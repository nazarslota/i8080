//! The `altair` binary. The code lives in the `app` library; see its
//! documentation.

use std::process::ExitCode;

use app::cli::Args;
use clap::Parser;

fn main() -> ExitCode {
    match app::run(&Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("altair: {error}");
            ExitCode::FAILURE
        }
    }
}
