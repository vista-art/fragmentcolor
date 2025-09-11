use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // Same strategy as build.rs meta::workspace_root
    let output =
        std::process::Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
            .arg("locate-project")
            .arg("--workspace")
            .arg("--message-format=plain")
            .output()
            .expect("failed to run cargo locate-project")
            .stdout;
    let cargo_path = std::path::Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

#[test]
fn js_shader_example_uses_template_literal_and_no_rust_raw_markers() {
    let root = workspace_root();
    let p = root.join("platforms/web/examples/core/shader/Shader.js");
    let src = fs::read_to_string(&p).expect("read JS shader example");

    assert!(
        src.contains("new Shader(`"),
        "JS should use template literal with backticks"
    );
    assert!(
        !src.contains("r#\""),
        "JS must not contain Rust raw string opener"
    );
    assert!(
        !src.contains("\"#"),
        "JS must not contain Rust raw string closer"
    );

    // Ensure we didn't mutate WGSL to add spurious semicolons (spot-check @vertex;)
    assert!(
        !src.contains("@vertex;"),
        "WGSL must remain unmodified (no added semicolons)"
    );
}

#[test]
fn py_shader_example_uses_triple_quotes_and_no_leading_space_before_comment() {
    let root = workspace_root();
    let p = root.join("platforms/python/examples/core/shader/Shader.py");
    let src = fs::read_to_string(&p).expect("read Py shader example");

    assert!(
        src.contains("Shader(\"\"\""),
        "Python should use triple-quoted string for WGSL"
    );
    assert!(
        !src.contains("r#\""),
        "Python must not contain Rust raw string opener"
    );
    assert!(
        !src.contains("\"#"),
        "Python must not contain Rust raw string closer"
    );

    // No lines should start with a leading space before '#'
    for (i, line) in src.lines().enumerate() {
        assert!(
            !line.starts_with(" #"),
            "Line {} has a leading space before '#': {:?}",
            i + 1,
            line
        );
    }
}
