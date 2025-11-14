
# Rusty Sorth

An experimental implementation of the Strange Forth language in Rust.  Like the C++ version, this
version byte-code compiles the source and runs the byte-code in a simple VM.

This version isn't up to full feature parity with the original but a slightly modified standard
library and fancy repl do work in this version.

Currently missing from the C++ version is the threading support and parity with the FFI interface.


[See the original version for more details.](https://github.com/cstrainge/sorth)


## Taskfile

This project includes a `Taskfile.yml` for convenient automation of common development tasks. You can use [cargo-task](https://github.com/go-task/task) or a compatible task runner to:

- Run all Rust and Forth integration tests
- Build, clean, or run scripts

Example:

```
task forth-rs-test
```

See the `Taskfile.yml` for available tasks and usage.


## Test Coverage


This project includes a comprehensive test suite for Forth word compatibility and language features. The Rust test suite is based on the reference tests from [forth-rs](https://github.com/cstrainge/forth-rs), ported and parameterized for rsorth. Deviations from Forth semantics are documented in the test file.

- **Rust Parameterized Tests:**
	- Located in `tests/forth_rs_param_tests.rs`.
	- Covers arithmetic, logic, stack, control flow, function definition, and error/panic cases.
	- Uses the [`test-case`](https://crates.io/crates/test-case) crate for parameterized coverage, matching the reference forth-rs test suite.
	- Known deviations from Forth semantics (e.g., `<>` returns `1` for true, not `-1`) are documented in the test file.

- **Integration Tests in Forth:**
	- The `tests/` directory contains `.f` scripts that exercise advanced features: structures, arrays, hashes, buffers, strings, exceptions, FFI, and REPL/user/terminal words.
	- These scripts are run as integration tests and provide coverage for features not directly asserted in Rust.

**Note:**
If you want to expand Rust-side assertions to cover FFI, JSON, or REPL/user/terminal words, see the `.f` scripts for examples.
