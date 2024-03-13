use std::{fmt::Write, path::Path};

#[cfg(feature = "cli")]
use clap::{Parser, ValueEnum};

use crate::{
    bits::{lsb_pos, mask_width},
    error::Error,
    regmap::{Docs, Enum, Field, FieldType, Register, RegisterBlock, RegisterMap, TypeBitwidth, TypeValue},
    utils::{
        byte_to_field_transform, field_to_byte_transform, filename, numbers_as_ranges, str_pad_to_length, str_table,
        Endianess, ShiftDirection,
    },
};

use super::{c_code, c_fitting_unsigned_type, c_generate_doxy_comment, c_generate_section_header_comment, c_macro};

// ====== Generator Opts =======================================================

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Element {
    Enums,
    EnumValidationFuncs,
    RegisterStructs,
    RegisterProperties,
    RegisterConversionFuncs,
    GenericMacros,
}

#[derive(Debug, Clone)]
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

    /// Endianess of input/output byte arrays.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "little"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub endian: Endianess,

    /// Make register structs bitfields to reduce their memory size
    ///
    /// Note that their memory layout will not match the actual register
    /// and the (un)packing functions must still be used.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub registers_as_bitfields: bool,

    /// Header file that should be included at the top of the generated header
    ///
    /// May be given multiple times.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub add_include: Vec<String>,

    /// Make all functions static inline.
    ///
    /// May be disabled if splitting code into header and source.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub funcs_static_inline: bool,

    /// Generate function prototypes instead of full implementations.
    ///
    /// May be enabled if splitting code into header and source.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "false"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub funcs_as_prototypes: bool,

    /// This enables the generation of validation functions/macros that
    /// check if a given value can be represented as an enum or struct.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub validation: bool,

    /// Surround file with a clang-format off guard
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub clang_format_guard: bool,

    /// Generate include guard
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub include_guards: bool,

    /// Generate doxygen comments.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub doxy_comments: bool,

    /// Only generate a subset of the elements/sections usually included in
    /// a complete output file.
    ///
    /// This option is mutually exclusive with 'dont_generate'
    /// If this option is not given, all elements are generated. This option
    /// may be given multiple times.
    /// Note that different components depend on each other. It is up to the
    /// user to generate all required sections, or add includes that provide
    /// those elements.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    #[cfg_attr(feature = "cli", arg(conflicts_with("dont_generate")))]
    pub only_generate: Vec<Element>,

    /// Skip generation of some element/section usually included in a complete
    /// output file.
    ///
    /// This option is mutually exclusive with 'only_generate'
    /// Note that different components depend on each other. It is up to the
    /// user to generate all required sections, or add includes that provide
    /// those elements.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    #[cfg_attr(feature = "cli", arg(conflicts_with("only_generate")))]
    pub dont_generate: Vec<Element>,
}

// ====== Generator ============================================================

struct Input<'a> {
    opts: GeneratorOpts,
    map: &'a RegisterMap,
    output_file: &'a Path,
}

pub fn generate(out: &mut dyn Write, map: &RegisterMap, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    let inp = Input {
        opts: opts.clone(),
        map,
        output_file,
    };

    generate_header(out, &inp)?;
    generate_shared_enums(out, &inp)?;
    for block in map.register_blocks.values() {
        generate_register_block_defines(out, &inp, block)?;
        for template in block.register_templates.values() {
            // Generate all template-specific output into a temporary buffer,
            // and then only output header if there was any content generated.
            // This avoids fairly complicated logic to decide if based on the
            // current options and properties of the current template, there
            // is any content to generated.
            let mut out_buf = String::new();
            generate_register_defines(&mut out_buf, &inp, block, template)?;
            generate_register_enums(&mut out_buf, &inp, block, template)?;
            generate_register_struct(&mut out_buf, &inp, block, template)?;
            generate_register_funcs(&mut out_buf, &inp, block, template)?;
            if !out_buf.is_empty() {
                generate_register_header(out, block, template)?;
                write!(out, "{}", out_buf)?;
            }
        }
    }
    generate_generic_macros(out, &inp)?;
    generate_footer(out, &inp)?;
    Ok(())
}

