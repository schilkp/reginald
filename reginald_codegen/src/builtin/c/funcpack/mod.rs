use std::{fmt::Write, path::Path};

#[cfg(feature = "cli")]
use clap::Parser;

use crate::{
    bits::{lsb_pos, mask_width, unpositioned_mask},
    error::Error,
    regmap::{Docs, Enum, Field, FieldType, Register, RegisterBlock, RegisterMap, TypeValue},
    utils::{filename, numbers_as_ranges, str_pad_to_length, str_table},
};

use super::{c_code, c_fitting_unsigned_type, c_generate_doxy_comment, c_generate_section_header_comment, c_macro};

// ====== Generator Opts =======================================================

#[derive(Debug)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Prefix the name of a local field enum with the name of the containing
    /// register
    ///
    /// This avoids naming conflicts, but is often not necessary and
    /// leads to much longer enum type names.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "false"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub field_enum_prefix: bool,

    /// Make register structs bitfields to reduce their memory size
    ///
    /// Note that their memory layout will not match the actual register
    /// and the (un)packing functions must still be used.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub registers_as_bitfields: bool,

    /// Surround header with a clang-format off guard
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub clang_format_guard: bool,

    /// Generate field/shared enums
    ///
    /// Note that enums are still used in register structs/functions
    /// if excluded. They must then be generated in a seperate header file.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_enums: bool,

    /// Generate register structs and property defines
    ///
    /// Note that the structs are still used in register (un)packing functions.
    /// They must then be generated in a seperate header file.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_registers: bool,

    /// Generate register packing and unpacking functions
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_register_functions: bool,

    /// Generate generic register packing and unpacking macros
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_generic_macros: bool,

    /// Generate enum and struct unpacking validation functions
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub generate_validation_functions: bool,

    /// Header file that should be included at the top of the generated header
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub add_include: Vec<String>,
}

// ====== Generator ============================================================

