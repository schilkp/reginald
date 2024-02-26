use std::path::PathBuf;

use reginald_codegen::{
    builtin::c::funcpack::{generate, GeneratorOpts},
    regmap::RegisterMap,
};

fn main() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../examples/maps/max77654.yaml");
    let reader = std::fs::File::open(path).unwrap();
    let map = RegisterMap::from_yaml(reader).unwrap();

    let mut out = String::new();
    generate(
        &mut out,
        &map,
        &PathBuf::from("max77654.h"),
        &GeneratorOpts {
            field_enum_prefix: false,
            registers_as_bitfields: true,
            clang_format_guard: true,
            generate_enums: true,
            generate_registers: true,
            generate_register_functions: true,
            generate_generic_macros: true,
            add_include: vec![],
        },
    );

    print!("{}", out);
}
