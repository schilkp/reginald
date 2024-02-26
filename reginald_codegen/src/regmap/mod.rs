use std::{collections::BTreeMap, io, path::PathBuf, rc::Rc};

use self::convert::convert_map;
use crate::error::Error;

pub mod bits;
mod convert;
mod listing;
mod validate;

pub type TypeValue = u64;
pub type TypeBitwidth = u32;
pub const MAX_BITWIDTH: TypeBitwidth = 64;
pub type TypeAdr = u64;

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

impl Docs {
    pub fn is_empty(&self) -> bool {
        self.brief.is_none() && self.doc.is_none()
    }

    pub fn as_multiline(&self, prefix: &str) -> String {
        let mut out = String::new();
        if let Some(brief) = &self.brief {
            out.push_str(prefix);
            out.push_str(brief);
            out.push('\n');
        }
        if let Some(doc) = &self.doc {
            for line in doc.lines() {
                out.push_str(prefix);
                out.push_str(line);
                out.push('\n');
            }
        }

        out
    }
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
    pub entries: BTreeMap<String, EnumEntry>,
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
    pub fields: BTreeMap<String, Field>,
    pub bitwidth: TypeBitwidth,
    pub is_block_template: bool,
    pub adr: Option<TypeAdr>,
    pub always_write: Option<AlwaysWrite>,
    pub reset_val: Option<TypeValue>,
    pub docs: Docs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Instance {
    pub name: String,
    pub adr: Option<TypeAdr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterBlock {
    pub name: String,
    pub instances: BTreeMap<String, Instance>,
    pub docs: Docs,
    pub register_templates: BTreeMap<String, Register>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisterMap {
    pub from_file: Option<PathBuf>,
    pub map_name: String,
    pub docs: Docs,
    pub register_blocks: BTreeMap<String, RegisterBlock>,
    pub shared_enums: BTreeMap<String, Rc<Enum>>,
}

impl RegisterMap {
    pub fn from_yaml<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read, {
        let listing = listing::RegisterMap::from_yaml(inp)?;
        convert_map(&listing, &None)
    }

    pub fn from_yaml_str(inp: &str) -> Result<Self, Error> {
        Self::from_yaml(inp.as_bytes())
    }

    pub fn from_hjson<R>(inp: R) -> Result<Self, Error>
    where
        R: io::Read,
    {
        let listing = listing::RegisterMap::from_hjson(inp)?;
        convert_map(&listing, &None)
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
