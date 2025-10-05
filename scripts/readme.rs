mod readme {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    #[derive(Clone, Debug)]
    struct CodeBlock {
        body: String,
    }

    #[derive(Clone, Debug)]
    struct Section {
        first_rust: Option<CodeBlock>,
        first_js: Option<CodeBlock>,
        first_py: Option<CodeBlock>,
    }

    fn workspace_root() -> PathBuf {
        crate::meta::workspace_root()
    }

    fn read(path: &str) -> String {
        let root = workspace_root();
        let p = root.join(path);
        fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {} failed: {}", p.display(), e))
    }

    fn write_if_changed(path: &str, contents: &str) {
        let root = workspace_root();
        let p = root.join(path);
        let _ = crate::meta::write_if_changed(&p, contents)
            .unwrap_or_else(|e| panic!("write {} failed: {}", p.display(), e));
    }

    fn normalize_lang(s: &str) -> String {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "javascript" => "js".into(),
            "typescript" => "js".into(), // treat ts examples as js for README purposes
            "py" => "python".into(),
            other => other.into(),
        }
    }

    fn parse_sections_with_blocks(md: &str) -> HashMap<String, Section> {
        // Very small Markdown walker specialized for headings + fenced blocks
        let mut out: HashMap<String, Section> = HashMap::new();
        let mut current_title: Option<(usize, String)> = None; // (level, title)
        let mut in_code = false;
        let mut code_lang = String::new();
        let mut code_buf: Vec<String> = Vec::new();

        let mut push_block = |title: &str, lang: &str, body: String| {
            let key = title.trim().to_string();
            let entry = out.entry(key.clone()).or_insert(Section {
                first_rust: None,
                first_js: None,
                first_py: None,
            });
            match lang {
                "rust" => {
                    if entry.first_rust.is_none() {
                        entry.first_rust = Some(CodeBlock { body });
                    }
                }
                "js" => {
                    if entry.first_js.is_none() {
                        entry.first_js = Some(CodeBlock { body });
                    }
                }
                "python" => {
                    if entry.first_py.is_none() {
                        entry.first_py = Some(CodeBlock { body });
                    }
                }
                _ => {}
            }
        };

        for line in md.lines() {
            let trimmed = line.trim_start();
            if !in_code {
                // heading?
                if trimmed.starts_with('#') {
                    let mut level = 0usize;
                    for ch in trimmed.chars() {
                        if ch == '#' { level += 1; } else { break; }
                    }
                    let title = trimmed[level..].trim().to_string();
                    current_title = Some((level, title));
                    continue;
                }
                // code block start?
                if trimmed.starts_with("```") {
                    let lang = trimmed.trim_start_matches("```").trim();
                    code_lang = normalize_lang(lang);
                    in_code = true;
                    code_buf.clear();
                    continue;
                }
            } else {
                // in code
                if trimmed.starts_with("```") {
                    // end block
                    if let Some((_lvl, title)) = &current_title {
                        push_block(title, &code_lang, code_buf.join("\n"));
                    }
                    in_code = false;
                    code_lang.clear();
                    code_buf.clear();
                    continue;
                }
                code_buf.push(line.to_string());
            }
        }
        out
    }

    // Replicate the hidden-line handling used by docs exporter: strip leading '#' and optional space
    fn lines_to_items(rust: &str) -> Vec<(String, bool)> {
        let mut items: Vec<(String, bool)> = Vec::new();
        for l in rust.lines() {
            let trimmed = l.trim_start();
            let indent_len = l.len() - trimmed.len();
            if trimmed.starts_with('#') {
                let after = trimmed.trim_start_matches('#').trim_start();
                let new_text = format!("{}{}", &l[..indent_len], after);
                items.push((new_text, true));
            } else {
                items.push((l.to_string(), false));
            }
        }
        items
    }

    fn generate_from_template(
        tpl: &str,
        sections: &HashMap<String, Section>,
        target: &str, // "js" or "python"
    ) -> String {
        // Replace markers: <!-- FC_EXAMPLE heading="..." prefer="rust" copy_native="false" -->
        fn parse_attr(raw: &str, key: &str) -> Option<String> {
            // naive key="..." extractor
            let needle = format!("{}=\"", key);
            if let Some(pos) = raw.find(&needle) {
                let rest = &raw[pos + needle.len()..];
                if let Some(end) = rest.find('\"') {
                    return Some(rest[..end].to_string());
                }
            }
            None
        }

        let mut out = String::new();
        for line in tpl.lines() {
            let lt = line.trim();
            if lt.starts_with("<!--") && lt.contains("FC_EXAMPLE") {
                let heading = parse_attr(lt, "heading").unwrap_or_default();
                let prefer = parse_attr(lt, "prefer").unwrap_or_else(|| "rust".into());
                let copy_native = parse_attr(lt, "copy_native").unwrap_or_else(|| "false".into());
                let copy_native = copy_native == "true";
                let sect = sections.get(&heading).unwrap_or_else(|| panic!("Heading '{}' not found in README.md", heading));

                let fence_lang = match target { "js" => "js", _ => "python" };
                let mut code = String::new();
                if prefer == "rust" {
                    if let Some(rust) = &sect.first_rust {
                        let items = lines_to_items(&rust.body);
                        code = match target {
                            "js" => crate::convert::to_js(&items),
                            _ => crate::convert::to_py(&items),
                        };
                    } else if copy_native {
                        match target {
                            "js" => {
                                if let Some(jsb) = &sect.first_js { code = jsb.body.clone(); }
                            }
                            _ => {
                                if let Some(pyb) = &sect.first_py { code = pyb.body.clone(); }
                            }
                        }
                    }
                }
                if code.trim().is_empty() {
                    panic!(
                        "No suitable example found for heading '{}' and target '{}' (prefer='{}', copy_native={})",
                        heading, target, prefer, copy_native
                    );
                }
                out.push_str(&format!("```{}\n{}\n```\n", fence_lang, code.trim_end()));
                continue;
            }
            out.push_str(line);
            out.push('\n');
        }
        out.trim_end().to_string() + "\n"
    }

    pub fn generate_readmes() {
        // Track changes for cargo builds
        println!("cargo::rerun-if-changed=README.md");
        println!("cargo::rerun-if-changed=README_JS.tpl.md");
        println!("cargo::rerun-if-changed=README_PY.tpl.md");

        let root_readme = read("README.md");
        let sections = parse_sections_with_blocks(&root_readme);

        let tpl_js = read("README_JS.tpl.md");
        let tpl_py = read("README_PY.tpl.md");

        let out_js = generate_from_template(&tpl_js, &sections, "js");
        let out_py = generate_from_template(&tpl_py, &sections, "python");

        write_if_changed("README_JS.md", &out_js);
        write_if_changed("README_PY.md", &out_py);
    }
}