/// Generate complete file.
pub fn generate(out: &mut dyn Write, map: &RegisterMap, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
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

/// Generate header comment, include guard, includes.
fn generate_header(
    out: &mut dyn Write,
    map: &RegisterMap,
    output_file: &Path,
    opts: &GeneratorOpts,
) -> Result<(), Error> {
    // Clang format guard, if enabled:
    if opts.clang_format_guard {
        writeln!(out, "// clang-format off")?;
    }

    // Doxy file comment:
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

    // Map docs/author/note, if present:
    if !map.docs.is_empty() {
        writeln!(out, " *")?;
        write!(out, "{}", map.docs.as_multiline(" * "))?;
    }
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

    // Include guard
    writeln!(out, "#ifndef REGINALD_{}", c_macro(&filename(output_file)?))?;
    writeln!(out, "#define REGINALD_{}", c_macro(&filename(output_file)?))?;

    // Includes
    writeln!(out)?;
    writeln!(out, "#include <stdint.h>")?;
    writeln!(out, "#include <stdbool.h>")?;
    for include in &opts.add_include {
        writeln!(out, "#include \"{include}\"")?;
    }

    Ok(())
}

/// Generate 'shared enums' section:
fn generate_shared_enums(out: &mut dyn Write, map: &RegisterMap, opts: &GeneratorOpts) -> Result<(), Error> {
    writeln!(out)?;
    c_generate_section_header_comment(out, "Shared Enums")?;
    for shared_enum in map.shared_enums.values() {
        generate_enum(out, map, shared_enum, &c_code(&shared_enum.name), opts)?;
    }
    Ok(())
}

/// Generate shared or field enum, and associated functions.
fn generate_enum(
    out: &mut dyn Write,
    map: &RegisterMap,
    e: &Enum,
    name: &str,
    opts: &GeneratorOpts,
) -> Result<(), Error> {
    let code_prefix = c_code(&map.map_name);
    let macro_prefix = c_macro(&map.map_name);

    // Enum proper:
    writeln!(out)?;
    c_generate_doxy_comment(out, &e.docs, "", None)?;
    writeln!(out, "enum {code_prefix}_{name} {{")?;
    for entry in e.entries.values() {
        c_generate_doxy_comment(out, &entry.docs, "  ", None)?;
        writeln!(out, "  {}_{}_{} = 0x{:X}U,", macro_prefix, c_macro(name), c_macro(&entry.name), entry.value)?;
    }
    writeln!(out, "}};")?;

    // Validation function:
    if opts.generate_validation_functions {
        let code_prefix = c_code(&map.map_name);
        let uint_type = c_fitting_unsigned_type(map.max_register_width())?;
        let accept_values: Vec<TypeValue> = e.entries.values().map(|x| x.value).collect();
        let accept_ranges = numbers_as_ranges(accept_values);

        // Doxy comment:
        writeln!(out,)?;
        let docs = Docs {
            brief: Some(format!(
                "Validate that a given value can be represented as an @ref enum {code_prefix}_{name}."
            )),
            doc: None,
        };
        c_generate_doxy_comment(out, &docs, "", None)?;

        // Validation Function
        writeln!(out, "static inline bool {code_prefix}_can_unpack_enum_{name}({uint_type} val) {{")?;

        // Convert possible ranges to continous ranges, and generate a check for each range.
        for range in accept_ranges {
            match (range.start(), range.end()) {
                (&start, &end) if start == end => {
                    writeln!(out, "  if (val == 0x{:X}U) return true;", range.start())?;
                }
                (0, &end) => {
                    writeln!(out, "  if (val <= 0x{end:X}U) return true;")?;
                }
                (&start, &end) => {
                    writeln!(out, "  if (0x{start:X}U <= val && val <= 0x{end:X}U) return true;")?;
                }
            }
        }

        writeln!(out, "  return false;")?;
        writeln!(out, "}}")?;
    }

    Ok(())
}

/// Generate register block defines (Instance adr,  register offsets)
fn generate_register_block_defines(out: &mut dyn Write, map: &RegisterMap, block: &RegisterBlock) -> Result<(), Error> {
    // Collect all defines into a table (vector of rows), so we can genreate a nicely alligned
    // code section:
    let mut defines = vec![]; // Each row: [define, value, comment]

    let block_name = &block.name;

    // Generate register block constants (starts of instances + offsets of registers), if that information is not
    // redundant from the actual register addresses.
    if block.instances.len() > 1 && block.register_templates.len() > 1 {
        let macro_prefix = c_macro(&map.map_name);
        let macro_block_name = c_macro(&block.name.clone());

        for instance in block.instances.values() {
            if let Some(adr) = &instance.adr {
                let macro_instance_name = c_macro(&instance.name);
                defines.push(vec![
                    format!("#define {macro_prefix}_{macro_block_name}_INSTANCE_{macro_instance_name}"),
                    format!("(0x{adr:X}U)"),
                    format!("//!< Start of {block_name} instance {}", instance.name),
                ]);
            }
        }

        for template in block.register_templates.values() {
            if let Some(template_offset) = template.adr {
                let reg_name_generic = template.name_in_block(block);
                let reg_name_generic_macro = c_macro(&reg_name_generic);
                defines.push(vec![
                    format!("#define {macro_prefix}_{reg_name_generic_macro}_OFFSET"),
                    format!("(0x{template_offset:X}U)"),
                    format!("//!< Offset of {reg_name_generic} register from {block_name} block start"),
                ]);
            }
        }
    }

    // Generate defines with value/comment aligned:
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

/// Generate register section header comment
fn generate_register_header(out: &mut dyn Write, block: &RegisterBlock, template: &Register) -> Result<(), Error> {
    let reg_name_generic = template.name_in_block(block);

    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{reg_name_generic} Register"))?;
    if !template.docs.is_empty() {
        write!(out, "{}", template.docs.as_multiline("// "))?;
    }

    Ok(())
}

/// Generate register defines (address, reset val, always write)
fn generate_register_defines(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    let reg_name_generic = template.name_in_block(block);
    let reg_name_generic_macro = c_macro(&reg_name_generic);
    let macro_prefix = c_macro(&map.map_name);

    // Collect all defines into a table (vector of rows), so we can genreate a nicely alligned
    // code section:
    let mut defines = vec![]; // Each row: [define, value, comment]

    // Register address for each instance:
    if let Some(template_offset) = template.adr {
        for instance in block.instances.values() {
            let instance_name = template.name_in_instance(instance);
            if let Some(instance_adr) = &instance.adr {
                defines.push(vec![
                    format!("#define {macro_prefix}_{}", c_macro(&instance_name)),
                    format!("(0x{:X}U)", template_offset + instance_adr),
                    format!("//!< {instance_name} register address"),
                ]);
            }
        }
    }

    // Reset value
    if let Some(reset_val) = &template.reset_val {
        defines.push(vec![
            format!("#define {macro_prefix}_{reg_name_generic_macro}_RESET"),
            format!("(0x{reset_val:X}U)"),
            format!("//!< {reg_name_generic} register reset value"),
        ]);
    }

    // Always write value:
    if let Some(always_write) = &template.always_write {
        defines.push(vec![
            format!("#define {macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_MASK"),
            format!("(0x{:X}U)", always_write.mask),
            format!("//!< {reg_name_generic} register always write mask"),
        ]);
        defines.push(vec![
            format!("#define {macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_VALUE"),
            format!("(0x{:X}U)", always_write.value),
            format!("//!< {reg_name_generic} register always write value"),
        ]);
    }

    // Generate aligned defines:
    write!(out, "{}", str_table(&defines, "", " "))?;

    Ok(())
}

/// Generate register field enums.
fn generate_register_enums(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    opts: &GeneratorOpts,
) -> Result<(), Error> {
    for field in template.fields.values() {
        if let FieldType::LocalEnum(local_enum) = &field.accepts {
            let enum_name = name_register_enum(block, template, local_enum, opts);
            generate_enum(out, map, local_enum, &enum_name, opts)?;
        }
    }
    Ok(())
}

/// Generate register struct
fn generate_register_struct(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    opts: &GeneratorOpts,
) -> Result<(), Error> {
    let struct_name = name_register_struct(map, block, template);

    // doxy comment
    writeln!(out)?;
    c_generate_doxy_comment(
        out,
        &template.docs,
        "",
        Some("use pack/unpack/overwrite functions for conversion to/from packed register value"),
    )?;

    // Struct proper:
    writeln!(out, "struct {struct_name} {{")?;

    for field in template.fields.values() {
        let field_type = register_struct_member_type(map, block, template, field, opts)?;
        let field_name = c_code(&field.name);
        c_generate_doxy_comment(out, &field.docs, "  ", None)?;

        // members are bitifields, if configured:
        if opts.registers_as_bitfields {
            writeln!(out, "  {field_type} {field_name} : {};", mask_width(field.mask))?;
        } else {
            writeln!(out, "  {field_type} {field_name};",)?;
        }
    }

    writeln!(out, "}};",)?;
    Ok(())
}

/// Generate register packing/unpacking/validation funcs
fn generate_register_functions(
    out: &mut dyn Write,
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    opts: &GeneratorOpts,
) -> Result<(), Error> {
    let reg_name_generic = template.name_in_block(block);
    let reg_name_generic_code = c_code(&reg_name_generic);
    let reg_name_generic_macro = c_macro(&reg_name_generic);

    let struct_name = name_register_struct(map, block, template);
    let packed_type = c_fitting_unsigned_type(template.bitwidth)?;

    let macro_prefix = c_macro(&map.map_name);
    let code_prefix = c_code(&map.map_name);

    // ==== FUNC 1: Validation functions: ====

    if opts.generate_validation_functions {
        // Doxy comment
        writeln!(out)?;
        let docs = Docs {
            brief: Some(format!("Validate that a given value can be unpacked to as a @ref struct {struct_name}.")),
            doc: Some("Verifies that all enum fields can represent the given value.".to_string()),
        };
        c_generate_doxy_comment(out, &docs, "", None)?;

        // Can-unpack function
        writeln!(out, "static inline bool {code_prefix}_can_unpack_{reg_name_generic_code}({packed_type} val) {{")?;

        let mut have_used_arg = false;
        for field in template.fields.values() {
            // Struct member name
            let name = match &field.accepts {
                FieldType::LocalEnum(local_enum) => name_register_enum(block, template, local_enum, opts),
                FieldType::SharedEnum(shared_enum) => c_code(&shared_enum.name),
                FieldType::UInt => continue, // Skip if field is uint/bool. Those do not have to be validated.
                FieldType::Bool => continue,
            };

            let unpos_mask = unpositioned_mask(field.mask);
            let shift = lsb_pos(field.mask);
            let uint_type = c_fitting_unsigned_type(map.max_register_width())?;

            let field_value = format!("({uint_type})(val >> {shift}U) & 0x{unpos_mask:X}U");
            let enum_validate_func = format!("{code_prefix}_can_unpack_enum_{name}");

            writeln!(out, "  if (!{enum_validate_func}({field_value})) return false;")?;
            have_used_arg = true;
        }

        // Prevent unused-argument warning if val was not used because no field requires validation.
        if !have_used_arg {
            writeln!(out, "  (void) val;")?;
        }
        writeln!(out, "  return true;")?;
        writeln!(out, "}}")?;
    }

    // ==== FUNC 2: Overwrite: ====

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: Some(
            "All bits that are not part of a field or specified as 'always write' are kept as in 'val'.".to_string(),
        ),
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // Function:
    writeln!(
        out,
        "static inline {packed_type} {struct_name}_overwrite(const struct {struct_name} *r, {packed_type} val) {{"
    )?;

    // If function has always write, apply:
    if template.always_write.is_some() {
        let mask_macro = format!("{macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_MASK");
        let val_macro = format!("{macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_VALUE");

        writeln!(out, "  val &= ({packed_type})~({packed_type}){mask_macro};")?;
        writeln!(out, "  val |= {val_macro};")?;
    }

    // Pack each field:
    for field in template.fields.values() {
        let field_name = c_code(&field.name);
        let mask = field.mask;
        let unpos_mask = unpositioned_mask(mask);
        let shift = lsb_pos(mask);
        writeln!(out, "  val &= ({packed_type})~({packed_type})0x{mask:X}U;")?;
        writeln!(out, "  val |= ({packed_type})(((({packed_type})r->{field_name}) & 0x{unpos_mask:X}U) << ({packed_type}){shift}U);")?;
    }
    writeln!(out, "  return val;",)?;
    writeln!(out, "}}",)?;

    // ==== FUNC 3: Pack : ====
    // (Calls overwrite with zero value)

    // Doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: None,
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // Function:
    writeln!(out, "static inline {packed_type} {struct_name}_pack(const struct {struct_name} *r) {{")?;
    writeln!(out, "  return {struct_name}_overwrite(r, 0);")?;
    writeln!(out, "}}",)?;

    // ==== FUNC 4: Unpack macro: ====

    // Doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed register value to register struct initialization".to_string()),
        doc: None,
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // Collect macro lines, to later allign the trailing newline-escape slashes:
    let mut macro_lines: Vec<String> = vec![];

    // Macro:
    macro_lines.push(format!("#define {}_UNPACK(_VAL_) {{", c_macro(&struct_name)));

    // Unpack for each field:
    for field in template.fields.values() {
        let field_name = c_code(&field.name);
        let mask = field.mask;
        let unpos_mask = unpositioned_mask(mask);
        let shift = lsb_pos(mask);
        let field_type = register_struct_member_type(map, block, template, field, opts)?;
        let unsigned_type = c_fitting_unsigned_type(mask_width(mask))?;

        if matches!(field.accepts, FieldType::UInt) {
            macro_lines.push(format!("  .{field_name} = ({field_type})((_VAL_) >> {shift}U) & 0x{unpos_mask:X}U,"));
        } else {
            macro_lines.push(format!(
                "  .{field_name} = ({field_type})(({unsigned_type})((_VAL_) >> {shift}U) & 0x{unpos_mask:X}U),"
            ));
        }
    }

    // Generate macro:
    macro_lines.push("}".to_string());
    generate_multiline_macro(out, macro_lines)?;

    // ==== FUNC 5: Unpack into: ====

    // doxy comment
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed register value into a register struct.".to_string()),
        doc: None,
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // function
    writeln!(out, "static inline void {struct_name}_unpack_into({packed_type} val,  struct {struct_name} *r) {{")?;

    for field in template.fields.values() {
        let field_name = c_code(&field.name);
        let mask = field.mask;
        let unpos_mask = unpositioned_mask(mask);
        let shift = lsb_pos(mask);
        let field_type = register_struct_member_type(map, block, template, field, opts)?;
        let unsigned_type = c_fitting_unsigned_type(mask_width(mask))?;
        if matches!(field.accepts, FieldType::UInt) {
            writeln!(out, "  r->{field_name} = ({field_type})(val >> {shift}U) & 0x{unpos_mask:X}U;")?;
        } else {
            writeln!(
                out,
                "  r->{field_name} = ({field_type})(({unsigned_type})(val >> {shift}U) & 0x{unpos_mask:X}U);"
            )?;
        }
    }
    writeln!(out, "}}",)?;

    // ==== FUNC 5: Try unpack into: ====

    if opts.generate_validation_functions {
        // doxy comment:
        writeln!(out)?;
        let docs = Docs {
            brief: Some("Convert packed register value into a register struct.".to_string()),
            doc: Some(
                "This function verifies if the given value can be unpacked into the struct.\n".to_string()
                    + "@returns 0 if the register was succesfully unpacked, 1 otherwise.",
            ),
        };
        c_generate_doxy_comment(out, &docs, "", None)?;

        // function:
        writeln!(
            out,
            "static inline int {struct_name}_try_unpack_into({packed_type} val,  struct {struct_name} *r) {{"
        )?;
        writeln!(out, "  if(!{code_prefix}_can_unpack_{reg_name_generic_code}(val)) return 1;",)?;
        writeln!(out, "  {struct_name}_unpack_into(val, r);",)?;
        writeln!(out, "  return 0;",)?;
        writeln!(out, "}}",)?;
    }

    Ok(())
}

/// Generate generic macros:
fn generate_generic_macros(out: &mut dyn Write, map: &RegisterMap) -> Result<(), Error> {
    let macro_prefix = c_macro(&map.map_name);

    // ==== MACRO 1: Overwrite: ====

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: Some(
            "All bits that are not part of a field or specified as 'always write' are kept as in 'val'.".to_string(),
        ),
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // macro:
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_OVERWRITE(_struct_ptr_, _val_) _Generic((_struct_ptr_),"));
    for block in map.register_blocks.values() {
        for template in block.register_templates.values() {
            let struct_name = name_register_struct(map, block, template);
            if template.fields.is_empty() {
                continue;
            }
            macro_lines.push(format!("    struct {struct_name}* : {struct_name}_overwrite,"));
        }
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push("  )(_struct_ptr_, _val_)".into());
    generate_multiline_macro(out, macro_lines)?;

    // ==== MACRO 2: Pack: ====

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: None,
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // macro:
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_PACK(_struct_ptr_) _Generic((_struct_ptr_),"));
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

    // ==== MACRO 3: Unpack into: ====

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed register value to register struct.".to_string()),
        doc: None,
    };
    c_generate_doxy_comment(out, &docs, "", None)?;

    // macro:
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_UNPACK_INTO(_val_, _struct_ptr_) _Generic((_struct_ptr_),"));
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

    // ==== MACRO 4: Try unpack into: ====

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Attempt to convert packed register value to register struct.".to_string()),
        doc: Some(
            "This function verifies if the given value can be unpacked into the struct.\n".to_string()
                + "@returns 0 if the register was succesfully unpacked, 1 otherwise.",
        ),
    };
    c_generate_doxy_comment(out, &docs, "", None)?;
    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_TRY_UNPACK_INTO(_val_, _struct_ptr_) _Generic((_struct_ptr_),"));
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

/// Generate file footer
fn generate_footer(out: &mut dyn Write, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    // Include guard:
    writeln!(out)?;
    writeln!(out, "#endif /* REGINALD_{} */", c_macro(&filename(output_file)?))?;

    // Clang format:
    if opts.clang_format_guard {
        writeln!(out, "// clang-format on")?;
    }

    Ok(())
}

/// Generate multi-line macro with allgined newline-escape slashes:
fn generate_multiline_macro(out: &mut dyn Write, mut lines: Vec<String>) -> Result<(), Error> {
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

/// Name of register field enum
fn name_register_enum(block: &RegisterBlock, template: &Register, field_enum: &Enum, opts: &GeneratorOpts) -> String {
    let regname = c_code(&template.name_in_block(block));
    let enumname = c_code(&field_enum.name);
    if opts.field_enum_prefix {
        format!("{regname}_{enumname}")
    } else {
        enumname.to_string()
    }
}

/// Name of register struct
fn name_register_struct(map: &RegisterMap, block: &RegisterBlock, template: &Register) -> String {
    let mapname = c_code(&map.map_name);
    let regname = c_code(&template.name_in_block(block));
    format!("{mapname}_{regname}")
}

/// Type of register struct member
fn register_struct_member_type(
    map: &RegisterMap,
    block: &RegisterBlock,
    template: &Register,
    field: &Field,
    opts: &GeneratorOpts,
) -> Result<String, Error> {
    let code_prefix = c_code(&map.map_name);
    match &field.accepts {
        FieldType::LocalEnum(local_enum) => {
            let name = name_register_enum(block, template, local_enum, opts);
            Ok(format!("enum {code_prefix}_{name}"))
        }
        FieldType::SharedEnum(shared_enum) => {
            let name = c_code(&shared_enum.name);
            Ok(format!("enum {code_prefix}_{name}"))
        }
        FieldType::UInt => c_fitting_unsigned_type(mask_width(field.mask)),
        FieldType::Bool => Ok("bool".to_string()),
    }
}

/// Check if template has any content to generate (based on current settings and
/// template properties)
fn template_has_content_to_generate(template: &Register, opts: &GeneratorOpts) -> bool {
    if opts.generate_registers {
        // Generate section for every register if 'generate_registers' is set.
        return true;
    }

    if opts.generate_enums {
        for field in template.fields.values() {
            if matches!(field.accepts, FieldType::LocalEnum(_)) {
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
