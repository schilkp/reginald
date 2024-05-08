mod generator_c_funcpack;
mod generator_c_macromap;
mod generator_rs_structs;

use std::{path::PathBuf, process::Output};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref TEST_MAP_FILE: PathBuf = find_test_map_file();
}

fn find_test_map_file() -> PathBuf {
    let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    d.push("tests/map.yaml");
    d
}

pub fn print_cmd_output(out: &Output) {
    println!("  {}", &out.status);
    if !out.stdout.is_empty() {
        println!("  stdout:");
        let stdout = String::from_utf8(out.stdout.clone()).unwrap();
        for line in stdout.lines() {
            println!("   > {line}");
        }
    }
    if !out.stderr.is_empty() {
        println!("  stderr:");
        let stderr = String::from_utf8(out.stderr.clone()).unwrap();
        for line in stderr.lines() {
            println!("   > {line}");
        }
    }
}
