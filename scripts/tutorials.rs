mod tutorials {
    //! Tutorial pipeline: discover `docs/tutorials/**/example.rs`, extract region
    //! markers (`// #region: NAME` / `// #endregion: NAME`), transpile each region
    //! to JS / Python / Swift / Kotlin via convert.rs, and emit a TS manifest the
    //! Astro `<Snippet>` and `<Demo>` components consume.
    //!
    //! Hand-written `example.js` files (alongside each `example.rs`) are the
    //! canonical JS source: they get used both for snippet display and as the
    //! runtime module that `<Demo>` imports. Auto-transpiled JS would be subtly
    //! different from a human-written port (no `await Shader.fetch`, no per-step
    //! ergonomic shifts) — easier to maintain by hand at this scale.
    //!
    //! Output:
    //! - `docs/website/src/generated/tutorials.ts` — manifest module
    //! - `docs/website/src/tutorials/<rel>/example.js` — copies for Vite to bundle
    //!
    //! Vite import.meta.glob in Demo.astro picks up the example.js copies.

    use std::collections::BTreeMap;
    use std::fs;
    use std::path::{Path, PathBuf};

    const TUTORIALS_DIR: &str = "docs/tutorials";
    const MANIFEST_OUT: &str = "docs/website/src/generated/tutorials.ts";
    const JS_COPY_ROOT: &str = "docs/website/src/tutorials";

    pub fn build() {
        let workspace_root = super::meta::workspace_root();
        let tut_root = workspace_root.join(TUTORIALS_DIR);
        if !tut_root.exists() {
            return;
        }

        let mut entries: BTreeMap<String, TutorialEntry> = BTreeMap::new();
        walk_tutorials(&tut_root, &tut_root, &mut entries);

        write_manifest(&workspace_root, &entries);
        copy_example_js(&workspace_root, &tut_root, &entries);

        println!("cargo::rerun-if-changed={}", TUTORIALS_DIR);
    }

    #[derive(Debug, Default)]
    struct TutorialEntry {
        /// Full file contents per language. Rust and JS are the canonical originals;
        /// other languages are best-effort transpilation of the full Rust file.
        full: LangSlice,
        /// One LangSlice per region name found in the Rust file.
        regions: BTreeMap<String, LangSlice>,
        /// Relative path of the example.{rs,js} pair under `docs/tutorials/`,
        /// e.g. `01-hello-triangle/04-postfx-pass/example.rs`. The companion
        /// example.js lives at the same path with `.js` extension.
        js_copy_rel: PathBuf,
        /// True when an example.js exists alongside the example.rs.
        has_js: bool,
    }

    #[derive(Debug, Default, Clone)]
    struct LangSlice {
        rust: Option<String>,
        js: Option<String>,
        py: Option<String>,
        swift: Option<String>,
        kotlin: Option<String>,
    }

    /// Walks the tutorials tree. `tut_root` is `<workspace>/docs/tutorials/`; `dir` is the current
    /// directory being scanned (starts at `tut_root`, recurses into subdirs). Manifest keys are
    /// relative to `tut_root` (e.g. `01-hello-triangle/04-postfx-pass/example.rs`) — short and
    /// stable across workspace reloacations.
    fn walk_tutorials(
        tut_root: &Path,
        dir: &Path,
        entries: &mut BTreeMap<String, TutorialEntry>,
    ) {
        let read = match fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        for de in read.flatten() {
            let path = de.path();
            if path.is_dir() {
                walk_tutorials(tut_root, &path, entries);
                continue;
            }
            if path.file_name().and_then(|n| n.to_str()) != Some("example.rs") {
                continue;
            }
            let rel = match path.strip_prefix(tut_root) {
                Ok(r) => r.to_path_buf(),
                Err(_) => continue,
            };
            let key = rel.to_string_lossy().into_owned();
            let entry = process_example(tut_root, &rel);
            entries.insert(key, entry);
        }
    }

    fn process_example(tut_root: &Path, rel_rust: &Path) -> TutorialEntry {
        let mut entry = TutorialEntry::default();

        // Rust source
        let abs_rust = tut_root.join(rel_rust);
        let rust_text = fs::read_to_string(&abs_rust).unwrap_or_default();
        entry.full.rust = Some(rust_text.clone());
        let full_items = lines_as_items(&rust_text);
        entry.full.py = Some(super::convert::to_py(&full_items));
        entry.full.swift = Some(super::convert::to_swift(&full_items));
        entry.full.kotlin = Some(super::convert::to_kotlin(&full_items));

        let rust_regions = extract_regions(&rust_text);
        for (name, body) in &rust_regions {
            let slice = entry.regions.entry(name.clone()).or_default();
            slice.rust = Some(body.clone());
            let items = lines_as_items(body);
            slice.py = Some(super::convert::to_py(&items));
            slice.swift = Some(super::convert::to_swift(&items));
            slice.kotlin = Some(super::convert::to_kotlin(&items));
        }

        // JS source (hand-written, alongside example.rs)
        let abs_js = abs_rust.with_file_name("example.js");
        if abs_js.exists() {
            let js_text = fs::read_to_string(&abs_js).unwrap_or_default();
            entry.full.js = Some(js_text.clone());
            for (name, body) in extract_regions(&js_text) {
                let slice = entry.regions.entry(name).or_default();
                slice.js = Some(body);
            }
            entry.has_js = true;
        }

        // Mirror the directory layout under JS_COPY_ROOT.
        // e.g. `01-hello-triangle/01-step/example.rs` → `01-hello-triangle/01-step/example.js`
        let parent_rel = rel_rust.parent().unwrap_or(Path::new(""));
        entry.js_copy_rel = parent_rel.join("example.js");

        entry
    }

    /// Parse `// #region: NAME` ... `// #endregion: NAME` markers (also `# #region:` for Python).
    /// Returns a map of region-name → dedented body. Marker lines themselves are excluded.
    fn extract_regions(text: &str) -> BTreeMap<String, String> {
        let mut out: BTreeMap<String, String> = BTreeMap::new();
        let mut active: Vec<(String, Vec<String>)> = Vec::new();

        for line in text.lines() {
            let trimmed = line.trim_start();
            // Accept either Rust/JS/Swift/Kotlin (`//`) or Python (`#`) comment markers.
            let marker = trimmed
                .strip_prefix("// ")
                .or_else(|| trimmed.strip_prefix("//"))
                .or_else(|| trimmed.strip_prefix("# "))
                .or_else(|| trimmed.strip_prefix("#"));
            if let Some(rest) = marker {
                if let Some(name) = rest.strip_prefix("#region:") {
                    active.push((name.trim().to_string(), Vec::new()));
                    continue;
                }
                if let Some(name) = rest.strip_prefix("#endregion:") {
                    let name = name.trim();
                    if let Some((open_name, lines)) = active.pop() {
                        if open_name == name {
                            out.insert(name.to_string(), dedent(&lines));
                        } else {
                            // Mismatched: re-push and bail this endregion gracefully.
                            active.push((open_name, lines));
                        }
                    }
                    continue;
                }
            }
            for (_, lines) in active.iter_mut() {
                lines.push(line.to_string());
            }
        }
        out
    }

    fn dedent(lines: &[String]) -> String {
        let min_indent = lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        let mut out = String::with_capacity(lines.iter().map(|l| l.len() + 1).sum());
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            if line.len() >= min_indent {
                out.push_str(&line[min_indent..]);
            } else {
                out.push_str(line);
            }
        }
        // Trim a trailing newline gain or leading blank line
        out.trim_matches('\n').to_string()
    }

    /// Wrap each line in the (text, is_executable=false) shape that convert.rs expects.
    /// All tutorial code is "display" content (not hidden behind `# ` markers like API doc examples),
    /// so the bool is always false.
    fn lines_as_items(text: &str) -> Vec<(String, bool)> {
        text.lines().map(|l| (l.to_string(), false)).collect()
    }

    fn write_manifest(root: &Path, entries: &BTreeMap<String, TutorialEntry>) {
        let mut out = String::new();
        out.push_str("// AUTO-GENERATED by build.rs (scripts/tutorials.rs). Do not edit by hand.\n");
        out.push_str(
            "// Source: docs/tutorials/**/example.rs (with companion example.js).\n\n",
        );
        out.push_str("export type LangSlice = {\n");
        out.push_str("    rust?: string;\n");
        out.push_str("    js?: string;\n");
        out.push_str("    py?: string;\n");
        out.push_str("    swift?: string;\n");
        out.push_str("    kotlin?: string;\n");
        out.push_str("};\n\n");
        out.push_str("export type TutorialEntry = {\n");
        out.push_str("    full: LangSlice;\n");
        out.push_str("    regions: Record<string, LangSlice>;\n");
        out.push_str("    /** Vite-relative path under src/tutorials/ used by Demo's import.meta.glob. */\n");
        out.push_str("    jsModulePath?: string;\n");
        out.push_str("};\n\n");
        out.push_str("export const tutorials: Record<string, TutorialEntry> = {\n");
        for (key, e) in entries {
            out.push_str(&format!("    {}: {{\n", json_string(key)));
            out.push_str("        full: ");
            write_lang_slice(&mut out, &e.full, "        ");
            out.push_str(",\n");
            out.push_str("        regions: {\n");
            for (region, slice) in &e.regions {
                out.push_str(&format!("            {}: ", json_string(region)));
                write_lang_slice(&mut out, slice, "            ");
                out.push_str(",\n");
            }
            out.push_str("        },\n");
            if e.has_js {
                out.push_str(&format!(
                    "        jsModulePath: {},\n",
                    json_string(&e.js_copy_rel.to_string_lossy())
                ));
            }
            out.push_str("    },\n");
        }
        out.push_str("};\n");

        let out_path = root.join(MANIFEST_OUT);
        if let Some(parent) = out_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = super::meta::write_if_changed(&out_path, &out);
    }

    fn write_lang_slice(out: &mut String, slice: &LangSlice, indent: &str) {
        out.push_str("{\n");
        if let Some(s) = &slice.rust {
            out.push_str(&format!("{indent}    rust: {},\n", json_string(s)));
        }
        if let Some(s) = &slice.js {
            out.push_str(&format!("{indent}    js: {},\n", json_string(s)));
        }
        if let Some(s) = &slice.py {
            out.push_str(&format!("{indent}    py: {},\n", json_string(s)));
        }
        if let Some(s) = &slice.swift {
            out.push_str(&format!("{indent}    swift: {},\n", json_string(s)));
        }
        if let Some(s) = &slice.kotlin {
            out.push_str(&format!("{indent}    kotlin: {},\n", json_string(s)));
        }
        out.push_str(indent);
        out.push('}');
    }

    fn copy_example_js(
        workspace_root: &Path,
        tut_root: &Path,
        entries: &BTreeMap<String, TutorialEntry>,
    ) {
        for entry in entries.values() {
            if !entry.has_js {
                continue;
            }
            let src = tut_root.join(&entry.js_copy_rel);
            let dst = workspace_root.join(JS_COPY_ROOT).join(&entry.js_copy_rel);
            if let Some(parent) = dst.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(body) = fs::read_to_string(&src) {
                let _ = super::meta::write_if_changed(&dst, &body);
            }
        }
    }

    /// Minimal JSON-string escaper that emits a TypeScript double-quoted string.
    /// Handles backslashes, double quotes, control characters, and surrogates correctly enough
    /// for JS/TS source embedding.
    fn json_string(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 2);
        out.push('"');
        for ch in s.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\u{08}' => out.push_str("\\b"),
                '\u{0c}' => out.push_str("\\f"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out.push('"');
        out
    }
}
