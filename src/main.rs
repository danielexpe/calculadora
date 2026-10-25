use std::process::ExitCode;

use calculadora::cli::Cli;
use clap::Parser;

fn main() -> ExitCode {
    run(Cli::parse())
}

fn run(cli: Cli) -> ExitCode {
    if let Some(path) = &cli.capture_frame {
        eprintln!(
            "error: frame capture is not implemented until phase 2 (requested {})",
            path.display()
        );
        return ExitCode::from(1);
    }

    eprintln!("error: framebuffer runtime is not implemented until phase 2");
    eprintln!("help: phase 1 provides the domain, CLI, i18n and tests only");
    ExitCode::from(3)
}
