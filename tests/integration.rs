use std::path::PathBuf;
use std::process::Command;

fn atomic_binary() -> PathBuf {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target");

    let candidates = [
        // Specific target dirs (when built with --target)
        base.join("x86_64-unknown-linux-gnu/debug/atomic"),
        base.join("aarch64-unknown-linux-gnu/debug/atomic"),
        // Default (no --target)
        base.join("debug/atomic"),
    ];

    for p in &candidates {
        if p.exists() {
            return p.clone();
        }
    }
    panic!("atomic binary not found — build with `cargo build` first");
}

fn run_example(name: &str) -> String {
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
    let output = Command::new(atomic_binary())
        .args(["run", example.to_str().unwrap()])
        .output()
        .expect(&format!("Failed to run example: {}", name));
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_example(name: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let example = std::path::Path::new(manifest_dir)
        .join("examples")
        .join(name);
    let output = Command::new(atomic_binary())
        .args(["run", example.to_str().unwrap()])
        .output()
        .expect(&format!("Failed to run example: {}", name));
    String::from_utf8_lossy(&output.stdout).to_string()
}

#[test]
fn test_hello() {
    assert_eq!(run_example("hello.at"), "Hello, World!\n");
}

#[test]
fn test_fn_ref() {
    assert_eq!(run_example("fn_ref.at"), "42");
}

#[test]
fn test_lambda() {
    assert_eq!(run_example("lambda.at"), "42423042");
}

#[test]
fn test_struct() {
    assert_eq!(run_example("struct.at"), "1020");
}

#[test]
fn test_shorthand_struct() {
    assert_eq!(run_example("shorthand_struct.at"), "1020");
}

#[test]
fn test_enum() {
    assert_eq!(run_example("enum.at"), "Red42");
}

#[test]
fn test_tuple() {
    assert_eq!(run_example("tuple.at"), "12342");
}

#[test]
fn test_destructure() {
    assert_eq!(run_example("destructure.at"), "4210");
}

#[test]
fn test_char_literal() {
    assert_eq!(run_example("char_literal.at"), "65");
}

#[test]
fn test_number_literals() {
    assert_eq!(run_example("number_literals.at"), "105112552408");
}

#[test]
fn test_power() {
    assert_eq!(run_example("power.at"), "8181102449");
}

#[test]
fn test_bitwise() {
    assert_eq!(run_example("bitwise.at"), "176-184");
}

#[test]
fn test_short_circuit() {
    assert_eq!(run_example("short_circuit.at"), "04200770");
}

#[test]
fn test_compound() {
    assert_eq!(run_example("compound.at"), "151312332");
}

#[test]
fn test_range_exclusive() {
    assert_eq!(run_example("range_exclusive.at"), "01234");
}

#[test]
fn test_for_loop() {
    assert_eq!(run_example("for_loop.at"), "012341011");
}

#[test]
fn test_yield() {
    assert_eq!(run_example("yield.at"), "125210127");
}

#[test]
fn test_nested_for() {
    assert_eq!(run_example("nested_for.at"), "110111210211111221223132");
}

#[test]
fn test_math() {
    assert_eq!(run_example("math_builtins.at"), "4209910-10720-57");
}

#[test]
fn test_const() {
    assert_eq!(run_example("const.at"), "1024390");
}

#[test]
fn test_fn_type() {
    assert_eq!(run_example("fn_type.at"), "20");
}

#[test]
fn test_fn_type2() {
    assert_eq!(run_example("fn_type2.at"), "2021");
}

#[test]
fn test_type_ann() {
    assert_eq!(run_example("type_ann.at"), "4212");
}

#[test]
fn test_list() {
    assert_eq!(run_example("list.at"), "103050");
}

#[test]
fn test_map_filter() {
    assert_eq!(run_example("map_filter.at"), "210215");
}

#[test]
fn test_str_match() {
    assert_eq!(run_example("str_match.at"), "1234");
}

#[test]
fn test_is_match() {
    assert_eq!(run_example("is_match.at"), "123");
}

#[test]
fn test_when_match() {
    assert_eq!(run_example("when_match.at"), "the answer42");
}

#[test]
fn test_when_chain() {
    assert_eq!(run_example("when_chain.at"), "positivemedium");
}

#[test]
fn test_stdlib() {
    assert_eq!(run_example("stdlib.at"), "42993150200");
}

#[test]
fn test_propagate() {
    assert_eq!(run_example("propagate.at"), "449");
}

#[test]
fn test_safe_access() {
    assert_eq!(run_example("safe_access.at"), "10429");
}

#[test]
fn test_multiline() {
    assert_eq!(run_example("multiline.at"), "Hello\nWorld");
}

#[test]
fn test_interp() {
    assert_eq!(
        run_example("interp.at"),
        "Hello, World!Age: 42World is 42 years olddone"
    );
}
