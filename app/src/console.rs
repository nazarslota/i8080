//! Your terminal window as the [`Terminal`] at the end of the serial cable.

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

/// Puts your terminal in raw mode and restores it when you drop the value.
///
/// In raw mode the terminal passes each key to the emulator at once, without
/// waiting for Enter, and doesn't echo it. Altair BASIC echoes what you type
/// itself.
pub struct RawMode;

impl RawMode {
    /// Switches the terminal to raw mode.
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

/// Keys from your keyboard and bytes to your screen, plus a paper tape
/// reader.
///
/// A background thread reads the keyboard and sends each key over a channel,
/// so [`receive`](Terminal::receive) never waits. The thread drops bytes
/// above 7FH: a Teletype sends 7-bit ASCII, and a letter from another
/// keyboard layout arrives as such bytes, which BASIC would read as junk.
/// The tape reader sits on
/// the same serial line, as on the Teletype that Altair owners used: the
/// console sends the whole tape first and your keys after it.
pub struct Console {
    tape: VecDeque<u8>,
    keys: Receiver<u8>,
    stdout: Stdout,
}

impl Console {
    /// Starts the keyboard thread with `tape` in the reader. Returns the
    /// console and a flag that turns `true` when you press [`QUIT_KEY`].
    pub fn start(tape: Vec<u8>) -> (Self, Arc<AtomicBool>) {
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
                if sender.send(key).is_err() {
                    break;
                }
            }
        });

        let console = Self {
            tape: tape.into(),
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
        // A program can't react to a broken screen, so the byte is lost.
        let _ = self.stdout.write_all(&[byte]);
        let _ = self.stdout.flush();
    }
}
