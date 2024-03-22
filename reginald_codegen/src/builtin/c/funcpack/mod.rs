use std::{fmt::Write, path::Path, rc::Rc};

#[cfg(feature = "cli")]
use clap::{Parser, ValueEnum};

use crate::{
    bits::{lsb_pos, mask_to_bit_ranges_str, mask_width, unpositioned_mask},
    error::Error,
    regmap::{
        Docs, Enum, FieldType, Layout, LayoutField, Register, RegisterBlock, RegisterBlockMember, RegisterMap,
        TypeBitwidth, TypeValue,
    },
    utils::{
        field_byte_to_packed_byte_transform, field_to_packed_byte_transform, filename, grab_byte, numbers_as_ranges,
        packed_byte_to_field_byte_transform, packed_byte_to_field_transform, str_pad_to_length, str_table, Endianess,
        ShiftDirection,
    },
    writer::header_writer::HeaderWriter,
};

use super::{
    c_code, c_fitting_unsigned_type, c_generate_doxy_comment, c_generate_header_comment,
    c_generate_section_header_comment, c_header_comment, c_layout_overview_comment, c_macro, c_section_header_comment,
};

// ====== Generator Opts =======================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum Element {
    Enums,
    EnumValidationFuncs,
    Structs,
    StructConversionFuncs,
    RegisterProperties,
    GenericMacros,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Endianess of input/output byte arrays.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "little"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub endian: Endianess,

    /// Include endianess in function and macro names
    ///
    /// Function/macro include a "_le" or "_be" to indicate if they operator
    /// on little/big endian binary arrays.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub endianess_in_names: bool,

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

    let mut out = HeaderWriter::new(out);

    generate_header(&mut out, &inp)?;

    // ===== Shared enums: =====
    out.push_section_with_header(&["\n", &c_section_header_comment("Shared Enums"), "\n"]);
    for e in map.shared_enums() {
        out.push_section_with_header(&["\n", &c_header_comment(&format!("{} Enum", e.name)), "\n"]);

        generate_enum(&mut out, &inp, e)?;
        generate_enum_validation_func(&mut out, &inp, e)?;

        out.pop_section();
    }
    out.pop_section();

    // ===== Shared layouts: =====
    out.push_section_with_header(&["\n", &c_section_header_comment("Shared Layout Structs"), "\n"]);
    for layout in map.shared_layouts() {
        out.push_section_with_header(&["\n", &c_header_comment(&format!("{} Layout", layout.name)), "\n"]);

        if is_enabled(&inp, Element::Structs) {
            writeln!(out, "// Fields:")?;
            writeln!(out, "{}", c_layout_overview_comment(layout))?;
        }
        generate_layout(&mut out, &inp, layout, true)?;

        out.pop_section();
    }
    out.pop_section();

    // ===== Individual Registers: =====
    for register in map.individual_registers() {
        let mut header = String::new();
        generate_register_header(&mut header, &inp, register)?;
        out.push_section_with_header(&[&header]);

        generate_register_properties(&mut out, &inp, register)?;

        // If the layout is local to this register, generate it:
        if register.layout.is_local {
            generate_layout(&mut out, &inp, &register.layout, true)?;
        } else if is_enabled(&inp, Element::Structs) {
            writeln!(&mut out)?;
            writeln!(
                out,
                "// Register uses the {}_{} struct and conversion funcs defined above.",
                c_code(&map.name),
                c_code(&register.layout.name)
            )?;
        }

        out.pop_section();
    }

    // Register blocks:
    for block in map.register_blocks.values() {
        let mut header = String::new();
        generate_register_block_header(&mut header, block)?;
        out.push_section_with_header(&[&header]);

        generate_register_block_properties(&mut out, &inp, block)?;

        for member in block.members.values() {
            let mut header = String::new();
            generate_register_block_member_header(&mut header, &inp, member)?;
            out.push_section_with_header(&[&header]);

            generate_register_block_member_properties(&mut out, &inp, member, block)?;
            if member.layout.is_local {
                generate_layout(&mut out, &inp, &member.layout, true)?;
            } else if is_enabled(&inp, Element::Structs) {
                writeln!(&mut out)?;
                writeln!(
                    out,
                    "// Register uses the {}_{} struct and conversion funcs defined above.",
                    c_code(&map.name),
                    c_code(&member.layout.name)
                )?;
            }

            out.pop_section();
        }

        out.pop_section();
    }

    generate_generic_macros(&mut out, &inp)?;

    generate_footer(&mut out, &inp)?;
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
    writeln!(out, " * @brief {}", inp.map.name)?;
    if let Some(input_file) = &inp.map.from_file {
        writeln!(out, " * @note do not edit directly: generated using reginald from {}.", filename(input_file)?)?;
    } else {
        writeln!(out, " * @note do not edit directly: generated using reginald.",)?;
    }
    writeln!(out, " *")?;
    writeln!(out, " * Generator: c.funcpack")?;
    if is_enabled(inp, Element::StructConversionFuncs) || is_enabled(inp, Element::RegisterProperties) {
        writeln!(out, " * Endianess: {}", inp.opts.endian)?;
    }

    // Map docs/author/note, if present:
    if !inp.map.docs.is_empty() {
        writeln!(out, " *")?;
        write!(out, "{}", inp.map.docs.as_multiline(" * "))?;
    }
    if let Some(author) = &inp.map.author {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file author: {author}")?;
    }
    if let Some(note) = &inp.map.notice {
        writeln!(out, " *")?;
        writeln!(out, " * Listing file notice:")?;
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

/// Generate an enum
fn generate_enum(out: &mut dyn Write, inp: &Input, e: &Enum) -> Result<(), Error> {
    if !is_enabled(inp, Element::Enums) {
        return Ok(());
    }

    let code_prefix = c_code(&inp.map.name);
    let macro_prefix = c_macro(&inp.map.name);

    let code_name = c_code(&e.name);
    let macro_name = c_macro(&e.name);

    // Enum proper:
    writeln!(out)?;
    generate_doxy_comment(out, inp, &e.docs, "", None)?;
    writeln!(out, "enum {code_prefix}_{code_name} {{")?;
    for entry in e.entries.values() {
        generate_doxy_comment(out, inp, &entry.docs, "  ", None)?;
        writeln!(out, "  {}_{}_{} = 0x{:X}U,", macro_prefix, macro_name, c_macro(&entry.name), entry.value)?;
    }
    writeln!(out, "}};")?;

    Ok(())
}

/// Generate an enum validation func
fn generate_enum_validation_func(out: &mut dyn Write, inp: &Input, e: &Enum) -> Result<(), Error> {
    if !is_enabled(inp, Element::EnumValidationFuncs) {
        return Ok(());
    }

    let code_prefix = c_code(&inp.map.name);

    let uint_type = c_fitting_unsigned_type(e.min_bitdwith())?;
    let accept_values: Vec<TypeValue> = e.entries.values().map(|x| x.value).collect();
    let accept_ranges = numbers_as_ranges(accept_values);
    let code_name = c_code(&e.name);

    // Doxy comment:
    writeln!(out,)?;
    let docs = Docs {
        brief: Some(format!("Check if a numeric value is a valid @ref enum {code_prefix}_{code_name}.")),
        doc: None,
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Validation Function
    let func_prefix = func_prefix(inp);
    let func_sig = format!("{func_prefix}bool {code_prefix}_is_{code_name}_enum({uint_type} val)");

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

fn generate_layout(out: &mut dyn Write, inp: &Input, layout: &Layout, generate_headers: bool) -> Result<(), Error> {
    let mut out = HeaderWriter::new(out);

    if generate_headers {
        if layout.is_local {
            out.push_section_with_header(&["\n", "// Register-specific enums and sub-layouts:", "\n"]);
        } else {
            out.push_section_with_header(&["\n", "// Layout-specific enums and sub-layouts:", "\n"]);
        }
    }

    for e in layout.local_enums() {
        generate_enum(&mut out, inp, e)?;
        generate_enum_validation_func(&mut out, inp, e)?;
    }

    for local_layout in layout.local_layouts() {
        generate_layout(&mut out, inp, local_layout, false)?;
    }

    if generate_headers {
        out.pop_section();
    }

    if generate_headers {
        if layout.is_local {
            out.push_section_with_header(&["\n", "// Register Layout Struct:", "\n"]);
        } else {
            out.push_section_with_header(&["\n", "// Layout Struct:", "\n"]);
        }
    }

    generate_layout_properties(&mut out, inp, layout)?;
    generate_layout_struct(&mut out, inp, layout)?;

    if generate_headers {
        out.pop_section();
    }

    if generate_headers {
        out.push_section_with_header(&["\n", "// Struct Conversion Functions:", "\n"]);
    }

    generate_layout_funcs(&mut out, inp, layout)?;

    if generate_headers {
        out.pop_section();
    }

    Ok(())
}

fn generate_layout_properties(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    if !is_enabled(inp, Element::Structs) {
        return Ok(());
    }

    let macro_prefix = c_macro(&inp.map.name);
    let macro_name = c_macro(&layout.name);

    let mut defines = vec![];

    if layout.contains_fixed_bits() {
        defines.push(vec![]);
        defines.push(vec![
            format!("#define {macro_prefix}_{macro_name}_ALWAYSWRITE_MASK"),
            to_array_init(inp, layout.fixed_bits_mask(), layout.width_bytes()),
            format!("//!< {} always write mask", layout.name),
        ]);
        defines.push(vec![
            format!("#define {macro_prefix}_{macro_name}_ALWAYSWRITE_VALUE"),
            to_array_init(inp, layout.fixed_bits_val(), layout.width_bytes()),
            format!("//!< {} always write value", layout.name),
        ]);
    }

    if !defines.is_empty() {
        write!(out, "{}", str_table(&defines, "", " "))?;
    }
    Ok(())
}

fn generate_layout_struct(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    if !is_enabled(inp, Element::Structs) {
        return Ok(());
    }

    let code_prefix = c_code(&inp.map.name);
    let struct_name = format!("{}_{}", c_code(&inp.map.name), c_code(&layout.name));

    // doxy comment
    writeln!(out)?;
    generate_doxy_comment(
        out,
        inp,
        &layout.docs,
        "",
        Some("use pack/unpack functions for conversion to/from packed binary value"),
    )?;

    // Struct proper:
    writeln!(out, "struct {struct_name} {{")?;
    for field in layout.fields.values() {
        let field_type = match &field.accepts {
            FieldType::Enum(e) => {
                let name = c_code(&e.name);
                format!("enum {code_prefix}_{name}")
            }
            FieldType::UInt => c_fitting_unsigned_type(mask_width(field.mask))?,
            FieldType::Bool => "bool".to_string(),
            FieldType::Layout(layout) => {
                let name = c_code(&layout.name);
                format!("struct {code_prefix}_{name}")
            }
            FieldType::Fixed(_) => continue,
        };

        let field_name = c_code(&field.name);
        generate_doxy_comment(out, inp, &field.docs, "  ", None)?;

        // Members are bitifields, if configured:
        if inp.opts.registers_as_bitfields && !matches!(field.accepts, FieldType::Layout(_)) {
            writeln!(out, "  {field_type} {field_name}: {};", mask_width(field.mask))?;
        } else {
            writeln!(out, "  {field_type} {field_name};",)?;
        }
    }
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  int dummy; // Register contains no variable fields.",)?;
    }

    writeln!(out, "}};")?;
    Ok(())
}

/// Generate register packing/unpacking funcs
fn generate_layout_funcs(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    if !is_enabled(inp, Element::StructConversionFuncs) {
        return Ok(());
    }

    generate_layout_pack_func(out, inp, layout)?;
    generate_layout_try_unpack_func(out, inp, layout)?;
    if layout.can_always_unpack() {
        generate_layout_unpack_func(out, inp, layout)?;
    }

    Ok(())
}

fn generate_layout_pack_func(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    // Strings/Properties:
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&layout.name);
    let func_prefix = func_prefix(inp);
    let width_bytes = layout.width_bytes();
    let endi = func_endianess_str(inp);

    // Doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some(format!("Convert @ref struct {code_prefix}_{code_name} struct to packed value.")),
        doc: None,
    };
    generate_doxy_comment(out, inp, &docs, "", Some("any unspecified fields are left untouched"))?;

    // Function:
    let func_sig = format!(
        "{}void {}_{}_pack{}(const struct {}_{} *r, uint8_t val[{}])",
        func_prefix, code_prefix, code_name, endi, code_prefix, code_name, width_bytes
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;

    // Pack each field:
    for field in layout.fields.values() {
        let field_name = c_code(&field.name);

        writeln!(out, "  // {} @ {code_name}[{}]:", field.name, mask_to_bit_ranges_str(field.mask))?;

        match &field.accepts {
            FieldType::UInt | FieldType::Bool | FieldType::Enum(_) => {
                // Numeric field that can be directly converted:
                for byte in 0..width_bytes {
                    let Some(transform) = field_to_packed_byte_transform(
                        inp.opts.endian,
                        unpositioned_mask(field.mask),
                        lsb_pos(field.mask),
                        byte,
                        width_bytes,
                    ) else {
                        continue;
                    };

                    let field_byte = match &transform.shift {
                        Some((ShiftDirection::Left, amnt)) => format!("(r->{field_name} << {amnt})"),
                        Some((ShiftDirection::Right, amnt)) => format!("(r->{field_name} >> {amnt})"),
                        None => format!("r->{field_name}"),
                    };

                    writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", transform.mask)?;
                    writeln!(out, "  val[{byte}] |= (uint8_t)(((uint8_t){field_byte}) & 0x{:X}U);", transform.mask)?;
                }
            }

            FieldType::Fixed(fixed) => {
                // Fixed value:
                for byte in 0..width_bytes {
                    let mask_byte = grab_byte(inp.opts.endian, field.mask, byte, width_bytes);
                    let value_byte = grab_byte(inp.opts.endian, *fixed << lsb_pos(field.mask), byte, width_bytes);
                    if mask_byte == 0 {
                        continue;
                    };

                    writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", mask_byte)?;
                    writeln!(out, "  val[{byte}] |= (uint8_t)0x{value_byte:x}; // Fixed value.")?;
                }
            }

            FieldType::Layout(sublayout) => {
                // Sub-layout has to delegate to other pack function:

                let array_name = c_code(&field.name);
                let array_len = sublayout.width_bytes();
                let code_sublayout_name = c_code(&sublayout.name);
                let function_prefix = format!("{code_prefix}_{code_sublayout_name}");

                writeln!(out, "  uint8_t {array_name}[{array_len}] = {{0}};")?;
                writeln!(out, "  {function_prefix}_pack(&r->{field_name}, {});", c_code(&field.name))?;

                for byte in 0..width_bytes {
                    for field_byte in 0..array_len {
                        // Determine required transform to put byte 'field_byte' of field into 'byte' of
                        // output:
                        let transform = field_byte_to_packed_byte_transform(
                            inp.opts.endian,
                            sublayout.occupied_mask(),
                            lsb_pos(field.mask),
                            field_byte,
                            sublayout.width_bytes(),
                            byte,
                            width_bytes,
                        );

                        let Some(transform) = transform else {
                            continue;
                        };

                        let field_byte = format!("{array_name}[{field_byte}]");
                        let field_byte = match &transform.shift {
                            Some((ShiftDirection::Left, amnt)) => format!("({field_byte} << {amnt})"),
                            Some((ShiftDirection::Right, amnt)) => format!("({field_byte} >> {amnt})"),
                            None => field_byte,
                        };

                        writeln!(out, "  val[{byte}] &= (uint8_t)~0x{:X}U;", transform.mask)?;
                        writeln!(out, "  val[{byte}] |= (uint8_t)((uint8_t){field_byte} & 0x{:X}U);", transform.mask)?;
                    }
                }
            }
        }
    }

    // Prevent unused args warnings:
    if layout.fields.is_empty() {
        writeln!(out, "  (void)val;")?;
    }
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  (void)r;")?;
    }

    writeln!(out, "}}",)?;

    Ok(())
}

