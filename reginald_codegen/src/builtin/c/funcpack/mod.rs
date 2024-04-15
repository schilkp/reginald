mod enums;
mod layouts;
mod registers;

use std::{fmt::Write, path::Path, rc::Rc};

#[cfg(feature = "cli")]
use clap::{Parser, ValueEnum};

use crate::{
    bits::{lsb_pos, mask_width, unpositioned_mask},
    error::Error,
    regmap::{Docs, FieldType, Layout, LayoutField, RegisterMap, TypeBitwidth, TypeValue},
    utils::{filename, packed_byte_to_field_transform, str_pad_to_length, Endianess, ShiftDirection},
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
    EnumValidationMacros,
    Structs,
    StructConversionFuncs,
    RegisterProperties,
    GenericMacros,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "cli", derive(Parser))]
pub struct GeneratorOpts {
    /// Generate functions and enums with the given endianess.
    ///
    /// May be given multiple times. If not specified, both endianess
    /// versions will be generated.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Append))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    #[cfg_attr(feature = "cli", arg(conflicts_with("dont_generate")))]
    pub endian: Vec<Endianess>,

    /// For other endianess, generate only simple functions that defers to this implementaiton.
    ///
    /// If generating both endianess versions, only generate one complete
    /// function implementation and have the other endianess defer to this
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub defer_to_endian: Option<Endianess>,

    /// Make register structs bitfields to reduce their memory size
    ///
    /// Note that their memory layout will not match the actual register
    /// and the (un)packing functions must still be used.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "true"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub registers_as_bitfields: bool,

    /// Max enum bitwidth before it is represented using macros instead of an enum. (default: 31)
    ///
    /// Set to zero to have all enums be represented using macros.
    #[cfg_attr(feature = "cli", arg(long))]
    #[cfg_attr(feature = "cli", arg(action = clap::ArgAction::Set))]
    #[cfg_attr(feature = "cli", arg(default_value = "31"))]
    #[cfg_attr(feature = "cli", arg(verbatim_doc_comment))]
    pub max_enum_bitwidth: TypeBitwidth,

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
    endian: Vec<Endianess>,
}

