use std::{fmt::Write, path::Path};

use crate::{
    error::GeneratorError,
    generator_cli::GeneratorCLI,
    regmap::{
        self,
        bits::{lsb_pos, mask_width, unpositioned_mask},
        Docs, Enum, Field, FieldEnum, Register, RegisterBlock, RegisterMap, TypeValue,
    },
    utils::{filename, numbers_as_ranges, str_pad_to_length, str_pad_to_table},
};

use super::{c_code, c_macro, generate_section_header_comment, c_fitting_unsigned_type};

pub struct FuncpackGenerator {}

impl GeneratorCLI for FuncpackGenerator {
    fn generate(
        &self,
        _map: regmap::RegisterMap,
        _output_file_name: &Path,
        _args: Vec<String>,
    ) -> Result<Vec<String>, crate::error::GeneratorError> {
        todo!()
    }

    fn description(&self) -> String {
        "C header with register structs and conversion functions.".into()
    }

    fn help(&self, _args: Vec<String>) {
        todo!()
    }
}

// ====== Generator ============================================================

pub struct GeneratorOpts {
    pub field_enum_prefix: bool,
    pub registers_as_bitfields: bool,
    pub clang_format_guard: bool,
    pub generate_enums: bool,
    pub generate_registers: bool,
    pub generate_register_functions: bool,
    pub generate_generic_macros: bool,
    pub generate_validation_functions: bool,
    pub add_include: Vec<String>,
}

