use std::{fs, path::PathBuf};

use crate::{builtin::rs::CONVERSION_TRAITS, error::Error};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Built-in tools and utilities")]
#[command(subcommand_value_name = "TOOL")]
#[command(subcommand_help_heading = "Tools")]
pub struct Command {
    #[command(subcommand)]
    /// Output file path or '-' for stdout.
    pub tool: Tool,
}

#[derive(Parser, Debug)]
pub enum Tool {
    /// Emit rust reginald trait definitions
    ///
    /// If a project requries multiple rs-structs register maps, it is
    /// desireable to have them share the same traits instead of having
    /// each map declare an indentical but seperate trait. This command
    /// generates the trait definitions which may be included in the project
    /// seperately, and used in the rs-structs register maps through
    /// the `--external-traits` flag.
    RsReginaldTraits(RsReginaldTraits),
}

#[derive(Debug, Clone, Parser)]
pub struct RsReginaldTraits {
    #[arg(short)]
    pub output: PathBuf,
}

pub fn cmd(tool: Command) -> Result<(), Error> {
    match tool.tool {
        Tool::RsReginaldTraits(opts) => {
            let traits = CONVERSION_TRAITS;
            if opts.output.to_string_lossy().trim() == "-" {
                println!("{traits}");
            } else {
                fs::write(opts.output, traits)?;
            }
        }
    }
    Ok(())
}
