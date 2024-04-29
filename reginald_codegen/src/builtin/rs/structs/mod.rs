mod enums;
mod layouts;
mod registers;

use std::fmt::Write;

use crate::{
    bits::{lsb_pos, mask_width, msb_pos},
    builtin::rs::rs_const,
    error::Error,
    regmap::{Enum, FieldType, Layout, LayoutField, Register, RegisterBlock, RegisterMap, TypeValue},
    utils::{grab_byte, packed_byte_to_field_transform, Endianess},
    writer::header_writer::HeaderWriter,
};
use clap::Parser;

use super::{
    generate_doc_comment, rs_fitting_unsigned_type, rs_generate_header_comment, rs_header_comment,
    rs_layout_overview_comment, rs_pascalcase, rs_snakecase, CONVERSION_TRAITS,
};

// ====== Generator Opts =======================================================

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Rust type to use for register addresses.
    ///
    /// If none is specified, the smallest unsigned type capable of storing
    /// the largest address will be used.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub address_type: Option<String>,

    /// Split registers and register blocks into seperate modules.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub split_into_modules: bool,

    /// Trait to derive on all register structs.
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub struct_derive: Vec<String>,

    /// Trait to derive on all enums.
    ///
    /// May be given multiple times. Note: All enums always derive
    /// the "Clone" and "Copy" traits.
    #[cfg_attr(feature = "cli", arg(long = "enum-derive"))]
    #[cfg_attr(feature = "cli", arg(value_name = "DERIVE"))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub raw_enum_derive: Vec<String>,

    /// Module should be 'use'ed at the top of the generated module.
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub add_use: Vec<String>,

    /// Module attributes that should be added at the top of the generated file.
    ///
    /// For example, a value of `allow(dead_code)` will result in `#![allow(dead_code)]` to be
    /// added to to the beginning of the generated module.
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub add_attribute: Vec<String>,

    /// Use an external definition of the `ToBytes`/`FromBytes`/`TryFromBytes` traits,
    ///
    /// No trait definition are generated, and implementations of the traits refeer
    /// to `[prefix]ToBytes`, `[prefix]FromBytes`, and `[prefix]TryFromBytes`,
    /// where `[preifx]` is the value given to this flag.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub external_traits: Option<String>,

    /// Generate `From/TryFrom/From` implementations that convert a register
    /// to/from the smallest rust unsigned integer value wide enough to hold the
    /// register, if one exists.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_uint_conversion: bool,
}

// ====== Generator ============================================================

struct Input<'a> {
    opts: GeneratorOpts,
    map: &'a RegisterMap,
    address_type: String,
    enum_derives: Vec<String>,
}

pub fn generate(out: &mut dyn Write, map: &RegisterMap, opts: &GeneratorOpts) -> Result<(), Error> {
    // Determine address type: Use option override, or smallest
    // unsigned type that fits the largest address in the map.
    let address_type = if let Some(address_type) = &opts.address_type {
        address_type.clone()
    } else {
        let max_addr = map.registers.values().map(|x| x.adr).max().unwrap_or(0);
        rs_fitting_unsigned_type(msb_pos(max_addr) + 1)?
    };

    // Gather derives to be applied to all enums.
    let mut enum_derives: Vec<String> = vec!["Clone".into(), "Copy".into()];
    enum_derives.extend(opts.raw_enum_derive.clone());

    // Generate
    let inp = Input {
        opts: opts.clone(),
        enum_derives,
        address_type,
        map,
    };

    let mut out = HeaderWriter::new(out);

    // File header/preamble:
    generate_header(&mut out, &inp)?;

    // ===== Shared enums: =====

    out.push_section_with_header(&["\n", &rs_header_comment("Shared Enums"), "\n"]);
    for shared_enum in inp.map.shared_enums() {
        enums::generate_enum(&mut out, &inp, shared_enum)?;
        enums::generate_enum_impls(&mut out, &inp, shared_enum, false)?;
    }
    out.pop_section();

    // ===== Shared layouts: =====

    for layout in inp.map.shared_layouts() {
        writeln!(&mut out)?;
        writeln!(&mut out, "{}", &rs_header_comment(&format!("`{}` Shared Layout", layout.name)))?;
        layouts::generate_layout(&mut out, &inp, layout, false)?;
    }

    // ===== Individual Registers: =====

    for register in inp.map.individual_registers() {
        registers::generate_register(&mut out, &inp, register)?;
    }

    // ===== Register Blocks: =====

    for block in inp.map.register_blocks.values() {
        registers::generate_register_block(&mut out, &inp, block)?;
    }

    // ===== Traits: =====

    if inp.opts.external_traits.is_none() {
        generate_traits(&mut out)?;
    }

    Ok(())
}

