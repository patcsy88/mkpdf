//! `mkpdf` command-line interface. Only `--version` exists in the foundation milestone.

#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    if matches!(arg.as_deref(), Some("--version" | "-V")) {
        println!("mkpdf {}", mkpdf_core::VERSION);
        ExitCode::SUCCESS
    } else {
        eprintln!("usage: mkpdf --version");
        ExitCode::from(2)
    }
}
