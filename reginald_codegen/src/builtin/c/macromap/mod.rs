use std::{fmt::Write, path::Path};

use crate::{
    error::GeneratorError,
    regmap::bits::lsb_pos,
    regmap::{Docs, FieldEnum, Register, RegisterBlock, RegisterMap},
    utils::{filename, str_pad_to_table},
};

use super::{c_macro, generate_section_header_comment};

pub struct GeneratorOpts {
    pub clang_format_guard: bool,
    pub add_include: Vec<String>,
}

pub fn generate(
    out: &mut dyn Write,
    map: &RegisterMap,
    output_file: &Path,
    opts: &GeneratorOpts,
) -> Result<(), GeneratorError> {
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
) -> Result<(), GeneratorError> {
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

fn generate_register_block_defines(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
) -> Result<(), GeneratorError> {
    let mut defines = vec![];

    if block.instances.len() > 1 && block.register_templates.len() > 1 {
        let macro_prefix = c_macro(&map.map_name);
        let macro_block_name = c_macro(&block.name.to_owned());

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
                let template_name = block.name.to_owned() + &template.name;
                let macro_template_name = c_macro(&template_name);
                defines.push(vec![
                    format!("#define {}_{}_OFFSET", macro_prefix, macro_template_name),
                    format!("(0x{:X}U)", template_offset),
                    format!("//!< Offset of {} register from {} block start", template_name, block.name),
                ])
            }
        }
    }

    if !defines.is_empty() {
        writeln!(out,)?;
        generate_section_header_comment(out, &format!("{} Register Block", block.name))?;
        if !block.docs.is_empty() {
            write!(out, "{}", block.docs.as_multiline("// "))?;
        }
        write!(out, "{}", str_pad_to_table(&defines, "", " "))?;
    }

    Ok(())
}

fn generate_register_header(
    out: &mut dyn Write,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), GeneratorError> {
    let generic_template_name = block.name.to_owned() + &template.name;

    // Register section header:
    writeln!(out)?;
    generate_section_header_comment(out, &format!("{} Register", generic_template_name))?;
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
                    writeln!(out, "//      {}", line)?;
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
) -> Result<(), GeneratorError> {
    let mut defines: Vec<Vec<String>> = vec![];

    let macro_prefix = c_macro(&map.map_name);
    let template_macro_prefix = format!("{macro_prefix}_{}", c_macro(&(block.name.to_owned() + &template.name)));
    let generic_template_name = block.name.to_owned() + &template.name;

    // Absolute address of all instances:
    if let Some(template_offset) = template.adr {
        for instance in block.instances.values() {
            let instance_name = instance.name.to_owned() + &template.name;
            if let Some(instance_adr) = &instance.adr {
                defines.push(vec![
                    format!("#define {}_{}", macro_prefix, c_macro(&instance_name)),
                    format!("(0x{:X}U)", template_offset + instance_adr),
                    format!("//!< {} register address", instance_name),
                ])
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
        ])
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

        if let Some(e) = &field.field_enum {
            let enum_entries = match e {
                FieldEnum::Local(field_enum) => field_enum.entries.values(),
                FieldEnum::Shared(shared_enun) => shared_enun.entries.values(),
            };
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
    write!(out, "{}", str_pad_to_table(&defines, "", " "))?;

    Ok(())
}

fn generate_footer(out: &mut dyn Write, output_file: &Path, opts: &GeneratorOpts) -> Result<(), GeneratorError> {
    writeln!(out)?;
    writeln!(out, "#endif /* REGINALD_{} */", c_macro(&filename(output_file)?))?;

    if opts.clang_format_guard {
        writeln!(out, "// clang-format on")?;
    }

    Ok(())
}