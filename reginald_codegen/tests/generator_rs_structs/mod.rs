use std::{path::PathBuf, process::Command};

use reginald_codegen::{
    builtin::rs::structs::GeneratorOpts,
    cli::cmd::gen::{self, Generator},
};

use crate::{print_cmd_output, TEST_MAP_FILE};

// ==== Utils ==================================================================

fn run_reginald(output_name: &str, opts: GeneratorOpts) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output_dir = manifest_dir.join(PathBuf::from("tests/generator_rs_structs/test_proj/src/"));
    let output_file = output_dir.join(output_name);

    gen::cmd(gen::Command {
        input: TEST_MAP_FILE.to_owned(),
        output: output_file,
        overwrite_map_name: None,
        verify: false,
        generator: Generator::RsStructs(opts),
    })
    .unwrap();
}

const GENERATOR_OPTS_DEFAULT: GeneratorOpts = GeneratorOpts {
    address_type: None,
    struct_derive: vec![],
    raw_enum_derive: vec![],
    add_use: vec![],
    add_attribute: vec![],
    external_traits: None,
    generate_uint_conversion: true,
};

// ==== Tests ==================================================================

#[test]
#[ignore]
fn generator_rs_structs() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_proj = manifest_dir.join(PathBuf::from("tests/generator_rs_structs/test_proj/"));

    run_reginald(
        "out.rs",
        GeneratorOpts {
            struct_derive: vec!["Debug".to_string(), "Clone".to_string()],
            raw_enum_derive: vec!["Debug".to_string(), "PartialEq".to_string()],
            ..GENERATOR_OPTS_DEFAULT
        },
    );

    run_reginald(
        "out_ext_traits.rs",
        GeneratorOpts {
            external_traits: Some("crate::out::".to_string()),
            ..GENERATOR_OPTS_DEFAULT
        },
    );

    run_reginald(
        "out_crate_traits.rs",
        GeneratorOpts {
            external_traits: Some("reginald::".to_string()),
            ..GENERATOR_OPTS_DEFAULT
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
