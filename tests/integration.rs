use std::path::PathBuf;
use std::process::Command;

fn atomic_binary() -> PathBuf {
    // CARGO_BIN_EXE_atomic is set by cargo test itself — trust it unconditionally.
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_atomic") {
        return PathBuf::from(&path);
    }

    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    let exe_suffix = if cfg!(target_os = "windows") {
        ".exe"
    } else {
        ""
    };

    let candidates: &[&str] = if cfg!(target_os = "windows") {
        &["x86_64-pc-windows-msvc/debug/atomic"]
    } else {
        &[
            "x86_64-unknown-linux-gnu/debug/atomic",
            "aarch64-unknown-linux-gnu/debug/atomic",
        ]
    };

    for c in candidates {
        let p = base.join(format!("{}{}", c, exe_suffix));
        if p.exists() {
            return p;
        }
    }

    // Fallback: default target dir (no --target)
    let p = base.join(format!("debug/atomic{}", exe_suffix));
    if p.exists() {
        return p;
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
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    // Normalize CRLF -> LF so tests pass on Windows where CRT emits \r\n.
    // Strip all \r to handle cases where git CRLF conversion adds an extra
    // carriage return (e.g. multiline string literals in .at source files).
    stdout.replace("\r\n", "\n").replace('\r', "")
}

fn run_example_with_stdin(name: &str, stdin_data: &str) -> String {
    use std::io::Write;
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
    let mut child = Command::new(atomic_binary())
        .args(["run", example.to_str().unwrap()])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect(&format!("Failed to spawn example: {}", name));

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(stdin_data.as_bytes()).expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect(&format!("Failed to run example: {}", name));
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    stdout.replace("\r\n", "\n").replace('\r', "")
}

/// Run an example and verify it exits successfully (ignoring output).
/// Used for network-dependent tests where output is not deterministic.
fn run_example_ok(name: &str) -> bool {
    let example = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join(name);
    let output = Command::new(atomic_binary())
        .args(["run", example.to_str().unwrap()])
        .output()
        .expect(&format!("Failed to run example: {}", name));
    output.status.success()
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

// ── read_line tests ──────────────────────────────────────

#[test]
fn test_read_line_with_input() {
    assert_eq!(
        run_example_with_stdin("io.at", "Alice\n"),
        "Hello, Alice\n"
    );
}

#[test]
fn test_read_line_eof() {
    // No stdin input → read_line returns None → unwrap_or uses "World"
    assert_eq!(
        run_example_with_stdin("io.at", ""),
        "Hello, World\n"
    );
}

#[test]
fn test_read_line_multiple() {
    assert_eq!(
        run_example_with_stdin("read_line_multiple.at", "first line\nsecond line\n"),
        "first line\nsecond line\n"
    );
}

#[test]
fn test_read_line_empty_line() {
    assert_eq!(
        run_example_with_stdin("read_line_empty.at", "\n"),
        "Got empty line\n"
    );
}

// ── Stability tests ────────────────────────────────────────────────

#[test]
fn test_stability_mergesort() {
    assert_eq!(
        run_example("stability_mergesort.at"),
        "3\n9\n10\n27\n38\n43\n82\nmerge_sort_ok\n"
    );
}

#[test]
fn test_stability_closure_chain() {
    assert_eq!(
        run_example("stability_closure_chain.at"),
        "17\n330\n360\n15\n35\nclosure_chain_ok\n"
    );
}

#[test]
fn test_stability_mapset() {
    assert_eq!(
        run_example("stability_mapset.at"),
        "4\n1\n20\n4\n6\n2\n2\nmapset_ok\n"
    );
}

#[test]
fn test_stability_string_transform() {
    assert_eq!(
        run_example("stability_string_transform.at"),
        "Hello, Atomic!\nAtomic! Hello,\nHello\nHELLO, ATOMIC!\nhello, atomic!\nFOO BAR BAZ\nstring_transform_ok\n"
    );
}

#[test]
fn test_stability_enum_match() {
    assert_eq!(
        run_example("stability_enum_match.at"),
        "42\n-7\n5\n30\n20\n-2\nenum_match_ok\n"
    );
}

#[test]
fn test_stability_for_comprehension() {
    assert_eq!(
        run_example("stability_for_comprehension.at"),
        "10\n1\n100\n10\n1\n19\n5\n9\n11\n41\nfor_comprehension_ok\n"
    );
}

#[test]
fn test_stability_collatz() {
    assert_eq!(
        run_example("stability_collatz.at"),
        "0\n1\n7\n8\n111\n324\ncollatz_ok\n"
    );
}

#[test]
fn test_stability_roman() {
    assert_eq!(
        run_example("stability_roman.at"),
        "1\n1\n1\n0\n1\n120\n3628800\nroman_recursion_ok\n"
    );
}

#[test]
fn test_stability_prime() {
    assert_eq!(
        run_example("stability_prime.at"),
        "25\n2\n11\n29\n229\n10\nprime_ok\n"
    );
}

#[test]
fn test_stability_nested_struct() {
    assert_eq!(
        run_example("stability_nested_struct.at"),
        "1\n30\n2\n10001\n31\n2\nnested_struct_ok\n"
    );
}

#[test]
fn test_stability_generic_algo() {
    assert_eq!(
        run_example("stability_generic_algo.at"),
        "1\n8\n8\n15\n120\ngeneric_algo_ok\n"
    );
}

#[test]
fn test_stability_kitchen_sink() {
    assert_eq!(
        run_example("stability_kitchen_sink.at"),
        "0\n1\n5\n55\n20\n100\n3\n120\nok\n10\n20\n99\n30\n55\n70\nHELLO ATOMIC\n42\n3.14\n1\n7\nkitchen_sink_ok\n"
    );
}

// ── HTTP / Networking stability tests ───────────────────────────────

#[test]
#[ignore = "requires network access"]
fn test_stability_http_get() {
    assert!(run_example_ok("stability_http_get.at"));
}

#[test]
#[ignore = "requires network access"]
fn test_stability_http_post() {
    assert!(run_example_ok("stability_http_post.at"));
}

#[test]
fn test_stability_http_error() {
    // Error handling test: connection refused is deterministic (no network needed)
    assert!(run_example_ok("stability_http_error.at"));
}

#[test]
#[ignore = "requires network access"]
fn test_stability_http_methods() {
    assert!(run_example_ok("stability_http_methods.at"));
}

#[test]
#[ignore = "requires network access"]
fn test_stability_deepseek() {
    assert!(run_example_ok("stability_deepseek.at"));
}
