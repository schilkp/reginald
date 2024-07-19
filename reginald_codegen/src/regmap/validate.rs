use std::collections::{BTreeMap, HashSet};

use super::{Docs, Enum, FieldType, Layout, LayoutField, Register, TypeBitwidth, TypeValue, MAX_BITWIDTH};
use crate::bits::{bitmask_from_width, fits_into_bitwidth, mask_width, unpositioned_mask};
use crate::error::Error;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"^[_a-zA-Z]").unwrap();
}

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

    Ok(())
}

fn validate_field_type(field: &LayoutField, bt: &str) -> Result<(), Error> {
    let bt = bt.to_owned() + ".accepts";
    match &field.accepts {
        FieldType::UInt => (),
        FieldType::Bool => {
            if mask_width(field.mask) != 1 {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!("Field {} accepts a boolean but is more than one bit wide!", field.name),
                });
            }
        }
        FieldType::Enum(e) => {
            for entry in e.entries.values() {
                let overshoot = (!unpositioned_mask(field.mask)) & entry.value;
                if overshoot != 0 {
                    return Err(Error::ConversionError {
                        bt: bt + "." + entry.name.as_str(),
                        msg: format!(
                            "Enum value 0x{:x} for entry {} does not fit into field {} (unpositioned mask: 0x{:x})!",
                            entry.value, entry.name, field.name, overshoot
                        ),
                    });
                }
            }
        }
        FieldType::Fixed(val) => {
            let overshoot = (!unpositioned_mask(field.mask)) & val;
            if overshoot != 0 {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "Fixed value 0x{:x} for  does not fit into field {} (unpositioned mask: 0x{:x})!",
                        val, field.name, overshoot
                    ),
                });
            }
        }
        FieldType::Layout(l) => {
            if l.bitwidth > mask_width(field.mask) {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "{}-bit field {} cannot hold {}-bit layout {}!",
                        mask_width(field.mask),
                        field.name,
                        l.bitwidth,
                        l.name,
                    ),
                });
            };

            let overshoot = (!unpositioned_mask(field.mask)) & l.occupied_mask();
            if overshoot != 0 {
                return Err(Error::ConversionError {
                    bt,
                    msg: format!(
                        "Layout {} does not fit into field {} (unpositioned mask: 0x{:x})!",
                        l.name, field.name, overshoot
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
            let bt = bt.to_owned() + " -> " + field.name.as_str();
            find_layout_loop(accepted_layout, visited_layouts.clone(), &bt)?;
        }
    }

    Ok(())
}

pub fn validate_layout(layout: &Layout, bt: &str) -> Result<(), Error> {
    let mut occupied_mask: TypeValue = 0;

    for field in layout.fields.values() {
        let bt = bt.to_owned() + ".fields." + field.name.as_str();

        // Validate that field fits into layout:
        let overlap = field.mask & !bitmask_from_width(layout.bitwidth);
        if overlap != 0 {
            return Err(Error::ConversionError {
                bt,
                msg: format!("Field {} is outside the {}-bit layout.", field.name, layout.bitwidth),
            });
        }

        validate_field_type(field, &bt)?;

        // Validate that no fields overlap:
        let overlap = field.mask & occupied_mask;
        if overlap != 0 {
            return Err(Error::ConversionError {
                bt,
                msg: format!(
                    "Field {} located at bits that are already occupied (overlap mask: 0x{:x})",
                    field.name, overlap
                ),
            });
        }
        occupied_mask |= field.mask;
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
        if !fits_into_bitwidth(field.mask, bitwidth) {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".bits",
                msg: format!("Field with bits 0x{:x} does not fit into a {}-bit register!", field.mask, bitwidth),
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
                        bits: [8]
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
                        bits: [\"0-7\"]
                    B:
                        bits: [3]

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
                        bits: [4,5]
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
                                    bits: [3,4]
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
