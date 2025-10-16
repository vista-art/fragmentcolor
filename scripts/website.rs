mod website {
    use super::*;
    use crate::codegen::ApiMap;

    // Public outcome for the export phase so steps can be run independently.
    pub struct ExportOutcome {
        pub expected: std::collections::HashSet<String>,
        pub ex_js: std::collections::HashSet<String>,
        pub ex_py: std::collections::HashSet<String>,
    }


    // Site base used by link normalization
    fn site_base() -> String {
        std::env::var("DOCS_SITE_BASE").unwrap_or_else(|_| "/api".to_string())
    }

    // Normalize links in MDX to site root
    fn rewrite_links_to_site_root(
        mdx: &str,
        top_categories: &std::collections::HashSet<String>,
    ) -> String {
        let base = site_base();
        let mut out = String::new();
        let bytes = mdx.as_bytes();
        let mut i = 0usize;

        let n_https = "](https://fragmentcolor.org/api/".as_bytes();
        let n_http = "](http://fragmentcolor.org/api/".as_bytes();
        let n_www_https = "](https://www.fragmentcolor.org/api/".as_bytes();
        let n_www_http = "](http://www.fragmentcolor.org/api/".as_bytes();

        while i < bytes.len() {
            let mut matched = None::<&[u8]>;
            if i + n_https.len() <= bytes.len() && &bytes[i..i + n_https.len()] == n_https {
                matched = Some(n_https);
            } else if i + n_http.len() <= bytes.len() && &bytes[i..i + n_http.len()] == n_http {
                matched = Some(n_http);
            } else if i + n_www_https.len() <= bytes.len()
                && &bytes[i..i + n_www_https.len()] == n_www_https
            {
                matched = Some(n_www_https);
            } else if i + n_www_http.len() <= bytes.len()
                && &bytes[i..i + n_www_http.len()] == n_www_http
            {
                matched = Some(n_www_http);
            }

            if let Some(m) = matched {
                out.push_str("](");
                i += m.len();
                out.push_str(&base);
                out.push('/');
                while i < bytes.len() && bytes[i] != b')' {
                    out.push(bytes[i] as char);
                    i += 1;
                }
            } else if i + 2 <= bytes.len() && bytes[i] == b']' && bytes[i + 1] == b'(' {
                out.push(']');
                out.push('(');
                i += 2;
                let start = i;
                while i < bytes.len() && bytes[i] != b')' {
                    i += 1;
                }
                let href = &mdx[start..i];
                let href_trim = href.trim();
                let lower = href_trim.to_ascii_lowercase();
                let is_abs = lower.starts_with("http://")
                    || lower.starts_with("https://")
                    || href_trim.starts_with('/')
                    || href_trim.starts_with('#')
                    || lower.starts_with("mailto:")
                    || lower.starts_with("tel:")
                    || lower.starts_with("data:");
                if is_abs {
                    out.push_str(href_trim);
                } else {
                    let top = href_trim.split('/').next().unwrap_or("");
                    if top_categories.contains(top) {
                        out.push_str(&base);
                        if !href_trim.starts_with('/') {
                            out.push('/');
                        }
                        out.push_str(href_trim);
                    } else {
                        out.push_str(href_trim);
                    }
                }
            } else {
                out.push(bytes[i] as char);
                i += 1;
            }
        }
        out
    }

    // Canonicalize a name for fuzzy matching (lowercase, alphanumeric only)
    fn canonicalize_name(s: &str) -> String {
        s.chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_ascii_lowercase()
    }

    // Parse a category _index.md and extract desired order labels from list items.
    // Supported forms:
    // - Renderer
    // * Shader
    // 1. Pass
    // 1) Frame
    // Also supports markdown links: - [Texture](texture.md)
    fn parse_index_order(md: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for line in md.lines() {
            let lt = line.trim_start();
            if lt.starts_with('-') || lt.starts_with('*') {
                let rest = lt[1..].trim_start();
                out.push(extract_label_from_list_item(rest));
                continue;
            }
            // numeric list: e.g. "1. Item" or "1) Item"
            let mut chars = lt.chars().peekable();
            let mut saw_digit = false;
            while let Some(c) = chars.peek() {
                if c.is_ascii_digit() { saw_digit = true; chars.next(); } else { break; }
            }
            if saw_digit
                && let Some(sep) = chars.peek()
                    && (*sep == '.' || *sep == ')') {
                        // consume separator
                        let _ = chars.next();
                        if matches!(chars.peek(), Some(' ')) { let _ = chars.next(); }
                        let rest: String = chars.collect();
                        let lab = extract_label_from_list_item(rest.trim());
                        if !lab.is_empty() { out.push(lab); }
                        continue;
                    }
        }
        // dedup while preserving order
        let mut seen = std::collections::HashSet::new();
        let mut res = Vec::new();
        for s in out {
            let t = s.trim();
            if t.is_empty() { continue; }
            if seen.insert(t.to_string()) { res.push(t.to_string()); }
        }
        res
    }

    fn extract_label_from_list_item(s: &str) -> String {
        if let Some(i) = s.find('[')
            && let Some(j_rel) = s[i + 1..].find(']') {
                let j = i + 1 + j_rel;
                return s[i + 1..j].trim().to_string();
            }
        // Fallback: take text up to a link start or end of line
        let before = s.split('(').next().unwrap_or(s);
        before.trim().to_string()
    }

    /// Step 2: Export examples and pages. Returns the sets needed by later steps.
    pub fn export_examples_and_pages(api_map: &ApiMap) -> ExportOutcome {
        use std::collections::{BTreeMap, HashSet};
        let root = meta::workspace_root();
        let docs_root = root.join("docs/api");
        let site_root = root.join("docs/website/src/content/docs/api");

        // Track expected output files for cleanup; store paths relative to site_root (forward slashes)
        let mut expected: HashSet<String> = HashSet::new();
        // Group objects by category (relative path under docs/api)
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        // Track which objects were processed (to avoid duplicates when scanning extras)
        let mut processed: HashSet<String> = HashSet::new();
        // Collected example file paths for platform healthchecks
        let mut ex_js: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut ex_py: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Build set of known top-level categories for link normalization
        let mut top_categories: HashSet<String> = HashSet::new();
        for (_object, _dir, cat_rel) in scan_docs_objects(&docs_root) {
            if !cat_rel.is_empty()
                && let Some(first) = cat_rel.split('/').next()
            {
                top_categories.insert(first.to_string());
            }
        }

        // Build custom order map per category from optional _index.md files under docs/api
        use std::collections::{BTreeMap as _BTreeMap, HashMap as _HashMap, HashSet as _HashSet};
        let mut all_by_cat: _BTreeMap<String, Vec<String>> = _BTreeMap::new();
        // From docs directory scan
        for (object, _obj_dir, cat_rel) in scan_docs_objects(&docs_root) {
            if is_hidden_category(&cat_rel) { continue; }
            all_by_cat.entry(cat_rel).or_default().push(object);
        }
        // Include base objects that may not have explicit docs yet
        for object in super::validation::base_public_objects().iter() {
            let dir_name = super::validation::object_dir_name(object);
            let obj_dir = find_object_dir(&docs_root, &dir_name).unwrap_or(docs_root.join(&dir_name));
            let cat_rel = if obj_dir.exists() { category_rel_from(&docs_root, &obj_dir) } else { String::new() };
            if is_hidden_category(&cat_rel) { continue; }
            let list = all_by_cat.entry(cat_rel).or_default();
            if !list.iter().any(|o| o == object) { list.push(object.to_string()); }
        }
        for list in all_by_cat.values_mut() { list.sort(); list.dedup(); }

        let mut orders_map: _HashMap<String, _HashMap<String, usize>> = _HashMap::new();
        for (cat, list) in all_by_cat.iter() {
            let dir = if cat.is_empty() { docs_root.clone() } else { docs_root.join(cat) };
            let idx_path = dir.join("_index.md");
            if let Ok(idx_md) = std::fs::read_to_string(&idx_path) {
                let desired = parse_index_order(&idx_md);
                // Build canonical map from canonical form to object name
                let mut canon_to_obj: _HashMap<String, String> = _HashMap::new();
                for obj in list.iter() {
                    let dir_name = super::validation::object_dir_name(obj);
                    canon_to_obj.insert(canonicalize_name(obj), obj.clone());
                    canon_to_obj.insert(canonicalize_name(&dir_name), obj.clone());
                }
                let mut listed: Vec<String> = Vec::new();
                let mut seen: _HashSet<String> = _HashSet::new();
                for name in desired {
                    let key = canonicalize_name(&name);
                    if let Some(obj) = canon_to_obj.get(&key)
                        && seen.insert(obj.clone()) { listed.push(obj.clone()); }
                }
                let mut unlisted: Vec<String> = list
                    .iter().filter(|&o| !seen.contains(o)).cloned()
                    .collect();
                unlisted.sort_by_key(|a| a.to_ascii_lowercase());
                listed.extend(unlisted);
                let mut m: _HashMap<String, usize> = _HashMap::new();
                for (i, obj) in listed.iter().enumerate() { m.insert(obj.clone(), i); }
                orders_map.insert(cat.clone(), m);
            }
        }

        // Helper to write a page given an object, its docs dir, category relative path, and ordered method files
        let mut write_page = |object: &str,
                              obj_dir: &std::path::Path,
                              cat_rel: &str,
                              method_files: Vec<String>|
         -> String {
            let obj_dirname = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let obj_md = std::fs::read_to_string(obj_dir.join(format!("{}.md", obj_dirname)))
                .unwrap_or_default();
            let description = first_paragraph(&obj_md);
            let body = strip_after_methods(&strip_leading_h1(&obj_md));

            let mut out = String::new();
            // Determine sidebar order, if any
            let order_opt: Option<usize> = orders_map
                .get(cat_rel)
                .and_then(|m| m.get(object))
                .cloned();

            out.push_str("---\n");
            out.push_str(&format!("title: {}\n", object));
            let desc = description.replace('\n', " ").replace('"', "\\\"");
            out.push_str(&format!("description: \"{}\"\n", desc));
            if !cat_rel.is_empty() {
                out.push_str(&format!("category: {}\n", cat_rel));
                out.push_str(&format!("categoryLabel: {}\n", category_title(cat_rel)));
            }
            if let Some(order) = order_opt {
                out.push_str("sidebar:\n");
                out.push_str(&format!("  order: {}\n", order));
            }
            out.push_str("---\n\n");

            // Tabs/Code components for examples
            out.push_str(
                "import { Code, Tabs, TabItem, Aside } from \"@astrojs/starlight/components\";\n\n",
            );

            out.push_str("## Description\n\n");

            // Extract and remove the main "## Example" section from the object body (pre-Methods)
            let mut desc_without = String::new();
            let mut main_rust = String::new();
            let mut in_example = false;
            let mut in_code = false;
            let mut capture_rust = false;
            let mut captured_any = false;
            for line in body.lines() {
                let t = line.trim_start();
                if !in_example {
                    if t.starts_with("## Example") {
                        in_example = true;
                        continue; // drop header
                    }
                    desc_without.push_str(line);
                    desc_without.push('\n');
                    continue;
                }
                // in_example section: skip everything until next top-level subsection or EOF
                if !in_code && t.starts_with("## ") {
                    // End of example section; resume copying
                    in_example = false;
                    desc_without.push_str(line);
                    desc_without.push('\n');
                    continue;
                }
                if !in_code && t.starts_with("```") {
                    in_code = true;
                    let is_unlabeled = t == "```";
                    let is_rust = t.starts_with("```rust") || is_unlabeled;
                    capture_rust = is_rust && !captured_any;
                    continue; // do not copy fences
                }
                if in_code && t.starts_with("```") {
                    in_code = false;
                    if capture_rust {
                        captured_any = true;
                        capture_rust = false;
                    }
                    continue; // do not copy fences
                }
                if in_code {
                    if capture_rust {
                        main_rust.push_str(line);
                        main_rust.push('\n');
                    }
                    continue; // do not copy other code content into desc
                }
                // non-code content within example section is dropped from desc
            }

            // Escape inline generics in prose so MDX doesn't treat them as JSX
            let desc_sanitized = sanitize_inline_generics_in_prose(&desc_without);
            out.push_str(&desc_sanitized);
            out.push('\n');

            if captured_any && !main_rust.trim().is_empty() {
                // Use the struct name as file stem per user preference
                let dir_slug = obj_dirname;
                let tabs = build_tabs_for_example(
                    &lines_to_items(&main_rust),
                    cat_rel,
                    dir_slug,
                    object,
                    &root,
                    &mut ex_js,
                    &mut ex_py,
                );
                out.push_str("\n## Example\n\n");
                out.push_str(&tabs);
            }

            out.push_str("\n## Methods\n\n");
            for file in method_files.iter() {
                let md = std::fs::read_to_string(obj_dir.join(format!("{}.md", file)))
                    .unwrap_or_default();

                // Split method content into description (pre) and the first Rust code block after '## Example'
                let mut pre = String::new();
                let mut rust_body = String::new();
                let mut after_example = false;
                let mut in_code = false;
                let mut taking_rust = false;
                for line in md.lines() {
                    let t = line.trim_start();
                    if !after_example && t.starts_with("## Example") {
                        after_example = true;
                        continue;
                    }
                    if after_example {
                        if !in_code && t.starts_with("```") {
                            in_code = true;
                            taking_rust = true; // assume the first block is Rust
                            continue;
                        } else if in_code && t.starts_with("```") {
                            in_code = false;
                            taking_rust = false;
                            continue;
                        }
                        if taking_rust {
                            rust_body.push_str(line);
                            rust_body.push('\n');
                        }
                    } else {
                        pre.push_str(line);
                        pre.push('\n');
                    }
                }

                // Separator before each method title for visual grouping
                out.push_str("\n---\n\n");
                // Sanitize prose for MDX generics first, then downshift headings
                let pre_sanitized = sanitize_inline_generics_in_prose(pre.trim_end());
                out.push_str(&downshift_headings(&pre_sanitized));
                out.push('\n');

                // Build example tabs via shared helper
                let dir_slug = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let items = lines_to_items(&rust_body);
                let tabs = build_tabs_for_example(
                    &items, cat_rel, dir_slug, file, &root, &mut ex_js, &mut ex_py,
                );
                out.push_str("\n#### Example\n\n");
                out.push_str(&tabs);
            }

            // Ensure exactly one trailing newline at EOF
            let mut out = out.trim_end().to_string();
            out.push('\n');

            // Normalize links in MDX to site root

            out = rewrite_links_to_site_root(&out, &top_categories);

            // Add platform WIP banner for Android/iOS pages
            out = inject_platform_banner_if_needed(object, &out);

            // Transform Rust code fences: strip hidden '#', move use-lines, add collapse ranges
            out = transform_rust_code_fences(&out);

            let site_file = if cat_rel.is_empty() {
                site_root.join(format!("{}.mdx", object.to_lowercase()))
            } else {
                site_root
                    .join(cat_rel)
                    .join(format!("{}.mdx", object.to_lowercase()))
            };
            if let Some(parent) = site_file.parent() {
                std::fs::create_dir_all(parent).unwrap();
            }
            let _ = super::meta::write_if_changed(&site_file, &out);

            // Return relative path for cleanup
            if cat_rel.is_empty() {
                format!("{}.mdx", object.to_lowercase())
            } else {
                format!("{}/{}.mdx", cat_rel, object.to_lowercase())
            }
        };

        // Iterate objects discovered from AST (base objects only)
        let objects = super::validation::base_public_objects();
        for object in objects.iter() {
            let dir_name = super::validation::object_dir_name(object);
            let obj_dir =
                find_object_dir(&docs_root, &dir_name).unwrap_or(docs_root.join(&dir_name));
            let cat_rel = if obj_dir.exists() {
                category_rel_from(&docs_root, &obj_dir)
            } else {
                String::new()
            };

            // Determine method files from codegen
            let mut method_files = Vec::new();
            if let Some(methods) = api_map.get(object) {
                for m in methods {
                    if let Some(fun) = &m.function {
                        let name = &fun.name;
                        let file = name.clone();
                        if obj_dir.join(format!("{}.md", file)).exists() {
                            method_files.push(file);
                        }
                    }
                }
            }

            // If no methods were discovered, fall back to docs files in the folder
            if method_files.is_empty()
                && obj_dir.exists()
                && obj_dir.is_dir()
                && let Ok(read_dir) = std::fs::read_dir(&obj_dir)
            {
                let mut files: Vec<String> = read_dir
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let p = e.path();
                        if p.extension()?.to_str()? == "md" {
                            let stem = p.file_stem()?.to_str()?.to_string();
                            if stem != dir_name { Some(stem) } else { None }
                        } else {
                            None
                        }
                    })
                    .collect();
                files.sort();
                method_files = files;
            }

            if is_hidden_category(&cat_rel) {
                processed.insert(object.to_string());
                continue;
            }
            let rel = write_page(object, &obj_dir, &cat_rel, method_files);
            expected.insert(rel);
            groups.entry(cat_rel).or_default().push(object.to_string());
            processed.insert(object.to_string());
        }

        // Extras: any docs-only objects in docs/api (recursively) not already processed
        for (object, obj_dir, cat_rel) in scan_docs_objects(&docs_root) {
            if processed.contains(&object) {
                continue;
            }
            let dir_name = obj_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let mut method_files: Vec<String> = Vec::new();
            if let Ok(files) = std::fs::read_dir(&obj_dir) {
                method_files = files
                    .filter_map(|e| e.ok())
                    .filter_map(|e| {
                        let p = e.path();
                        if p.extension()?.to_str()? == "md" {
                            let stem = p.file_stem()?.to_str()?.to_string();
                            if stem != dir_name { Some(stem) } else { None }
                        } else {
                            None
                        }
                    })
                    .collect();
                method_files.sort();
            }
            if is_hidden_category(&cat_rel) {
                processed.insert(object);
                continue;
            }
            let rel = write_page(&object, &obj_dir, &cat_rel, method_files);
            expected.insert(rel);
            groups.entry(cat_rel).or_default().push(object.clone());
            processed.insert(object);
        }

        // Groups sorted for index
        for list in groups.values_mut() {
            list.sort();
        }
        let mut top = groups.remove("").unwrap_or_default();
        top.sort();
        let mut cats: Vec<String> = groups.keys().cloned().collect();
        cats.sort();

        // Build index.mdx
        let mut idx = String::new();
        idx.push_str("---\n");
        idx.push_str("title: API\n");
        idx.push_str("description: \"Auto-generated API index\"\n");
        idx.push_str("---\n\n");
        idx.push_str("# API\n\n");

        // Build site base for index links
        let base = std::env::var("DOCS_SITE_BASE").unwrap_or_else(|_| "/api".to_string());
        let base = base.trim_end_matches('/');

        // Top-level (no category) first
        for o in &top {
            let path = format!("{}/{}", base, o.to_lowercase());
            idx.push_str(&format!("- [{}]({})\n", o, path));
        }
        if !top.is_empty() {
            idx.push('\n');
        }

        // Then each category alphabetically
        for cat in cats {
            if let Some(list) = groups.get(&cat) {
                let label = category_title(&cat);
                idx.push_str(&format!("## {}\n\n", label));
                for o in list {
                    let path = format!("{}/{}/{}", base, cat, o.to_lowercase());
                    idx.push_str(&format!("- [{}]({})\n", o, path));
                }
                idx.push('\n');
            }
        }

        let mut idx = idx.trim_end().to_string();
        idx.push('\n');
        let _ = super::meta::write_if_changed(&site_root.join("index.mdx"), &idx);

        ExportOutcome {
            expected,
            ex_js,
            ex_py,
        }
    }

    /// Step 3: cleanup stale site files (non-destructive to source docs)
    pub fn cleanup_site(expected: &std::collections::HashSet<String>) {
        let root = meta::workspace_root();
        let site_root = root.join("docs/website/src/content/docs/api");
        fn walk_and_cleanup(
            root: &std::path::Path,
            base: &std::path::Path,
            expected: &std::collections::HashSet<String>,
        ) {
            if let Ok(read_dir) = std::fs::read_dir(root) {
                for entry in read_dir.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        walk_and_cleanup(&path, base, expected);
                        continue;
                    }
                    if let Some(ext) = path.extension().and_then(|s| s.to_str())
                        && ext == "mdx"
                        && let Ok(rel) = path.strip_prefix(base)
                    {
                        let key = rel.to_string_lossy().replace('\\', "/");
                        if !expected.contains(&key) {
                            let _ = std::fs::remove_file(&path);
                        }
                    }
                }
            }
        }
        walk_and_cleanup(&site_root, &site_root, expected);
    }

    /// Step 4: write healthcheck aggregators referencing all generated example files
    pub fn write_healthcheck_aggregators(
        ex_js: &std::collections::HashSet<String>,
        ex_py: &std::collections::HashSet<String>,
    ) {
        let root = meta::workspace_root();

        // Sort and write JS aggregator
        let mut js_list: Vec<String> = ex_js.iter().cloned().collect();
        js_list.sort();
        let js_path = root.join("platforms/web/healthcheck/generated_examples.mjs");
        if let Some(parent) = js_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut js_out = String::new();
        js_out.push_str("// Auto-generated: runs all JS examples with cargo-like output.\n");
        js_out.push_str(
            "const GREEN='\\u001b[1;32m'; const RED='\\u001b[1;31m'; const RESET='\\u001b[0m';\n",
        );
        js_out.push_str("const EXAMPLES = [\n");
        for rel in &js_list {
            js_out.push_str(&format!("  '../examples/{}',\n", rel));
        }
        js_out.push_str("]\n\n");
        js_out.push_str("function fq(rel){ return 'platforms.web.examples.' + rel.replace('../examples/','').replace(/\\\\.js$/, '').replaceAll('/', '.'); }\n");
        js_out.push_str("export async function runExamples() {\n  const total = EXAMPLES.length;\n  let passed = 0;\n  let failed = 0;\n  console.log(`running ${total} tests`);\n  globalThis.__HC = globalThis.__HC || { currentModule: null };\n  for (const rel of EXAMPLES) {\n    const name = fq(rel);\n    const head = `test ${name} ... `;\n    try {\n      globalThis.__HC.currentModule = name;\n      await import(rel);\n      passed++;\n      console.log(head + GREEN + 'OK' + RESET);\n    } catch (e) {\n      failed++;\n      console.log(head + RED + 'FAILED' + RESET);\n      console.error(e);\n    } finally {\n      globalThis.__HC.currentModule = null;\n    }\n  }\n  return { passed, failed };\n}\n");
        let _ = super::meta::write_if_changed(&js_path, &js_out);

        // Sort and write Python aggregator
        let mut py_list: Vec<String> = ex_py.iter().cloned().collect();
        py_list.sort();
        let py_path = root.join("platforms/python/examples/main.py");
        if let Some(parent) = py_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut py_out = String::new();
        py_out.push_str("# Auto-generated: executes all Python examples with cargo-like output.\n");
        py_out.push_str("import runpy, pathlib, sys, traceback, os\n\n");
        py_out.push_str("GREEN='\x1b[1;32m'\nRED='\x1b[1;31m'\nRESET='\x1b[0m'\n\n");
        py_out.push_str("def run_all():\n");
        py_out.push_str("    base = pathlib.Path(__file__).parent\n");
        py_out.push_str("    files = [\n");
        for rel in &py_list {
            let rel_norm = rel.replace('\\', "/");
            py_out.push_str(&format!("        '{}',\n", rel_norm));
        }
        py_out.push_str("    ]\n");
        py_out.push_str("\n    # Announce test count and optionally prepare summary file\n");
        py_out.push_str("    total = len(files)\n");
        py_out.push_str("    print(f\"running {total} tests\")\n");
        py_out.push_str("    summary_path = os.environ.get('FC_PY_SUMMARY_PATH')\n");
        py_out.push_str("    if summary_path:\n");
        py_out.push_str("        try:\n");
        py_out.push_str("            with open(summary_path, 'w') as f:\n");
        py_out.push_str("                f.write(f\"total={total}\\npassed=0\\nfailed=0\\n\")\n");
        py_out.push_str("        except Exception:\n");
        py_out.push_str("            pass\n");
        py_out.push_str("\n    passed = 0\n");
        py_out.push_str("    failed = 0\n");
        py_out.push_str("    for rel in files:\n");
        py_out.push_str("        name = 'platforms.python.examples.' + rel.replace('/', '.').removesuffix('.py')\n");
        py_out.push_str("        head = f'test {name} ... '\n");
        py_out.push_str("        os.environ['FC_RUNNER'] = 'python'\n");
        py_out.push_str("        os.environ['FC_CURRENT_TEST'] = name\n");
        py_out.push_str("        try:\n");
        py_out.push_str("            runpy.run_path(str(base / rel), run_name='__main__')\n");
        py_out.push_str("            passed += 1\n");
        py_out.push_str("            print(head + GREEN + 'OK' + RESET)\n");
        py_out.push_str("        except Exception:\n");
        py_out.push_str("            failed += 1\n");
        py_out.push_str("            print(head + RED + 'FAILED' + RESET)\n");
        py_out.push_str("            traceback.print_exc()\n");
        py_out.push_str("\n    if summary_path:\n");
        py_out.push_str("        try:\n");
        py_out.push_str("            with open(summary_path, 'w') as f:\n");
        py_out.push_str("                f.write(f\"total={total}\\npassed={passed}\\nfailed={failed}\\n\")\n");
        py_out.push_str("        except Exception:\n");
        py_out.push_str("            pass\n");
        py_out.push_str("\n    if failed:\n");
        py_out.push_str("        print(f\"\\n{RED}test result: FAILED{RESET}. {passed} passed; {failed} failed\")\n");
        py_out.push_str("        raise SystemExit(1)\n");
        py_out.push_str("    else:\n");
        py_out.push_str("        print(f\"\\n{GREEN}test result: ok{RESET}. {passed} passed; {failed} failed\")\n");
        py_out.push_str("\nif __name__ == '__main__':\n    run_all()\n");
        let _ = super::meta::write_if_changed(&py_path, &py_out);
    }

    // Escape backticks for MDX template literal usage in <Code code={`...`} />
    fn sanitize_for_template(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0usize;
        while i < chars.len() {
            let ch = chars[i];
            if ch == '`' {
                // Count consecutive backslashes immediately before this backtick
                let mut bs = 0usize;
                let mut j = i;
                while j > 0 {
                    if chars[j - 1] == '\\' {
                        bs += 1;
                        j -= 1;
                    } else {
                        break;
                    }
                }
                // If the number of preceding backslashes is even (including 0),
                // this backtick is not escaped yet. Add one backslash to escape it.
                if bs.is_multiple_of(2) {
                    out.push('\\');
                }
                out.push('`');
                i += 1;
                continue;
            }
            out.push(ch);
            i += 1;
        }
        out
    }

    // Sanitize inline generics like Result<(), E> or vec2<f32> in prose (non-code) so MDX doesn't parse them as JSX
    // This wraps tokens of the form IDENT<...> in backticks, respecting existing inline code spans.
    fn sanitize_inline_generics_in_prose(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 16);
        let chars: Vec<char> = s.chars().collect();
        let mut i = 0usize;
        let mut in_tick = false;
        while i < chars.len() {
            let c = chars[i];
            if c == '`' {
                in_tick = !in_tick;
                out.push('`');
                i += 1;
                continue;
            }
            // Escape bare braces, which MDX treats as expression delimiters
            if !in_tick && (c == '{' || c == '}') {
                if c == '{' { out.push_str("&#123;"); } else { out.push_str("&#125;"); }
                i += 1;
                continue;
            }
            if !in_tick && c == '<' {
                // Try to find the start of the identifier before '<'
                let mut j = i;
                let mut start = i;
                if j > 0 {
                    j -= 1;
                    while j > 0 {
                        let cj = chars[j];
                        if cj.is_ascii_alphanumeric() || cj == '_' || cj == ':' {
                            j -= 1;
                        } else {
                            break;
                        }
                    }
                    // Adjust start position
                    if j == 0 {
                        if chars[j].is_ascii_alphanumeric() || chars[j] == '_' {
                            start = 0;
                        } else {
                            start = j + 1;
                        }
                    } else {
                        start = j + 1;
                    }
                }
                // Ensure there is actually an identifier immediately before '<'
                if start < i && (chars[start].is_ascii_alphabetic() || chars[start] == '_') {
                    // Find the matching '>' (do not cross line)
                    let mut k = i + 1;
                    let mut found_gt = None;
                    while k < chars.len() {
                        if chars[k] == '>' {
                            found_gt = Some(k);
                            break;
                        }
                        // stop if we hit backtick or start of an MD link/image '![', or a code fence marker (shouldn't appear in prose blocks we pass in)
                        if chars[k] == '\n' { break; }
                        k += 1;
                    }
                    if let Some(end_gt) = found_gt {
                        // Wrap from start..=end_gt in backticks
                        out.push('`');
                        for &ch in &chars[start..=end_gt] {
                            out.push(ch);
                        }
                        out.push('`');
                        i = end_gt + 1;
                        continue;
                    }
                }
                // Fallback: no identifier, or no closing '>' — just output '<'
                out.push(c);
                i += 1;
                continue;
            }
            out.push(c);
            i += 1;
        }
        out
    }

    // Reusable: turn a Rust code body into (text, hidden) items using doc-test '#'
    fn lines_to_items(rust: &str) -> Vec<(String, bool)> {
        let mut items: Vec<(String, bool)> = Vec::new();
        for l in rust.lines() {
            let trimmed = l.trim_start();
            let indent_len = l.len() - trimmed.len();
            if trimmed.starts_with('#') {
                let after = if let Some(stripped) = trimmed.strip_prefix('#') {
                    stripped
                } else {
                    trimmed
                };
                // Optional single space after '#'
                let after = if let Some(stripped) = after.strip_prefix(' ') {
                    stripped
                } else {
                    after
                };
                let new_text = format!("{}{}", &l[..indent_len], after);
                items.push((new_text, true));
            } else {
                items.push((l.to_string(), false));
            }
        }
        items
    }

    // Central helper to build Tabs for an example and persist JS/Py files
    fn build_tabs_for_example(
        items: &[(String, bool)],
        cat_rel: &str,
        obj_dir_slug: &str,
        file_stem: &str,
        root: &std::path::Path,
        ex_js: &mut std::collections::HashSet<String>,
        ex_py: &mut std::collections::HashSet<String>,
    ) -> String {
        // Compute collapse ranges (1-based inclusive) from hidden runs
        let mut ranges: Vec<(usize, usize)> = Vec::new();
        let mut i_ln = 0usize;
        while i_ln < items.len() {
            if items[i_ln].1 {
                let start = i_ln + 1;
                while i_ln < items.len() && items[i_ln].1 {
                    i_ln += 1;
                }
                let end = i_ln; // inclusive
                ranges.push((start, end));
            } else {
                i_ln += 1;
            }
        }
        let rust_lines: Vec<String> = items.iter().map(|(t, _)| t.clone()).collect();
        let rust_processed = rust_lines.join("\n");
        let meta_attr = if ranges.is_empty() {
            String::new()
        } else {
            let mut s = String::from(" meta=\"collapse={");
            for (idx, (s1, e1)) in ranges.iter().enumerate() {
                if idx > 0 {
                    s.push_str(", ");
                }
                s.push_str(&format!("{}-{}", s1, e1));
            }
            s.push_str("}\"");
            s
        };

        // Convert to JS/Python
        let js_code = crate::convert::to_js(items);
        let py_code = crate::convert::to_py(items);

        // Persist example files for healthchecks
        let js_rel = if cat_rel.is_empty() {
            format!("{}/{}.js", obj_dir_slug, file_stem)
        } else {
            format!("{}/{}/{}.js", cat_rel, obj_dir_slug, file_stem)
        };
        let py_rel = if cat_rel.is_empty() {
            format!("{}/{}.py", obj_dir_slug, file_stem)
        } else {
            format!("{}/{}/{}.py", cat_rel, obj_dir_slug, file_stem)
        };
        let js_abs = root.join("platforms/web/examples").join(&js_rel);
        let py_abs = root.join("platforms/python/examples").join(&py_rel);
        if let Some(p) = js_abs.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        if let Some(p) = py_abs.parent() {
            let _ = std::fs::create_dir_all(p);
        }
        let _ = super::meta::write_if_changed(&js_abs, &js_code);
        let _ = super::meta::write_if_changed(&py_abs, &py_code);
        ex_js.insert(js_rel.clone());
        ex_py.insert(py_rel.clone());

        // Swift/Kotlin placeholders
        let sk_rel = if cat_rel.is_empty() {
            format!("{}/{}.txt", obj_dir_slug, file_stem)
        } else {
            format!("{}/{}/{}.txt", cat_rel, obj_dir_slug, file_stem)
        };
        let swift_abs = root.join("platforms/swift/examples").join(&sk_rel);
        let kotlin_abs = root.join("platforms/kotlin/examples").join(&sk_rel);
        if let Some(parent) = swift_abs.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Some(parent) = kotlin_abs.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if !swift_abs.exists() {
            let _ =
                super::meta::write_if_changed(&swift_abs, "// Swift placeholder: bindings WIP\n");
        }
        if !kotlin_abs.exists() {
            let _ =
                super::meta::write_if_changed(&kotlin_abs, "// Kotlin placeholder: bindings WIP\n");
        }
        let swift_code = std::fs::read_to_string(&swift_abs)
            .unwrap_or_else(|_| "// Swift placeholder: bindings WIP\n".to_string());
        let kotlin_code = std::fs::read_to_string(&kotlin_abs)
            .unwrap_or_else(|_| "// Kotlin placeholder: bindings WIP\n".to_string());

        // Build Tabs snippet
        let mut tabs = String::new();
        tabs.push_str("<Tabs>\n\n");

        tabs.push_str("<TabItem label=\"Rust\">\n");
        tabs.push_str("<Code\n");
        tabs.push_str("code={`\n");
        tabs.push_str(&rust_processed);
        tabs.push_str("\n`}\n");
        tabs.push_str("lang=\"rust\"");
        tabs.push_str(&meta_attr);
        tabs.push_str("\n/>\n\n</TabItem>\n\n");

        tabs.push_str("<TabItem label=\"JavaScript\">\n");
        tabs.push_str("<Code\n");
        tabs.push_str("code={`\n");
        tabs.push_str(&sanitize_for_template(&js_code));
        tabs.push_str("\n`}\nlang=\"js\"\n/>\n\n</TabItem>\n\n");

        tabs.push_str("<TabItem label=\"Python\">\n");
        tabs.push_str("<Code\n");
        tabs.push_str("code={`\n");
        tabs.push_str(&sanitize_for_template(&py_code));
        tabs.push_str("\n`}\nlang=\"python\"\n/>\n\n</TabItem>\n\n");

        tabs.push_str("<TabItem label=\"Swift\">\n");
        tabs.push_str("<Aside type=\"caution\" title=\"Work in progress\">Swift bindings are not yet available; implementation is in the works.</Aside>\n");
        tabs.push_str("<Code\n");
        tabs.push_str("code={`\n");
        tabs.push_str(&swift_code);
        tabs.push_str("\n`}\nlang=\"text\"\n/>\n\n</TabItem>\n\n");

        tabs.push_str("<TabItem label=\"Kotlin\">\n");
        tabs.push_str("<Aside type=\"caution\" title=\"Work in progress\">Kotlin/Android bindings are not yet available; implementation is in the works.</Aside>\n");
        tabs.push_str("<Code\n");
        tabs.push_str("code={`\n");
        tabs.push_str(&kotlin_code);
        tabs.push_str("\n`}\nlang=\"text\"\n/>\n\n</TabItem>\n\n");

        tabs.push_str("</Tabs>\n\n");
        tabs
    }

    fn first_paragraph(md: &str) -> String {
        let mut lines = md.lines();
        // Skip the first H1 line entirely
        for line in lines.by_ref() {
            if line.trim_start().starts_with('#') {
                break;
            }
        }

        let mut out = String::new();
        let mut started = false;
        for line in lines {
            let t = line.trim();
            // Skip any heading lines
            if t.starts_with('#') {
                if started && !out.is_empty() {
                    break;
                } else {
                    continue;
                }
            }
            // Stop on blank line once we've started
            if t.is_empty() {
                if started && !out.is_empty() {
                    break;
                } else {
                    continue;
                }
            }
            started = true;
            out.push_str(line);
            out.push('\n');
        }
        out.trim().to_string()
    }

    fn escape_mdx_specials_in_heading_text(s: &str) -> String {
        // Escape MDX-reserved characters in headings so MDX doesn't parse them as JSX/expressions.
        // Respect inline code spans (`...`), leaving their contents untouched.
        let mut out = String::new();
        let mut in_tick = false;
        for ch in s.chars() {
            if ch == '`' {
                in_tick = !in_tick;
                out.push(ch);
                continue;
            }
            if in_tick {
                out.push(ch);
            } else {
                match ch {
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '{' => out.push_str("&#123;"),
                    '}' => out.push_str("&#125;"),
                    _ => out.push(ch),
                }
            }
        }
        out
    }

    fn downshift_headings(md: &str) -> String {
        // Shift headings down by two levels so that method H1 becomes H3 under the "## Methods" section.
        let mut out = String::new();
        let mut in_code = false;
        for l in md.lines() {
            let line = l;
            if line.trim_start().starts_with("```") {
                in_code = !in_code;
                out.push_str(line);
                out.push('\n');
                continue;
            }
            if in_code {
                out.push_str(line);
                out.push('\n');
                continue;
            }
            let mut shifted = if let Some(stripped) = line.strip_prefix("######") {
                format!("######{}", stripped)
            } else if let Some(stripped) = line.strip_prefix("#####") {
                format!("######{}", stripped)
            } else if let Some(stripped) = line.strip_prefix("####") {
                format!("######{}", stripped)
            } else if let Some(stripped) = line.strip_prefix("###") {
                format!("#####{}", stripped)
            } else if let Some(stripped) = line.strip_prefix("##") {
                format!("####{}", stripped)
            } else if let Some(stripped) = line.strip_prefix('#') {
                format!("###{}", stripped)
            } else {
                line.to_string()
            };
            // Escape MDX-reserved characters in headings (outside code blocks) to avoid parsing errors.
            if shifted.trim_start().starts_with('#') {
                shifted = escape_mdx_specials_in_heading_text(&shifted);
            }
            out.push_str(&shifted);
            out.push('\n');
        }
        out.trim_end().to_string()
    }

    fn find_object_dir(docs_root: &std::path::Path, dir_name: &str) -> Option<std::path::PathBuf> {
        fn walk(root: &std::path::Path, target: &str) -> Option<std::path::PathBuf> {
            if !root.is_dir() {
                return None;
            }
            for entry in std::fs::read_dir(root).ok()?.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if p.file_name().and_then(|s| s.to_str()) == Some(target) {
                        let md = p.join(format!("{}.md", target));
                        if md.exists() {
                            return Some(p);
                        }
                    }
                    if let Some(found) = walk(&p, target) {
                        return Some(found);
                    }
                }
            }
            None
        }
        walk(docs_root, dir_name)
    }

    fn category_rel_from(docs_root: &std::path::Path, obj_dir: &std::path::Path) -> String {
        let parent = obj_dir.parent().unwrap_or(docs_root);
        if let Ok(rel) = parent.strip_prefix(docs_root) {
            rel.to_string_lossy().replace('\\', "/")
        } else {
            String::new()
        }
    }

    fn scan_docs_objects(docs_root: &std::path::Path) -> Vec<(String, std::path::PathBuf, String)> {
        fn walk(
            dir: &std::path::Path,
            root: &std::path::Path,
            out: &mut Vec<(String, std::path::PathBuf, String)>,
        ) {
            if !dir.is_dir() {
                return;
            }
            for entry in std::fs::read_dir(dir).ok().into_iter().flatten().flatten() {
                let p = entry.path();
                if p.is_dir() {
                    if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                        let md = p.join(format!("{}.md", name));
                        if md.exists() {
                            let object = super::validation::dir_to_object_name(name);
                            let cat = category_rel_from(root, &p);
                            out.push((object, p.clone(), cat));
                            // Do not descend into this object dir further
                            continue;
                        }
                    }
                    walk(&p, root, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(docs_root, docs_root, &mut out);
        out
    }

    fn is_hidden_category(cat_rel: &str) -> bool {
        cat_rel.split('/').any(|seg| seg == "hidden")
    }

    fn category_title(cat_rel: &str) -> String {
        if cat_rel.is_empty() {
            return String::new();
        }
        cat_rel
            .split('/')
            .map(|seg| {
                let mut chars = seg.chars();
                match chars.next() {
                    Some(first) => {
                        first.to_uppercase().collect::<String>()
                            + &chars.as_str().to_ascii_lowercase()
                    }
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }
    fn strip_leading_h1(md: &str) -> String {
        let mut out = String::new();
        let mut first = true;
        for line in md.lines() {
            if first && line.trim_start().starts_with('#') {
                first = false;
                continue; // skip the H1 line entirely
            }
            first = false;
            out.push_str(line);
            out.push('\n');
        }
        out.trim_start().to_string()
    }

    fn strip_after_methods(md: &str) -> String {
        let mut out = String::new();
        for line in md.lines() {
            if line.trim_start().starts_with("## Methods") {
                break;
            }
            out.push_str(line);
            out.push('\n');
        }
        out.trim_end().to_string()
    }

    /// Post-process MDX content to transform Rust code fences:
    /// - Strip leading `#` from hidden lines (doc-test convention)
    /// - Move top-level `use ...` lines inside the first hidden wrapper block
    /// - Compute collapse ranges for all contiguous hidden runs and annotate the fence
    ///
    /// Only affects ```rust* code fences. All other languages are left unchanged.
    fn transform_rust_code_fences(mdx: &str) -> String {
        #[derive(Clone)]
        struct LineItem {
            text: String,
            hidden: bool,
        }

        fn process_rust_block(header: &str, body: &[String]) -> (String, Vec<String>) {
            // Build items with hidden flags by stripping leading '#'
            let mut items: Vec<LineItem> = Vec::with_capacity(body.len());
            for l in body {
                let trimmed = l.trim_start();
                let indent_len = l.len() - trimmed.len();
                if trimmed.starts_with('#') {
                    // Strip one leading '#' and one optional following space
                    let after = if let Some(stripped) = trimmed.strip_prefix('#') {
                        stripped
                    } else {
                        trimmed
                    };

                    let after = if let Some(stripped) = after.strip_prefix(' ') {
                        stripped
                    } else {
                        after
                    };

                    let new_text = format!("{}{}", &l[..indent_len], after);
                    items.push(LineItem {
                        text: new_text,
                        hidden: true,
                    });
                } else {
                    items.push(LineItem {
                        text: l.clone(),
                        hidden: false,
                    });
                }
            }

            // Compute collapse ranges over final items
            let mut ranges: Vec<(usize, usize)> = Vec::new();
            let mut i = 0usize;
            let mut line_no = 1usize;
            while i < items.len() {
                if items[i].hidden {
                    let start = line_no;
                    while i < items.len() && items[i].hidden {
                        i += 1;
                        line_no += 1;
                    }
                    let end = line_no.saturating_sub(1);
                    if end >= start {
                        ranges.push((start, end));
                    }
                    continue;
                }
                i += 1;
                line_no += 1;
            }

            // Prepare new header with collapse={...}
            let mut new_header = header.to_string();
            if !ranges.is_empty() {
                let mut collapse = String::new();
                collapse.push_str("collapse={");
                for (idx, (s, e)) in ranges.iter().enumerate() {
                    if idx > 0 {
                        collapse.push_str(", ");
                    }
                    // Ranges are already computed as 0-based above.
                    collapse.push_str(&format!("{}-{}", s, e));
                }
                collapse.push('}');

                // Remove any existing collapse={...} chunk in header (best-effort without regex)
                if let Some(pos) = new_header.find("collapse={")
                    && let Some(end_rel) = new_header[pos..].find('}')
                {
                    let end = pos + end_rel + 1;
                    new_header.replace_range(pos..end, "");
                }
                if !new_header.ends_with(' ') {
                    new_header.push(' ');
                }
                new_header.push_str(&collapse);
            }

            let new_body: Vec<String> = items.into_iter().map(|it| it.text).collect();
            (new_header, new_body)
        }

        let mut out = String::new();
        let mut in_code = false;
        let mut header = String::new();
        let mut is_rust = false;
        let mut block: Vec<String> = Vec::new();

        for line in mdx.lines() {
            if !in_code {
                if line.starts_with("```") {
                    header = line.to_string();
                    let trimmed = header.trim();
                    if trimmed == "```" {
                        // Default unlabeled fences to rust for our docs pages
                        header = "```rust".to_string();
                        is_rust = true;
                    } else {
                        is_rust = header.starts_with("```rust");
                    }
                    in_code = true;
                    block.clear();
                } else {
                    out.push_str(line);
                    out.push('\n');
                }
            } else if line.starts_with("```") {
                // End of block
                if is_rust {
                    let (new_header, new_body) = process_rust_block(&header, &block);
                    out.push_str(&new_header);
                    out.push('\n');
                    for l in new_body {
                        out.push_str(&l);
                        out.push('\n');
                    }
                    out.push_str("```");
                    out.push('\n');
                } else {
                    out.push_str(&header);
                    out.push('\n');
                    for l in &block {
                        out.push_str(l);
                        out.push('\n');
                    }
                    out.push_str("```");
                    out.push('\n');
                }
                in_code = false;
                header.clear();
                is_rust = false;
                block.clear();
            } else {
                block.push(line.to_string());
            }
        }

        // If the MDX ended while inside a code block, flush and ensure it is closed
        if in_code {
            out.push_str(&header);
            out.push('\n');
            for l in &block {
                out.push_str(l);
                out.push('\n');
            }
            out.push_str("```");
            out.push('\n');
        }

        out
    }

    fn inject_platform_banner_if_needed(object: &str, mdx: &str) -> String {
        fn needs_banner(obj: &str) -> bool {
            obj.starts_with("Android") || obj.starts_with("Ios")
        }
        if !needs_banner(object) {
            return mdx.to_string();
        }
        // Insert after front matter end (---\n\n). If not found, prepend at top.
        let banner_import = "import { Aside } from \"@astrojs/starlight/components\";\n\n";
        let banner = "<Aside type=\"caution\" title=\"Work in progress\">\nThese platform bindings are not yet available; implementation is in the works. APIs may change.\n</Aside>\n\n";
        if let Some(idx) = mdx.find("---\n\n") {
            let (head, tail) = mdx.split_at(idx + 4);
            let mut out = String::with_capacity(mdx.len() + banner.len() + banner_import.len());
            out.push_str(head);
            out.push_str(banner_import);
            out.push_str(banner);
            out.push_str(tail);
            out
        } else {
            let mut out = String::new();
            out.push_str(banner_import);
            out.push_str(banner);
            out.push_str(mdx);
            out
        }
    }
}
