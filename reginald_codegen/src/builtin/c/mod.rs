use std::fmt::Write;

use crate::{
    error::Error,
    regmap::{Docs, Layout, TypeBitwidth},
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

fn c_section_header_comment(title: &str) -> String {
    let lbar = str_pad_to_length("// ", '=', 80);
    let lcontent = str_pad_to_length(&format!("// ==== {title} "), '=', 80);

    format!("{lbar}\n{lcontent}\n{lbar}")
}

fn c_generate_section_header_comment(out: &mut dyn Write, title: &str) -> Result<(), Error> {
    writeln!(out, "{}", c_section_header_comment(title))?;
    Ok(())
}

fn c_header_comment(title: &str) -> String {
    str_pad_to_length(&format!("// ---- {title} "), '-', 80)
}

fn c_generate_header_comment(out: &mut dyn Write, title: &str) -> Result<(), Error> {
    writeln!(out, "{}", c_header_comment(title))?;
    Ok(())
}

fn c_generate_doxy_comment(
    out: &mut dyn Write,
    docs: &Docs,
    prefix: &str,
    mut nodes: Vec<(String, String)>,
) -> Result<(), Error> {
    if let Some(brief) = &docs.brief {
        nodes.push((String::from("brief"), brief.to_owned()));
    }

    nodes.sort_by_key(|x| match x.0.as_str() {
        "name" => 0,
        "brief" => 1,
        "warning" => 2,
        "note" => 3,
        "returns" => 4,
        _ => 5,
    });

    if nodes.is_empty() && docs.doc.is_none() {
        Ok(())
    } else if nodes.len() == 1 && docs.doc.is_none() {
        writeln!(out, "{prefix}/** @{} {} */", nodes[0].0, nodes[0].1)?;
        Ok(())
    } else {
        writeln!(out, "{prefix}/**")?;
        for (node_name, node) in &nodes {
            writeln!(out, "{prefix} * @{node_name} {node}")?
        }
        if let Some(doc) = &docs.doc {
            for line in doc.lines() {
                writeln!(out, "{prefix} * {line}")?;
            }
        }
        writeln!(out, "{prefix} */")?;
        Ok(())
    }
}

fn c_layout_overview_comment(layout: &Layout) -> String {
    layout
        .overview_text(false)
        .lines()
        .map(|x| String::from("//  ") + x)
        .collect::<Vec<String>>()
        .join("\n")
}