fn generate_layout_try_unpack_func(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    // Strings:
    let struct_name = format!("{}_{}", c_code(&inp.map.name), c_code(&layout.name));
    let func_prefix = func_prefix(inp);
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&layout.name);
    let endi = func_endianess_str(inp);

    let width_bytes = layout.width_bytes();

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Attempt to convert packed binary value to struct.".to_string()),
        doc: Some(
            "@returns 0 if succesfull.\n@returns the position of the field that could not be unpacked plus one, if enums can not represent content.".to_string(),
        ),
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Function signature
    let func_sig = format!(
        "{}int {}_try_unpack{}(const uint8_t val[{}], struct {} *r)",
        func_prefix, struct_name, endi, width_bytes, struct_name
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;

    // Unpack each field:
    for field in layout.fields_with_content() {
        let code_field_name = c_code(&field.name);
        let error_code = lsb_pos(field.mask) + 1;

        writeln!(out, "  // {} @ {code_name}[{}]:", field.name, mask_to_bit_ranges_str(field.mask))?;

        match &field.accepts {
            FieldType::UInt | FieldType::Bool => {
                // Numeric fields can be directly converted:
                let numeric_value = assemble_numeric_field(inp, layout, field)?;

                writeln!(out, "  r->{code_field_name} = {numeric_value};")?;
            }
            FieldType::Enum(e) => {
                // Enums may require validation:
                let enum_name = c_code(&e.name);

                let numeric_value = assemble_numeric_field(inp, layout, field)?;

                if field.can_always_unpack() {
                    writeln!(out, "  r->{code_field_name} = (enum {code_prefix}_{enum_name})({numeric_value});")?;
                } else {
                    let unsigned_type = c_fitting_unsigned_type(e.min_bitdwith())?;
                    writeln!(out, "  {unsigned_type} {code_field_name} = ({unsigned_type})({numeric_value});")?;
                    writeln!(
                        out,
                        "  if (!{code_prefix}_is_{enum_name}_enum({code_field_name})) {{ return {error_code}; }}",
                    )?;
                    writeln!(out, "  r->{code_field_name} = (enum {code_prefix}_{enum_name}){code_field_name};")?;
                }
            }
            FieldType::Layout(sublayout) => {
                // Sub-layout has to delegate to other unpack function:
                let array_len = sublayout.width_bytes();
                let code_sublayout_name = c_code(&sublayout.name);

                // Array to contain unpacked/binary sublayout:
                writeln!(out, "  uint8_t {code_field_name}[{array_len}] = {{0}};")?;

                for byte in 0..width_bytes {
                    for field_byte in 0..array_len {
                        // Determine required transform to put byte 'byte' of packed input into 'field_byte' of
                        // field:
                        let transform = packed_byte_to_field_byte_transform(
                            inp.opts.endian,
                            sublayout.occupied_mask(),
                            lsb_pos(field.mask),
                            field_byte,
                            array_len,
                            byte,
                            width_bytes,
                        );

                        let Some(transform) = transform else {
                            continue;
                        };

                        let masked = format!("(val[{byte}] & 0x{:X}U)", transform.mask);
                        let shifted = match &transform.shift {
                            Some((ShiftDirection::Left, amnt)) => format!("{masked} << {amnt}"),
                            Some((ShiftDirection::Right, amnt)) => format!("{masked} >> {amnt}"),
                            None => masked,
                        };

                        writeln!(out, "  {code_field_name}[{field_byte}] |= (uint8_t)({shifted});")?;
                    }
                }

                let function_prefix = format!("{code_prefix}_{code_sublayout_name}");

                // Unpack using sublayout's function:
                if sublayout.can_always_unpack() {
                    writeln!(out, "  r->{code_field_name} = {function_prefix}_unpack({code_field_name});",)?;
                } else {
                    writeln!(
                        out,
                        "  if ({function_prefix}_try_unpack(&r->{code_field_name}, {code_field_name})) {{ return {error_code} }};",
                    )?;
                }
            }
            FieldType::Fixed(_) => unreachable!(),
        }
    }

    // Prevent unused args warnings:
    if layout.fields_with_content().count() == 0 {
        writeln!(out, "  (void)val;")?;
        writeln!(out, "  (void)r;")?;
    }

    writeln!(out, "  return 0;")?;
    writeln!(out, "}}")?;
    Ok(())
}

