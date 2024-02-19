use std::{collections::HashMap, io, rc::Rc};

use self::convert::convert_map;
use crate::error::Error;

pub mod bits;
mod convert;
mod listing;
mod validate;

type TypeValue = u64;
type TypeBitwidth = u32;
const MAX_BITWIDTH: TypeBitwidth = 64;
type TypeAdr = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccessMode {
    R,
    W,
}

pub type Access = Vec<AccessMode>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Docs {
    pub brief: Option<String>,
    pub doc: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumEntry {
    pub name: String,
    pub value: TypeValue,
    pub docs: Docs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Enum {
    pub name: String,
    pub is_shared: bool,
    pub docs: Docs,
    pub entries: Vec<EnumEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldEnum {
    Local(Enum),
    Shared(Rc<Enum>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlwaysWrite {
    pub mask: TypeValue,
    pub value: TypeValue,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub mask: TypeValue,
    pub access: Option<Access>,
    pub docs: Docs,
    pub field_enum: Option<FieldEnum>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Register {
    pub name: String,
    pub fields: Vec<Field>,
    pub bitwidth: TypeBitwidth,
    pub is_block_template: bool,
    pub adr: Option<TypeAdr>,
    pub always_write: Option<AlwaysWrite>,
    pub reset_val: Option<TypeValue>,
    pub docs: Docs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    name: String,
    adr: Option<TypeAdr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterBlock {
    pub name: String,
    pub instances: Vec<Instance>,
    pub docs: Docs,
    pub register_templates: Vec<Register>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterMap {
    pub map_name: String,
    pub docs: Docs,
    pub register_blocks: Vec<RegisterBlock>,
    pub shared_enums: HashMap<String, Rc<Enum>>,
}

impl RegisterMap {
    pub fn from_yaml<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        let listing = listing::RegisterMap::from_yaml(inp)?;
        convert_map(&listing)
    }

    pub fn from_yaml_str(inp: &str) -> Result<Self, Error> {
        Self::from_yaml(inp.as_bytes())
    }

    pub fn from_hjson<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        let listing = listing::RegisterMap::from_hjson(inp)?;
        convert_map(&listing)
    }

    pub fn from_hjson_str(inp: &str) -> Result<Self, Error> {
        Self::from_hjson(inp.as_bytes())
    }
}

#[cfg(test)]
pub fn assert_regmap_eq(left: RegisterMap, right: RegisterMap) {
    use std::iter::zip;

    use pretty_assertions::assert_eq;
    assert_eq!(left.map_name, right.map_name);
    assert_eq!(left.docs, right.docs);
    for (left, right) in zip(&left.register_blocks, &right.register_blocks) {
        assert_eq!(left, right);
    }
    assert_eq!(left.register_blocks, right.register_blocks);
    for (name, left) in left.shared_enums {
        assert_eq!(left, *right.shared_enums.get(&name).unwrap());
    }
}
