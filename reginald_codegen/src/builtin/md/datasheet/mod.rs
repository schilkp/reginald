pub mod regdump;

use std::fmt::Write;

use crate::{
    bits::{bitmask_from_range, lsb_pos, mask_to_bit_ranges_str, msb_pos},
    error::Error,
    regmap::{
        access_str, DecodedField, FieldType, FlattenedLayoutField, Layout, LayoutField, Register, RegisterMap,
        TypeValue,
    },
};

use super::md_table;

pub fn generate(out: &mut dyn Write, map: &RegisterMap) -> Result<(), Error> {
    writeln!(out, "# {}", map.name)?;
    writeln!(out)?;
    writeln!(out, "## Register Map")?;
    generate_overview(out, map)?;

    writeln!(out)?;
    writeln!(out, "## Register Details")?;
    for register in map.registers.values() {
        generate_register_infos(out, map, register, None)?;
    }

    Ok(())
}

fn generate_overview(out: &mut dyn Write, map: &RegisterMap) -> Result<(), Error> {
    if let Some(input_file) = &map.from_file {
        writeln!(out)?;
        writeln!(out, "Generated from listing file: {}.", input_file.to_string_lossy())?;
    }
    if let Some(author) = &map.author {
        writeln!(out)?;
        writeln!(out, "Listing file author: {author}.")?;
    }
    if let Some(notice) = &map.notice {
        writeln!(out,)?;
        writeln!(out, "Listing file notice:")?;
        writeln!(out, "```")?;
        for line in notice.lines() {
            writeln!(out, "  {line}")?;
        }
        writeln!(out, "```")?;
    }

    let mut rows = vec![];
    rows.push(vec![
        "**Address**".to_string(),
        "**Register**".to_string(),
        "**Brief**".to_string(),
    ]);
    for reg in map.registers.values() {
        let adr = format!("0x{:X}", reg.adr);
        let name = reg.name.clone();
        let brief = reg.docs.brief.clone().unwrap_or("-".to_string());
        rows.push(vec![adr, name, brief]);
    }
    writeln!(out)?;
    md_table(out, &rows, "")?;

    Ok(())
}

fn generate_register_infos(
    out: &mut dyn Write,
    map: &RegisterMap,
    register: &Register,
    value: Option<TypeValue>,
) -> Result<(), Error> {
    // Header:
    writeln!(out)?;
    writeln!(out, "### {}", register.name)?;

    writeln!(out)?;
    writeln!(out, "#### Info:")?;
    writeln!(out)?;
    if let Some(value) = value {
        writeln!(out, "  - **Current Value: 0x{value:X}**")?;
    }
    writeln!(out, "  - Address: 0x{:X}", register.adr)?;
    if let Some(reset_val) = register.reset_val {
        writeln!(out, "  - Reset: 0x{reset_val:X}")?;
    }

    writeln!(out)?;
    writeln!(out, "#### Register:")?;
    writeln!(out)?;

    generate_layout_table(out, &register.layout, value)?;

    writeln!(out)?;
    write!(out, "{}", register.docs.as_twoline("  - "))?;

    if let Some(from_block) = &register.from_block {
        writeln!(out, "  - In '{}' instance of '{}' block", from_block.instance, from_block.block)?;
        let block = &map.register_blocks[&from_block.block];
        let member = &block.members[&from_block.block_member];
        let instance = &block.instances[&from_block.instance];

        writeln!(out, "    - Offset from block start: 0x{:X}", member.offset)?;
        writeln!(out, "    - Instance {} start address: 0x{:X}", instance.name, instance.adr)?;
    }

    let sublayouts: Vec<FlattenedLayoutField> = register
        .layout
        .nested_fields()
        .iter()
        .filter(|&x| matches!(&x.field.accepts, FieldType::Layout(_)))
        .cloned()
        .collect();

    if !sublayouts.is_empty() {
        writeln!(out)?;
        writeln!(out, "#### Structured Fields:")?;

        for sublayout in sublayouts {
            let name = sublayout.name.join(".");
            let bits = mask_to_bit_ranges_str(sublayout.mask);
            let sublayout_value = value.map(|x| (x & sublayout.mask) >> lsb_pos(sublayout.mask));

            let FieldType::Layout(subfield_layout) = &sublayout.field.accepts else {
                unreachable!();
            };

            writeln!(out)?;
            writeln!(out, " - [{bits}]: {name}")?;
            if let Some(value) = sublayout_value {
                writeln!(out, "    - **Current Value: 0x{value:X}**")?;
            }
            writeln!(out)?;
            generate_layout_table(out, subfield_layout, sublayout_value)?;
        }
    }

    // Field Info:
    writeln!(out)?;
    writeln!(out, "#### Fields:")?;
    writeln!(out)?;

    for field in register.layout.nested_fields() {
        let value_field = value.map(|x| (x & field.mask) >> lsb_pos(field.mask));

        let indent = field.name.len();
        let indent = String::from_iter(std::iter::repeat("  ").take(indent));

        let value_string = value_field.map(|x| format!(": 0x{x:X}")).unwrap_or_default();
        let bits = mask_to_bit_ranges_str(field.mask);

        let name = field.name.join(".");

        writeln!(out, "{indent}- {name} [{bits}]{value_string}")?;
        write!(out, "{}", field.field.docs.as_twoline(&format!("{indent}  - ")))?;

        if let Some(access) = &field.field.access {
            writeln!(out, "{indent} - [{}]", access_str(access))?;
        }

        match &field.field.accepts {
            FieldType::UInt => {
                writeln!(out, "{indent} - Type: uint")?;
            }
            FieldType::Bool => {
                writeln!(out, "{indent} - Type: bool")?;
            }
            FieldType::Fixed(fix) => {
                writeln!(out, "{indent} - Type: fixed 0x{fix:X}")?;
            }
            FieldType::Layout(l) => {
                writeln!(out, "{indent} - Type: struct {}", l.name)?;
            }
            FieldType::Enum(e) => {
                writeln!(out, "{indent} - Type: enum {}", e.name)?;
                for entry in e.entries.values() {
                    match value_field {
                        Some(val_field) if val_field == entry.value => {
                            writeln!(out, "{indent}   - **0x{:X}: {} (SELECTED)**", entry.value, entry.name)?;
                        }
                        _ => {
                            writeln!(out, "{indent}   - 0x{:X}: {}", entry.value, entry.name)?;
                        }
                    }
                    write!(out, "{}", entry.docs.as_twoline("        - "))?;
                }
            }
        }
    }

    Ok(())
}

