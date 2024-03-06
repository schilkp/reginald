use std::fmt::Write;

use crate::{error::Error, regmap::TypeBitwidth, utils::str_pad_to_length};
use lazy_static::lazy_static;
use regex::Regex;

pub mod funcpack;
pub mod macromap;

fn c_macro(s: &str) -> String {
    c_sanitize(&s.to_uppercase())
}

fn c_code(s: &str) -> String {
    c_sanitize(&s.to_lowercase())
}

fn generate_section_header_comment(out: &mut dyn Write, title: &str) -> Result<(), Error> {
    writeln!(out, "{}", str_pad_to_length(&format!("// ==== {} ", title), '=', 80))?;
    Ok(())
}

lazy_static! {
    static ref C_SANITIZE_SCHEMATIC: Regex = Regex::new(r"[^_a-zA-Z0-9]").unwrap();
}

fn c_sanitize(s: &str) -> String {
    C_SANITIZE_SCHEMATIC.replace_all(s, "_").into()
}

fn c_fitting_unsigned_type(width: TypeBitwidth) -> Result<String, Error> {
    match width {
        1..=8 => Ok("uint8_t".to_string()),
        9..=16 => Ok("uint16_t".to_string()),
        17..=32 => Ok("uint32_t".to_string()),
        33..=64 => Ok("uint64_t".to_string()),
        _ => Err(Error::GeneratorError(format!("Cannot represent {width}-bit wide value as C type!"))),
    }
}
