use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// A single violation found during source scanning.
/// kind describes the category (e.g., "panic!", "unwrap", "expect").
#[derive(Debug, Clone)]
struct Offense {
    line_number: usize,
    kind: &'static str,
}

/// Mutable scanning state that persists across lines while parsing a file.
#[derive(Debug, Default, Clone)]
struct ScanState {
    comment_depth: usize,
    brace_depth: usize,
    pending_test: bool,
    test_stack: Vec<usize>,
}

/// Scan the workspace library sources for panic-prone usage and fail the build
/// with a clear message if any violations are found.
fn enforce_no_panic_policy() -> Result<(), String> {
    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(value) => PathBuf::from(value),
        Err(_) => PathBuf::from("."),
    };

    let scan_roots = [
        manifest_dir.join("src"),
        manifest_dir.join("crates/kyoso-components/src"),
        manifest_dir.join("crates/kyoso-core/src"),
        manifest_dir.join("crates/kyoso-crdt/src"),
        manifest_dir.join("crates/kyoso-ecs/src"),
        manifest_dir.join("crates/kyoso-renderer/src"),
        manifest_dir.join("crates/kyoso-scheduler/src"),
    ];

    let mut findings: Vec<(PathBuf, Vec<Offense>)> = Vec::new();
    for root in scan_roots.iter() {
        if !root.exists() {
            continue;
        }
        walk_rust_files(root, &mut |path| {
            let path_string = path.to_string_lossy();
            if path_string.contains("/tests/") || path_string.contains("/benches/") {
                return;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("build.rs") {
                return;
            }
            if let Some(offenses) = analyze_source_file(path)
                && !offenses.is_empty()
            {
                findings.push((path.to_path_buf(), offenses));
            }
        });
    }

    if findings.is_empty() {
        return Ok(());
    }

    println!("cargo::warning=Found panic-prone usage in library code (build will fail):");
    for (path, offenses_for_path) in &findings {
        for offense in offenses_for_path {
            println!(
                "cargo::warning=❌ - {}:{}: {}",
                path.display(),
                offense.line_number,
                offense.kind
            );
        }
    }

    Err(
        "panic-prone usage found in library source files; replace with Result-based errors (thiserror) or justify with expect(\"SAFETY: ...\")"
            .to_string(),
    )
}

/// Recursively visit all .rs files under the given directory and invoke the callback.
fn walk_rust_files<F: FnMut(&Path)>(directory: &Path, visit: &mut F) {
    if let Ok(read_dir) = fs::read_dir(directory) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_rust_files(&path, visit);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                visit(&path);
            }
        }
    }
}

/// Analyze one Rust source file, returning any violations found.
/// Returns None if the file could not be read.
fn analyze_source_file(path: &Path) -> Option<Vec<Offense>> {
    let content = fs::read_to_string(path).ok()?;
    let mut offenses: Vec<Offense> = Vec::new();

    let mut state = ScanState::default();

    for (index, raw_line) in content.lines().enumerate() {
        let line_number = index + 1;

        // First, remove comments while keeping string contents, and update
        // brace depth so we can track test-only regions accurately.
        let line_text = strip_comments_and_track_braces(raw_line, &mut state);

        // Detect attributes that start test-only regions.
        let trimmed = line_text.trim();
        if trimmed.contains("#[cfg") && trimmed.contains("test") {
            state.pending_test = true;
        }
        if trimmed.contains("#[test]") {
            state.pending_test = true;
        }

        // Skip scanning inside #[test] or #[cfg(test)] regions.
        let in_test_only = !state.test_stack.is_empty();
        if in_test_only {
            continue;
        }

        // Panic-like macros
        push_pattern_occurrences(&line_text, "panic!(", "panic!", line_number, &mut offenses);
        push_pattern_occurrences(&line_text, "todo!(", "todo!", line_number, &mut offenses);
        push_pattern_occurrences(
            &line_text,
            "unimplemented!(",
            "unimplemented!",
            line_number,
            &mut offenses,
        );
        push_pattern_occurrences(
            &line_text,
            "unreachable!(",
            "unreachable!",
            line_number,
            &mut offenses,
        );

        // unwrap variants
        push_pattern_occurrences(&line_text, ".unwrap(", "unwrap", line_number, &mut offenses);
        push_pattern_occurrences(
            &line_text,
            ".unwrap_err(",
            "unwrap_err",
            line_number,
            &mut offenses,
        );

        // expect variants, tolerating expect("SAFETY: ...")
        push_expect_occurrences(&line_text, ".expect(", "expect", line_number, &mut offenses);
        push_expect_occurrences(
            &line_text,
            ".expect_err(",
            "expect_err",
            line_number,
            &mut offenses,
        );
    }

    Some(offenses)
}

