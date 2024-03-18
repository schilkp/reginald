#![allow(clippy::large_enum_variant)]

mod diff;

use std::fs;
use std::io;
use std::{path::PathBuf, process::ExitCode};

use clap::{CommandFactory, Parser};
use reginald_codegen::builtin::c;
use reginald_codegen::builtin::md;
use reginald_codegen::builtin::rs;
use reginald_codegen::error::Error;
use reginald_codegen::regmap::RegisterMap;

#[derive(Parser, Debug)]
#[command(version, about)]
#[command(name = "reginald")]
enum Cli {
    Gen(CommandGenerate),
    Completion(CommandCompletion),
    Tool(CommandTool),
}

#[derive(Parser, Debug)]
#[command(about = "Generate register management code from register listing")]
#[command(subcommand_value_name = "GENERATOR")]
#[command(subcommand_help_heading = "Generators")]
struct CommandGenerate {
    /// Input yaml or (h)json listing file path
    #[arg(short)]
    input: PathBuf,

    /// Output file path
    #[arg(short)]
    output: PathBuf,

    /// Overwrite map name
    #[arg(long)]
    overwrite_map_name: Option<String>,

    /// Verify that existing output file is up-to-date
    ///
    /// Instead of generating the specified output file, verify that a file
    /// exists at that location, and matches what would have been generated
    /// using the given input file, generator, and settings,.
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    verify: bool,

    /// Output generator
    #[command(subcommand)]
    generator: Generator,
}

#[derive(Parser, Debug)]
enum Generator {
    /// C header with register structs, and packing/unpacking functions
    CFuncpack(c::funcpack::GeneratorOpts),
    /// C header with field mask/shift macros
    CMacromap(c::macromap::GeneratorOpts),
    /// Markdown datasheet
    MdDatasheet,
    /// Markdown decode report of register dump
    MdRegdumpDecode(md::datasheet::regdump::GeneratorOpts),
    /// Rust module with register structs and no dependencies
    RsStructs(rs::structs::GeneratorOpts),
}

#[derive(Parser, Debug)]
#[command(about = "Print completion script for specified shell")]
struct CommandCompletion {
    shell: clap_complete::Shell,
}

#[derive(Parser, Debug)]
#[command(about = "Built-in tools and utilities.")]
#[command(subcommand_value_name = "TOOL")]
#[command(subcommand_help_heading = "Tools")]
struct CommandTool {
    #[command(subcommand)]
    tool: Tool,
}

#[derive(Parser, Debug)]
enum Tool {
    RsStructsTraits,
}

pub fn cli_main() -> ExitCode {
    let cli = Cli::parse();

    let err = match cli {
        Cli::Gen(gen) => cmd_generate(gen),
        Cli::Completion(c) => cmd_completion(c),
        Cli::Tool(_) => todo!(),
    };

    match err {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{}", err);
            ExitCode::FAILURE
        }
    }
}

fn cmd_generate(gen: CommandGenerate) -> Result<(), Error> {
    // Read input map:
    let mut map = RegisterMap::from_file(&gen.input)?;

    if let Some(name) = &gen.overwrite_map_name {
        map.map_name = name.to_string();
    }

    // Generate output:
    let mut out = String::new();
    match &gen.generator {
        Generator::CFuncpack(opts) => c::funcpack::generate(&mut out, &map, &gen.output, opts)?,
        Generator::CMacromap(opts) => c::macromap::generate(&mut out, &map, &gen.output, opts)?,
        Generator::MdDatasheet => md::datasheet::generate(&mut out, &map)?,
        Generator::MdRegdumpDecode(opts) => md::datasheet::regdump::generate(&mut out, &map, opts)?,
        Generator::RsStructs(opts) => rs::structs::generate(&mut out, &map, opts)?,
    };

    // Verify or write ouput:
    if gen.verify {
        let output_content = fs::read_to_string(&gen.output)?;
        if output_content != out {
            let diff_msg = diff::diff_report(&output_content, &out);
            let msg = format!("File {} differs from generator output!\n{}", gen.output.to_string_lossy(), diff_msg);
            Err(Error::VerificationError(msg))?;
        }
    } else {
        fs::write(gen.output, out)?;
    }
    Ok(())
}

fn cmd_completion(compl: CommandCompletion) -> Result<(), Error> {
    clap_complete::generate(compl.shell, &mut Cli::command(), "reginald", &mut io::stdout());
    Ok(())
}
