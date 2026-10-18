//! Your terminal window as the [`Terminal`] at the end of the serial cable.
//!
//! ```text
//!   keyboard ──► thread ──► channel ──┐
//!                                     ├──► receive() ──► 88-2SIO ──► the program
//!   paper tape ───────────────────────┘    tape first, then keys
//!
//!   the program ──► 88-2SIO ──► send() ──► your screen
//! ```
//!
//! The thread drops non-ASCII bytes, turns Backspace into the erase key the
//! program expects, and sets the quit flag on Ctrl+].
//!
//! ```
//! use altair::cards::Terminal;
//! use app::host::console::{Console, RUBOUT_KEY};
//!
//! let (mut console, _quit) = Console::start(b"10 PRINT", RUBOUT_KEY);
//!
//! assert_eq!(console.receive(), Some(b'1')); // the tape comes before your keys
//! ```

use std::collections::VecDeque;
use std::io::{self, Read, Stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver};
use std::thread;

use altair::cards::Terminal;

/// The key that quits the emulator: Ctrl+], as in telnet. The Altair
/// programs don't use it, so every other key reaches the machine.
const QUIT_KEY: u8 = 0x1D;
/// The byte that your Backspace key sends: DEL, which the Teletype called
/// RUBOUT. Altair BASIC erases a character on it.
pub const RUBOUT_KEY: u8 = 0x7F;
/// Ctrl+H, the byte that CP/M and MBASIC erase a character on. They take
/// [`RUBOUT_KEY`] as an erase too, but echo the erased character instead
/// of moving the cursor back.
pub const BACKSPACE_KEY: u8 = 0x08;

/// Puts your terminal in raw mode and restores it when you drop the value.
///
/// In raw mode the terminal passes each key to the emulator at once, without
/// waiting for Enter, and doesn't echo it. Altair BASIC echoes what you type
/// itself.
pub struct RawMode;

impl RawMode {
    /// Switches the terminal to raw mode.
    ///
    /// ```no_run
    /// use app::host::console::RawMode;
    ///
    /// let raw_mode = RawMode::enable()?;
    /// // Each key now reaches the emulator at once, without an echo.
    /// drop(raw_mode);
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn enable() -> io::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // Nothing to do if this fails: the process is about to exit.
        let _ = crossterm::terminal::disable_raw_mode();
    }
}

/// Keys from your keyboard and bytes to your screen, plus the paper tape
/// reader that sat on the same serial line of the Teletype.
///
/// [`receive`](Terminal::receive) never waits: the keyboard thread hands
/// over each key through a channel. The thread drops bytes above 7FH,
/// because a Teletype sends 7-bit ASCII and a letter from another keyboard
/// layout would reach BASIC as junk.
pub struct Console {
    tape: VecDeque<u8>,
    keys: Receiver<u8>,
    stdout: Stdout,
}

impl Console {
    /// Starts the keyboard thread with `tape` in the reader. The thread
    /// sends `erase_key` for each [`RUBOUT_KEY`]. Returns the console and a
    /// flag that turns `true` when you press Ctrl+].
    ///
    /// ```
    /// use std::sync::atomic::Ordering;
    ///
    /// use altair::cards::Terminal;
    /// use app::host::console::{BACKSPACE_KEY, Console};
    ///
    /// let (mut console, quit) = Console::start(b"", BACKSPACE_KEY);
    ///
    /// assert_eq!(console.receive(), None); // no tape, and you typed nothing yet
    /// assert!(!quit.load(Ordering::Relaxed));
    /// ```
    pub fn start(tape: &[u8], erase_key: u8) -> (Self, Arc<AtomicBool>) {
        let (sender, keys) = mpsc::channel();
        let quit = Arc::new(AtomicBool::new(false));
        let quit_pressed = Arc::clone(&quit);

        thread::spawn(move || {
            for key in io::stdin().lock().bytes() {
                let Ok(key) = key else { break };
                if key == QUIT_KEY {
                    quit_pressed.store(true, Ordering::Relaxed);
                    break;
                }
                if !key.is_ascii() {
                    continue;
                }
                let key = if key == RUBOUT_KEY { erase_key } else { key };
                if sender.send(key).is_err() {
                    break;
                }
            }
        });

        let console = Self {
            tape: tape.iter().copied().collect(),
            keys,
            stdout: io::stdout(),
        };
        (console, quit)
    }
}

impl Terminal for Console {
    fn receive(&mut self) -> Option<u8> {
        self.tape.pop_front().or_else(|| self.keys.try_recv().ok())
    }

    fn send(&mut self, byte: u8) {
        // A program can't react to a broken screen, so we drop the byte.
        let _ = self.stdout.write_all(&[byte]);
        let _ = self.stdout.flush();
    }
}