fn generate_layout_unpack_func(out: &mut dyn Write, inp: &Input, layout: &Layout) -> Result<(), Error> {
    // Strings:
    let struct_name = format!("{}_{}", c_code(&inp.map.name), c_code(&layout.name));
    let func_prefix = func_prefix(inp);
    let endi = func_endianess_str(inp);

    let width_bytes = layout.width_bytes();

    // doxy comment:
    writeln!(out)?;
    let docs = Docs {
        brief: Some("Convert packed binary value to struct.".to_string()),
        doc: None,
    };
    generate_doxy_comment(out, inp, &docs, "", None)?;

    // Function signature
    let func_sig = format!(
        "{}struct {} {}_unpack{}(const uint8_t val[{}])",
        func_prefix, struct_name, struct_name, endi, width_bytes
    );

    if inp.opts.funcs_as_prototypes {
        writeln!(out, "{func_sig};")?;
        return Ok(());
    }

    writeln!(out, "{func_sig} {{")?;
    writeln!(out, "  // All possible layout field values can be unpacked without error.")?;
    writeln!(out, "  struct {struct_name} r = {{0}};")?;
    writeln!(out, "  (void) {struct_name}_try_unpack(val, &r);")?;
    writeln!(out, "  return r;")?;
    writeln!(out, "}}")?;
    Ok(())
}

