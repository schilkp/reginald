use std::fs;
use std::path::PathBuf;

use crate::builtin::c;
use crate::builtin::md;
use crate::builtin::rs;
use crate::error::Error;
use crate::regmap::RegisterMap;
use clap::Parser;

use crate::cli::diff;

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

pub fn cmd(gen: Command) -> Result<(), Error> {
    // Read input map:
    let mut map = RegisterMap::from_file(&gen.input)?;

    if let Some(name) = &gen.overwrite_map_name {
        map.name = name.to_string();
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
        return Ok(());
    }

    if gen.output.to_string_lossy().trim() == "-" {
        println!("{}", out);
    } else {
        fs::write(gen.output, out)?;
    }

    Ok(())
}