pub fn generate(out: &mut dyn Write, map: &RegisterMap, output_file: &Path, opts: &GeneratorOpts) -> Result<(), Error> {
    let mut endian = if opts.endian.is_empty() {
        Vec::from([Endianess::Little, Endianess::Big])
    } else {
        Vec::from_iter(opts.endian.iter().copied())
    };

    // If impls defer to a given endianess, sort to have that impl appear first:
    if let Some(defer_to) = &opts.defer_to_endian {
        endian.sort_by_key(|x| if x == defer_to { 0 } else { 1 })
    }

    let inp = Input {
        opts: opts.clone(),
        map,
        output_file,
        endian,
    };

    let mut out = HeaderWriter::new(out);

    generate_header(&mut out, &inp)?;

    // ===== Shared enums: =====
    out.push_section_with_header(&["\n", &c_section_header_comment("Shared Enums"), "\n"]);
    for e in map.shared_enums() {
        out.push_section_with_header(&["\n", &c_header_comment(&format!("{} Enum", e.name)), "\n"]);

        enums::generate_enum(&mut out, &inp, e)?;
        enums::generate_enum_validation_macro(&mut out, &inp, e)?;

        out.pop_section();
    }
    out.pop_section();

    // ===== Shared layouts: =====
    out.push_section_with_header(&["\n", &c_section_header_comment("Shared Layout Structs"), "\n"]);
    for layout in map.shared_layouts() {
        out.push_section_with_header(&["\n", &c_header_comment(&format!("{} Layout", layout.name)), "\n"]);
        if is_enabled(&inp, Element::Structs) {
            // Field details only in comment if struct is generated.
            writeln!(out, "// Fields:")?;
            writeln!(out, "{}", c_layout_overview_comment(layout))?;
        }
        layouts::generate_layout(&mut out, &inp, layout)?;
        out.pop_section();
    }
    out.pop_section();

    // ===== Individual Registers: =====
    for register in map.individual_registers() {
        registers::generate_register(&mut out, &inp, register)?;
    }

    // Register blocks:
    for block in map.register_blocks.values() {
        registers::generate_register_block(&mut out, &inp, block)?;
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
    if inp.opts.defer_to_endian.is_some() {
        writeln!(out, "#include <stddef.h>")?;
    }
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

/// Generate generic macros:
fn generate_generic_macros(out: &mut dyn Write, inp: &Input) -> Result<(), Error> {
    if !is_enabled(inp, Element::GenericMacros) {
        return Ok(());
    }

    let mut out = HeaderWriter::new(out);
    out.push_section_with_header(&["\n", &c_section_header_comment("Generic Macros"), "\n"]);

    for endian in &inp.endian {
        let docs = Docs {
            brief: Some("Convert struct to packed {endian} binary value.".to_string()),
            doc: Some("All non-field/always write bits are left untouched.".to_string()),
        };
        writeln!(out)?;
        c_generate_doxy_comment(&mut out, &docs, "", vec![])?;

        let func_suffix = format!("pack_{}", endian.short());
        generate_generic_macro(&mut out, inp, &func_suffix, "_struct_ptr_, _val_", inp.map.layouts.values())?;
    }

    let docs = Docs {
        brief: Some("Validate struct".to_string()),
        doc: Some("Confirms that all enums are valid, and all values fit into respective fields".to_string()),
    };
    writeln!(out)?;
    c_generate_doxy_comment(
        &mut out,
        &docs,
        "",
        vec![
            (String::from("returns"), String::from("0 if valid.")),
            (String::from("returns"), String::from("the position of the first invalid field if invalid.")),
        ],
    )?;
    generate_generic_macro(&mut out, inp, "validate", "_struct_ptr_", inp.map.layouts.values())?;

    for endian in &inp.endian {
        let docs = Docs {
            brief: Some("Attempt to convert packed {endian} binary value to struct.".to_string()),
            doc: None,
        };
        writeln!(out)?;
        c_generate_doxy_comment(
            &mut out,
            &docs,
            "",
            vec![
                (String::from("returns"), String::from("0 if valid.")),
                (String::from("returns"), String::from("the position of the first invalid field if invalid.")),
            ],
        )?;
        let func_suffix = format!("try_unpack_{}", endian.short());
        generate_generic_macro(&mut out, inp, &func_suffix, "_val_, _struct_ptr_", inp.map.layouts.values())?;
    }

    out.pop_section();

    Ok(())
}

fn generate_generic_macro<'a>(
    out: &mut dyn Write,
    inp: &Input,
    func_name_suffix: &str,
    func_args: &str,
    layouts: impl Iterator<Item = &'a Rc<Layout>>,
) -> Result<(), Error> {
    let macro_name_suffix = c_macro(func_name_suffix);
    let macro_prefix = c_macro(&inp.map.name);
    let code_prefix = c_code(&inp.map.name);

    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_{macro_name_suffix}({func_args}) _Generic((_struct_ptr_),"));

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

fn swap_loop(from: &str, to: &str, width_bytes: TypeBitwidth) -> String {
    format!("for(size_t i = 0; i < {width_bytes}; i++) {{ {to}[i] = {from}[{width_bytes}-(i+1)]; }}")
}

/// Convert a value to an array initialiser of correct endianess
fn to_array_init(val: TypeValue, width_bytes: TypeBitwidth, endian: Endianess) -> String {
    let mut bytes: Vec<String> = vec![];

    for i in 0..width_bytes {
        let byte = format!("0x{:X}U", ((val >> (8 * i)) & 0xFF) as u8);
        bytes.push(byte);
    }

    if matches!(endian, Endianess::Big) {
        bytes.reverse();
    }

    format!("{{{}}}", bytes.join(", "))
}

fn assemble_numeric_field(layout: &Layout, field: &LayoutField, endian: Endianess) -> Result<String, Error> {
    let layout_width_bytes = layout.width_bytes();

    let field_bitwidth = mask_width(field.mask);

    let pre_cast = if field_bitwidth <= 8 {
        String::new()
    } else {
        format!("({})", c_fitting_unsigned_type(mask_width(field.mask))?)
    };

    let mut unpacked_value: Vec<String> = vec![];
    for byte in 0..layout_width_bytes {
        let Some(transform) = packed_byte_to_field_transform(
            endian,
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
