use std::{fmt::Write, path::Path};

#[cfg(feature = "cli")]
use clap::Parser;

use handlebars::{
    handlebars_helper, no_escape, Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output,
    RenderContext, RenderError, Renderable,
};
use reginald_utils::str_table;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use crate::{ bits::lsb_pos,
    builtin::c::c_section_header_comment,
    error::Error,
    regmap::{
        EnumEntry, FieldType, Layout, LayoutField, Register, RegisterBlock, RegisterBlockMember, RegisterMap, TypeValue,
    },
};

use super::{c_generate_section_header_comment, c_layout_overview_comment, c_macro};

// ====== Generator Opts =======================================================

#[derive(Debug, Serialize)]
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

#[derive(Serialize, Debug)]
struct TemplateInput<'a> {
    map: &'a RegisterMap,
    output: String,
    output_file: String,
    opts: &'a GeneratorOpts,
}

// Define a struct for your block helper
struct PrefixLinesWithHelper;

// Implement the HelperDef trait for your block helper
impl HelperDef for PrefixLinesWithHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        // Get the parameter from the helper (if any)
        let prefix = h.param(0).map(|v| v.value().render()).unwrap_or_else(|| "".to_string());

        // Render the block content
        let block_content = h.template().unwrap().renders(r, ctx, rc).unwrap();

        // You can process the block content here and output it as needed
        for line in block_content.lines() {
            out.write(&prefix)?;
            out.write(line)?;
            out.write("\n")?;
        }

        Ok(())
    }
}

// Define a struct for your block helper
struct AlignLines;

// Implement the HelperDef trait for your block helper
impl HelperDef for AlignLines {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'rc>,
        r: &'reg Handlebars,
        ctx: &'rc Context,
        rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let seperator = h
            .param(0)
            .map(|v| v.value().render())
            .unwrap_or_else(|| " ".to_string());
        let max_cols = h.param(1).map(|v| v.value().render());
        let max_cols: Option<usize> = match max_cols {
            None => None,
            Some(n) => Some(usize::from_str_radix(&n, 10).map_err(|_x| todo!()).unwrap()),
        };
        let swallow_sep = h
            .param(2)
            .map(|v| v.value().render().to_lowercase())
            .unwrap_or(String::from("true"));
        let swallow_sep: bool = match swallow_sep.as_str() {
            "true" => true,
            "false" => false,
            _ => todo!(),
        };

        let emit_sep = if swallow_sep { String::new() } else { seperator.clone() };

        let mut rows: Vec<Vec<String>> = vec![];
        let block_content = h.template().unwrap().renders(r, ctx, rc).unwrap();
        for line in block_content.lines() {
            if let Some(max_cols) = max_cols {
                rows.push(line.splitn(max_cols, &seperator).map(|x| x.to_string()).collect())
            } else {
                rows.push(line.split(&seperator).map(|x| x.to_string()).collect())
            }
        }

        out.write(&str_table(&rows, "", &emit_sep))?;

        Ok(())
    }
}

