mod cmd;
mod diff;

use clap::Parser;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(version, about)]
#[command(name = "reginald")]
#[allow(clippy::large_enum_variant)]
enum Cli {
    Gen(cmd::generate::Command),
    Completion(cmd::completion::Command),
    Tool(cmd::tool::Command),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let err = match cli {
        Cli::Gen(generate) => cmd::generate::cmd(generate),
        Cli::Completion(c) => cmd::completion::cmd(c),
        Cli::Tool(tool) => cmd::tool::cmd(tool),
    };

    match err {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}
