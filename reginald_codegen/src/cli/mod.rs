mod cmd;
mod diff;

use std::process::ExitCode;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about)]
#[command(name = "reginald")]
enum Cli {
    Gen(cmd::gen::Command),
    Completion(cmd::completion::Command),
    Tool(cmd::tool::Command),
}

pub fn cli_main() -> ExitCode {
    let cli = Cli::parse();

    let err = match cli {
        Cli::Gen(gen) => cmd::gen::cmd(gen),
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
