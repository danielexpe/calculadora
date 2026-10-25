use std::process::ExitCode;

use calculadora::{app, capture, cli::Cli};
use clap::Parser;

fn main() -> ExitCode {
    run(Cli::parse())
}

fn run(cli: Cli) -> ExitCode {
    if cli.is_capture() {
        return match capture::write_frame(&cli) {
            Ok(info) => {
                if let Some(path) = &cli.capture_frame {
                    println!("wrote {} ({}x{})", path.display(), info.width, info.height);
                }
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("error: {error}");
                ExitCode::from(1)
            }
        };
    }

    match app::run_fb(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}
