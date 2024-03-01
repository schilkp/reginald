use std::{collections::HashMap, path::PathBuf};

use reginald_codegen::{
    builtin::{
        c::funcpack,
        md::datasheet::{self, regdump::RegDump},
    },
    regmap::RegisterMap,
};

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../examples/maps/max77654.yaml");
    let reader = std::fs::File::open(path).unwrap();
    let map = RegisterMap::from_yaml(reader).unwrap();

    // let mut out = String::new();
    // funcpack::generate(
    //     &mut out,
    //     &map,
    //     &PathBuf::from("max77654.h"),
    //     &funcpack::GeneratorOpts {
    //         field_enum_prefix: false,
    //         registers_as_bitfields: true,
    //         clang_format_guard: true,
    //         generate_enums: true,
    //         generate_registers: true,
    //         generate_register_functions: true,
    //         generate_generic_macros: true,
    //         generate_validation_functions: true,
    //         add_include: vec![],
    //     },
    // )
    // .unwrap();
    // print!("{}", out);

    // let mut out = String::new();
    // macromap::generate(
    //     &mut out,
    //     &map,
    //     &PathBuf::from("max77654.h"),
    //     &macromap::GeneratorOpts {
    //         clang_format_guard: true,
    //         add_include: vec![],
    //     },
    // )
    // .unwrap();
    // print!("{}", out);

    // let mut out = String::new();
    // datasheet::generate(&mut out, &map).unwrap();
    // print!("{}", out);

    let mut out = String::new();
    datasheet::regdump::generate(
        &mut out,
        &map,
        &RegDump::from([(0, 0x1F), (1, 0x3F), (0x18, 0x1F)]),
    )
    .unwrap();
    print!("{}", out);
}
