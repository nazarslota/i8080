//! A minimal CP/M environment for running `.COM` programs.
//!
//! This is not an emulation of real CP/M or of any hardware. The program is
//! loaded at `0x0100`, and its system calls (`CALL 0x0005`) are handled in
//! Rust instead of by a real operating system. This technique is known as
//! high-level emulation (HLE).
//!
//! Its main job is running the classic 8080 diagnostic programs (TST8080,
//! 8080PRE, CPUTEST, 8080EXM), which were written for CP/M and report their
//! results through it.
//!
//! # Status
//!
//! Work in progress: the public API is not defined yet.
