mod enums;
mod layouts;
mod registers;

use std::fmt::Write;

use crate::{
    bits::{lsb_pos, mask_width, msb_pos},
    builtin::{md::md_table, rs::rs_const},
    error::Error,
    regmap::{Enum, FieldType, Layout, LayoutField, Register, RegisterBlock, RegisterMap, TypeValue},
    utils::{grab_byte, packed_byte_to_field_transform, Endianess},
    writer::header_writer::HeaderWriter,
};
use clap::Parser;

use self::layouts::LayoutStructKind;

use super::{
    generate_doc_comment, rs_fitting_unsigned_type, rs_generate_header_comment, rs_header_comment, rs_pascalcase,
    rs_snakecase, CONVERSION_TRAITS,
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
    #[cfg_attr(feature = "cli", arg(default_value_t = Self::default().generate_uint_conversion))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_uint_conversion: bool,
}

impl Default for GeneratorOpts {
    fn default() -> Self {
        Self {
            address_type: None,
            struct_derive: vec![],
            raw_enum_derive: vec![],
            add_use: vec![],
            add_attribute: vec![],
            external_traits: None,
            generate_uint_conversion: true,
        }
    }
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

    // ===== Traits: =====
    if inp.opts.external_traits.is_none() {
        generate_traits(&mut out)?;
    }

    // ===== Registers: =====
    let mut regs: Vec<_> = inp.map.registers.values().collect();
    regs.sort_by_key(|x| x.adr);
    for reg in regs {
        registers::generate_register(&mut out, &inp, reg)?;
    }

    // ===== Register Blocks: =====
    for block in inp.map.register_blocks.values() {
        registers::generate_register_block(&mut out, &inp, block)?;
    }

    // ===== Shared enums: =====
    out.push_section_with_header(&["\n", &rs_header_comment("Shared Enums"), "\n"]);
    for shared_enum in inp.map.shared_enums() {
        enums::generate_enum(&mut out, &inp, shared_enum)?;
    }
    out.pop_section();

    // ===== Shared layouts: =====
    for layout in inp.map.shared_layouts() {
        writeln!(&mut out)?;
        writeln!(&mut out, "{}", &rs_header_comment(&format!("`{}` Shared Layout", layout.name)))?;
        layouts::generate_layout(&mut out, &inp, layout, &LayoutStructKind::Layout)?;
    }

    // ===== Conversion functions: =====
    for e in inp.map.enums.values() {
        writeln!(&mut out)?;
        writeln!(&mut out, "{}", &rs_header_comment(&format!("`{}` Enum Conversion Functions", e.name)))?;
        enums::generate_enum_impls(&mut out, &inp, e)?;
    }
    for layout in inp.map.layouts.values() {
        writeln!(&mut out)?;
        writeln!(&mut out, "{}", &rs_header_comment(&format!("`{}` Layout Conversion Functions", layout.name)))?;
        layouts::generate_layout_impls(&mut out, &inp, layout)?;
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
    writeln!(out, "//! # `{}` Registers.", inp.map.name)?;

    // Map top-level documentation:
    if !inp.map.docs.is_empty() {
        writeln!(out, "//!")?;
        write!(out, "{}", inp.map.docs.as_multiline("//! "))?;
    }

    // Generated-with-reginald note, including original file name if known:
    writeln!(out, "//!")?;
    writeln!(out, "//! ## Infos")?;
    if let Some(input_file) = &inp.map.from_file {
        writeln!(out, "//!")?;
        writeln!(out, "//! Generated using reginald from `{}`.", input_file.to_string_lossy())?;
    } else {
        writeln!(out, "//!")?;
        writeln!(out, "//! Generated using reginald.")?;
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

    writeln!(out, "//!")?;
    writeln!(out, "//! ## Register Overview")?;
    let mut rows = vec![];
    rows.push(vec!["Address".to_string(), "Name".to_string(), "Brief".to_string()]);
    let mut regs: Vec<_> = inp.map.registers.values().collect();
    regs.sort_by_key(|x| x.adr);
    for reg in regs {
        let adr = format!("0x{:02X}", reg.adr);
        let name = format!("[`{}`]", rs_pascalcase(&reg.name));
        let brief = reg.docs.brief.clone().unwrap_or("".to_string());
        rows.push(vec![adr, name, brief]);
    }
    md_table(out, &rows, "//! ")?;

    // Additional uses:
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
fn trait_prefix(inp: &Input) -> String {
    if let Some(overide) = &inp.opts.external_traits {
        String::from(overide)
    } else {
        String::new()
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
