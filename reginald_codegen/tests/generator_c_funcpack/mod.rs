use std::{fs, path::PathBuf, process::Command};

use tempfile::{tempdir, TempDir};

use reginald_codegen::{
    builtin::c::funcpack::{Element, GeneratorOpts},
    cli::cmd::gen::{self, Generator},
    utils::Endianess,
};

use crate::{print_cmd_output, TEST_MAP_FILE};

// ==== Utils ==================================================================

const ADDITIONAL_COMPILERS: Option<&str> = option_env!("REGINALD_TEST_C_FUNCPACK_ADDITIONAL_COMPILERS");

fn test_generated_code(test_dir: &TempDir, extra_cflags: &[&str], extra_sources: &[&str]) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let resources_dir = manifest_dir.join(PathBuf::from("tests/generator_c_funcpack/resources"));

    // Sources:
    let mut sources = vec![];
    sources.push(resources_dir.join("test.c").to_str().unwrap().to_string());
    sources.push(resources_dir.join("Unity/unity.c").to_str().unwrap().to_string());
    sources.append(&mut extra_sources.iter().map(|x| x.to_string()).collect());

    // c flags:
    let mut cflags = vec![];
    cflags.push("-Wall".to_string());
    cflags.push("-Wextra".to_string());
    cflags.push("-Wpedantic".to_string());
    cflags.push("-Wconversion".to_string());
    cflags.push("-Warith-conversion".to_string());
    cflags.push("-Wint-conversion".to_string());
    cflags.push("-Werror".to_string());
    // include resources dir (for test framework):
    cflags.push(format!("-I{}", resources_dir.to_str().unwrap()));
    // include test dir (for generated files):
    cflags.push(format!("-I{}", test_dir.path().to_str().unwrap()));
    // Extra c flags:
    cflags.append(&mut extra_cflags.iter().map(|x| x.to_string()).collect());

    // ==== Compile for host + run ====

    let mut compile_args = vec![];
    compile_args.extend(sources.clone());
    compile_args.extend(cflags.clone());
    compile_args.push("-fsanitize=undefined".to_string());
    compile_args.push("-fanalyzer".to_string());

    // output:
    let test_exe = test_dir.path().join("test.out").to_str().unwrap().to_string();
    compile_args.push("-o".to_string());
    compile_args.push(test_exe.to_string());

    println!("Compiling...");
    let compile_output = Command::new("gcc").args(&compile_args).output().unwrap();
    print_cmd_output(&compile_output);
    assert!(compile_output.status.success());

    println!("Testing...");
    let test_output = Command::new(test_exe).output().unwrap();
    print_cmd_output(&test_output);
    assert!(test_output.status.success());

    // ==== Compile with ADDITIONAL_COMPILERS ====

    if let Some(additional_compilers) = ADDITIONAL_COMPILERS {
        for compiler in additional_compilers.split(",") {
            let compiler = compiler.trim();

            let mut compile_args = vec![];
            compile_args.extend(sources.clone());
            compile_args.extend(cflags.clone());

            // only compile to object, don't link:
            compile_args.push("-c".to_string());

            println!("Compiling with `{}`...", compiler);
            let compile_output = Command::new(compiler)
                .current_dir(test_dir.path())
                .args(&compile_args)
                .output()
                .unwrap();
            print_cmd_output(&compile_output);
            assert!(compile_output.status.success());
        }
    }
}

fn finish_test(d: TempDir) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let artifacts_dir = manifest_dir.join(PathBuf::from("tests/generator_c_funcpack/artifacts"));
    fs::create_dir_all(&artifacts_dir).unwrap();

    for entry in fs::read_dir(d.path()).unwrap() {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_dir() {
            fs::copy(entry.path(), artifacts_dir.join(entry.file_name())).unwrap();
        }
    }

    d.close().unwrap();
}

fn run_reginald(d: &TempDir, output_name: &str, opts: GeneratorOpts) {
    gen::cmd(gen::Command {
        input: TEST_MAP_FILE.to_owned(),
        output: d.path().to_owned().join(PathBuf::from(output_name)),
        overwrite_map_name: None,
        verify: false,
        generator: Generator::CFuncpack(opts),
    })
    .unwrap();
}

// ==== Tests ==================================================================

#[test]
#[ignore]
fn generator_c_funcpack_c99() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            dont_generate: vec![Element::GenericMacros],
            ..GeneratorOpts::default()
        },
    );

    test_generated_code(&d, &["-DTEST_SKIP_GENERIC_MACROS", "-std=c99"], &[]);

    finish_test(d);
}

#[test]
#[ignore]
fn generator_c_funcpack_c11() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            ..GeneratorOpts::default()
        },
    );

    test_generated_code(&d, &["-std=c11"], &[]);

    finish_test(d);
}

#[test]
#[ignore]
fn generator_c_funcpack_defer_to_le() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            defer_to_endian: Some(Endianess::Little),
            ..GeneratorOpts::default()
        },
    );

    test_generated_code(&d, &["-std=c11"], &[]);

    finish_test(d);
}

#[test]
#[ignore]
fn generator_c_funcpack_defer_to_be() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            defer_to_endian: Some(Endianess::Big),
            ..GeneratorOpts::default()
        },
    );

    test_generated_code(&d, &["-std=c11"], &[]);

    finish_test(d);
}

#[test]
#[ignore]
fn generator_c_funcpack_split_header_source() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            funcs_as_prototypes: true,
            funcs_static_inline: false,
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        &d,
        "out.c",
        GeneratorOpts {
            funcs_static_inline: false,
            add_include: vec!["out.h".to_string()],
            include_guards: false,
            only_generate: vec![Element::StructConversionFuncs],
            ..GeneratorOpts::default()
        },
    );

    test_generated_code(&d, &["-std=c11"], &[d.path().join("out.c").to_str().unwrap()]);

    finish_test(d);
}

#[test]
#[ignore]
fn generator_c_funcpack_split_headers() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out_enum.h",
        GeneratorOpts {
            only_generate: vec![Element::Enums],
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        &d,
        "out_enum_validation.h",
        GeneratorOpts {
            only_generate: vec![Element::EnumValidationMacros],
            add_include: vec!["out_enum.h".to_string()],
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        &d,
        "out_structs.h",
        GeneratorOpts {
            only_generate: vec![Element::Structs],
            add_include: vec!["out_enum_validation.h".to_string()],
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        &d,
        "out_struct_conv.h",
        GeneratorOpts {
            only_generate: vec![Element::StructConversionFuncs],
            add_include: vec!["out_structs.h".to_string()],
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        &d,
        "out_reg_props.h",
        GeneratorOpts {
            only_generate: vec![Element::RegisterProperties],
            add_include: vec!["out_struct_conv.h".to_string()],
            ..GeneratorOpts::default()
        },
    );

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            only_generate: vec![Element::GenericMacros],
            add_include: vec!["out_reg_props.h".to_string()],
            ..GeneratorOpts::default()
        },
    );

    test_generated_code(&d, &["-std=c11"], &[]);

    finish_test(d);
}
