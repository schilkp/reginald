use std::{fs, path::PathBuf, process::Command};

use reginald_codegen::{
    builtin::rs::{self, structs::GeneratorOpts},
    regmap::RegisterMap,
};

use crate::{TEST_MAP_FILE, print_cmd_output};

// ==== Utils ==================================================================

fn run_reginald(output_name: &str, opts: GeneratorOpts) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join(PathBuf::from("tests/generator_rs_structs/test_proj/src/"));
    let output_file = output_dir.join(output_name);

    let map = RegisterMap::from_file(&TEST_MAP_FILE).unwrap();

    let mut out = String::new();
    rs::structs::generate(&mut out, &map, &opts).unwrap();

    // Write to output file:
    fs::create_dir_all(&output_dir).unwrap();
    fs::write(output_file, &out).unwrap();
}

// ==== Tests ==================================================================

#[test]
#[cfg_attr(not(feature = "test_gen_output"), ignore)]
fn generator_rs_structs() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_proj = manifest_dir.join(PathBuf::from("tests/generator_rs_structs/test_proj/"));

    run_reginald(
        "out.rs",
        GeneratorOpts {
            struct_derive: vec!["Debug".to_string(), "Clone".to_string()],
            raw_enum_derive: vec!["Debug".to_string(), "PartialEq".to_string()],
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        "out_ext_traits.rs",
        GeneratorOpts {
            external_traits: Some("crate::out::".to_string()),
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        "out_crate_traits.rs",
        GeneratorOpts {
            external_traits: Some("reginald::".to_string()),
            ..GeneratorOpts::default()
        },
    );

    let output = Command::new("cargo")
        .args(&["test".to_string()])
        .current_dir(&test_proj)
        .output()
        .unwrap();
    print_cmd_output(&output);
    assert!(output.status.success());

    let output = Command::new("cargo")
        .args(&["clippy".to_string()])
        .args(&["--".to_string()])
        .args(&["-D".to_string()])
        .args(&["warnings".to_string()])
        .current_dir(&test_proj)
        .output()
        .unwrap();
    print_cmd_output(&output);
    assert!(output.status.success());
}
