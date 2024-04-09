use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Write;

use crate::{
    error::Error,
    regmap::{Docs, Layout, TypeBitwidth},
    utils::str_pad_to_length,
};

pub mod structs;

fn rs_pascalcase(s: &str) -> String {
    let mut result = String::new();
    for part in rs_sanitize(s).to_lowercase().split('_') {
        let mut chars = part.chars();
        if let Some(f) = chars.next() {
            result.push_str(&(f.to_uppercase().collect::<String>() + chars.as_str()));
        }
    }
    result
}

fn rs_snakecase(s: &str) -> String {
    rs_sanitize(s).to_lowercase()
}

fn rs_const(s: &str) -> String {
    rs_sanitize(s).to_uppercase()
}

lazy_static! {
    static ref RS_SANITIZE_REGEX: Regex = Regex::new(r"[^_a-zA-Z0-9]").unwrap();
}

fn rs_sanitize(s: &str) -> String {
    RS_SANITIZE_REGEX.replace_all(s, "_").into()
}

fn generate_doc_comment(out: &mut dyn Write, docs: &Docs, prefix: &str) -> Result<(), Error> {
    match (&docs.brief, &docs.doc) {
        (None, None) => (),
        (Some(brief), None) => {
            writeln!(out, "{prefix}/// {brief}")?;
        }
        (None, Some(doc)) => {
            for line in doc.lines() {
                writeln!(out, "{prefix}/// {line}")?;
            }
        }
        (Some(brief), Some(doc)) => {
            writeln!(out, "{prefix}/// {brief}")?;
            writeln!(out, "{prefix}///")?;
            for line in doc.lines() {
                writeln!(out, "{prefix}/// {line}")?;
            }
        }
    }
    Ok(())
}

fn rs_fitting_unsigned_type(width: TypeBitwidth) -> Result<String, Error> {
    match width {
        1..=8 => Ok("u8".to_string()),
        9..=16 => Ok("u16".to_string()),
        17..=32 => Ok("u32".to_string()),
        33..=64 => Ok("u64".to_string()),
        65..=128 => Ok("u128".to_string()),
        _ => Err(Error::GeneratorError(format!("Cannot represent {width}-bit wide value as a rust type!"))),
    }
}

fn rs_header_comment(title: &str) -> String {
    str_pad_to_length(&format!("// ==== {title} "), '=', 80)
}

fn rs_generate_header_comment(out: &mut dyn Write, title: &str) -> Result<(), Error> {
    writeln!(out, "{}", rs_header_comment(title))?;
    Ok(())
}

fn rs_layout_overview_comment(layout: &Layout) -> String {
    layout
        .overview_text(true)
        .lines()
        .map(|x| String::from("///  ") + x)
        .collect::<Vec<String>>()
        .join("\n")
}
