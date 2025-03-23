use std::collections::{BTreeMap, HashSet};
use std::ops::Deref;
use std::sync::LazyLock;

use super::{Docs, Enum, FieldType, Layout, LayoutField, MAX_BITWIDTH, Register, TypeBitwidth, TypeValue};
use crate::bits::{bitmask_from_width, fits_into_bitwidth};
use crate::error::Error;
use regex::Regex;

static NAME_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[_a-zA-Z]").unwrap());

pub fn validate_name(name: &str, bt: &str, bt_extra: &str) -> Result<(), Error> {
    if !name.is_empty() && !NAME_REGEX.is_match(name) {
        return Err(Error::ConversionError {
            bt: bt.to_owned() + bt_extra,
            msg: "Name may only begin with an ASCII letter or an underscore ([_a-zA-Z})".to_owned(),
        });
    }
    Ok(())
}

pub struct NamespaceEntry {
    pub name: String,
    pub origin: String,
}

pub type Namespace = BTreeMap<String, NamespaceEntry>;

pub fn validate_name_unique(name: &str, namespace: &mut Namespace, bt: &str) -> Result<(), Error> {
    let key = name.to_lowercase();

    if let Some(existing_entry) = namespace.get(&key) {
        return Err(Error::ConversionError {
            bt: bt.to_owned(),
            msg: format!(
                "Name '{}' collides with name '{}' defined at '{}'",
                name, existing_entry.name, existing_entry.origin
            ),
        });
    }

    let entry = NamespaceEntry {
        name: name.to_string(),
        origin: bt.to_string(),
    };

    namespace.insert(key, entry);

    Ok(())
}

pub fn validate_map_author(author: &Option<String>, bt: &str) -> Result<(), Error> {
    if let Some(author) = author {
        if author.contains('\n') {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".author",
                msg: "Author may not contain more than one line.".to_owned(),
            });
        }
    }
    Ok(())
}

pub fn validate_bitwidth(bitwidth: TypeBitwidth, bt: &str) -> Result<(), Error> {
    if bitwidth == 0 {
        return Err(Error::ConversionError {
            bt: bt.to_owned(),
            msg: "Bitwidth may not be zero.".into(),
        });
    }

    if bitwidth > MAX_BITWIDTH {
        Err(Error::ConversionError {
            bt: bt.to_owned(),
            msg: format!("Bitwidth of {bitwidth} is greater than the maximal bitwidth {MAX_BITWIDTH}!"),
        })
    } else {
        Ok(())
    }
}

pub fn validate_bitpos(bitpos: TypeBitwidth, bt: &str) -> Result<(), Error> {
    if bitpos >= MAX_BITWIDTH {
        Err(Error::ConversionError {
            bt: bt.to_owned(),
            msg: format!("Bit position {bitpos} is outside the maximal bitwidth {MAX_BITWIDTH}!"),
        })
    } else {
        Ok(())
    }
}

pub fn validate_docs(docs: Docs, bt: &str) -> Result<Docs, Error> {
    if let Some(brief_content) = &docs.brief {
        if brief_content.is_empty() {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".brief",
                msg: "Empty string".to_owned(),
            });
        }

        if brief_content.contains('\n') {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".brief",
                msg: "Brief may not contain more than one line.".to_owned(),
            });
        }
    };

    if let Some(doc_content) = &docs.doc {
        if doc_content.is_empty() {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".doc",
                msg: "Empty string".into(),
            });
        }
    };

    Ok(docs)
}

pub fn validate_enum(e: &Enum, bt: &str) -> Result<(), Error> {
    if e.entries.is_empty() {
        return Err(Error::ConversionError {
            bt: bt.to_owned(),
            msg: "Empty enum.".into(),
        });
    }

    let mut enum_vals: HashSet<TypeValue> = HashSet::new();

    for entry in e.entries.values() {
        if !enum_vals.insert(entry.value) {
            return Err(Error::ConversionError {
                bt: bt.to_owned(),
                msg: format!("Enum contains multiple entries with value 0x{:X}", entry.value),
            });
        }
    }

    for entry in e.entries.values() {
        let max_value = bitmask_from_width(e.bitwidth);

        if (entry.value & (!max_value)) != 0 {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + "." + &entry.name,
                msg: format!(
                    "Enum value 0x{:x} for entry {} does not fit into {}-bit enum!",
                    entry.value, entry.name, e.bitwidth
                ),
            });
        }
    }

    Ok(())
}

