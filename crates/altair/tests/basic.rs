//! Loads Altair BASIC from its paper tape and runs a line of BASIC.
//!
//! The test needs `roms/8K BASIC Ver 4-0.tap`; see `roms/README.md`.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;

use altair::{Altair, Ram, Sio2, Terminal};

/// The bootstrap loader for 8K BASIC 4.0, the program an Altair owner toggled
/// in to read the tape. The listing comes from [Loading Basic with the 88-2SIO
/// Board] by Martin Eberhard (2013), in octal, as MITS printed it.
///
/// [Loading Basic with the 88-2SIO Board]: https://deramp.com/downloads/mfe_archive/010-S100%20Computers%20and%20Boards/00-MITS/10-MITS%20S100%20Boards/88-2SIO%20Dual%20Serial%20Board/Loading%20Basic%20with%20the%2088-2SIO%20Board.pdf
const BOOTSTRAP_LOADER: [u8; 28] = [
    0o076, 0o003, // 000  MVI A,3      reset command for the ACIA
    0o323, 0o020, // 002  OUT 10H      reset the ACIA
    0o076, 0o025, // 004  MVI A,25Q    8 data bits, 1 stop bit
    0o323, 0o020, // 006  OUT 10H      set up the ACIA
    0o041, 0o302, 0o037, // 010  LXI H,1FC2H  the checksum loader ends here
    0o061, 0o032, 0o000, // 013  LXI SP,032Q  each RET below jumps to 013
    0o333, 0o020, // 016  IN 10H       read the status
    0o017, //        020  RRC          bit 0 (a byte waits) into CY
    0o320, //        021  RNC          no byte: back to 013
    0o333, 0o021, // 022  IN 11H       read the byte
    0o275, //        024  CMP L        a leader byte?
    0o310, //        025  RZ           yes: back to 013
    0o055, //        026  DCR L        next address down
    0o167, //        027  MOV M,A      store the byte
    0o300, //        030  RNZ          more to load: back to 013
    0o351, //        031  PCHL         run the checksum loader
    0o013, 0o000, // 032  DW 013Q      the return address for RET
];
/// The sense switches for BASIC 4.x on the 88-2SIO port A: all down.
const SENSE_SWITCHES: u8 = 0x00;
/// Enough instructions to load BASIC and run one line, with room to spare.
const MAX_STEPS: u64 = 10_000_000;

/// A Teletype with a paper tape reader: the line carries the tape first and
/// then the keys typed in advance. The test keeps a second handle to the
/// paper, because the machine owns the terminal.
struct Teletype {
    input: VecDeque<u8>,
    paper: Rc<RefCell<Vec<u8>>>,
}

impl Terminal for Teletype {
    fn receive(&mut self) -> Option<u8> {
        self.input.pop_front()
    }

    fn send(&mut self, byte: u8) {
        self.paper.borrow_mut().push(byte);
    }
}

#[test]
fn eight_k_basic_prints_sum() {
    let mut input: VecDeque<u8> = read_tape("8K BASIC Ver 4-0.tap").into();
    // MEMORY SIZE? all; TERMINAL WIDTH? 72; WANT SIN-COS-TAN-ATN? yes
    input.extend(b"\r\rY\rPRINT 2+2\r");
    let paper = Rc::new(RefCell::new(Vec::new()));
    let teletype = Teletype {
        input,
        paper: Rc::clone(&paper),
    };
    let mut altair = Altair::builder()
        .plug(Ram::new(0x0000..=0xFFFF))
        .plug(Sio2::new(teletype))
        .build();
    let mut panel = altair.front_panel();
    panel.toggle_in(0x0000, &BOOTSTRAP_LOADER);
    panel.set_switches(0x0000);
    panel.examine();
    panel.set_switches(u16::from(SENSE_SWITCHES) << 8);
    panel.run();

    for _ in 0..MAX_STEPS {
        altair.step();
    }

    let paper = String::from_utf8_lossy(&paper.borrow()).into_owned();
    assert!(paper.contains("ALTAIR BASIC REV. 4.0"), "{paper}");
    assert!(paper.contains("PRINT 2+2\r\n 4 \r\nOK"), "{paper}");
}

fn read_tape(name: &str) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../roms")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("{}: {error}. See roms/README.md", path.display()))
}
