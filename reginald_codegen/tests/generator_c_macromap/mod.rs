use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::{TempDir, tempdir};

use reginald_codegen::{
    builtin::c::{self, macromap::GeneratorOpts},
    regmap::RegisterMap,
};

use crate::{TEST_MAP_FILE, print_cmd_output};

// ==== Utils ==================================================================

fn test_generated_code(test_dir: &TempDir, extra_cflags: &[&str], extra_sources: &[&str]) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_resources_dir = manifest_dir.join(PathBuf::from("tests/generator_c_macromap/resources"));
    let shared_resources_dir = manifest_dir.join(PathBuf::from("tests/resources"));

    // Sources:
    let mut sources = vec![];
    sources.push(test_resources_dir.join("test.c").to_str().unwrap().to_string());
    sources.push(shared_resources_dir.join("Unity/unity.c").to_str().unwrap().to_string());
    sources.append(&mut extra_sources.iter().map(|x| x.to_string()).collect());

    // c flags:
    let mut cflags = vec![];
    cflags.push("-Wall".to_string());
    cflags.push("-Wextra".to_string());
    cflags.push("-Wpedantic".to_string());
    cflags.push("-Werror".to_string());
    // include test resources dir (for test files):
    cflags.push(format!("-I{}", test_resources_dir.to_str().unwrap()));
    // include resources dir (for test framework):
    cflags.push(format!("-I{}", shared_resources_dir.to_str().unwrap()));
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

    println!("  GCC host args:");
    for arg in &compile_args {
        println!("    {}", arg);
    }
    println!("  Compiling for host...");
    let compile_output = Command::new("gcc").args(&compile_args).output().unwrap();
    print_cmd_output(&compile_output);
    assert!(compile_output.status.success());

    println!("  Running tests...");
    let test_output = Command::new(test_exe).output().unwrap();
    print_cmd_output(&test_output);
    assert!(test_output.status.success());

    println!("  >>> OK!");
}

fn finish_test(d: TempDir) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let artifacts_dir = manifest_dir.join(PathBuf::from("tests/generator_c_macromap/artifacts"));
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
    let output_path = d.path().to_owned().join(PathBuf::from(output_name));
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let artifacts_dir = manifest_dir.join(PathBuf::from("tests/generator_c_funcpack/artifacts"));

    let map = RegisterMap::from_file(&TEST_MAP_FILE).unwrap();

    let mut out = String::new();
    c::macromap::generate(&mut out, &map, Path::new(output_name), &opts).unwrap();

    // Write to output file:
    fs::write(&output_path, &out).unwrap();

    // Write to artifacts_dir file:
    fs::create_dir_all(&artifacts_dir).unwrap();
    fs::write(artifacts_dir.join(PathBuf::from(output_name)), out).unwrap();
}

// ==== Tests ==================================================================

#[test]
#[cfg_attr(not(feature = "test_gen_output"), ignore)]
fn generator_c_macromap_c99() {
    let d = tempdir().unwrap();

    run_reginald(
        &d,
        "out.h",
        GeneratorOpts {
            clang_format_guard: true,
            add_include: vec![],
        },
    );

    test_generated_code(&d, &["-std=c99"], &[]);

    finish_test(d);
}
