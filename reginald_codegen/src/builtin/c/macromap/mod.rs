use std::{fmt::Write, path::Path};

#[cfg(feature = "cli")]
use clap::Parser;
use reginald_utils::str_table;

use crate::{
    bits::lsb_pos,
    error::Error,
    regmap::{FieldType, Layout, Register, RegisterBlock, RegisterBlockMember, RegisterMap},
};

use super::{c_generate_section_header_comment, c_layout_overview_comment, c_macro};

// ====== Generator Opts =======================================================

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Surround header with a clang-format off guard
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value_t = Self::default().clang_format_guard))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub clang_format_guard: bool,

    /// Header file that should be included at the top of the generated header
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub add_include: Vec<String>,
}

impl Default for GeneratorOpts {
    fn default() -> Self {
        Self {
            clang_format_guard: true,
            add_include: vec![],
        }
    }
}

// ====== Generator ============================================================

pub fn generate(out: &mut dyn Write, map: &RegisterMap, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    generate_header(out, map, output_file, opts)?;

    for register in map.individual_registers() {
        generate_register_header(out, register)?;
        generate_register_defines(out, map, register)?;
        generate_layout_defines(out, map, &register.layout)?;
    }

    for block in map.register_blocks.values() {
        generate_register_block_header(out, block)?;
        generate_register_block_defines(out, map, block)?;
        for member in block.members.values() {
            generate_register_block_member_header(out, member)?;
            generate_register_block_member_defines(out, map, block, member)?;
            generate_layout_defines(out, map, &member.layout)?;
        }
    }

    generate_footer(out, output_file, opts)?;
    Ok(())
}

fn generate_header(
    out: &mut dyn Write,
    map: &RegisterMap,
    output_file: &Path,
    opts: &GeneratorOpts,
) -> Result<(), Error> {
    if opts.clang_format_guard {
        writeln!(out, "// clang-format off")?;
    }

    writeln!(out, "/**")?;
    writeln!(out, " * @file {}", output_file.to_string_lossy())?;
    writeln!(out, " * @brief {}", map.name)?;
    if let Some(input_file) = &map.from_file {
        writeln!(
            out,
            " * @note do not edit directly: generated using reginald from {}.",
            input_file.to_string_lossy()
        )?;
    } else {
        writeln!(out, " * @note do not edit directly: generated using reginald.",)?;
    }
    writeln!(out, " *")?;
    writeln!(out, " * Generator: c.macromap")?;
    if let Some(author) = &map.author {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file author: {author}")?;
    }
    if let Some(note) = &map.notice {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file notice:")?;
        for line in note.lines() {
            writeln!(out, " *   {line}")?;
        }
    }
    writeln!(out, " */")?;
    writeln!(out, "#ifndef REGINALD_{}", c_macro(&output_file.to_string_lossy()))?;
    writeln!(out, "#define REGINALD_{}", c_macro(&output_file.to_string_lossy()))?;
    writeln!(out)?;
    writeln!(out, "#include <stdint.h>")?;
    for include in &opts.add_include {
        writeln!(out, "#include \"{include}\"")?;
    }

    Ok(())
}

fn generate_register_header(out: &mut dyn Write, register: &Register) -> Result<(), Error> {
    let name = &register.name;
    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{name} Register"))?;
    if !register.docs.is_empty() {
        write!(out, "{}", register.docs.as_multiline("// "))?;
    }
    writeln!(out, "// Fields:")?;
    writeln!(out, "{}", c_layout_overview_comment(&register.layout))?;
    Ok(())
}

fn generate_register_block_header(out: &mut dyn Write, block: &RegisterBlock) -> Result<(), Error> {
    let name = &block.name;

    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{name} Register Block"))?;
    if !block.docs.is_empty() {
        write!(out, "{}", block.docs.as_multiline("// "))?;
    }

    if !block.members.is_empty() {
        writeln!(out, "//")?;
        writeln!(out, "// Contains registers:")?;
        for member in block.members.values() {
            if let Some(brief) = &member.docs.brief {
                writeln!(out, "// - [0x{:02}] {}: {}", member.offset, member.name, brief)?;
            } else {
                writeln!(out, "// - [0x{:02}] {}", member.offset, member.name)?;
            }
        }
    }

    if !block.instances.is_empty() {
        writeln!(out, "//")?;
        writeln!(out, "// Instances:")?;
        for instance in block.instances.values() {
            if let Some(brief) = &instance.docs.brief {
                writeln!(out, "// - [0x{:02}] {}: {}", instance.adr, instance.name, brief)?;
            } else {
                writeln!(out, "// - [0x{:02}] {}", instance.adr, instance.name)?;
            }
        }
    }

    Ok(())
}

fn generate_register_block_member_header(out: &mut dyn Write, member: &RegisterBlockMember) -> Result<(), Error> {
    writeln!(out)?;
    writeln!(out, "// ==== {} Block Register ==== ", member.name)?;

    if !member.docs.is_empty() {
        write!(out, "{}", member.docs.as_multiline("// "))?;
    }
    writeln!(out, "// Fields:")?;
    writeln!(out, "{}", c_layout_overview_comment(&member.layout))?;

    Ok(())
}

