//! Keeps the last instructions the CPU ran, for `--trace`.
//!
//! The clock adds a line before each instruction. When the trace is full,
//! each new line pushes out the oldest, so memory stays the same however
//! long the program runs. You write the trace to a file after the run:
//!
//! ```text
//!   push ──► [ oldest │ ... │ newest ] ──► write_to: the header, then
//!            at most `capacity` lines        oldest to newest
//! ```
//!
//! ```
//! use i8080::{Registers, TraceLine};
//! use app::trace::Trace;
//!
//! let mut trace = Trace::new(2);
//! for pc in 0..3 {
//!     let registers = Registers { pc, ..Registers::default() };
//!     trace.push(TraceLine::new(registers, |_| 0x00)); // NOP
//! }
//! let mut file = Vec::new();
//!
//! trace.write_to(&mut file)?;
//!
//! let text = String::from_utf8(file)?;
//! let pcs: Vec<&str> = text.lines().skip(1).map(|line| &line[..4]).collect();
//! assert_eq!(pcs, ["0001", "0002"]); // 0000H fell out
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::collections::VecDeque;
use std::io::{self, Write};

use i8080::TraceLine;

/// How many lines `--trace` keeps: the last 10,000 instructions.
pub const TRACE_LINES: usize = 10_000;

/// The last trace lines, up to a fixed count. See the module documentation
/// for an example.
pub struct Trace {
    lines: VecDeque<TraceLine>,
    capacity: usize,
}

impl Trace {
    /// Makes an empty trace that keeps the last `capacity` lines.
    pub fn new(capacity: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Adds `line`, and drops the oldest line when the trace is full.
    pub fn push(&mut self, line: TraceLine) {
        if self.capacity == 0 {
            return;
        }
        if self.lines.len() == self.capacity {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Writes [`TraceLine::HEADER`], then the lines from the oldest to the
    /// newest, one per text line.
    pub fn write_to(&self, mut out: impl Write) -> io::Result<()> {
        writeln!(out, "{}", TraceLine::HEADER)?;
        for line in &self.lines {
            writeln!(out, "{line}")?;
        }
        out.flush()
    }
}

#[cfg(test)]
mod tests {
    use i8080::Registers;

    use super::*;

    fn nop_at(pc: u16) -> TraceLine {
        let registers = Registers {
            pc,
            ..Registers::default()
        };
        TraceLine::new(registers, |_| 0x00)
    }

    fn written(trace: &Trace) -> String {
        let mut file = Vec::new();
        trace.write_to(&mut file).unwrap();
        String::from_utf8(file).unwrap()
    }

    #[test]
    fn empty_trace_writes_header_only() {
        let trace = Trace::new(10);

        let text = written(&trace);

        assert_eq!(text, format!("{}\n", TraceLine::HEADER));
    }

    #[test]
    fn full_trace_drops_oldest_line() {
        let mut trace = Trace::new(2);

        trace.push(nop_at(0x0000));
        trace.push(nop_at(0x0001));
        trace.push(nop_at(0x0002));

        assert_eq!(
            written(&trace),
            format!(
                "{}\n{}\n{}\n",
                TraceLine::HEADER,
                nop_at(0x0001),
                nop_at(0x0002)
            ),
        );
    }

    #[test]
    fn zero_capacity_keeps_nothing() {
        let mut trace = Trace::new(0);

        trace.push(nop_at(0x0000));

        assert_eq!(written(&trace), format!("{}\n", TraceLine::HEADER));
    }
}
