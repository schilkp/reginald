mod c;

use std::fs;
use std::path::PathBuf;

use clap::Parser;
use reginald_codegen::builtin::c as codegen_c;
use reginald_codegen::builtin::md;
use reginald_codegen::builtin::rs;
use reginald_codegen::error::Error;
use reginald_codegen::regmap::RegisterMap;

use crate::diff;

#[derive(Parser, Debug)]
#[command(about = "Generate register management code from register listing")]
#[command(subcommand_value_name = "GENERATOR")]
#[command(subcommand_help_heading = "Generators")]
pub struct Command {
    /// Input yaml or (h)json listing file path
    #[arg(short)]
    pub input: PathBuf,

    /// Output file path or '-' for stdout.
    #[arg(short)]
    pub output: PathBuf,

    /// Overwrite map name
    #[arg(long)]
    pub overwrite_map_name: Option<String>,

    /// Verify that existing output file is up-to-date
    ///
    /// Instead of generating the specified output file, verify that a file
    /// exists at that location, and matches what would have been generated
    /// using the given input file, generator, and settings,.
    #[arg(long, default_value = "false", verbatim_doc_comment)]
    pub verify: bool,

    /// Output generator
    #[command(subcommand)]
    pub generator: Generator,
}

#[derive(Parser, Debug)]
pub enum Generator {
    /// C header with register structs, and packing/unpacking functions
    CFuncpack(c::funcpack::Cli),
    /// C header with field mask/shift macros
    CMacromap(codegen_c::macromap::GeneratorOpts),
    /// Markdown datasheet
    MdDatasheet,
    /// Markdown decode report of register dump
    MdRegdumpDecode(md::datasheet::regdump::GeneratorOpts),
    /// Rust module with register structs and no dependencies
    RsStructs(rs::structs::GeneratorOpts),
}

pub fn cmd(generate: Command) -> Result<(), Error> {
    // Read input map:
    let mut map = RegisterMap::from_file(&generate.input)?;

    if let Some(name) = &generate.overwrite_map_name {
        map.name = name.to_string();
    }

    // Generate output:
    let mut out = String::new();
    match &generate.generator {
        Generator::CFuncpack(opts) => codegen_c::funcpack::generate(&mut out, &map, &generate.output, opts.into())?,
        Generator::CMacromap(opts) => codegen_c::macromap::generate(&mut out, &map, &generate.output, opts)?,
        Generator::MdDatasheet => md::datasheet::generate(&mut out, &map)?,
        Generator::MdRegdumpDecode(opts) => md::datasheet::regdump::generate(&mut out, &map, opts)?,
        Generator::RsStructs(opts) => rs::structs::generate(&mut out, &map, opts)?,
    };

    // Verify or write ouput:
    if generate.verify {
        let output_content = fs::read_to_string(&generate.output)?;
        if output_content != out {
            let diff_msg = diff::diff_report(&output_content, &out);
            let msg =
                format!("File {} differs from generator output!\n{}", generate.output.to_string_lossy(), diff_msg);
            Err(Error::VerificationError(msg))?;
        }
        return Ok(());
    }

    if generate.output.to_string_lossy().trim() == "-" {
        println!("{}", out);
    } else {
        fs::write(generate.output, out)?;
    }

    Ok(())
}