fn validate_field_type(field: &LayoutField, bt: &str) -> Result<(), Error> {
    let bt = bt.to_owned() + ".accepts";

    let field_width = field.bits.width();

    match &field.accepts {
        FieldType::UInt => (),
        FieldType::Bool => {
            if field.bits.width() != 1 {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!("Field {} accepts a boolean but is more than one bit wide!", field.name),
                });
            }
        }
        FieldType::Enum(e) => {
            if field.bits.width() != e.bitwidth {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "Field {} is {} bits wide, and therefor incompatible with the {} bit wide Enum {}",
                        field.name, field_width, e.bitwidth, e.name
                    ),
                });
            }
            validate_enum(e, &bt)?;
        }
        FieldType::Fixed(val) => {
            // Check if fixed value fits in the field width
            if field_width >= MAX_BITWIDTH {
                // Field is maximum size, so all values fit
                return Ok(());
            }

            // Generate maximum value that fits in field_width bits
            let max_value = if field_width == 0 {
                0
            } else if field_width >= 64 {
                u64::MAX
            } else {
                (1u64 << field_width) - 1
            };

            if *val > max_value {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "Fixed value 0x{:x} does not fit into field {} (max value: 0x{:x})!",
                        val, field.name, max_value
                    ),
                });
            }
        }
        FieldType::Layout(l) => {
            // TODO: LAYOUTS SHOULD ALSO BE CONT.!
            if l.bitwidth > field_width {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "{}-bit field {} cannot hold {}-bit layout {}!",
                        field_width, field.name, l.bitwidth, l.name,
                    ),
                });
            };

            // Check if the layout's occupied bits fit within the field width
            if field_width >= MAX_BITWIDTH {
                // Field is maximum size, so all values fit
                return Ok(());
            }

            // Generate maximum value that fits in field_width bits
            let max_value = if field_width == 0 {
                0
            } else if field_width >= 64 {
                u64::MAX
            } else {
                (1u64 << field_width) - 1
            };

            if l.occupied_mask() > max_value {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "Layout {} does not fit into field {} (max value: 0x{:x})!",
                        l.name, field.name, max_value
                    ),
                });
            }
        }
    };
    Ok(())
}

fn find_layout_loop(layout: &Layout, mut visited_layouts: HashSet<String>, bt: &str) -> Result<(), Error> {
    if visited_layouts.contains(&layout.name) {
        return Err(Error::ConversionError {
            bt: bt.to_owned(),
            msg: format!("Circular layout loop containing layout {} ", layout.name,),
        });
    }

    visited_layouts.insert(layout.name.clone());

    for field in layout.fields.values() {
        if let FieldType::Layout(accepted_layout) = &field.accepts {
            let bt = bt.to_owned() + " -> " + &field.name;
            find_layout_loop(accepted_layout, visited_layouts.clone(), &bt)?;
        }
    }

    Ok(())
}

pub fn validate_layout(layout: &Layout, bt: &str) -> Result<(), Error> {
    let mut occupied_bits = HashSet::new();

    for field in layout.fields.values() {
        let bt = bt.to_owned() + ".fields." + &field.name;

        // Validate that field fits into layout:
        if *field.bits.end() >= layout.bitwidth {
            return Err(Error::ConversionError {
                bt,
                msg: format!("Field {} is outside the {}-bit layout.", field.name, layout.bitwidth),
            });
        }

        validate_field_type(field, &bt)?;

        // Validate that no fields overlap by checking for any overlapping bits
        for bit_pos in field.bits.deref().clone() {
            if occupied_bits.contains(&bit_pos) {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "Field {} located at bits that are already occupied (bit position: {})",
                        field.name, bit_pos
                    ),
                });
            }
            occupied_bits.insert(bit_pos);
        }
    }

    find_layout_loop(layout, HashSet::new(), bt)?;

    Ok(())
}

