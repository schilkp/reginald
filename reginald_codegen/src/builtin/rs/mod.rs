use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Write;

use crate::{
    error::Error,
    regmap::{Docs, Layout, TypeBitwidth, TypeValue},
    utils::{grab_byte, str_pad_to_length, Endianess},
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

fn rs_layout_overview_comment(layout: &Layout, prefix: &str) -> String {
    layout
        .overview_text(true)
        .lines()
        .map(|x| String::from(prefix) + x)
        .collect::<Vec<String>>()
        .join("\n")
}

/// Convert a value to an array literal of given endianess
fn array_literal(endian: Endianess, val: TypeValue, width_bytes: TypeBitwidth) -> String {
    let mut bytes: Vec<String> = vec![];

    for i in 0..width_bytes {
        let byte = format!("0x{:X}", ((val >> (8 * i)) & 0xFF) as u8);
        bytes.push(byte);
    }

    if matches!(endian, Endianess::Big) {
        bytes.reverse();
    }

    format!("[{}]", bytes.join(", "))
}

/// Convert a value to an array literal of given endianess
fn masked_array_literal(endian: Endianess, val_name: &str, mask: TypeValue, width_bytes: TypeBitwidth) -> String {
    let mut bytes = vec![];

    for i in 0..width_bytes {
        let mask = grab_byte(Endianess::Little, mask, i, width_bytes);
        let byte = if mask == 0 {
            String::from("0")
        } else if mask == 0xFF {
            format!("{val_name}[{i}]")
        } else {
            format!("{val_name}[{i}] & 0x{mask:X}")
        };
        bytes.push(byte);
    }

    if matches!(endian, Endianess::Big) {
        bytes.reverse();
    }

    format!("[{}]", bytes.join(", "))
}

pub const CONVERSION_TRAITS: &str = include_str!("../../../../reginald/src/lib.rs");