fn is_enabled(inp: &Input, e: Element) -> bool {
    if inp.opts.only_generate.is_empty() {
        !inp.opts.dont_generate.contains(&e)
    } else {
        inp.opts.only_generate.contains(&e)
    }
}

/// Generate header comment, include guard, includes.
fn generate_header(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    // Clang format guard, if enabled:
    if inp.opts.clang_format_guard {
        writeln!(out, "// clang-format off")?;
    }

    // Doxy file comment:
    writeln!(out, "/**")?;
    writeln!(out, " * @file {}", filename(inp.output_file)?)?;
    writeln!(out, " * @brief {}", inp.map.map_name)?;
    if let Some(input_file) = &inp.map.from_file {
        writeln!(out, " * @note do not edit directly: generated using reginald from {}.", filename(input_file)?)?;
    } else {
        writeln!(out, " * @note do not edit directly: generated using reginald.",)?;
    }
    writeln!(out, " *")?;
    writeln!(out, " * Generator: c.funcpack")?;

    // Map docs/author/note, if present:
    if !inp.map.docs.is_empty() {
        writeln!(out, " *")?;
        write!(out, "{}", inp.map.docs.as_multiline(" * "))?;
    }
    if let Some(author) = &inp.map.author {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file author: {author}")?;
    }
    if let Some(note) = &inp.map.note {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file note:")?;
        for line in note.lines() {
            writeln!(out, " *   {line}")?;
        }
    }
    writeln!(out, " */")?;

    // Include guard
    if inp.opts.include_guards {
        writeln!(out, "#ifndef REGINALD_{}", c_macro(&filename(inp.output_file)?))?;
        writeln!(out, "#define REGINALD_{}", c_macro(&filename(inp.output_file)?))?;
    }

    // Includes
    writeln!(out)?;
    writeln!(out, "#include <stdint.h>")?;
    writeln!(out, "#include <stdbool.h>")?;
    for include in &inp.opts.add_include {
        writeln!(out, "#include \"{include}\"")?;
    }

    Ok(())
}

/// Generate 'shared enums' section:
fn generate_shared_enums(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    if inp.map.shared_enums.is_empty() {
        return Ok(());
    }

    writeln!(out)?;
    c_generate_section_header_comment(out, "Shared Enums")?;

    if is_enabled(inp, Element::Enums) {
        for shared_enum in inp.map.shared_enums.values() {
            generate_enum(out, inp, shared_enum, &c_code(&shared_enum.name))?;
        }
    }

    if is_enabled(inp, Element::EnumValidationFuncs) {
        for shared_enum in inp.map.shared_enums.values() {
            generate_enum_validation_func(out, inp, shared_enum, &c_code(&shared_enum.name))?;
        }
    }

    Ok(())
}

/// Generate shared or field enum
fn generate_enum(out: &mut dyn Write, inp: &Input, e: &Enum, name: &str) -> Result<(), Error> {
    let code_prefix = c_code(&inp.map.map_name);
    let macro_prefix = c_macro(&inp.map.map_name);

    if is_enabled(inp, Element::Enums) {
        // Enum proper:
        writeln!(out)?;
        generate_doxy_comment(out, inp, &e.docs, "", None)?;
        writeln!(out, "enum {code_prefix}_{name} {{")?;
        for entry in e.entries.values() {
            generate_doxy_comment(out, inp, &entry.docs, "  ", None)?;
            writeln!(out, "  {}_{}_{} = 0x{:X}U,", macro_prefix, c_macro(name), c_macro(&entry.name), entry.value)?;
        }
        writeln!(out, "}};")?;
    }

    Ok(())
}

