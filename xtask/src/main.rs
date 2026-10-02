//! Development tasks for the marcasite workspace. Run `cargo xtask help`.
// A command-line tool: printing is its output, and its string slicing is at indices found by
// searching the same string.
#![allow(clippy::print_stdout, clippy::print_stderr, clippy::indexing_slicing)]

mod docs;

use std::path::Path;
use std::process::ExitCode;

const USAGE: &str = "\
Usage: cargo xtask <task>

Tasks:
  docs [--check]  Generate docs/sdk/ from rustdoc JSON (built by `just docs-md`);
                  with --check, fail if docs/sdk/ is out of date";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or(Path::new("."));
    let result = match args.as_slice() {
        ["docs"] => docs::run(root, false),
        ["docs", "--check"] => docs::run(root, true),
        ["help" | "--help" | "-h"] | [] => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e:#}");
            ExitCode::FAILURE
        }
    }
}