/// Generate generic macros:
fn generate_generic_macros(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    if !is_enabled(inp, Element::GenericMacros) {
        return Ok(());
    }

    let mut out = HeaderWriter::new(out);
    out.push_section_with_header(&["\n", &c_section_header_comment("Generic Macros"), "\n"]);

    let docs = Docs {
        brief: Some("Convert struct to packed binary value.".to_string()),
        doc: Some("All non-field/always write bits are left untouched.".to_string()),
    };
    generate_generic_macro(&mut out, inp, &docs, "pack", "_struct_ptr_, _val_", inp.map.layouts.values())?;

    let docs = Docs {
       brief: Some("Attempt to convert packed binary value to struct.".to_string()),
       doc: Some(
           "@returns 0 if succesfull.\n@returns the position of the field that could not be unpacked plus one, if enums can not represent content.".to_string(),
       ),
    };
    generate_generic_macro(&mut out, inp, &docs, "try_unpack", "_val_, _struct_ptr_", inp.map.layouts.values())?;

    out.pop_section();

    Ok(())
}

fn generate_generic_macro<'a>(
    out: &mut dyn Write,
    inp: &Input,
    docs: &Docs,
    func_name_suffix: &str,
    func_args: &str,
    layouts: impl Iterator<Item = &'a Rc<Layout>>,
) -> Result<(), Error> {
    let macro_name_suffix = c_macro(func_name_suffix);
    let macro_prefix = c_macro(&inp.map.name);
    let code_prefix = c_code(&inp.map.name);
    let endi = func_endianess_str(inp).to_uppercase();

    // doxy comment:
    writeln!(out)?;
    generate_doxy_comment(out, inp, docs, "", None)?;

    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_{macro_name_suffix}{endi}({func_args}) _Generic((_struct_ptr_),"));

    for layout in layouts {
        let struct_name = format!("{}_{}", code_prefix, c_code(&layout.name));
        macro_lines.push(format!("    struct {struct_name}* : {struct_name}_{func_name_suffix},"));
    }
    let last_line = macro_lines.pop().unwrap().replace(',', "");
    macro_lines.push(last_line);
    macro_lines.push(format!("  )({func_args})"));
    generate_multiline_macro(out, macro_lines)?;
    Ok(())
}

/// Generate multi-line macro with allgined newline-escape slashes:
fn generate_multiline_macro(out: &mut dyn Write, mut lines: Vec<String>) -> Result<(), Error> {
    if !lines.is_empty() {
        let last_line = lines.pop().unwrap();
        for line in lines {
            writeln!(out, "{}\\", str_pad_to_length(&line, ' ', 99))?;
        }
        writeln!(out, "{last_line}")?;
    }
    Ok(())
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

/// Generate register section header comment
fn generate_register_header(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    let name = &register.name;
    writeln!(out)?;
    c_generate_section_header_comment(out, &format!("{name} Register"))?;
    if !register.docs.is_empty() {
        write!(out, "{}", register.docs.as_multiline("// "))?;
    }
    if is_enabled(inp, Element::Structs) {
        writeln!(out, "// Fields:")?;
        writeln!(out, "{}", c_layout_overview_comment(&register.layout))?;
    }
    Ok(())
}

fn generate_register_properties(out: &mut dyn Write, inp: &Input, register: &Register) -> Result<(), Error> {
    if !is_enabled(inp, Element::RegisterProperties) {
        return Ok(());
    }

    let mut defines: Vec<Vec<String>> = vec![];

    let macro_prefix = c_macro(&inp.map.name);
    let reg_macro_prefix = format!("{macro_prefix}_{}", c_macro(&register.name));
    let endi = func_endianess_str(inp).to_uppercase();

    // Address:
    defines.push(vec![
        format!("#define {}_ADDRESS", reg_macro_prefix),
        format!("(0x{:X}U)", register.adr),
        format!("//!< {} register address", register.name),
    ]);

    // Reset value:
    if let Some(reset_val) = &register.reset_val {
        defines.push(vec![
            format!("#define {}_RESET{}", reg_macro_prefix, endi),
            to_array_init(inp, *reset_val, register.layout.width_bytes()),
            format!("//!< {} register reset value", register.name),
        ]);
    }

    if !defines.is_empty() {
        writeln!(out)?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

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

fn generate_register_block_properties(out: &mut dyn Write, inp: &Input, block: &RegisterBlock) -> Result<(), Error> {
    if !is_enabled(inp, Element::RegisterProperties) {
        return Ok(());
    }

    let macro_prefix = c_macro(&inp.map.name);
    let macro_block_name = c_macro(&block.name);

    let mut defines = vec![];
    for member in block.members.values() {
        let macro_member_name = c_macro(&member.name);
        defines.push(vec![
            format!("#define {}_{}_OFFSET", macro_prefix, macro_member_name),
            format!("(0x{:X}U)", member.offset),
            format!("//!< Offset of {} register from {} block start", member.name, block.name),
        ]);
    }

    if !defines.is_empty() {
        writeln!(out)?;
        writeln!(out, "// Contained registers:")?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    let mut defines = vec![];
    for instance in block.instances.values() {
        let macro_instance_name = c_macro(&instance.name);
        defines.push(vec![
            format!("#define {}_{}_INSTANCE_{}", macro_prefix, macro_block_name, macro_instance_name),
            format!("(0x{:X}U)", instance.adr),
            format!("//!< Start of {} instance {}", block.name, instance.name),
        ]);
    }

    if !defines.is_empty() {
        writeln!(out)?;
        writeln!(out, "// Instances:")?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}

fn generate_register_block_member_header(
    out: &mut dyn Write,
    inp: &Input,
    member: &RegisterBlockMember,
) -> Result<(), Error> {
    let name = &member.name;
    writeln!(out)?;
    c_generate_header_comment(out, &format!("{name} Register Block Member "))?;

    if !member.docs.is_empty() {
        write!(out, "{}", member.docs.as_multiline("// "))?;
    }

    if is_enabled(inp, Element::Structs) {
        writeln!(out, "// Fields:")?;
        writeln!(out, "{}", c_layout_overview_comment(&member.layout))?;
    }
    Ok(())
}

fn generate_register_block_member_properties(
    out: &mut dyn Write,
    inp: &Input,
    member: &RegisterBlockMember,
    block: &RegisterBlock,
) -> Result<(), Error> {
    if !is_enabled(inp, Element::RegisterProperties) {
        return Ok(());
    }

    let macro_prefix = c_macro(&inp.map.name);

    let mut defines = vec![];
    for block_instance in block.instances.values() {
        let member_instance = &block_instance.registers[&member.name];
        let reg_macro_prefix = format!("{macro_prefix}_{}", c_macro(&member_instance.name));

        // Address:
        defines.push(vec![
            format!("#define {}_ADDRESS", reg_macro_prefix),
            format!("(0x{:X}U)", member_instance.adr),
            format!("//!< {} register address", member_instance.name),
        ]);

        // Reset value:
        if let Some(reset_val) = &member_instance.reset_val {
            defines.push(vec![
                format!("#define {}_RESET", reg_macro_prefix),
                to_array_init(inp, *reset_val, member_instance.layout.width_bytes()),
                format!("//!< {} register reset value", member_instance.name),
            ]);
        }
    }

    if !defines.is_empty() {
        writeln!(out)?;
        write!(out, "{}", str_table(&defines, "", " "))?;
    }

    Ok(())
}

// ====== Generator Utils ======================================================

/// Decide what each function should be prefixed with, depending on
/// given opts.
fn func_prefix(inp: &Input) -> &'static str {
    if inp.opts.funcs_static_inline {
        "static inline "
    } else {
        ""
    }
}

fn func_endianess_str(inp: &Input) -> &'static str {
    match (&inp.opts.endianess_in_names, &inp.opts.endian) {
        (true, Endianess::Little) => "_le",
        (true, Endianess::Big) => "_be",
        (false, _) => "",
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

fn assemble_numeric_field(inp: &Input, layout: &Layout, field: &LayoutField) -> Result<String, Error> {
    let layout_width_bytes = layout.width_bytes();

    let field_bitwidth = match &field.accepts {
        FieldType::UInt => mask_width(field.mask),
        FieldType::Bool => 1,
        FieldType::Enum(e) => e.min_bitdwith(),
        FieldType::Fixed(_) => unreachable!(),
        FieldType::Layout(_) => unreachable!(),
    };

    let pre_cast = if field_bitwidth <= 8 {
        String::new()
    } else {
        format!("({})", c_fitting_unsigned_type(mask_width(field.mask))?)
    };

    let mut unpacked_value: Vec<String> = vec![];
    for byte in 0..layout_width_bytes {
        let Some(transform) = packed_byte_to_field_transform(
            inp.opts.endian,
            unpositioned_mask(field.mask),
            lsb_pos(field.mask),
            byte,
            layout_width_bytes,
        ) else {
            continue;
        };

        let masked = if pre_cast.is_empty() {
            format!("(val[{byte}] & 0x{:X}U)", transform.mask)
        } else {
            format!("({pre_cast}(val[{byte}] & 0x{:X}U))", transform.mask)
        };

        match &transform.shift {
            Some((ShiftDirection::Left, amnt)) => unpacked_value.push(format!("({masked} << {amnt})")),
            Some((ShiftDirection::Right, amnt)) => unpacked_value.push(format!("({masked} >> {amnt})")),
            None => unpacked_value.push(masked),
        };
    }
    assert!(!unpacked_value.is_empty());

    let unpacked_value = unpacked_value.join(" | ");

    let post_cast = match &field.accepts {
        FieldType::UInt => format!("({})", c_fitting_unsigned_type(mask_width(field.mask))?),
        FieldType::Bool => String::from("(bool)"),
        FieldType::Enum(_) => String::new(),
        FieldType::Fixed(_) => unreachable!(),
        FieldType::Layout(_) => unreachable!(),
    };

    if post_cast.is_empty() {
        Ok(unpacked_value.to_string())
    } else {
        Ok(format!("{post_cast}({unpacked_value})"))
    }
}