/// Remove comments while preserving string literals and update brace/test state.
fn strip_comments_and_track_braces(raw_line: &str, state: &mut ScanState) -> String {
    let mut output = String::new();
    let mut chars = raw_line.chars().peekable();
    let mut in_string = false;
    let mut string_delimiter = '\0';

    while let Some(character) = chars.next() {
        if state.comment_depth > 0 {
            if character == '*' && chars.peek() == Some(&'/') {
                chars.next();
                state.comment_depth -= 1;
            }
            continue;
        }

        if in_string {
            // Keep string content intact; handle escapes.
            if character == '\\' {
                output.push(character);
                if let Some(next_character) = chars.next() {
                    output.push(next_character);
                }
                continue;
            }
            if character == string_delimiter {
                in_string = false;
            }
            output.push(character);
            continue;
        }

        // Not in a block comment or string.
        if character == '/' && chars.peek() == Some(&'*') {
            chars.next();
            state.comment_depth += 1;
            continue;
        }
        if character == '/' && chars.peek() == Some(&'/') {
            // Line comment: ignore the remainder of this line.
            break;
        }
        if character == '"' || character == '\'' {
            in_string = true;
            string_delimiter = character;
            output.push(character);
            continue;
        }

        // Update brace depth outside strings/comments.
        if character == '{' {
            state.brace_depth += 1;
            if state.pending_test {
                state.test_stack.push(state.brace_depth);
                state.pending_test = false;
            }
        } else if character == '}' {
            if state.brace_depth > 0 {
                state.brace_depth -= 1;
            }
            while state
                .test_stack
                .last()
                .copied()
                .is_some_and(|depth| state.brace_depth < depth)
            {
                state.test_stack.pop();
            }
        }

        output.push(character);
    }

    output
}

/// Find all occurrences of a simple pattern and record an offense for each.
fn push_pattern_occurrences(
    line: &str,
    pattern: &str,
    kind: &'static str,
    line_number: usize,
    out: &mut Vec<Offense>,
) {
    let mut start = 0usize;
    while let Some(position) = line[start..].find(pattern) {
        out.push(Offense { line_number, kind });
        start += position + pattern.len();
    }
}

/// Find occurrences of expect-like calls and record an offense unless the message
/// begins with "SAFETY:".
fn push_expect_occurrences(
    line: &str,
    pattern: &str,
    kind: &'static str,
    line_number: usize,
    out: &mut Vec<Offense>,
) {
    let mut start = 0usize;
    while let Some(position) = line[start..].find(pattern) {
        let absolute = start + position;
        if !is_safety_expect_message_at(line, absolute + pattern.len()) {
            out.push(Offense { line_number, kind });
        }
        start = absolute + pattern.len();
    }
}

/// Return true if the expect message at the given index begins with "SAFETY:".
fn is_safety_expect_message_at(line: &str, after_paren_index: usize) -> bool {
    // after_paren_index points right after the '('
    let mut index = after_paren_index;
    let bytes = line.as_bytes();

    // Skip whitespace
    while index < bytes.len() && (bytes[index] as char).is_whitespace() {
        index += 1;
    }

    if index >= bytes.len() || bytes[index] != b'"' {
        return false;
    }

    // Consume opening quote
    index += 1;

    let mut message = String::new();
    while index < bytes.len() {
        let character = bytes[index] as char;
        if character == '\\' {
            // Skip escaped char
            index += 2;
            continue;
        }
        if character == '"' {
            break;
        }
        message.push(character);
        index += 1;
    }

    message.trim_start().starts_with("SAFETY:")
}