fn generate_register_block_member_defines(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    member: &RegisterBlockMember,
) -> Result<(), Error> {
    if !block.instances.is_empty() {
        for block_instance in block.instances.values() {
            let member_instance = &block_instance.registers[&member.name];
            generate_register_defines(out, map, member_instance)?;
        }
    }
    Ok(())
}

fn generate_register_block_defines(out: &mut dyn Write, map: &RegisterMap, block: &RegisterBlock) -> Result<(), Error> {
    let macro_prefix = c_macro(&map.name);
    let macro_block_name = c_macro(&block.name.clone());

    if !block.members.is_empty() {
        let mut defines = vec![];
        for member in block.members.values() {
            let macro_member_name = c_macro(&member.name);
            defines.push(vec![
                format!("#define {}_{}_OFFSET", macro_prefix, macro_member_name),
                format!("(0x{:X}U)", member.offset),
                format!("//!< Offset of {} register from {} block start", member.name, block.name),
            ]);
        }
        writeln!(out)?;
        writeln!(out, "// Contained registers:")?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    if !block.instances.is_empty() {
        let mut defines = vec![];
        for instance in block.instances.values() {
            let macro_instance_name = c_macro(&instance.name);
            defines.push(vec![
                format!("#define {}_{}_INSTANCE_{}", macro_prefix, macro_block_name, macro_instance_name),
                format!("(0x{:X}U)", instance.adr),
                format!("//!< Start of {} instance {}", block.name, instance.name),
            ]);
        }
        writeln!(out)?;
        writeln!(out, "// Instances:")?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}

fn generate_register_defines(out: &mut dyn Write, map: &RegisterMap, register: &Register) -> Result<(), Error> {
    let mut defines: Vec<Vec<String>> = vec![];

    let macro_prefix = c_macro(&map.name);
    let reg_macro_prefix = format!("{macro_prefix}_{}", c_macro(&register.name));

    // Address:
    defines.push(vec![
        format!("#define {}_ADDRESS", reg_macro_prefix),
        format!("(0x{:X}U)", register.adr),
        format!("//!< {} register address", register.name),
    ]);

    // Reset value:
    if let Some(reset_val) = &register.reset_val {
        defines.push(vec![
            format!("#define {}_RESET", reg_macro_prefix),
            format!("(0x{:X}U)", reset_val),
            format!("//!< {} register reset value", register.name),
        ]);
    }

    writeln!(out)?;
    write!(out, "{}", str_table(&defines, "", " "))?;

    Ok(())
}

fn generate_layout_defines(out: &mut dyn Write, map: &RegisterMap, layout: &Layout) -> Result<(), Error> {
    let macro_prefix = c_macro(&map.name);
    let layout_macro_prefix = format!("{macro_prefix}_{}", c_macro(&layout.name));

    if layout.contains_fixed_bits() {
        let mut defines: Vec<Vec<String>> = vec![];
        defines.push(vec![
            format!("#define {}_ALWAYSWRITE_MASK", layout_macro_prefix),
            format!("(0x{:X}U)", layout.fixed_bits_mask()),
            format!("//!< {} register always write mask", layout.name),
        ]);
        defines.push(vec![
            format!("#define {}_ALWAYSWRITE_VALUE", layout_macro_prefix),
            format!("(0x{:X}U)", layout.fixed_bits_val()),
            format!("//!< {} register always write value", layout.name),
        ]);

        writeln!(out)?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    let mut defines: Vec<Vec<String>> = vec![];

    // Register fields & enums:
    for field in layout.nested_fields_with_content() {
        let name_macro = c_macro(&field.name.join("_"));
        let name_comment = c_macro(&field.name.join("_"));

        if !defines.is_empty() {
            defines.push(vec![]);
        }

        defines.push(vec![
            format!("#define {}_{}_MASK", layout_macro_prefix, name_macro),
            format!("(0x{:X}U)", field.mask),
            format!("//!< {}.{}: bit mask (shifted)", layout.name, name_comment),
        ]);
        defines.push(vec![
            format!("#define {}_{}_SHIFT", layout_macro_prefix, name_macro),
            format!("({}U)", lsb_pos(field.mask)),
            format!("//!< {}.{}: bit shift", layout.name, name_comment),
        ]);

        if let FieldType::Enum(e) = &field.field.accepts {
            for entry in e.entries.values() {
                defines.push(vec![
                    format!("#define {}_{}_VAL_{}", layout_macro_prefix, name_macro, c_macro(&entry.name)),
                    format!("(0x{:X}U)", entry.value),
                    format!("//!< {}.{}: Value {}", layout_macro_prefix, name_comment, entry.name),
                ]);
            }
        }
    }

    writeln!(out)?;
    writeln!(out, "// Fields: ")?;
    write!(out, "{}", str_table(&defines, "", " "))?;

    Ok(())
}

fn generate_footer(out: &mut dyn Write, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    writeln!(out)?;
    writeln!(out, "#endif /* REGINALD_{} */", c_macro(&output_file.to_string_lossy()))?;

    if opts.clang_format_guard {
        writeln!(out, "// clang-format on")?;
    }

    Ok(())
}
