use std::fmt::Write;

use crate::{
    error::Error,
    regmap::{Docs, TypeBitwidth},
    utils::str_pad_to_length,
};
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

lazy_static! {
    static ref C_SANITIZE_REGEX: Regex = Regex::new(r"[^_a-zA-Z0-9]").unwrap();
}

fn c_sanitize(s: &str) -> String {
    C_SANITIZE_REGEX.replace_all(s, "_").into()
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

fn c_generate_section_header_comment(out: &mut dyn Write, title: &str) -> Result<(), Error> {
    writeln!(out, "{}", str_pad_to_length(&format!("// ==== {} ", title), '=', 80))?;
    Ok(())
}

fn c_generate_doxy_comment(out: &mut dyn Write, docs: &Docs, prefix: &str, note: Option<&str>) -> Result<(), Error> {
    match (&docs.brief, note, &docs.doc) {
        (None, None, None) => (),
        (Some(brief), None, None) => {
            writeln!(out, "{prefix}/** @brief {brief} */")?;
        }
        (None, Some(note), None) => {
            writeln!(out, "{prefix}/** @note {note} */")?;
        }
        (brief, note, doc) => {
            writeln!(out, "{prefix}/**")?;
            if let Some(brief) = brief {
                writeln!(out, "{prefix} * @brief {brief}")?;
            }
            if let Some(note) = note {
                writeln!(out, "{prefix} * @note {note}")?;
            }
            if let Some(doc) = doc {
                for line in doc.lines() {
                    writeln!(out, "{prefix} * {line}")?;
                }
            }
            writeln!(out, "{prefix} */")?;
        }
    }
    Ok(())
}
