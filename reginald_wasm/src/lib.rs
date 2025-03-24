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
pub enum Endianess {
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

// ==== FUNCPACK  ==============================================================

mod c_funcpack {
    use std::{collections::HashSet, path::Path};

    use crate::{Endianess, ListingFormat};
    use reginald_codegen::{
        builtin::c::funcpack::{Element, GeneratorOpts, generate},
        regmap::{RegisterMap, TypeBitwidth},
        utils::Endianess as ActualEndianess,
    };
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    #[derive(Copy, Clone, Default)]
    pub enum EndianessImpl {
        Little,
        Big,
        #[default]
        Both,
    }

    #[wasm_bindgen]
    #[derive(Default)]
    pub struct CFuncpackOpts {
        pub endianess: EndianessImpl,
        pub defer_to_endianess: Option<Endianess>,
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
    pub fn run(inp: String, in_format: ListingFormat, wasm_opts: CFuncpackOpts) -> Result<String, String> {
        let map: RegisterMap = match in_format {
            ListingFormat::Yaml => RegisterMap::from_yaml_str(&inp),
            ListingFormat::Json => RegisterMap::from_hjson_str(&inp),
        }
        .map_err(|e| e.to_string())?;

        let endian = match wasm_opts.endianess {
            EndianessImpl::Little => vec![ActualEndianess::Little],
            EndianessImpl::Big => vec![ActualEndianess::Big],
            EndianessImpl::Both => vec![],
        };

        let defer_to_endian = wasm_opts.defer_to_endianess.map(|x| match x {
            Endianess::Little => ActualEndianess::Little,
            Endianess::Big => ActualEndianess::Big,
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
        generate(&mut out, &map, Path::new(&format!("{}.c", map.name)), opts).map_err(|e| e.to_string())?;

        Ok(out)
    }
}