/// Generate shared or field enum validation func
fn generate_enum_validation_func(out: &mut dyn Write, inp: &Input, e: &Enum, name: &str) -> Result<(), Error> {
    let code_prefix = c_code(&inp.map.map_name);
    let uint_type = c_fitting_unsigned_type(e.min_bitdwith())?;
    let accept_values: Vec<TypeValue> = e.entries.values().map(|x| x.value).collect();
    let accept_ranges = numbers_as_ranges(accept_values);

    if !inp.opts.validation {
        return Ok(());
    }

    // Doxy comment:
    writeln!(out,)?;
    let docs = Docs {
        brief: Some(format!("Validate a value can be represented as an @ref enum {code_prefix}_{name}.")),
        doc: None,
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Validation Function
    let func_prefix = func_prefix(inp);
    let func_sig = format!("{func_prefix}bool {code_prefix}_is_{name}_enum({uint_type} val)");
    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
    } else {
        writeln!(out, "{func_sig} {{")?;

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
fn generate_register_block_defines(out: &mut dyn Write, inp: &Input, block: &RegisterBlock) -> Result<(), Error> {
    if !is_enabled(inp, Element::RegisterProperties) {
        return Ok(());
    }

    // Collect all defines into a table (vector of rows), so we can generate a nicely alligned
    // code section:
    let mut defines = vec![]; // Each row: [define, value, comment]

    let block_name = &block.name;

    // Generate register block constants (starts of instances + offsets of registers), if that information is not
    // redundant from the actual register addresses.
    if block.instances.len() > 1 && block.register_templates.len() > 1 {
        let macro_prefix = c_macro(&inp.map.map_name);
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
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    if !is_enabled(inp, Element::RegisterProperties) {
        return Ok(());
    }
    let reg_name_generic = template.name_in_block(block);
    let reg_name_generic_macro = c_macro(&reg_name_generic);
    let macro_prefix = c_macro(&inp.map.map_name);

    // Collect all defines into a table (vector of rows), so we can generate a nicely alligned
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
            to_array_init(inp, *reset_val, template.width_bytes()),
            format!("//!< {reg_name_generic} register reset value"),
        ]);
    }

    // Always write value:
    if let Some(always_write) = &template.always_write {
        defines.push(vec![
            format!("#define {macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_MASK"),
            to_array_init(inp, always_write.mask, template.width_bytes()),
            format!("//!< {reg_name_generic} register always write mask"),
        ]);
        defines.push(vec![
            format!("#define {macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_VALUE"),
            to_array_init(inp, always_write.value, template.width_bytes()),
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
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    for field in template.fields.values() {
        if let FieldType::LocalEnum(local_enum) = &field.accepts {
            let name = name_register_enum(inp, block, template, local_enum);
            generate_enum(out, inp, local_enum, &name)?;
            if is_enabled(inp, Element::EnumValidationFuncs) {
                generate_enum_validation_func(out, inp, local_enum, &name)?;
            }
        }
    }
    Ok(())
}

/// Generate register struct
fn generate_register_struct(
    out: &mut dyn Write,
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    if template.fields.is_empty() {
        return Ok(());
    }

    if !is_enabled(inp, Element::RegisterStructs) {
        return Ok(());
    }

    let struct_name = name_register_struct(inp, block, template);

    // doxy comment
    writeln!(out)?;
    generate_doxy_comment(
        out,
        inp,
        &template.docs,
        "",
        Some("use pack/unpack/overwrite functions for conversion to/from packed register value"),
    )?;

    // Struct proper:
    writeln!(out, "struct {struct_name} {{")?;

    for field in template.fields.values() {
        let field_type = register_struct_member_type(inp, block, template, field)?;
        let field_name = c_code(&field.name);
        generate_doxy_comment(out, inp, &field.docs, "  ", None)?;

        // Members are bitifields, if configured:
        if inp.opts.registers_as_bitfields {
            writeln!(out, "  {field_type} {field_name} : {};", mask_width(field.mask))?;
        } else {
            writeln!(out, "  {field_type} {field_name};",)?;
        }
    }

    writeln!(out, "}};",)?;
    Ok(())
}

/// Generate register packing/unpacking/validation funcs
fn generate_register_funcs(
    out: &mut dyn Write,
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    if template.fields.is_empty() {
        return Ok(());
    }

    if !is_enabled(inp, Element::RegisterConversionFuncs) {
        return Ok(());
    }

    generate_register_func_overwrite(out, inp, block, template)?;
    generate_register_func_pack(out, inp, block, template)?;
    generate_register_func_unpack(out, inp, block, template, false)?;
    generate_register_func_unpack(out, inp, block, template, true)?;

    Ok(())
}

fn generate_register_func_overwrite(
    out: &mut dyn Write,
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    // Strings:
    let reg_name_generic = template.name_in_block(block);
    let reg_name_generic_macro = c_macro(&reg_name_generic);
    let struct_name = name_register_struct(inp, block, template);
    let macro_prefix = c_macro(&inp.map.map_name);
    let func_prefix = func_prefix(inp);

    let width_bytes = template.width_bytes();

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: Some("All non-field/always write bits are left untouched.".to_string()),
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Function:
    let func_sig = format!(
        "{}void {}_overwrite(const struct {} *r, uint8_t val[{}])",
        func_prefix, struct_name, struct_name, width_bytes
    );
    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;

    // Apply always-write:
    if template.always_write.is_some() {
        let mask_macro = format!("{macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_MASK");
        let val_macro = format!("{macro_prefix}_{reg_name_generic_macro}_ALWAYSWRITE_VALUE");
        writeln!(out, "  uint8_t aw_m[{width_bytes}] = {mask_macro}; // Always-write mask")?;
        writeln!(out, "  uint8_t aw_v[{width_bytes}] = {val_macro}; // Always-write value")?;
        writeln!(out, "  for (uint32_t i = 0; i < {width_bytes}; i++) {{")?;
        writeln!(out, "      val[i] &= (uint8_t)~aw_m[i];")?;
        writeln!(out, "      val[i] |= aw_v[i];")?;
        writeln!(out, "  }}")?;
    }

    // Pack each field:
    for field in template.fields.values() {
        let field_name = c_code(&field.name);
        for byte in 0..width_bytes {
            let Some(transform) = field_to_byte_transform(inp.opts.endian, field.mask, byte, width_bytes) else {
                continue;
            };

            writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", transform.mask)?;

            let field_byte = match &transform.shift {
                Some((ShiftDirection::Left, amnt)) => format!("(r->{field_name} << {amnt})"),
                Some((ShiftDirection::Right, amnt)) => format!("(r->{field_name} >> {amnt})"),
                None => format!("r->{field_name}"),
            };

            writeln!(out, "  val[{byte}] |= (uint8_t)(((uint8_t){field_byte}) & 0x{:X}U);", transform.mask)?;
        }
    }

    writeln!(out, "}}",)?;

    Ok(())
}

fn generate_register_func_pack(
    out: &mut dyn Write,
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
) -> Result<(), Error> {
    // Strings:
    let struct_name = name_register_struct(inp, block, template);
    let func_prefix = func_prefix(inp);

    let width_bytes = template.width_bytes();

    // Doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Function:
    let func_sig = format!(
        "{}void {}_pack(const struct {} *r, uint8_t val[{}])",
        func_prefix, struct_name, struct_name, width_bytes
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;
    writeln!(out, "  for (uint32_t i = 0; i < {width_bytes}; i++) {{ val[i] = 0; }}")?;
    writeln!(out, "  {struct_name}_overwrite(r, val);")?;
    writeln!(out, "}}",)?;

    Ok(())
}

fn generate_register_func_unpack(
    out: &mut dyn Write,
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
    try_unpack: bool,
) -> Result<(), Error> {
    if try_unpack && !inp.opts.validation {
        // Cannot generate a 'try-unpack' function if validation is turned off
        // and no enum validation funcs are generated.
        return Ok(());
    }

    // Strings:
    let struct_name = name_register_struct(inp, block, template);
    let func_prefix = func_prefix(inp);
    let code_prefix = c_code(&inp.map.map_name);

    let width_bytes = template.width_bytes();

    // doxy comment:
    writeln!(out)?;
    let docs = if try_unpack {
        Docs {
            brief: Some("Attempt to convert packed register value to register struct.".to_string()),
            doc: Some(
                "@returns 0 if succesfull.\n@returns the position of the field that could not be unpacked plus one, if enums can not represent register content.".to_string(),
            ),
        }
    } else {
        Docs {
            brief: Some("Convert packed register value to register struct.".to_string()),
            doc: None,
        }
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Function signature (depending on what flavour unpack function we are
    // generating)
    let func_sig = if try_unpack {
        format!(
            "{}int {}_try_unpack(const uint8_t val[{}], struct {} *r)",
            func_prefix, struct_name, width_bytes, struct_name
        )
    } else {
        format!("{}struct {} {}_unpack(const uint8_t val[{}])", func_prefix, struct_name, struct_name, width_bytes)
    };

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;

    if try_unpack && template.can_always_unpack() {
        // No need to generate complete try unpack, since unpack will
        // always succeed.
        writeln!(out, "  // Can always be unpacked.")?;
        writeln!(out, "  *r = {struct_name}_unpack(val);")?;
        writeln!(out, "  return 0;")?;
        writeln!(out, "}}")?;
        return Ok(());
    }

    if !try_unpack {
        writeln!(out, "  struct {struct_name} r;")?;
    }

    // Unpack each field:
    for field in template.fields.values() {
        let field_name = c_code(&field.name);

        // Generate code to assemble field value from input bytes:
        let pre_cast = match &field.accepts {
            FieldType::UInt => format!("({})", c_fitting_unsigned_type(mask_width(field.mask))?),
            FieldType::Bool => String::new(),
            FieldType::LocalEnum(e) => format!("({})", c_fitting_unsigned_type(e.min_bitdwith())?),
            FieldType::SharedEnum(e) => format!("({})", c_fitting_unsigned_type(e.min_bitdwith())?),
        };
        let post_cast = match &field.accepts {
            FieldType::UInt => format!("({})", c_fitting_unsigned_type(mask_width(field.mask))?),
            FieldType::Bool => String::from("(bool)"),
            FieldType::LocalEnum(e) => {
                if try_unpack {
                    format!("({})", c_fitting_unsigned_type(e.min_bitdwith())?)
                } else {
                    format!("(enum {code_prefix}_{})", name_register_enum(inp, block, template, e))
                }
            }
            FieldType::SharedEnum(e) => {
                if try_unpack {
                    format!("({})", c_fitting_unsigned_type(e.min_bitdwith())?)
                } else {
                    format!("(enum {code_prefix}_{})", c_code(&e.name))
                }
            }
        };

        let mut unpacked_value: Vec<String> = vec![];

        for byte in 0..width_bytes {
            let Some(transform) = byte_to_field_transform(inp.opts.endian, field.mask, byte, width_bytes) else {
                continue;
            };

            let masked = format!("({pre_cast}(val[{byte}] & 0x{:X}U))", transform.mask);

            match &transform.shift {
                Some((ShiftDirection::Left, amnt)) => unpacked_value.push(format!("({masked} << {amnt})")),
                Some((ShiftDirection::Right, amnt)) => unpacked_value.push(format!("({masked} >> {amnt})")),
                None => unpacked_value.push(masked),
            };
        }

        let unpacked_value = format!("{post_cast}({})", unpacked_value.join(" | "));
        assert!(!unpacked_value.is_empty());

        if try_unpack {
            if let Some(e) = field.get_enum() {
                let unpacked_type = c_fitting_unsigned_type(e.min_bitdwith())?;
                let enum_name = if e.is_shared {
                    c_code(&e.name)
                } else {
                    name_register_enum(inp, block, template, e)
                };
                let error_code = lsb_pos(field.mask) + 1;
                writeln!(out, "  {unpacked_type} {field_name} = {unpacked_value};")?;
                writeln!(out, "  if (!{code_prefix}_is_{enum_name}_enum({field_name})) {{ return {error_code}; }}",)?;
                writeln!(out, "  r->{field_name} = (enum {code_prefix}_{enum_name}){field_name};")?;
            } else {
                writeln!(out, "  r->{field_name} = {unpacked_value};")?;
            }
        } else {
            writeln!(out, "  r.{field_name} = {unpacked_value};")?;
        }
    }

    if try_unpack {
        writeln!(out, "  return 0;")?;
    } else {
        writeln!(out, "  return r;")?;
    }
    writeln!(out, "}}",)?;
    Ok(())
}

/// Generate generic macros:
fn generate_generic_macros(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    if !is_enabled(inp, Element::GenericMacros) {
        return Ok(());
    }

    c_generate_section_header_comment(out, "Generic Macros")?;

    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: Some("All non-field/always write bits are left untouched.".to_string()),
    };
    generate_generic_macro(out, inp, &docs, "overwrite", "_struct_ptr_, _val_")?;

    let docs = Docs {
        brief: Some("Convert register struct to packed register value.".to_string()),
        doc: None,
    };
    generate_generic_macro(out, inp, &docs, "pack", "_struct_ptr_, _val_")?;

    if inp.opts.validation {
        let docs = Docs {
            brief: Some("Attempt to convert packed register value to register struct.".to_string()),
            doc: Some(
                "This function verifies if the given value can be unpacked into the struct.\n".to_string()
                    + "@returns 0 if the register was succesfully unpacked, 1 otherwise.",
            ),
        };
        generate_generic_macro(out, inp, &docs, "try_unpack", "_val_, _struct_ptr_")?;
    }

    Ok(())
}

fn generate_generic_macro(
    out: &mut dyn Write,
    inp: &Input,
    docs: &Docs,
    func_name_suffix: &str,
    func_args: &str,
) -> Result<(), Error> {
    let macro_name_suffix = c_macro(func_name_suffix);
    let macro_prefix = c_macro(&inp.map.map_name);

    // doxy comment:
    writeln!(out)?;
    generate_doxy_comment(out, inp, docs, "", None)?;

    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_{macro_name_suffix}({func_args}) _Generic((_struct_ptr_),"));
    for block in inp.map.register_blocks.values() {
        for template in block.register_templates.values() {
            let struct_name = name_register_struct(inp, block, template);
            if template.fields.is_empty() {
                continue;
            }
            macro_lines.push(format!("    struct {struct_name}* : {struct_name}_{func_name_suffix},"));
        }
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push(format!("  )({func_args})"));
    generate_multiline_macro(out, macro_lines)?;
    Ok(())
}

/// Generate file footer
fn generate_footer(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    // Include guard:
    writeln!(out)?;

    if inp.opts.include_guards {
        writeln!(out, "#endif /* REGINALD_{} */", c_macro(&filename(inp.output_file)?))?;
    }

    // Clang format:
    if inp.opts.clang_format_guard {
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

fn generate_doxy_comment(
    out: &mut dyn Write,
    inp: &Input,
    docs: &Docs,
    prefix: &str,
    note: Option<&str>,
) -> Result<(), Error> {
    if inp.opts.doxy_comments {
        c_generate_doxy_comment(out, docs, prefix, note)
    } else {
        Ok(())
    }
}

// ====== Generator Utils ======================================================

/// Name of register field enum
fn name_register_enum(inp: &Input, block: &RegisterBlock, template: &Register, field_enum: &Enum) -> String {
    let regname = c_code(&template.name_in_block(block));
    let enumname = c_code(&field_enum.name);
    if inp.opts.field_enum_prefix {
        format!("{regname}_{enumname}")
    } else {
        enumname.to_string()
    }
}

/// Name of register struct
fn name_register_struct(inp: &Input, block: &RegisterBlock, template: &Register) -> String {
    let mapname = c_code(&inp.map.map_name);
    let regname = c_code(&template.name_in_block(block));
    format!("{mapname}_{regname}")
}

/// Type of register struct member
fn register_struct_member_type(
    inp: &Input,
    block: &RegisterBlock,
    template: &Register,
    field: &Field,
) -> Result<String, Error> {
    let code_prefix = c_code(&inp.map.map_name);
    match &field.accepts {
        FieldType::LocalEnum(local_enum) => {
            let name = name_register_enum(inp, block, template, local_enum);
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

/// Decide what each function should be prefixed with, depending on
/// given opts.
fn func_prefix(inp: &Input) -> &'static str {
    if inp.opts.funcs_static_inline {
        "static inline "
    } else {
        ""
    }
}

/// Convert a value to an array initialiser of correct endianess
fn to_array_init(inp: &Input, val: TypeValue, width_bytes: TypeBitwidth) -> String {
    let mut bytes: Vec<String> = vec![];

    for i in 0..width_bytes {
        let byte = format!("0x{:X}U", ((val >> (8 * i)) & 0xFF) as u8);
        bytes.push(byte);
    }

    if matches!(inp.opts.endian, Endianess::Big) {
        bytes.reverse();
    }

    format!("{{{}}}", bytes.join(", "))
}