pub fn generate(
    out: &mut dyn Write,
    map: &RegisterMap,
    output_file: &Path,
    opts: &GeneratorOpts,
) -> Result<(), GeneratorError> {
    generate_header(out, map, output_file, opts)?;

    if opts.generate_enums && !map.shared_enums.is_empty() {
        generate_shared_enums(out, map, opts)?;
    }

    for block in map.register_blocks.values() {
        generate_register_block_defines(out, map, block)?;

        for template in block.register_templates.values() {
            if !template_has_content_to_generate(template, opts) {
                continue;
            }

            generate_register_header(out, block, template)?;

            if opts.generate_registers {
                generate_register_defines(out, map, block, template)?;
            }

            if opts.generate_enums {
                generate_register_enums(out, map, block, template, opts)?;
            }

            if !template.fields.is_empty() {
                if opts.generate_registers {
                    generate_register_struct(out, map, block, template, opts)?;
                }

                if opts.generate_register_functions {
                    generate_register_functions(out, map, block, template, opts)?;
                }
            }
        }
    }

    if opts.generate_generic_macros {
        generate_generic_macros(out, map)?;
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
    writeln!(out, " * Generator: c.funcpack")?;
    writeln!(out, " */")?;
    writeln!(out, "#ifndef REGINALD_{}", c_macro(&filename(output_file)?))?;
    writeln!(out, "#define REGINALD_{}", c_macro(&filename(output_file)?))?;
    writeln!(out)?;
    writeln!(out, "#include <stdint.h>")?;
    if opts.generate_validation_functions {
        writeln!(out, "#include <stdbool.h>")?;
    }
    for include in &opts.add_include {
        writeln!(out, "#include \"{include}\"")?;
    }

    Ok(())
}

fn generate_shared_enums(out: &mut dyn Write, map: &RegisterMap, opts: &GeneratorOpts) -> Result<(), GeneratorError> {
    writeln!(out)?;
    generate_section_header_comment(out, "Shared Enums")?;

    for shared_enum in map.shared_enums.values() {
        generate_enum(out, map, shared_enum, &c_code(&shared_enum.name), opts)?;
    }

    Ok(())
}

fn generate_enum(
    out: &mut dyn Write,
    map: &RegisterMap,
    e: &Enum,
    name: &str,
    opts: &GeneratorOpts,
) -> Result<(), GeneratorError> {
    let code_prefix = c_code(&map.map_name);
    let macro_prefix = c_macro(&map.map_name);

    writeln!(out)?;
    generate_doxy_comment(out, &e.docs, "", None)?;
    writeln!(out, "enum {}_{} {{", code_prefix, name)?;
    for entry in e.entries.values() {
        generate_doxy_comment(out, &entry.docs, "  ", None)?;
        writeln!(out, "  {}_{}_{} = 0x{:X}U,", macro_prefix, c_macro(name), c_macro(&entry.name), entry.value)?;
    }
    writeln!(out, "}};")?;

    if opts.generate_validation_functions {
        let code_prefix = c_code(&map.map_name);
        let uint_type = c_fitting_unsigned_type(map.max_register_width())?;
        let accept_values: Vec<TypeValue> = e.entries.values().map(|x| x.value).collect();
        let accept_ranges = numbers_as_ranges(accept_values);

        writeln!(out,)?;
        let docs = Docs {
            brief: Some(format!(
                "Validate that a given value can be represented as an @ref enum {code_prefix}_{name}."
            )),
            doc: None,
        };
        generate_doxy_comment(out, &docs, "", None)?;
        writeln!(out, "static inline bool {code_prefix}_can_unpack_enum_{name}({uint_type} val) {{")?;
        for range in accept_ranges {
            if range.start() == range.end() {
                writeln!(out, "  if (val == 0x{:X}U) return true;", range.start())?;
            } else {
                writeln!(out, "  if (0x{:X}U <= val && val <= 0x{:X}U) return true;", range.start(), range.end())?;
            }
        }
        writeln!(out, "  return false;")?;
        writeln!(out, "}}")?;
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

    Ok(())
}

fn generate_register_defines(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), GeneratorError> {
    let mut defines: Vec<Vec<String>> = vec![];

    let macro_reg_template = c_macro(&(block.name.to_owned() + &template.name));
    let macro_prefix = c_macro(&map.map_name);
    let generic_template_name = block.name.to_owned() + &template.name;

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

    if let Some(reset_val) = &template.reset_val {
        defines.push(vec![
            format!("#define {}_{}_RESET", macro_prefix, macro_reg_template),
            format!("(0x{:X}U)", reset_val),
            format!("//!< {} register reset value", generic_template_name),
        ])
    }

    if let Some(always_write) = &template.always_write {
        defines.push(vec![
            format!("#define {}_{}_ALWAYSWRITE_MASK", macro_prefix, macro_reg_template),
            format!("(0x{:X}U)", always_write.mask),
            format!("//!< {} register always write mask", generic_template_name),
        ]);
        defines.push(vec![
            format!("#define {}_{}_ALWAYSWRITE_VALUE", macro_prefix, macro_reg_template),
            format!("(0x{:X}U)", always_write.value),
            format!("//!< {} register always write value", generic_template_name),
        ]);
    }

    write!(out, "{}", str_pad_to_table(&defines, "", " "))?;

    Ok(())
}

fn generate_register_enums(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    opts: &GeneratorOpts,
) -> Result<(), GeneratorError> {
    for field in template.fields.values() {
        if let Some(FieldEnum::Local(local_enum)) = &field.field_enum {
            let enum_name = name_register_enum(block, template, local_enum, opts);
            generate_enum(out, map, local_enum, &enum_name, opts)?;
        }
    }

    Ok(())
}

fn generate_register_struct(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    opts: &GeneratorOpts,
) -> Result<(), GeneratorError> {
    let struct_name = name_register_struct(map, block, template);

    writeln!(out)?;
    generate_doxy_comment(
        out,
        &template.docs,
        "",
        Some("use pack/unpack/overwrite functions for conversion to/from packed register value"),
    )?;
    writeln!(out, "struct {struct_name} {{")?;
    for field in template.fields.values() {
        let field_type = register_struct_member_type(map, block, template, field, opts)?;
        let field_name = c_code(&field.name);
        generate_doxy_comment(out, &field.docs, "  ", None)?;
        if opts.registers_as_bitfields {
            writeln!(out, "  {field_type} {field_name} : {};", mask_width(field.mask))?;
        } else {
            writeln!(out, "  {field_type} {field_name};",)?;
        }
    }
    writeln!(out, "}};",)?;
    Ok(())
}

fn generate_register_functions(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    opts: &GeneratorOpts,
) -> Result<(), GeneratorError> {
    let regname = c_code(&(block.name.to_owned() + &template.name));
    let struct_name = name_register_struct(map, block, template);
    let packed_type = c_fitting_unsigned_type(template.bitwidth)?;
    let macro_reg_template = c_macro(&(block.name.to_owned() + &template.name));
    let macro_prefix = c_macro(&map.map_name);
    let code_prefix = c_code(&map.map_name);

    if opts.generate_validation_functions {
        writeln!(out)?;
        let docs = Docs {
            brief: Some(format!("Validate that a given value can be unpacked to as a @ref struct {struct_name}.")),
            doc: Some("Verifies that all enum fields can represent the given value.".to_string()),
        };
        generate_doxy_comment(out, &docs, "", None)?;
        writeln!(out, "static inline bool {code_prefix}_can_unpack_{regname}({packed_type} val) {{")?;
        for field in template.fields.values() {
            if let Some(field_enum) = &field.field_enum {
                let unpos_mask = unpositioned_mask(field.mask);
                let shift = lsb_pos(field.mask);
                let field_value = format!("(val >> {shift}) & 0x{unpos_mask:X}U");
                let name = match field_enum {
                    FieldEnum::Local(local_enum) => name_register_enum(block, template, local_enum, opts),
                    FieldEnum::Shared(shared_enum) => c_code(&shared_enum.name),
                };
                let enum_validate_func = format!("{code_prefix}_can_unpack_enum_{name}");
                writeln!(out, "  if (!{enum_validate_func}({field_value})) return false;")?;
            }
        }
        writeln!(out, "  return true;")?;
        writeln!(out, "}}")?;
    }

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: Some(
            "All bits that are not part of a field or specified as 'always write' are kept as in 'val'.".to_string(),
        ),
    };
    generate_doxy_comment(out, &docs, "", None)?;
    writeln!(
        out,
        "static inline {packed_type} {struct_name}_overwrite(const struct {struct_name} *r, {packed_type} val) {{"
    )?;
    if template.always_write.is_some() {
        writeln!(out, "  val &= ~({packed_type}){macro_prefix}_{macro_reg_template}_ALWAYSWRITE_MASK;")?;
        writeln!(out, "  val |= {macro_prefix}_{macro_reg_template}_ALWAYSWRITE_VALUE;")?;
    }
    for field in template.fields.values() {
        let field_name = c_code(&field.name);
        let mask = field.mask;
        let unpos_mask = unpositioned_mask(mask);
        let shift = lsb_pos(mask);
        writeln!(out, "  val &= ~({packed_type})0x{mask:X}U;")?;
        writeln!(out, "  val |= ((({packed_type})r->{field_name}) & 0x{unpos_mask:X}U) << ({packed_type}){shift}U;")?;
    }
    writeln!(out, "  return val;",)?;
    writeln!(out, "}}",)?;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, &docs, "", None)?;
    writeln!(out, "static inline {packed_type} {struct_name}_pack(const struct {struct_name} *r) {{")?;
    writeln!(out, "  return {struct_name}_overwrite(r, 0);")?;
    writeln!(out, "}}",)?;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed register value to register struct initialization".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, &docs, "", None)?;

    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {}_UNPACK(_VAL_) {{", c_macro(&struct_name)));
    for field in template.fields.values() {
        let mask = unpositioned_mask(field.mask);
        let shift = lsb_pos(field.mask);
        let field_name = c_code(&field.name);
        macro_lines.push(format!("  .{field_name} = ((_VAL_) >> {shift}U) & 0x{mask:X}U,"));
    }
    macro_lines.push("}}".to_string());
    generate_multiline_macro(out, macro_lines)?;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed register value into a register struct.".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, &docs, "", None)?;
    writeln!(out, "static inline void {struct_name}_unpack_into({packed_type} val,  struct {struct_name} *r) {{")?;
    for field in template.fields.values() {
        let field_name = c_code(&field.name);
        let mask = field.mask;
        let unpos_mask = unpositioned_mask(mask);
        let shift = lsb_pos(mask);
        let field_type = register_struct_member_type(map, block, template, field, opts)?;
        if field.field_enum.is_some() {
            writeln!(out, "  r->{field_name} = ({field_type})((val >> {shift}U) & 0x{unpos_mask:X}U);")?;
        } else {
            writeln!(out, "  r->{field_name} = (val >> {shift}U) & 0x{unpos_mask:X}U;")?;
        }
    }
    writeln!(out, "}}",)?;

    if opts.generate_validation_functions {
        writeln!(out)?;
        let docs = Docs {
            brief: Some("Convert packed register value into a register struct.".to_string()),
            doc: Some(
                "This function verifies if the given value can be unpacked into the struct.\n".to_string()
                    + "@returns 0 if the register was succesfully unpacked, 1 otherwise.",
            ),
        };
        generate_doxy_comment(out, &docs, "", None)?;
        writeln!(
            out,
            "static inline int {struct_name}_try_unpack_into({packed_type} val,  struct {struct_name} *r) {{"
        )?;
        writeln!(out, "  if(!{code_prefix}_can_unpack_{regname}(val)) return 1;",)?;
        writeln!(out, "  {struct_name}_unpack_into(val, r);",)?;
        writeln!(out, "  return 0;",)?;
        writeln!(out, "}}",)?;
    }

    Ok(())
}

fn generate_generic_macros(out: &mut dyn Write, map: &RegisterMap) -> Result<(), GeneratorError> {
    let macro_prefix = c_macro(&map.map_name);

    let mut entry_count = 0;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: Some(
            "All bits that are not part of a field or specified as 'always write' are kept as in 'val'.".to_string(),
        ),
    };
    generate_doxy_comment(out, &docs, "", None)?;
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_OVERWRITE(_struct_ptr, _val_) _Generic((_struct_ptr),"));
    for block in map.register_blocks.values() {
        for template in block.register_templates.values() {
            let struct_name = name_register_struct(map, block, template);
            if template.fields.is_empty() {
                continue;
            }
            entry_count += 1;
            macro_lines.push(format!("    struct {struct_name}* : {struct_name}_overwrite,"));
        }
    }
    if entry_count == 0 {
        return Ok(());
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push("  )(_struct_ptr_, _val_)".into());
    generate_multiline_macro(out, macro_lines)?;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, &docs, "", None)?;
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_PACK(_struct_ptr) _Generic((_struct_ptr),"));
    for block in map.register_blocks.values() {
        for template in block.register_templates.values() {
            let struct_name = name_register_struct(map, block, template);
            if template.fields.is_empty() {
                continue;
            }
            macro_lines.push(format!("    struct {struct_name}* : {struct_name}_pack,"));
        }
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push("  )(_struct_ptr_)".into());
    generate_multiline_macro(out, macro_lines)?;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed register value to register struct.".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, &docs, "", None)?;
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_UNPACK_INTO(_val_, _struct_ptr) _Generic((_struct_ptr),"));
    for block in map.register_blocks.values() {
        for template in block.register_templates.values() {
            let struct_name = name_register_struct(map, block, template);
            if template.fields.is_empty() {
                continue;
            }
            macro_lines.push(format!("    struct {struct_name}* : {struct_name}_unpack_into,"));
        }
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push("  )(_val_, _struct_ptr_)".into());
    generate_multiline_macro(out, macro_lines)?;

    writeln!(out)?;
    let docs = Docs {
        brief: Some("Attempt to convert packed register value to register struct.".to_string()),
        doc: Some(
            "This function verifies if the given value can be unpacked into the struct.\n".to_string()
                + "@returns 0 if the register was succesfully unpacked, 1 otherwise.",
        ),
    };
    generate_doxy_comment(out, &docs, "", None)?;
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_TRY_UNPACK_INTO(_val_, _struct_ptr) _Generic((_struct_ptr),"));
    for block in map.register_blocks.values() {
        for template in block.register_templates.values() {
            let struct_name = name_register_struct(map, block, template);
            if template.fields.is_empty() {
                continue;
            }
            macro_lines.push(format!("    struct {struct_name}* : {struct_name}_try_unpack_into,"));
        }
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push("  )(_val_, _struct_ptr_)".into());
    generate_multiline_macro(out, macro_lines)?;

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

fn generate_doxy_comment(
    out: &mut dyn Write,
    docs: &Docs,
    prefix: &str,
    note: Option<&str>,
) -> Result<(), GeneratorError> {
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

fn generate_multiline_macro(out: &mut dyn Write, mut lines: Vec<String>) -> Result<(), GeneratorError> {
    if lines.is_empty() {
        Ok(())
    } else {
        let last_line = lines.pop().unwrap();
        for line in lines {
            writeln!(out, "{}\\", str_pad_to_length(&line, ' ', 99))?;
        }
        writeln!(out, "{last_line}")?;
        Ok(())
    }
}

// ====== Generator Utils ======================================================

fn name_register_enum(block: &RegisterBlock, template: &Register, field_enum: &Enum, opts: &GeneratorOpts) -> String {
    let regname = c_code(&(block.name.to_owned() + &template.name));
    let enumname = c_code(&field_enum.name);
    if opts.field_enum_prefix {
        format!("{regname}_{enumname}")
    } else {
        enumname.to_string()
    }
}

fn name_register_struct(map: &RegisterMap, block: &RegisterBlock, template: &Register) -> String {
    let mapname = c_code(&map.map_name);
    let regname = c_code(&(block.name.to_owned() + &template.name));
    format!("{mapname}_{regname}")
}

fn register_struct_member_type(
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    field: &Field,
    opts: &GeneratorOpts,
) -> Result<String, GeneratorError> {
    let code_prefix = c_code(&map.map_name);
    match &field.field_enum {
        Some(FieldEnum::Local(local_enum)) => {
            let name = name_register_enum(block, template, local_enum, opts);
            Ok(format!("{code_prefix}_{name}"))
        }
        Some(FieldEnum::Shared(shared_enum)) => {
            let name = c_code(&shared_enum.name);
            Ok(format!("{code_prefix}_{name}"))
        }
        None => c_fitting_unsigned_type(mask_width(field.mask)),
    }
}

fn template_has_content_to_generate(template: &Register, opts: &GeneratorOpts) -> bool {
    if opts.generate_registers {
        // Generate section for every register if 'generate_registers' is set.
        return true;
    }

    if opts.generate_enums {
        for field in template.fields.values() {
            if matches!(field.field_enum, Some(FieldEnum::Local(_))) {
                return true;
            }
        }
    }

    if opts.generate_register_functions && !template.fields.is_empty() {
        // Register requires functions
        return true;
    }

    false
}