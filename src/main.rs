use std::process::ExitCode;

use swab::run::{Options, run};

const USAGE: &str = "usage: swab [--base <ref>] [--dry-run]

Removes comments introduced by the current branch's changes.";

fn main() -> ExitCode {
    let options = match parse(std::env::args().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("swab: {message}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match run(&options) {
        Ok(report) => {
            for file in &report.files {
                let plural = if file.removed == 1 {
                    "comment"
                } else {
                    "comments"
                };
                println!("{}: removed {} {plural}", file.path, file.removed);
            }
            if report.files.is_empty() {
                println!("no comments introduced since {}", report.base);
            }
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("swab: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse(args: impl Iterator<Item = String>) -> Result<Option<Options>, String> {
    let mut options = Options {
        base: None,
        dry_run: false,
    };
    let mut args = args.peekable();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => return Ok(None),
            "--dry-run" | "-n" => options.dry_run = true,
            "--base" | "-b" => {
                options.base = Some(args.next().ok_or("--base needs a ref")?);
            }
            other => return Err(format!("unexpected argument: {other}")),
        }
    }

    Ok(Some(options))
}
