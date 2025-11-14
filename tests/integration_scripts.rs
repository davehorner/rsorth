use std::process::Command;
use std::path::{Path, PathBuf};

// For library-based tests
use sorth::runtime::interpreter::sorth_interpreter::SorthInterpreter;
use sorth::runtime::interpreter::{Interpreter, CodeManagement};
use sorth::runtime::built_ins::{
    base_words::register_base_words,
    io_words::register_io_words,
    terminal_words::register_terminal_words,
    user_words::register_user_words,
    ffi_words::register_ffi_words,
};
use std::fs;

// Helper to get absolute path from manifest dir
fn manifest_path(rel: &str) -> PathBuf {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    Path::new(manifest_dir).join(rel)
}

// Helper to run the interpreter binary with a script and capture output
fn run_script(script: &str) -> String {
    let exe = if cfg!(windows) { "target\\debug\\sorth.exe" } else { "target/debug/sorth" };
    assert!(Path::new(exe).exists(), "Interpreter binary not found: {}", exe);
    let output = Command::new(exe)
        .arg(script)
        .output()
        .expect("Failed to run interpreter");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn assert_00_test_words_output(output: &str) {
    assert!(output.contains("Hello world!"), "Missing 'Hello world!' in output");
    assert!(output.contains("Goodbye all."), "Missing 'Goodbye all.' in output");
    assert!(output.contains("The 10 is greater than 0!"), "Missing 'The 10 is greater than 0!' in output");
    assert!(output.contains("What about failed ifs?"), "Missing 'What about failed ifs?' in output");
    assert!(output.contains("All done."), "Missing 'All done.' in output");
    assert!(output.contains("Depth: 0"), "Missing 'Depth: 0' in output");
}

fn assert_01_test_loops_output(output: &str) {
    for n in (0..=9).rev() {
        assert!(output.contains(&format!("Looping until: {}", n)), "Missing 'Looping until: {}'", n);
    }
    for n in (0..=9).rev() {
        assert!(output.contains(&format!("Looping while: {}", n)), "Missing 'Looping while: {}'", n);
    }
    let looping_count = output.matches("Looping.").count();
    assert_eq!(looping_count, 10, "Expected 10 'Looping.' lines, got {}", looping_count);
}

#[test]
fn test_00_test_words() {
    let output = run_script("tests/00_test_words.f");
    println!("\n--- Output of 00_test_words.f ---\n{}\n-------------------------------", output);
    assert_00_test_words_output(&output);
}

#[test]
fn test_01_test_loops() {
    let output = run_script("tests/01_test_loops.f");
    println!("\n--- Output of 01_test_loops.f ---\n{}\n-------------------------------", output);
    assert_01_test_loops_output(&output);
}

#[test]
fn test_00_test_words_lib() {
    let mut interpreter = SorthInterpreter::new();
    register_base_words(&mut interpreter);
    register_io_words(&mut interpreter);
    register_terminal_words(&mut interpreter);
    register_user_words(&mut interpreter);
    register_ffi_words(&mut interpreter);
    let std_path = manifest_path("std");
    interpreter.add_search_path(std_path.to_str().unwrap()).unwrap();
    interpreter.process_source_file(manifest_path("std.f").to_str().unwrap()).unwrap();
    let script = fs::read_to_string(manifest_path("tests/00_test_words.f")).unwrap();
    let result = interpreter.process_source(manifest_path("tests/00_test_words.f").to_str().unwrap(), &script);
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
    // If you add output capturing to the interpreter, call assert_00_test_words_output here.
}

#[test]
fn test_01_test_loops_lib() {
    let mut interpreter = SorthInterpreter::new();
    register_base_words(&mut interpreter);
    register_io_words(&mut interpreter);
    register_terminal_words(&mut interpreter);
    register_user_words(&mut interpreter);
    register_ffi_words(&mut interpreter);
    let std_path = manifest_path("std");
    interpreter.add_search_path(std_path.to_str().unwrap()).unwrap();
    interpreter.process_source_file(manifest_path("std.f").to_str().unwrap()).unwrap();
    let script = fs::read_to_string(manifest_path("tests/01_test_loops.f")).unwrap();
    let result = interpreter.process_source(manifest_path("tests/01_test_loops.f").to_str().unwrap(), &script);
    assert!(result.is_ok(), "Script failed: {:?}", result.err());
    // If you add output capturing to the interpreter, call assert_01_test_loops_output here.
}
