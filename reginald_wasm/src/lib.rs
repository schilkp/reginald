use reginald_codegen::regmap::listing::RegisterMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum ListingFormat {
    Yaml,
    Json,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum Endianness {
    Little,
    Big,
}

// TODO: Could not figure out how to make this an impl via bindgen?
#[wasm_bindgen]
pub fn listing_format_to_string(inp: ListingFormat) -> String {
    match inp {
        ListingFormat::Yaml => "yaml",
        ListingFormat::Json => "json",
    }
    .to_string()
}

#[wasm_bindgen]
pub fn is_parseable_listing(inp: String, format: ListingFormat) -> bool {
    match format {
        ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
        ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
    }
    .is_ok()
}

#[wasm_bindgen]
pub fn convert_listing_format(
    inp: String,
    in_format: ListingFormat,
    out_format: ListingFormat,
) -> Result<String, String> {
    let map: RegisterMap = match in_format {
        ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
        ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
    }
    .map_err(|e| e.to_string())?;

    match out_format {
        ListingFormat::Yaml => map.to_yaml(),
        ListingFormat::Json => map.to_json(),
    }
    .map_err(|e| e.to_string())
}

// ==== C.FUNCPACK  ============================================================

mod c_funcpack {
    use std::{collections::HashSet, path::Path};

    use crate::{Endianness, ListingFormat};
    use reginald_codegen::{
        builtin::c::funcpack::{Element, GeneratorOpts, generate},
        regmap::{RegisterMap, TypeBitwidth},
        utils::Endianness as ActualEndianness,
    };
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Copy, Clone, Default)]
    pub enum EndiannessImpl {
        Little,
        Big,
        #[default]
        Both,
    }

    #[wasm_bindgen]
    #[derive(Default)]
    pub struct CFuncpackOpts {
        pub endianness: EndiannessImpl,
        pub defer_to_endianness: Option<Endianness>,
        pub registers_as_bitfields: bool,
        pub max_enum_bitwidth: TypeBitwidth,
        #[wasm_bindgen(skip)]
        pub add_include: Vec<String>,
        pub funcs_static_inline: bool,
        pub funcs_as_prototypes: bool,
        pub clang_format_guard: bool,
        pub include_guards: bool,
        pub gen_enums: bool,
        pub gen_enum_validation: bool,
        pub gen_structs: bool,
        pub gen_struct_conv: bool,
        pub gen_reg_properties: bool,
        pub gen_generics: bool,
    }

    #[wasm_bindgen]
    impl CFuncpackOpts {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_include_push(&mut self, value: String) {
            self.add_include.push(value);
        }
    }

    #[wasm_bindgen]
    pub fn run_c_funcpack(inp: String, in_format: ListingFormat, wasm_opts: CFuncpackOpts) -> Result<String, String> {
        let map: RegisterMap = match in_format {
            ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
            ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
        }
        .map_err(|e| e.to_string())?;

        let endian = match wasm_opts.endianness {
            EndiannessImpl::Little => vec![ActualEndianness::Little],
            EndiannessImpl::Big => vec![ActualEndianness::Big],
            EndiannessImpl::Both => vec![ActualEndianness::Little, ActualEndianness::Big],
        };

        let defer_to_endian = wasm_opts.defer_to_endianness.map(|x| match x {
            Endianness::Little => ActualEndianness::Little,
            Endianness::Big => ActualEndianness::Big,
        });

        let mut to_generate: HashSet<Element> = HashSet::new();
        if wasm_opts.gen_enums {
            to_generate.insert(Element::Enums);
        }
        if wasm_opts.gen_enum_validation {
            to_generate.insert(Element::EnumValidationMacros);
        }
        if wasm_opts.gen_structs {
            to_generate.insert(Element::Structs);
        }
        if wasm_opts.gen_struct_conv {
            to_generate.insert(Element::StructConversionFuncs);
        }
        if wasm_opts.gen_reg_properties {
            to_generate.insert(Element::RegisterProperties);
        }
        if wasm_opts.gen_generics {
            to_generate.insert(Element::GenericMacros);
        }

        let opts: GeneratorOpts = GeneratorOpts {
            endian,
            defer_to_endian,
            registers_as_bitfields: wasm_opts.registers_as_bitfields,
            max_enum_bitwidth: wasm_opts.max_enum_bitwidth,
            add_include: wasm_opts.add_include,
            funcs_static_inline: wasm_opts.funcs_static_inline,
            funcs_as_prototypes: wasm_opts.funcs_as_prototypes,
            clang_format_guard: wasm_opts.clang_format_guard,
            include_guards: wasm_opts.include_guards,
            to_generate,
        };

        let mut out = String::new();
        generate(&mut out, &map, Path::new(&format!("{}.h", map.name)), opts).map_err(|e| e.to_string())?;

        Ok(out)
    }
}