pub fn validate_register_properties(
    layout: &Layout,
    bitwidth: TypeBitwidth,
    reset_val: Option<TypeValue>,
    bt: &str,
) -> Result<(), Error> {
    validate_bitwidth(bitwidth, bt)?;

    // Validate that the field fits into the register:
    for field in layout.fields.values() {
        if *field.bits.end() >= bitwidth {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".bits",
                msg: format!(
                    "Field with bit position {} does not fit into a {}-bit register!",
                    field.bits.end(),
                    bitwidth
                ),
            });
        }
    }

    // Validate that reset value fits into register:
    if let Some(reset_val) = reset_val {
        if !fits_into_bitwidth(reset_val, bitwidth) {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".reset_val",
                msg: format!("Reset value 0x{:x} does not fit into a {}-bit register!", reset_val, bitwidth),
            });
        }
    }

    Ok(())
}

pub fn validate_register(reg: &Register, bt: &str) -> Result<(), Error> {
    validate_register_properties(&reg.layout, reg.layout.bitwidth, reg.reset_val, bt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::regmap::RegisterMap;

    use super::*;

    #[test]
    fn test_catch_zero_bitwidth() {
        let err = validate_bitwidth(0, "").unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("zero"));
    }

    #[test]
    fn test_catch_large_bitwidth() {
        // Max:
        validate_bitwidth(MAX_BITWIDTH, "").unwrap();
        // Outside max:
        let err = validate_bitwidth(MAX_BITWIDTH + 1, "").unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("greater than the maximal bitwidth"));
    }

    #[test]
    fn test_catch_large_bitpos() {
        // Max:
        validate_bitpos(MAX_BITWIDTH - 1, "").unwrap();
        // Outside max:
        let err = validate_bitpos(MAX_BITWIDTH, "").unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("outside the maximal bitwidth"));
    }

    #[test]
    fn test_doc_validation() {
        validate_docs(
            Docs {
                brief: Some("Test".into()),
                doc: Some("Test".into()),
            },
            "",
        )
        .unwrap();

        // Empty brief:
        validate_docs(
            Docs {
                brief: Some("".into()),
                doc: None,
            },
            "",
        )
        .unwrap_err();

        // Empty doc:
        validate_docs(
            Docs {
                brief: None,
                doc: Some("".into()),
            },
            "",
        )
        .unwrap_err();

        // multiline brief
        validate_docs(
            Docs {
                brief: Some("Hi\nThere!".into()),
                doc: None,
            },
            "",
        )
        .unwrap_err();
    }

    #[test]
    fn test_catch_bad_reset_value() {
        let yaml = "
        name: DummyChip
        defaults:
            layout_bitwidth: 8
        registers:
            REG: !Register
                adr: 0x01
                reset_val: 0x100
                layout: !Layout

        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("Reset value"));
        assert!(format!("{}", err).contains("does not fit"));
    }

    #[test]
    fn test_catch_large_field() {
        let yaml = "
        name: DummyChip
        defaults:
            layout_bitwidth: 8
        registers:
            REG: !Register
                adr: 0x100
                layout: !Layout
                    A:
                        bits: 8
        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("is outside the 8-bit layout"));
    }

    #[test]
    fn test_catch_field_overlap() {
        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                adr: 0x1
                bitwidth: 8
                layout: !Layout
                    A:
                        bits: \"0-7\"
                    B:
                        bits: 3

        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("that are already occupied"));
    }

    #[test]
    fn test_catch_bad_enum() {
        let yaml = "
        name: DummyChip
        registers:
            REG: !Register
                bitwidth: 8
                adr: 0x1000
                layout: !Layout
                    A:
                        bits: \"4-5\"
                        accepts: !Enum
                            A:
                                val: 0x4
        ";
        let err = RegisterMap::from_yaml_str(yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit into"));

        let hjson = "
        {
            name: \"DummyChip2\",
            defaults: {
                layout_bitwidth: 8,
            },
            registers: {
                REG: {
                    Register: {
                        adr: 10,
                        layout: {
                            Layout: {
                                A: {
                                    bits: \"3-4\"
                                    accepts: {
                                        SharedEnum: \"MyEnum\"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            enums: {
                MyEnum: {
                    bitwidth: 2,
                    enum: {
                        A: {
                            val: 0
                        },
                        B: {
                            val: 15
                        }
                    }
                }
            }
        }
        ";
        let err = RegisterMap::from_hjson_str(hjson).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit into"));
    }
}
