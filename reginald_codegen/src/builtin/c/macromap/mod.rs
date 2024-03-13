use std::{fmt::Write, path::Path};

#[cfg(feature = "cli")]
use clap::Parser;

use crate::{
    bits::lsb_pos,
    error::Error,
    regmap::{Register, RegisterBlock, RegisterMap},
    utils::{filename, str_table},
};

use super::{c_generate_section_header_comment, c_macro};

// ====== Generator Opts =======================================================

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Surround header with a clang-format off guard
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
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

// ====== Generator ============================================================

pub fn generate(out: &mut dyn Write, map: &RegisterMap, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    generate_header(out, map, output_file, opts)?;

    for block in map.register_blocks.values() {
        generate_register_block_defines(out, map, block)?;
        for template in block.register_templates.values() {
            generate_register_header(out, block, template)?;
            generate_register_defines(out, map, block, template)?;
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
    writeln!(out, " * @file {}", filename(output_file)?)?;
    writeln!(out, " * @brief {}", map.map_name)?;
    if let Some(input_file) = &map.from_file {
        writeln!(out, " * @note do not edit directly: generated using reginald from {}.", filename(input_file)?)?;
    } else {
        writeln!(out, " * @note do not edit directly: generated using reginald.",)?;
    }
    writeln!(out, " *")?;
    writeln!(out, " * Generator: c.macromap")?;
    if let Some(author) = &map.author {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file author: {author}")?;
    }
    if let Some(note) = &map.note {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file note:")?;
        for line in note.lines() {
            writeln!(out, " *   {line}")?;
        }
    }
    writeln!(out, " */")?;
    writeln!(out, "#ifndef REGINALD_{}", c_macro(&filename(output_file)?))?;
    writeln!(out, "#define REGINALD_{}", c_macro(&filename(output_file)?))?;
    writeln!(out)?;
    writeln!(out, "#include <stdint.h>")?;
    for include in &opts.add_include {
        writeln!(out, "#include \"{include}\"")?;
    }

    Ok(())
}

fn generate_register_block_defines(out: &mut dyn Write, map: &RegisterMap, block: &RegisterBlock) -> Result<(), Error> {
    let mut defines = vec![];

    if block.instances.len() > 1 && block.register_templates.len() > 1 {
        let macro_prefix = c_macro(&map.map_name);
        let macro_block_name = c_macro(&block.name.clone());

        for instance in block.instances.values() {
            if let Some(adr) = &instance.adr {
                let macro_instance_name = c_macro(&instance.name);
                defines.push(vec![
                    format!("#define {}_{}_INSTANCE_{}", macro_prefix, macro_block_name, macro_instance_name),
                    format!("(0x{:X}U)", adr),
                    format!("//!< Start of {} instance {}", block.name, instance.name),
                ]);
            }
        }

        for template in block.register_templates.values() {
            if let Some(template_offset) = template.adr {
                let template_name = template.name_in_block(block);
                let macro_template_name = c_macro(&template_name);
                defines.push(vec![
                    format!("#define {}_{}_OFFSET", macro_prefix, macro_template_name),
                    format!("(0x{:X}U)", template_offset),
                    format!("//!< Offset of {} register from {} block start", template_name, block.name),
                ]);
            }
        }
    }

    if !defines.is_empty() {
        writeln!(out,)?;
        c_generate_section_header_comment(out, &format!("{} Register Block", block.name))?;
        if !block.docs.is_empty() {
            write!(out, "{}", block.docs.as_multiline("// "))?;
        }
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}

fn generate_register_header(out: &mut dyn Write, block: &RegisterBlock, template: &Register) -> Result<(), Error> {
    let generic_template_name = template.name_in_block(block);

    // Register section header:
    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{generic_template_name} Register"))?;
    if !template.docs.is_empty() {
        write!(out, "{}", template.docs.as_multiline("// "))?;
    }
    if !template.fields.is_empty() {
        writeln!(out, "// Fields:")?;

        for field in template.fields.values() {
            if let Some(brief) = &field.docs.brief {
                writeln!(out, "//  - {}: {}", field.name, brief)?;
            } else {
                writeln!(out, "//  - {}", field.name)?;
            }
            if let Some(doc) = &field.docs.doc {
                for line in doc.lines() {
                    writeln!(out, "//      {line}")?;
                }
            }
        }
    }

    Ok(())
}

fn generate_register_defines(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    let mut defines: Vec<Vec<String>> = vec![];

    let macro_prefix = c_macro(&map.map_name);
    let generic_template_name = template.name_in_block(block);
    let template_macro_prefix = format!("{macro_prefix}_{}", c_macro(&generic_template_name));

    // Absolute address of all instances:
    if let Some(template_offset) = template.adr {
        for instance in block.instances.values() {
            let instance_name = template.name_in_instance(instance);
            if let Some(instance_adr) = &instance.adr {
                defines.push(vec![
                    format!("#define {}_{}", macro_prefix, c_macro(&instance_name)),
                    format!("(0x{:X}U)", template_offset + instance_adr),
                    format!("//!< {} register address", instance_name),
                ]);
            }
        }
    }

    // Reset value:
    if let Some(reset_val) = &template.reset_val {
        defines.push(vec![]);
        defines.push(vec![
            format!("#define {}__RESET", template_macro_prefix),
            format!("(0x{:X}U)", reset_val),
            format!("//!< {} register reset value", generic_template_name),
        ]);
    }

    //  Always write mask:
    if let Some(always_write) = &template.always_write {
        defines.push(vec![]);
        defines.push(vec![
            format!("#define {}__ALWAYSWRITE_MASK", template_macro_prefix),
            format!("(0x{:X}U)", always_write.mask),
            format!("//!< {} register always write mask", generic_template_name),
        ]);
        defines.push(vec![
            format!("#define {}__ALWAYSWRITE_VALUE", template_macro_prefix),
            format!("(0x{:X}U)", always_write.value),
            format!("//!< {} register always write value", generic_template_name),
        ]);
    }

    // Register fields & enums:
    for field in template.fields.values() {
        defines.push(vec![]);
        let field_name = c_macro(&field.name);
        defines.push(vec![
            format!("#define {}_{}__MASK", template_macro_prefix, field_name),
            format!("(0x{:X}U)", field.mask),
            format!("//!< {}.{}: bit mask (shifted)", generic_template_name, field.name),
        ]);
        defines.push(vec![
            format!("#define {}_{}__SHIFT", template_macro_prefix, field_name),
            format!("({}U)", lsb_pos(field.mask)),
            format!("//!< {}.{}: bit shift", generic_template_name, field.name),
        ]);

        if let Some(enum_entries) = field.enum_entries() {
            for entry in enum_entries {
                defines.push(vec![
                    format!("#define {}_{}_{}", template_macro_prefix, field_name, c_macro(&entry.name)),
                    format!("(0x{:X}U)", entry.value),
                    format!("//!< {}.{}: {}", generic_template_name, field.name, entry.name),
                ]);
            }
        }
    }

    writeln!(out)?;
    write!(out, "{}", str_table(&defines, "", " "))?;

    Ok(())
}

fn generate_footer(out: &mut dyn Write, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    writeln!(out)?;
    writeln!(out, "#endif /* REGINALD_{} */", c_macro(&filename(output_file)?))?;

    if opts.clang_format_guard {
        writeln!(out, "// clang-format on")?;
    }

    Ok(())
}
