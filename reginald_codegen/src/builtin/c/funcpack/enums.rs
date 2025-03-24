use std::fmt::Write;

use crate::{
    error::Error,
    regmap::{Docs, Enum, TypeValue},
};

use reginald_utils::numbers_as_ranges;

use super::{Element, Input, c_code, c_generate_doxy_comment, c_macro, generate_multiline_macro};

/// Generate an enum
pub fn generate_enum(out: &mut dyn Write, inp: &Input, e: &Enum) -> Result<(), Error> {
    if !inp.opts.is_enabled(Element::Enums) {
        return Ok(());
    }

    let code_prefix = c_code(&inp.map.name);
    let macro_prefix = c_macro(&inp.map.name);

    let code_name = c_code(&e.name);
    let macro_name = c_macro(&e.name);

    if e.bitwidth <= inp.opts.max_enum_bitwidth {
        // Enum proper:
        writeln!(out)?;
        c_generate_doxy_comment(out, &e.docs, "", vec![])?;
        writeln!(out, "enum {code_prefix}_{code_name} {{")?;
        for entry in e.entries.values() {
            c_generate_doxy_comment(out, &entry.docs, "  ", vec![])?;
            writeln!(out, "  {}_{}_{} = 0x{:X}U,", macro_prefix, macro_name, c_macro(&entry.name), entry.value)?;
        }
        writeln!(out, "}};")?;
    } else {
        // Defines
        writeln!(out)?;
        c_generate_doxy_comment(out, &e.docs, "", vec![(String::from("name"), c_macro(&e.name))])?;
        writeln!(out, "///@{{")?;
        for entry in e.entries.values() {
            c_generate_doxy_comment(out, &entry.docs, "  ", vec![])?;
            writeln!(out, "#define {}_{}_{} (0x{:X}U)", macro_prefix, macro_name, c_macro(&entry.name), entry.value)?;
        }
        writeln!(out, "///@}}")?;
    }

    Ok(())
}

/// Generate an enum validation func
pub fn generate_enum_validation_macro(out: &mut dyn Write, inp: &Input, e: &Enum) -> Result<(), Error> {
    if !inp.opts.is_enabled(Element::EnumValidationMacros) {
        return Ok(());
    }

    // let code_prefix = c_code(&inp.map.name);
    let macro_prefix = c_macro(&inp.map.name);
    let code_prefix = c_code(&inp.map.name);
    let code_name = c_code(&e.name);
    let macro_name = c_macro(&e.name);

    let accept_values: Vec<TypeValue> = e.entries.values().map(|x| x.value).collect();
    let accept_ranges = numbers_as_ranges(accept_values);

    // Doxy comment:
    let enum_ref = if e.bitwidth > inp.opts.max_enum_bitwidth {
        c_macro(&e.name)
    } else {
        format!("enum {code_prefix}_{code_name}")
    };

    writeln!(out,)?;
    c_generate_doxy_comment(
        out,
        &Docs::default(),
        "",
        vec![
            (String::from("brief"), format!("Check if a numeric value is a valid @ref {enum_ref}.")),
            (String::from("returns"), String::from("bool (true/false)")),
        ],
    )?;

    let mut macro_lines: Vec<String> = vec![];
    macro_lines.push(format!("#define {macro_prefix}_IS_VALID_{macro_name}(_VAL_) ("));

    // Convert possible ranges to continous ranges, and generate a check for each range.
    for range in accept_ranges {
        match (range.start(), range.end()) {
            (&start, &end) if start == end => {
                macro_lines.push(format!("  ((_VAL_) == 0x{:X}U) ? true :", range.start()));
            }
            (0, &end) => {
                macro_lines.push(format!("  ((_VAL_) <= 0x{end:X}U) ? true : "));
            }
            (&start, &end) => {
                macro_lines.push(format!("  (0x{start:X}U <= (_VAL_) && (_VAL_) <= 0x{end:X}U) ? true :"));
            }
        }
    }

    macro_lines.push("  false )".to_string());

    generate_multiline_macro(out, macro_lines)?;

    Ok(())
}
