use crate::{
    error::Error,
    regmap::{TypeAdr, TypeBitwidth, TypeValue},
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, io};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(untagged, deny_unknown_fields)]
pub enum BitRange {
    Bit(TypeBitwidth),
    Range(String),
}

pub type Bits = Vec<BitRange>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum AccessMode {
    R,
    W,
}

pub type Access = Vec<AccessMode>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnumEntry {
    pub val: TypeValue,
    pub doc: Option<String>,
    pub brief: Option<String>,
}

pub type EnumEntries = BTreeMap<String, EnumEntry>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
#[derive(Default)]
pub enum FieldType {
    #[default]
    UInt,
    Bool,
    LocalEnum(EnumEntries),
    SharedEnum(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub bits: Bits,
    #[serde(default = "Vec::new")]
    pub access: Access,
    pub doc: Option<String>,
    pub brief: Option<String>,
    #[serde(default)]
    pub accepts: FieldType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AlwaysWrite {
    pub mask: TypeValue,
    pub val: TypeValue,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct Register {
    #[serde(default = "BTreeMap::new")]
    pub fields: BTreeMap<String, Field>,
    pub adr: Option<TypeAdr>,
    pub bitwidth: Option<TypeBitwidth>,
    pub reset_val: Option<TypeValue>,
    pub always_write: Option<AlwaysWrite>,
    pub doc: Option<String>,
    pub brief: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RegisterBlock {
    pub instances: BTreeMap<String, Option<TypeAdr>>,
    pub doc: Option<String>,
    pub brief: Option<String>,
    pub registers: BTreeMap<String, Register>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum RegisterListing {
    Register(Register),
    Block(RegisterBlock),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SharedEnum {
    pub doc: Option<String>,
    pub brief: Option<String>,
    #[serde(rename = "enum")]
    pub entries: EnumEntries,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct RegisterMap {
    pub map_name: String,
    pub default_register_bitwidth: TypeBitwidth,
    pub doc: Option<String>,
    pub brief: Option<String>,
    pub note: Option<String>,
    pub author: Option<String>,
    #[serde(default = "BTreeMap::new")]
    pub registers: BTreeMap<String, RegisterListing>,
    #[serde(default = "BTreeMap::new")]
    pub enums: BTreeMap<String, SharedEnum>,
}

impl RegisterMap {
    pub fn from_yaml<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        Ok(serde_yaml::from_reader(inp)?)
    }

    pub fn from_hjson<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        Ok(deser_hjson::from_reader(inp)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;

    use deser_hjson;
    use lazy_static::lazy_static;
    use pretty_assertions::assert_eq;
    use serde_yaml;

    #[test]
    fn deser_yaml_bits() {
        let yaml = "[\"2-3\"]";
        let v: Bits = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(v, vec![BitRange::Range("2-3".into())]);

        let yaml = "[2]";
        let v: Bits = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(v, vec![BitRange::Bit(2)]);

        let yaml = "[1-3, 4, \"5-6\"]";
        let v: Bits = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(
            v,
            vec![
                BitRange::Range("1-3".into()),
                BitRange::Bit(4),
                BitRange::Range("5-6".into())
            ]
        );
    }

    #[test]
    fn deser_hjson_bits() {
        let hjson = "[\"2-3\"]";
        let v: Bits = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(v, vec![BitRange::Range("2-3".into())]);

        let hjson = "[2]";
        let v: Bits = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(v, vec![BitRange::Bit(2)]);

        let hjson = "[\"1-3\", 4, \"5-6\"]";
        let v: Bits = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(
            v,
            vec![
                BitRange::Range("1-3".into()),
                BitRange::Bit(4),
                BitRange::Range("5-6".into())
            ]
        );
    }

    #[test]
    fn deser_yaml_access() {
        let yaml = "['R']";
        let v: Access = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(v, vec![AccessMode::R]);

        let yaml = "[W]";
        let v: Access = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(v, vec![AccessMode::W]);

        let yaml = "['W', R]";
        let v: Access = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(v, vec![AccessMode::W, AccessMode::R]);
    }

    #[test]
    fn deser_hjson_access() {
        let hjson = "['R']";
        let v: Access = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(v, vec![AccessMode::R]);

        let hjson = "[W]";
        let v: Access = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(v, vec![AccessMode::W]);

        let hjson = "['W', R]";
        let v: Access = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(v, vec![AccessMode::W, AccessMode::R]);
    }

    #[test]
    fn deser_yaml_empty_map() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        ";
        let is: RegisterMap = serde_yaml::from_str(yaml).unwrap();
        let expect = RegisterMap {
            map_name: "DummyChip".to_string(),
            default_register_bitwidth: 8,
            ..Default::default()
        };
        assert_eq!(is, expect);
    }

    #[test]
    fn deser_hjson_empty_map() {
        let hjson = "
        map_name: DummyChip
        default_register_bitwidth: 8
        ";
        let is: RegisterMap = deser_hjson::from_str(hjson).unwrap();
        let expect = RegisterMap {
            map_name: "DummyChip".to_string(),
            default_register_bitwidth: 8,
            ..Default::default()
        };
        assert_eq!(is, expect);
    }

    lazy_static! {
        static ref SHARED_ENUM_EXPECT: RegisterMap = RegisterMap {
            map_name: "DummyChip".to_string(),
            default_register_bitwidth: 8,
            enums: BTreeMap::from([(
                "MyEnum".into(),
                SharedEnum {
                    doc: Some("Testdoc".into()),
                    brief: Some("very brief brief".into()),
                    entries: BTreeMap::from([(
                        "OFF".into(),
                        EnumEntry {
                            val: 0x0,
                            brief: Some("off".into()),
                            doc: Some("this is turned off".into()),
                        },
                    )]),
                },
            )]),
            ..Default::default()
        };
    }

    #[test]
    fn deser_yaml_shared_enums() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        enums:
            MyEnum:
                doc: Testdoc
                brief: very brief brief
                enum:
                    OFF:
                        val: 0
                        brief:  off
                        doc: this is turned off
        ";
        let is: RegisterMap = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(is, *SHARED_ENUM_EXPECT);
    }

    #[test]
    fn deser_hjson_shared_enums() {
        let hjson = "
        map_name: DummyChip
        default_register_bitwidth: 8
        enums: {
            MyEnum: {
                doc: Testdoc
                brief: very brief brief
                enum: {
                    OFF: {
                        val: 0
                        brief:  off
                        doc: this is turned off
                    }
                }
            }
        }
        ";
        let is: RegisterMap = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(is, *SHARED_ENUM_EXPECT);
    }

    lazy_static! {
        static ref BASIC_REGISTER_EXPECT: RegisterMap = RegisterMap {
            map_name: "DummyChip".to_string(),
            default_register_bitwidth: 8,
            registers: BTreeMap::from([(
                "FIFOCTRL4".into(),
                RegisterListing::Register(Register {
                    doc: Some("Testdoc".into()),
                    brief: Some("very brief brief".into()),
                    fields: BTreeMap::from([
                        (
                            "F7".into(),
                            Field {
                                bits: vec![BitRange::Bit(7)],
                                ..Default::default()
                            },
                        ),
                        (
                            "F1".into(),
                            Field {
                                bits: vec![BitRange::Bit(1)],
                                ..Default::default()
                            },
                        ),
                    ]),
                    ..Default::default()
                }),
            )]),
            ..Default::default()
        };
    }

    #[test]
    fn deser_yaml_basic_register() {
        let yaml = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers:
            FIFOCTRL4: !Register
                doc: Testdoc
                brief: very brief brief
                fields:
                    F7:
                        bits: [7]
                    F1:
                        bits: [1]
        ";
        let is: RegisterMap = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(is, *BASIC_REGISTER_EXPECT);
    }

    #[test]
    fn deser_hjson_basic_register() {
        let hjson = "
        map_name: DummyChip
        default_register_bitwidth: 8
        registers: {
            FIFOCTRL4: {
                Register: {
                    doc: Testdoc
                    brief: very brief brief
                    fields: {
                        F7: {
                            bits: [7]
                        },
                        F1: {
                            bits: [1]
                        }
                    }
                }
            }
        }
        ";
        let is: RegisterMap = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(is, *BASIC_REGISTER_EXPECT);
    }

    lazy_static! {
        static ref FIELD_ENUM_EXCEPT: Field = Field {
            bits: vec![BitRange::Bit(1)],
            access: vec![],
            doc: None,
            brief: None,
            accepts: FieldType::LocalEnum(BTreeMap::from([
                (
                    "A".into(),
                    EnumEntry {
                        val: 0x1,
                        doc: None,
                        brief: None,
                    },
                ),
                (
                    "B".into(),
                    EnumEntry {
                        val: 0x0,
                        doc: None,
                        brief: None,
                    },
                ),
            ])),
        };
    }

    #[test]
    fn deser_yaml_field_enum() {
        let yaml = "
        bits: [1]
        accepts: !LocalEnum
            A:
                val: 0x1
            B:
                val: 0x0
        ";
        let field_is: Field = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(field_is, *FIELD_ENUM_EXCEPT);
    }

    #[test]
    fn deser_hjson_field_enum() {
        let hjson = "
        bits: [1]
        accepts: {
            LocalEnum: {
                A: {
                    val: 1
                },
                B: {
                    val: 0
                },
            }
        }
        ";
        let field_is: Field = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(field_is, *FIELD_ENUM_EXCEPT);
    }

    lazy_static! {
        static ref FIELD_SHARED_ENUM_EXPECT: Field = Field {
            bits: vec![BitRange::Bit(1)],
            access: vec![],
            doc: None,
            brief: None,
            accepts: FieldType::SharedEnum("TestEnum".into()),
        };
    }

    #[test]
    fn deser_yaml_field_shared_enum() {
        let yaml = "
        bits: [1]
        accepts: !SharedEnum 'TestEnum'
        ";
        let field_is: Field = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(field_is, *FIELD_SHARED_ENUM_EXPECT);
    }

    #[test]
    fn deser_hjson_field_shared_enum() {
        let hjson = "
        bits: [1]
        accepts: {
            SharedEnum: 'TestEnum'
        }
        ";
        let field_is: Field = deser_hjson::from_str(hjson).unwrap();
        assert_eq!(field_is, *FIELD_SHARED_ENUM_EXPECT);
    }

    fn parse_yaml_example(file: &str) -> RegisterMap {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/maps/");
        path.push(file);
        let reader = std::fs::File::open(path).unwrap();
        RegisterMap::from_yaml(reader).unwrap()
    }

    fn parse_hjson_example(file: &str) -> RegisterMap {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/maps/");
        path.push(file);
        let reader = std::fs::File::open(path).unwrap();
        RegisterMap::from_hjson(reader).unwrap()
    }

    #[test]
    fn deser_example_dummy_yaml() {
        parse_yaml_example("dummy.yaml");
    }

    #[test]
    fn deser_example_dummy_hjson() {
        parse_hjson_example("dummy.hjson");
    }

    #[test]
    fn deser_example_max77654_yaml() {
        parse_yaml_example("max77654.yaml");
    }

    #[test]
    fn deser_example_max77654_hjson() {
        parse_hjson_example("max77654.hjson");
    }

    #[test]
    fn deser_example_lsm6dsv16bx_yaml() {
        parse_yaml_example("lsm6dsv16bx.yaml");
    }

    #[test]
    fn deser_example_lsm6dsv16bx_hjson() {
        parse_hjson_example("lsm6dsv16bx.hjson");
    }
}