/// Generate file header
fn generate_header(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    writeln!(out, "#![allow(clippy::unnecessary_cast)]")?;
    writeln!(out, "#![allow(clippy::module_name_repetitions)]")?;
    writeln!(out, "#![allow(unused_imports)]")?;
    for attr in &inp.opts.add_attribute {
        writeln!(out, "#![{}]", attr)?;
    }

    // Top doc comment:
    writeln!(out, "//! `{}` Registers", inp.map.name)?;
    writeln!(out, "//!")?;

    // Generated-with-reginald note, including original file name if known:
    if let Some(input_file) = &inp.map.from_file {
        writeln!(out, "//! Generated using reginald from `{}`.", input_file.to_string_lossy())?;
    } else {
        writeln!(out, "//! Generated using reginald.")?;
    }

    // Indicate which generator was used:
    writeln!(out, "//! Generator: rs-structs")?;

    // Map top-level documentation:
    if !inp.map.docs.is_empty() {
        writeln!(out, "//!")?;
        write!(out, "{}", inp.map.docs.as_multiline("//! "))?;
    }

    // Map author and note:
    if let Some(author) = &inp.map.author {
        writeln!(out, "//! ")?;
        writeln!(out, "//! Listing file author: {author}")?;
    }
    if let Some(notice) = &inp.map.notice {
        writeln!(out, "//!")?;
        writeln!(out, "//! Listing file notice:")?;
        for line in notice.lines() {
            writeln!(out, "//!   {line}")?;
        }
    }

    if !inp.opts.add_use.is_empty() {
        writeln!(out)?;
        for add_use in &inp.opts.add_use {
            writeln!(out, "use {add_use};")?;
        }
    }

    Ok(())
}

/// Traits section
fn generate_traits(out: &mut dyn Write) -> Result<(), Error> {
    writeln!(out)?;
    rs_generate_header_comment(out, "Traits")?;
    writeln!(out)?;
    write!(out, "{}", CONVERSION_TRAITS)?;
    Ok(())
}

/// Decide trait prefix. If an external override is given, use that.
/// Otherwise, use the local definition (Which may be in the parent
/// module)
fn trait_prefix(inp: &Input, in_module: bool) -> String {
    if let Some(overide) = &inp.opts.external_traits {
        String::from(overide)
    } else if in_module {
        String::from("super::")
    } else {
        String::new()
    }
}

/// Prefix a given identifier with `super::` if required.
fn prefix_with_super(inp: &Input, s: &str, is_local: bool, in_module: bool) -> String {
    if inp.opts.split_into_modules && !is_local && in_module {
        format!("super::{s}")
    } else {
        String::from(s)
    }
}

#[allow(clippy::enum_variant_names)]
enum FromBytesImpl {
    FromBytes,
    FromMaskedBytes,
    TryFromBytes,
}

fn enum_impl(e: &Enum) -> FromBytesImpl {
    if e.can_unpack_min_bitwidth() && [8, 16, 32, 64, 128].contains(&e.min_bitdwith()) {
        FromBytesImpl::FromBytes
    } else if e.can_unpack_masked() {
        FromBytesImpl::FromMaskedBytes
    } else {
        FromBytesImpl::TryFromBytes
    }
}