// ==== C.MACROMAP  ============================================================

mod c_macromap {
    use std::path::Path;

    use crate::ListingFormat;
    use reginald_codegen::{
        builtin::c::macromap::{GeneratorOpts, generate},
        regmap::RegisterMap,
    };
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Default)]
    pub struct CMacromapOpts {
        #[wasm_bindgen(skip)]
        pub add_include: Vec<String>,
        pub clang_format_guard: bool,
    }

    #[wasm_bindgen]
    impl CMacromapOpts {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        pub fn add_include_push(&mut self, value: String) {
            self.add_include.push(value);
        }
    }

    #[wasm_bindgen]
    pub fn run_c_macromap(inp: String, in_format: ListingFormat, wasm_opts: CMacromapOpts) -> Result<String, String> {
        let map: RegisterMap = match in_format {
            ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
            ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
        }
        .map_err(|e| e.to_string())?;

        let opts: GeneratorOpts = GeneratorOpts {
            add_include: wasm_opts.add_include,
            clang_format_guard: wasm_opts.clang_format_guard,
        };

        let mut out = String::new();
        generate(&mut out, &map, Path::new(&format!("{}.h", map.name)), &opts).map_err(|e| e.to_string())?;

        Ok(out)
    }
}

// ==== MD.DATASHEET  ==========================================================

mod md_datasheet {
    use crate::ListingFormat;
    use reginald_codegen::{builtin::md::datasheet::generate, regmap::RegisterMap};
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn run_md_datasheet(inp: String, in_format: ListingFormat) -> Result<String, String> {
        let map: RegisterMap = match in_format {
            ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
            ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
        }
        .map_err(|e| e.to_string())?;

        let mut out = String::new();
        generate(&mut out, &map).map_err(|e| e.to_string())?;

        Ok(out)
    }
}

// ==== RS.STRUCTS  ============================================================

mod rs_struts {
    use crate::ListingFormat;
    use reginald_codegen::{
        builtin::rs::structs::{GeneratorOpts, generate},
        regmap::RegisterMap,
    };
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Default)]
    pub struct RsStructsOpts {
        #[wasm_bindgen(skip)]
        pub address_type: String,
        #[wasm_bindgen(skip)]
        pub struct_derive: Vec<String>,
        #[wasm_bindgen(skip)]
        pub enum_derive: Vec<String>,
        #[wasm_bindgen(skip)]
        pub add_use: Vec<String>,
        #[wasm_bindgen(skip)]
        pub add_attribute: Vec<String>,
        #[wasm_bindgen(skip)]
        pub external_traits: String,
        pub generate_uint_conversion: bool,
    }

    #[wasm_bindgen]
    impl RsStructsOpts {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Self {
            Self::default()
        }

        #[wasm_bindgen(setter)]
        pub fn set_address_type(&mut self, value: String) {
            self.address_type = value;
        }

        #[wasm_bindgen(getter)]
        pub fn address_type(&self) -> String {
            self.address_type.clone()
        }

        #[wasm_bindgen(setter)]
        pub fn set_external_traits(&mut self, value: String) {
            self.external_traits = value;
        }

        #[wasm_bindgen(getter)]
        pub fn external_traits(&self) -> String {
            self.external_traits.clone()
        }

        pub fn struct_derive_push(&mut self, value: String) {
            self.struct_derive.push(value);
        }

        pub fn enum_derive_push(&mut self, value: String) {
            self.enum_derive.push(value);
        }

        pub fn add_use_push(&mut self, value: String) {
            self.add_use.push(value);
        }

        pub fn add_attribute_push(&mut self, value: String) {
            self.add_attribute.push(value);
        }
    }

    #[wasm_bindgen]
    pub fn run_rs_structs(inp: String, in_format: ListingFormat, wasm_opts: RsStructsOpts) -> Result<String, String> {
        let map: RegisterMap = match in_format {
            ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
            ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
        }
        .map_err(|e| e.to_string())?;

        let opts: GeneratorOpts = GeneratorOpts {
            address_type: if wasm_opts.address_type.is_empty() {
                None
            } else {
                Some(wasm_opts.address_type)
            },
            struct_derive: wasm_opts.struct_derive,
            raw_enum_derive: wasm_opts.enum_derive,
            add_use: wasm_opts.add_use,
            add_attribute: wasm_opts.add_attribute,
            external_traits: if wasm_opts.external_traits.is_empty() {
                None
            } else {
                Some(wasm_opts.external_traits)
            },
            generate_uint_conversion: wasm_opts.generate_uint_conversion,
        };

        let mut out = String::new();
        generate(&mut out, &map, &opts).map_err(|e| e.to_string())?;

        Ok(out)
    }
}
