use std::collections::HashSet;

use super::{Docs, Enum, EnumEntry, Field, FieldType, Register, TypeBitwidth, TypeValue, MAX_BITWIDTH};
use crate::bits::{fits_into_bitwidth, mask_width, unpositioned_mask};
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

    // TODO This is only required for the rust output ATM. Should this be a
    // general limitation, or specific to the rust output?
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

pub fn validate_field_type(field: &Field, bt: &str) -> Result<(), Error> {
    match &field.accepts {
        FieldType::UInt => Ok(()),
        FieldType::Bool => {
            if mask_width(field.mask) == 1 {
                Ok(())
            } else {
                Err(Error::ConversionError {
                    bt: bt.to_owned() + ".accepts",
                    msg: format!("Field {} accepts a boolean but is more than one bit wide!", field.name),
                })
            }
        }
        FieldType::LocalEnum(field_enum) => validate_field_enum(field, field_enum.entries.values(), bt),
        FieldType::SharedEnum(shared_enun) => validate_field_enum(field, shared_enun.entries.values(), bt),
    }
}

pub fn validate_field_enum<'a>(
    field: &Field,
    entries: impl Iterator<Item = &'a EnumEntry>,
    bt: &str,
) -> Result<(), Error> {
    for entry in entries {
        let overshoot = !(unpositioned_mask(field.mask)) & entry.value;
        if overshoot != 0 {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".accepts." + &entry.name,
                msg: format!(
                    "Enum value 0x{:x} for entry {} does not fit into field {} (unpositioned mask: 0x{:x})!",
                    entry.value, entry.name, field.name, overshoot
                ),
            });
        }
    }
    Ok(())
}

pub fn validate_register_template(template: Register, bt: &str) -> Result<Register, Error> {
    validate_bitwidth(template.bitwidth, bt)?;

    // Validate that reset value fits into register:
    if let Some(reset_val) = &template.reset_val {
        if !fits_into_bitwidth(*reset_val, template.bitwidth) {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".reset_val",
                msg: format!("Reset value 0x{:x} does not fit into a {}-bit register!", reset_val, template.bitwidth),
            });
        }
    }

    let mut occupied_mask: TypeValue = 0;

    for field in template.fields.values() {
        let bt = bt.to_owned() + ".fields." + &field.name;

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

        // Validate that the field fits into the register:
        if !fits_into_bitwidth(field.mask, template.bitwidth) {
            return Err(Error::ConversionError {
                bt: bt + ".bits",
                msg: format!(
                    "Field with bits 0x{:x} does not fit into a {}-bit register!",
                    field.mask, template.bitwidth
                ),
            });
        }

        validate_field_type(field, &bt)?;
    }

    if let Some(always_write) = &template.always_write {
        // Validate that always write mask fits into register:
        if !fits_into_bitwidth(always_write.mask, template.bitwidth) {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".always_write.mask",
                msg: format!(
                    "Always-write mask 0x{:x} does not fit into a {}-bit register!",
                    always_write.mask, template.bitwidth
                ),
            });
        }

        // Validate that always write mask does not overlap with registers:
        let overlap = always_write.mask & occupied_mask;
        if overlap != 0 {
            return Err(Error::ConversionError {
                bt: bt.to_owned() + ".fields",
                msg: format!(
                    "Always-write mask 0x{:x} covers bits already occupied by other fields (overlap mask: 0x{:x})!.",
                    always_write.mask, overlap
                ),
            });
        }
    }

    Ok(template)
}

#[cfg(test)]
mod tests {
    use crate::regmap::RegisterMap;

    use super::*;

    #[test]
    fn test_catch_zero_bitwidth() {
        let err = validate_bitwidth(0, "".into()).unwrap_err();
        assert!(format!("{}", err).contains("zero"));
    }

    #[test]
    fn test_catch_large_bitwidth() {
        // Max:
        validate_bitwidth(MAX_BITWIDTH, "".into()).unwrap();
        // Outside max:
        let err = validate_bitwidth(MAX_BITWIDTH + 1, "".into()).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("greater than the maximal bitwidth"));
    }

    #[test]
    fn test_catch_large_bitpos() {
        // Max:
        validate_bitpos(MAX_BITWIDTH - 1, "".into()).unwrap();
        // Outside max:
        let err = validate_bitpos(MAX_BITWIDTH, "".into()).unwrap_err();
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
            "".into(),
        )
        .unwrap();

        // Empty brief:
        validate_docs(
            Docs {
                brief: Some("".into()),
                doc: None,
            },
            "".into(),
        )
        .unwrap_err();

        // Empty doc:
        validate_docs(
            Docs {
                brief: None,
                doc: Some("".into()),
            },
            "".into(),
        )
        .unwrap_err();

        // multiline brief
        validate_docs(
            Docs {
                brief: Some("Hi\nThere!".into()),
                doc: None,
            },
            "".into(),
        )
        .unwrap_err();
    }

    #[test]
    fn test_catch_bad_reset_value() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                reset_val: 0x100

        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("Reset value"));
        assert!(format!("{}", err).contains("does not fit"));
    }

    #[test]
    fn test_catch_large_field() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                fields:
                    A:
                        bits: [8]
        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit"));
    }

    #[test]
    fn test_catch_field_overlap() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                fields:
                    A:
                        bits: [\"0-7\"]
                    B:
                        bits: [3]

        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("that are already occupied"));
    }

    #[test]
    fn test_catch_bad_always_write() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                always_write:
                    - { bits: [8], val: 1 }
        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit into"));

        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                always_write:
                    - { bits: [1], val: 0x10 }

        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit into specified bits/mask"));

        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                always_write:
                    - { bits: [0], val : 0x0 }
                fields:
                    A:
                        bits: [\"7-0\"]

        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("covers bits already occupied by other"));
    }

    #[test]
    fn test_catch_bad_enum() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            REG: !Register
                fields:
                    A:
                        bits: [4,5]
                        accepts: !LocalEnum
                            A:
                                val: 0x4
        ";
        let err = RegisterMap::from_yaml_str(&yaml).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit into"));

        let hjson = "
        {
            map_name: \"DummyChip\",
            default_register_bitwidth: 8,
            registers: {
                REG: {
                    Register: {
                        fields: {
                            A: {
                                bits: [3,4]
                                accepts: {
                                    SharedEnum: \"MyEnum\"
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
        let err = RegisterMap::from_hjson_str(&hjson).unwrap_err();
        println!("{}", err);
        assert!(format!("{}", err).contains("does not fit into"));
    }
}
