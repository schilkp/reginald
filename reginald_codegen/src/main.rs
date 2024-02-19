use std::process::ExitCode;

#[cfg(feature = "cli")]
mod cli;

#[cfg(feature = "cli")]
fn main() -> ExitCode {
    cli::cli_main()
}

#[cfg(not(feature = "cli"))]
fn main() -> ExitCode {
    eprintln!("Error: Reginald codegen compiled without cli.");
    ExitCode::FAILURE
}