fn generate_layout_table(out: &mut dyn Write, layout: &Layout, value: Option<TypeValue>) -> Result<(), Error> {
    // Overview table:
    let ranges = layout.split_to_bitranges();

    let mut row_bits: Vec<String> = vec!["**Bits:**".to_string()];
    let mut row_field: Vec<String> = vec!["**Field:**".to_string()];
    let mut row_access: Vec<String> = vec!["**Access:**".to_string()];
    let mut row_state: Vec<String> = vec!["**State:**".to_string()];
    let mut row_decode: Vec<String> = vec!["**Decode:**".to_string()];

    for range in ranges.iter().rev() {
        if range.bits.start() == range.bits.end() {
            row_bits.push(format!("{}", range.bits.end()));
        } else {
            row_bits.push(format!("{}:{}", range.bits.end(), range.bits.start()));
        }

        if let Some(content) = &range.content {
            if let Some(access) = &content.field.access {
                row_access.push(access_str(access));
            } else {
                row_access.push("/".into());
            }

            if content.is_split {
                let lsb = lsb_pos(content.subfield_mask);
                let msb = msb_pos(content.subfield_mask);
                if lsb == msb {
                    row_field.push(format!("{}[{}]", content.field.name, msb));
                } else {
                    row_field.push(format!("{}[{}:{}]", content.field.name, msb, lsb));
                }
            } else {
                row_field.push(content.field.name.clone());
            }
        } else {
            row_field.push("/".into());
            row_access.push("/".into());
        }

        if let Some(value) = value {
            let value_range = (value & bitmask_from_range(&range.bits)) >> range.bits.start();
            row_state.push(format!("**0b{value_range:b}**"));
            if let Some(content) = &range.content {
                let value_field = (value & content.field.mask) >> lsb_pos(content.field.mask);
                row_decode.push(decode_field(value_field, content.field));
            } else {
                row_decode.push(String::new());
            }
        }
    }

    if value.is_some() {
        md_table(out, &vec![row_bits, row_field, row_access, row_state, row_decode], "")
    } else {
        md_table(out, &vec![row_bits, row_field, row_access], "")
    }
}

fn decode_field(field_value: TypeValue, field: &LayoutField) -> String {
    match field.decode_value(field_value) {
        Ok(DecodedField::UInt(_)) => String::new(),
        Ok(DecodedField::Bool(b)) => if b { "**true**" } else { "**false**" }.to_string(),
        Ok(DecodedField::EnumEntry(e)) => format!("**{e}**"),
        Ok(DecodedField::Fixed { val: _, is_correct }) => {
            if is_correct {
                "**OK**".to_string()
            } else {
                "**ERROR**".to_string()
            }
        }
        Err(_) => "**ERROR**".to_string(),
    }
}