pub fn generate(out: &mut dyn Write, map: &RegisterMap, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    // generate_header(out, map, output_file, opts)?;
    //
    // for register in map.individual_registers() {
    //     generate_register_header(out, register)?;
    //     generate_register_defines(out, map, register)?;
    //     generate_layout_defines(out, map, &register.layout)?;
    // }
    //
    // for block in map.register_blocks.values() {
    //     generate_register_block_header(out, block)?;
    //     generate_register_block_defines(out, map, block)?;
    //     for member in block.members.values() {
    //         generate_register_block_member_header(out, member)?;
    //         generate_register_block_member_defines(out, map, block, member)?;
    //         generate_layout_defines(out, map, &member.layout)?;
    //     }
    // }
    //
    // generate_footer(out, output_file, opts)?;
    // Ok(())

    // FIXME: Move to io::Write
    let mut template = handlebars::Handlebars::new();
    template.register_template_string("macromap", include_str!("./macromap_template.h"))?;
    template.register_escape_fn(no_escape);

    template.register_helper("prefix_lines_with", Box::new(PrefixLinesWithHelper));

    template.register_helper("align_lines", Box::new(AlignLines));

    handlebars_helper!(field_enum_entries: |field: LayoutField| {
        if let FieldType::Enum(e) = field.accepts {
            let entries: Vec<EnumEntry> = e.entries.values().map(|x| x.clone()).collect();
            serde_json::to_value(entries).unwrap()
        } else {
            Value::Array(vec![])
        }
    });
    template.register_helper("field_enum_entries", Box::new(field_enum_entries));

    handlebars_helper!(is_null: |name: Value| name.is_null());
    template.register_helper("is_null", Box::new(is_null));

    handlebars_helper!(c_section_header_comment_helper: |name: String| c_section_header_comment(&name));
    template.register_helper("c_section_header_comment", Box::new(c_section_header_comment_helper));

    handlebars_helper!(c_macro_helper: |name: String| c_macro(&name));
    template.register_helper("c_macro", Box::new(c_macro_helper));

    handlebars_helper!(mask_lsb_pos: |mask: TypeValue| lsb_pos(mask) );
    template.register_helper("mask_lsb_pos", Box::new(mask_lsb_pos));

    handlebars_helper!(concat: |*args| args.iter().map(|a| a.render()).collect::<Vec<String>>().join("")
    );
    template.register_helper("concat", Box::new(concat));

    handlebars_helper!(join: |pieces: Vec<String>, sep: String|  {
        pieces.join(sep.as_str())
    }
    );
    template.register_helper("join", Box::new(join));

    handlebars_helper!(layout_contains_fixed_bits: |layout: Layout|
        layout.contains_fixed_bits()
    );
    template.register_helper("layout_contains_fixed_bits", Box::new(layout_contains_fixed_bits));

    handlebars_helper!(layout_fixed_bits_val: |layout: Layout|
        layout.fixed_bits_val()
    );
    template.register_helper("layout_fixed_bits_val", Box::new(layout_fixed_bits_val));

    handlebars_helper!(layout_fixed_bits_mask: |layout: Layout|
        layout.fixed_bits_mask()
    );
    template.register_helper("layout_fixed_bits_mask", Box::new(layout_fixed_bits_mask));

    handlebars_helper!(layout_nested_fields_with_content: |layout: Layout| {
        let a = layout.nested_fields_with_content();
        serde_json::to_value(a).unwrap()
    }
    );
    template.register_helper("layout_nested_fields_with_content", Box::new(layout_nested_fields_with_content));

    handlebars_helper!(hex: |val: u64, {digits:usize=1}| {
        let s = format!("{val:X}");
        if s.len() < digits {
            let mut result = s.clone();
            for _ in 0..=(digits - s.len()) {
                result.insert(0, '0');
            }
            result
        } else {
            s
        }
    }
    );
    template.register_helper("hex", Box::new(hex));

    let output_file_name = output_file
        .file_name()
        .ok_or(Error::GeneratorError("Failed to extract file name from output path!".to_string()))?
        .to_string_lossy();
    let inputs = TemplateInput {
        map,
        output: output_file.to_string_lossy().into(),
        output_file: output_file_name.into(),
        opts,
    };
    let s = template.render("macromap", &inputs)?;
    out.write_str(&s)?;
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

    let output_file_name = output_file
        .file_name()
        .ok_or(Error::GeneratorError("Failed to extract file name from output path!".to_string()))?
        .to_string_lossy();

    writeln!(out, "/**")?;
    writeln!(out, " * @file {output_file_name}")?;
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
    writeln!(out, "#ifndef REGINALD_{}", c_macro(&output_file_name))?;
    writeln!(out, "#define REGINALD_{}", c_macro(&output_file_name))?;
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

    let output_file_name = output_file
        .file_name()
        .ok_or(Error::GeneratorError("Failed to extract file name from output path!".to_string()))?
        .to_string_lossy();

    writeln!(out, "#endif /* REGINALD_{} */", c_macro(&output_file_name))?;

    if opts.clang_format_guard {
        writeln!(out, "// clang-format on")?;
    }

    Ok(())
}
