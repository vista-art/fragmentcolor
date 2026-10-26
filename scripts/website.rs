mod website {
    use super::*;
    use crate::codegen::ApiMap;
    use crate::example::{Example, Lang};
    use std::collections::HashMap;

    // Public outcome for the export phase so steps can be run independently.
    pub struct ExportOutcome {
        pub expected: std::collections::HashSet<String>,
        pub ex_js: std::collections::HashSet<String>,
        pub ex_py: std::collections::HashSet<String>,
        pub ex_swift: std::collections::HashSet<String>,
        pub ex_kotlin: std::collections::HashSet<String>,
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
        // Byte buffer rather than String: the scanner copies source bytes
        // verbatim, so a `String` push of `bytes[i] as char` would reinterpret
        // each UTF-8 continuation byte as its own Latin-1 codepoint and mangle
        // every non-ASCII character (em dashes, ellipses, accents) into mojibake.
        // Everything inserted here is ASCII, so the result stays valid UTF-8.
        let mut out: Vec<u8> = Vec::with_capacity(mdx.len());
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
                out.extend_from_slice(b"](");
                i += m.len();
                out.extend_from_slice(base.as_bytes());
                out.push(b'/');
                while i < bytes.len() && bytes[i] != b')' {
                    out.push(bytes[i]);
                    i += 1;
                }
            } else if i + 2 <= bytes.len() && bytes[i] == b']' && bytes[i + 1] == b'(' {
                out.push(b']');
                out.push(b'(');
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
                    out.extend_from_slice(href_trim.as_bytes());
                } else {
                    let top = href_trim.split('/').next().unwrap_or("");
                    if top_categories.contains(top) {
                        out.extend_from_slice(base.as_bytes());
                        if !href_trim.starts_with('/') {
                            out.push(b'/');
                        }
                        out.extend_from_slice(href_trim.as_bytes());
                    } else {
                        out.extend_from_slice(href_trim.as_bytes());
                    }
                }
            } else {
                out.push(bytes[i]);
                i += 1;
            }
        }
        String::from_utf8_lossy(&out).into_owned()
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
    pub fn export_examples_and_pages(
        catalog: &super::codegen::ApiCatalog,
        api_map: &ApiMap,
    ) -> ExportOutcome {
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
        let mut ex_swift: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut ex_kotlin: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Build set of known top-level categories for link normalization
        let mut top_categories: HashSet<String> = HashSet::new();
        for (_object, _dir, cat_rel) in super::docs::scan_docs_objects(&docs_root) {
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
        for (object, _obj_dir, cat_rel) in super::docs::scan_docs_objects(&docs_root) {
            if is_hidden_category(&cat_rel) { continue; }
            all_by_cat.entry(cat_rel).or_default().push(object);
        }
        // Include base objects that may not have explicit docs yet
        for object in catalog.base_public_objects().iter() {
            let dir_name = super::docs::object_dir_name(object);
            let obj_dir = super::docs::find_object_dir(&docs_root, &dir_name).unwrap_or(docs_root.join(&dir_name));
            let cat_rel = if obj_dir.exists() { super::docs::category_rel_from(&docs_root, &obj_dir) } else { String::new() };
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
                    let dir_name = super::docs::object_dir_name(obj);
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
            let mut headings = vec![Heading::new("Description")];

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

            headings.extend(list_headings(&desc_without, None));

            // Escape inline generics in prose so MDX doesn't treat them as JSX
            let desc_sanitized = sanitize_inline_generics_in_prose(&desc_without);
            out.push_str(&desc_sanitized);
            out.push('\n');

            if captured_any && !main_rust.trim().is_empty() {
                // Use the struct name as file stem per user preference
                let dir_slug = obj_dirname;
                let tabs = build_tabs_for_example(
                    &parse_example(&main_rust, obj_dir, obj_dirname),
                    cat_rel,
                    dir_slug,
                    object,
                    &root,
                    &mut ex_js,
                    &mut ex_py,
                    &mut ex_swift,
                    &mut ex_kotlin,
                );
                out.push_str("\n## Example\n\n");
                out.push_str(&tabs);
                headings.push(Heading::new("Example"));
            }

            // An object with no methods, such as an enum, gets no Methods heading.
            if !method_files.is_empty() {
                out.push_str("\n## Methods\n\n");
                headings.push(Heading::new("Methods"));
            }
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
                headings.extend(list_headings(&pre, Some(file.as_str())));
                // Sanitize prose for MDX generics first, then downshift headings
                let pre_sanitized = sanitize_inline_generics_in_prose(pre.trim_end());
                out.push_str(&anchor_title(&downshift_headings(&pre_sanitized), file));
                out.push('\n');

                // Build example tabs via shared helper
                let dir_slug = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
                let tabs = build_tabs_for_example(
                    &parse_example(&rust_body, obj_dir, file),
                    cat_rel,
                    dir_slug,
                    file,
                    &root,
                    &mut ex_js,
                    &mut ex_py,
                    &mut ex_swift,
                    &mut ex_kotlin,
                );
                out.push_str("\n#### Example\n\n");
                out.push_str(&tabs);
                headings.push(Heading::new("Example"));
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
            check_heading_ids(&site_file, &headings);
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
        let objects = catalog.base_public_objects();
        for object in objects.iter() {
            let dir_name = super::docs::object_dir_name(object);
            let obj_dir =
                super::docs::find_object_dir(&docs_root, &dir_name).unwrap_or(docs_root.join(&dir_name));
            let cat_rel = if obj_dir.exists() {
                super::docs::category_rel_from(&docs_root, &obj_dir)
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
        for (object, obj_dir, cat_rel) in super::docs::scan_docs_objects(&docs_root) {
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
            ex_swift,
            ex_kotlin,
        }
    }

    /// Step 3: cleanup stale site files (non-destructive to source docs)
    pub fn cleanup_site(expected: &std::collections::HashSet<String>) {
        let site_root = meta::workspace_root().join("docs/website/src/content/docs/api");
        remove_unlisted(&site_root, "mdx", expected);
    }

    /// Step 3b: removes the example files this export did not write,
    /// so a deleted or renamed docs/api page leaves no example behind.
    pub fn cleanup_examples(outcome: &ExportOutcome) {
        let root = meta::workspace_root();
        let examples = [
            ("platforms/web/examples", "js", &outcome.ex_js),
            ("platforms/python/examples", "py", &outcome.ex_py),
            ("platforms/swift/examples", "swift", &outcome.ex_swift),
            ("platforms/kotlin/examples", "kt", &outcome.ex_kotlin),
        ];
        for (dir, ext, written) in examples {
            // An export that wrote nothing failed; keep what is on disk.
            if written.is_empty() {
                continue;
            }
            let mut keep = written.clone();
            // The Python aggregator lives next to the examples it runs.
            if ext == "py" {
                keep.insert("main.py".to_string());
            }
            let dir = root.join(dir);
            remove_unlisted(&dir, ext, &keep);
            remove_empty_dirs(&dir);
        }
    }

    /// Removes each `.{ext}` file under `dir` whose path relative to `dir`
    /// is not in `keep`. Matching ignores case, so on a case-insensitive disk
    /// a file whose name differs from a kept one only in case is never removed.
    fn remove_unlisted(
        dir: &std::path::Path,
        ext: &str,
        keep: &std::collections::HashSet<String>,
    ) {
        fn walk(
            dir: &std::path::Path,
            base: &std::path::Path,
            ext: &str,
            keep: &std::collections::HashSet<String>,
        ) {
            let Ok(read_dir) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in read_dir.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, base, ext, keep);
                    continue;
                }
                if path.extension().and_then(|s| s.to_str()) == Some(ext)
                    && let Ok(rel) = path.strip_prefix(base)
                {
                    let key = rel.to_string_lossy().replace('\\', "/").to_lowercase();
                    if !keep.contains(&key) {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
        }
        let keep = keep.iter().map(|k| k.to_lowercase()).collect();
        walk(dir, dir, ext, &keep);
    }

    /// Removes the empty directories under `dir`, deepest first.
    fn remove_empty_dirs(dir: &std::path::Path) {
        let Ok(read_dir) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() {
                remove_empty_dirs(&path);
                // remove_dir refuses a directory that still holds anything.
                let _ = std::fs::remove_dir(&path);
            }
        }
    }

    /// Step 4: write healthcheck aggregators referencing all generated example files
    pub fn write_healthcheck_aggregators(
        ex_js: &std::collections::HashSet<String>,
        ex_py: &std::collections::HashSet<String>,
        ex_swift: &std::collections::HashSet<String>,
        ex_kotlin: &std::collections::HashSet<String>,
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
            let rel_norm = rel.replace('\\', "/");
            let stem = rel_norm.strip_suffix(".js").unwrap_or(&rel_norm);
            js_out.push_str(&format!(
                "  ['../examples/{}', '{}'],\n",
                rel_norm,
                example_page(stem)
            ));
        }
        js_out.push_str("]\n\n");
        js_out.push_str("function fq(rel){ return 'platforms.web.examples.' + rel.replace('../examples/','').replace(/\\\\.js$/, '').replaceAll('/', '.'); }\n");
        // WebGPU reports an example's GPU errors after its calls return. The runner waits
        // for the error scopes each example opened before it starts the next one, so a
        // late error fails the example that caused it and names its `docs/api` page.
        js_out.push_str(
            r#"
// The error scopes WebGPU has yet to resolve.
const pending = new Set();

// Counts each popErrorScope until its promise settles.
function trackScopes() {
  const device = globalThis.GPUDevice;
  if (!device || device.prototype.popErrorScope.__fcTracked) return;
  const pop = device.prototype.popErrorScope;
  const tracked = function () {
    const promise = pop.call(this);
    pending.add(promise);
    const forget = () => pending.delete(promise);
    promise.then(forget, forget);
    return promise;
  };
  tracked.__fcTracked = true;
  device.prototype.popErrorScope = tracked;
}

// Waits until every open scope has resolved and the library has logged what it caught.
// The rounds are capped, so an example that keeps rendering can't hold the run.
async function settleScopes() {
  for (let round = 0; round < 100 && pending.size > 0; round++) {
    await Promise.allSettled([...pending]);
  }
  await new Promise((resolve) => setTimeout(resolve, 0));
}

export async function runExamples() {
  const total = EXAMPLES.length;
  let passed = 0;
  let failed = 0;
  console.log(`running ${total} tests`);
  globalThis.__HC = globalThis.__HC || { currentModule: null };
  trackScopes();
  const consoleError = console.error;
  let logged = [];
  console.error = (...args) => {
    logged.push(args.map(String).join(' '));
    consoleError.apply(console, args);
  };
  try {
    for (const [rel, page] of EXAMPLES) {
      const name = fq(rel);
      const head = `test ${name} ... `;
      logged = [];
      let problem = null;
      try {
        globalThis.__HC.currentModule = name;
        await import(rel);
      } catch (e) {
        problem = e;
      }
      await settleScopes();
      globalThis.__HC.currentModule = null;
      if (problem === null && logged.length > 0) problem = logged.join('\n');
      if (problem === null) {
        passed++;
        console.log(head + GREEN + 'OK' + RESET);
      } else {
        failed++;
        console.log(head + RED + 'FAILED' + RESET);
        console.error(`${page}:`, problem);
      }
    }
  } finally {
    console.error = consoleError;
  }
  return { passed, failed };
}
"#,
        );
        let _ = super::meta::write_if_changed(&js_path, &js_out);

        // Sort and write Python aggregator
        let mut py_list: Vec<String> = ex_py.iter().cloned().collect();
        py_list.sort();
        let py_path = root.join("platforms/python/examples/main.py");
        if let Some(parent) = py_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        // Each example fails on an exception or on a GPU error that the
        // library appended to the file FC_GPU_ERROR_LOG names, and a failing
        // example names its `docs/api` page.
        let mut py_out = String::new();
        py_out.push_str(
            r#"# Auto-generated: executes all Python examples with cargo-like output.
import runpy, pathlib, sys, traceback, os, tempfile

GREEN='\x1b[1;32m'
RED='\x1b[1;31m'
RESET='\x1b[0m'

def gpu_error_log():
    """The file where FragmentColor appends each GPU error that no error scope caught."""
    path = os.environ.get('FC_GPU_ERROR_LOG')
    if not path:
        handle, path = tempfile.mkstemp(prefix='fc-gpu-errors-', suffix='.log')
        os.close(handle)
        os.environ['FC_GPU_ERROR_LOG'] = path
    return path

def take_gpu_errors():
    """Empties the GPU error log and returns what it held."""
    with open(gpu_error_log(), 'a+') as log:
        log.seek(0)
        errors = log.read().strip()
        log.truncate(0)
    return errors

def run_all():
    base = pathlib.Path(__file__).parent
    examples = [
"#,
        );
        for rel in &py_list {
            let rel_norm = rel.replace('\\', "/");
            let stem = rel_norm.strip_suffix(".py").unwrap_or(&rel_norm);
            py_out.push_str(&format!(
                "        ('{}', '{}'),\n",
                rel_norm,
                example_page(stem)
            ));
        }
        py_out.push_str(
            r#"    ]

    # Announce test count and optionally prepare summary file
    total = len(examples)
    print(f"running {total} tests")
    summary_path = os.environ.get('FC_PY_SUMMARY_PATH')
    if summary_path:
        try:
            with open(summary_path, 'w') as f:
                f.write(f"total={total}\npassed=0\nfailed=0\n")
        except Exception:
            pass

    passed = 0
    failed = 0
    take_gpu_errors()
    for rel, page in examples:
        name = 'platforms.python.examples.' + rel.replace('/', '.').removesuffix('.py')
        head = f'test {name} ... '
        os.environ['FC_RUNNER'] = 'python'
        os.environ['FC_CURRENT_TEST'] = name
        problems = []
        try:
            runpy.run_path(str(base / rel), run_name='__main__')
        except SystemExit as stop:
            if stop.code != 0:
                problems.append(traceback.format_exc())
        except Exception:
            problems.append(traceback.format_exc())
        errors = take_gpu_errors()
        if errors:
            problems.append(f'GPU error: {errors}')
        if problems:
            failed += 1
            print(head + RED + 'FAILED' + RESET)
            print(f'{page}: ' + '\n'.join(problems))
        else:
            passed += 1
            print(head + GREEN + 'OK' + RESET)

    if summary_path:
        try:
            with open(summary_path, 'w') as f:
                f.write(f"total={total}\npassed={passed}\nfailed={failed}\n")
        except Exception:
            pass

    if failed:
        print(f"\n{RED}test result: FAILED{RESET}. {passed} passed; {failed} failed")
        raise SystemExit(1)
    else:
        print(f"\n{GREEN}test result: ok{RESET}. {passed} passed; {failed} failed")

if __name__ == '__main__':
    run_all()
"#,
        );
        let _ = super::meta::write_if_changed(&py_path, &py_out);

        // Sort and write the Swift example tests. Each transpiled example,
        // minus its `import` lines, becomes the body of one XCTest that
        // `./fc healthcheck ios` runs on an iOS simulator.
        let mut swift_list: Vec<String> = ex_swift.iter().cloned().collect();
        swift_list.sort();
        let swift_aggregator_path =
            root.join("platforms/swift/healthcheck/GeneratedExamples.swift");
        if let Some(parent) = swift_aggregator_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut swift_out = String::new();
        swift_out.push_str("// AUTO-GENERATED by build.rs (scripts/website.rs). DO NOT EDIT.\n");
        swift_out.push_str("//\n");
        swift_out.push_str("// One test per transpiled Swift example under `platforms/swift/examples/`.\n");
        swift_out.push_str("// `./fc healthcheck ios` runs them on an iOS simulator, where a failing test\n");
        swift_out.push_str("// names the `docs/api` page of its example. Fix that page, the binding it calls,\n");
        swift_out.push_str("// or `scripts/convert.rs::to_swift`, since every cargo build rewrites this file.\n");
        swift_out.push('\n');
        // Foundation gives us `Data`, which the texture-input examples need
        // when wrapping `[UInt8]` byte arrays. Importing FragmentColor alone
        // doesn't transitively re-export Foundation symbols into this file.
        swift_out.push_str("import Foundation\n");
        swift_out.push_str("import FragmentColor\n");
        for import in example_imports(&root.join("platforms/swift/examples"), &swift_list, &swift_out) {
            swift_out.push_str(&import);
            swift_out.push('\n');
        }
        swift_out.push('\n');
        swift_out.push_str("final class GeneratedExamples: ExampleCase {\n");
        for (index, rel) in swift_list.iter().enumerate() {
            let stem = rel.strip_suffix(".swift").unwrap_or(rel);
            let example_path = root.join("platforms/swift/examples").join(rel);
            let body = std::fs::read_to_string(&example_path).unwrap_or_default();
            let body_clean = strip_lang_lines(&body, &["import "]);
            if index > 0 {
                swift_out.push('\n');
            }
            swift_out.push_str(&format!("    func test_{}() async {{\n", ident_slug(stem)));
            swift_out.push_str(&format!(
                "        await example(\"{}\") {{\n",
                example_page(stem)
            ));
            push_indented(&mut swift_out, &body_clean, "            ");
            swift_out.push_str("        }\n");
            swift_out.push_str("    }\n");
        }
        swift_out.push_str("}\n");
        let _ = super::meta::write_if_changed(&swift_aggregator_path, &swift_out);

        // Sort and write the Kotlin example tests. Each transpiled example,
        // minus its `import` and `package` lines, becomes the body of one
        // JUnit test that `./fc healthcheck android` runs on a device.
        let mut kotlin_list: Vec<String> = ex_kotlin.iter().cloned().collect();
        kotlin_list.sort();
        let kotlin_aggregator_path = root.join(
            "platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/GeneratedExamples.kt",
        );
        if let Some(parent) = kotlin_aggregator_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut k_out = String::new();
        k_out.push_str("package org.fragmentcolor\n\n");
        k_out.push_str("// AUTO-GENERATED by build.rs (scripts/website.rs). DO NOT EDIT.\n");
        k_out.push_str("//\n");
        k_out.push_str("// One test per transpiled Kotlin example under `platforms/kotlin/examples/`.\n");
        k_out.push_str("// `./fc healthcheck android` runs them on a device or emulator. A failing test\n");
        k_out.push_str("// names the `docs/api` page of its example. Fix that page, the binding it calls,\n");
        k_out.push_str("// or `scripts/convert.rs::to_kotlin`, since every cargo build rewrites this file.\n");
        k_out.push('\n');
        k_out.push_str("import androidx.test.ext.junit.runners.AndroidJUnit4\n");
        k_out.push_str("import org.junit.Test\n");
        k_out.push_str("import org.junit.runner.RunWith\n");
        let kotlin_dir = root.join("platforms/kotlin/examples");
        // The aggregator sits in the package itself.
        let in_package = "import org.fragmentcolor.*\n".to_string() + &k_out;
        for import in example_imports(&kotlin_dir, &kotlin_list, &in_package) {
            k_out.push_str(&import);
            k_out.push('\n');
        }
        k_out.push('\n');
        k_out.push_str("@RunWith(AndroidJUnit4::class)\n");
        k_out.push_str("class GeneratedExamples : ExampleCase() {\n");
        for (index, rel) in kotlin_list.iter().enumerate() {
            let stem = rel.strip_suffix(".kt").unwrap_or(rel);
            let example_path = kotlin_dir.join(rel);
            let body = std::fs::read_to_string(&example_path).unwrap_or_default();
            let body_clean = strip_lang_lines(&body, &["import ", "package "]);
            if index > 0 {
                k_out.push('\n');
            }
            k_out.push_str("    @Test\n");
            k_out.push_str(&format!(
                "    fun {}() = example(\"{}\") {{\n",
                ident_slug(stem),
                example_page(stem)
            ));
            push_indented(&mut k_out, &body_clean, "        ");
            k_out.push_str("    }\n");
        }
        k_out.push_str("}\n");
        let _ = super::meta::write_if_changed(&kotlin_aggregator_path, &k_out);
    }

    /// The `docs/api` page an example comes from.
    /// A method's example takes the name of its page.
    /// An object's example takes its type's name instead,
    /// so `core/pass/Pass` comes from the object page `core/pass/pass.md`.
    fn example_page(stem: &str) -> String {
        let (dir, name) = stem.rsplit_once('/').unwrap_or(("", stem));
        let page = if name.starts_with(char::is_uppercase) {
            dir.rsplit('/').next().unwrap_or(name)
        } else {
            name
        };
        format!("docs/api/{dir}/{page}.md")
    }

    /// Appends the lines of `body` to `out` behind `indent`.
    /// Blank lines around the body are dropped, and those inside it stay empty.
    fn push_indented(out: &mut String, body: &str, indent: &str) {
        let lines: Vec<&str> = body.lines().collect();
        let blank = |line: &&str| line.trim().is_empty();
        let start = lines.iter().position(|line| !blank(line)).unwrap_or(lines.len());
        let end = lines.iter().rposition(|line| !blank(line)).map_or(start, |last| last + 1);
        for line in &lines[start..end] {
            if !line.trim().is_empty() {
                out.push_str(indent);
                out.push_str(line);
            }
            out.push('\n');
        }
    }

    /// The import lines of the examples at `rels` under `dir`
    /// that `header` doesn't import yet, sorted, each once.
    fn example_imports(dir: &std::path::Path, rels: &[String], header: &str) -> Vec<String> {
        let imported: std::collections::BTreeSet<&str> = header.lines().collect();
        let mut imports = std::collections::BTreeSet::new();
        for rel in rels {
            let body = std::fs::read_to_string(dir.join(rel)).unwrap_or_default();
            for line in body.lines().map(str::trim) {
                if line.starts_with("import ") && !imported.contains(line) {
                    imports.insert(line.to_string());
                }
            }
        }
        imports.into_iter().collect()
    }

    fn ident_slug(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    }

    fn strip_lang_lines(body: &str, prefixes: &[&str]) -> String {
        body.lines()
            .filter(|line| {
                let t = line.trim_start();
                !prefixes.iter().any(|p| t.starts_with(p))
            })
            .collect::<Vec<_>>()
            .join("\n")
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
                        // Wrap from start..=end_gt in backticks. The identifier
                        // is already in `out`, so take it back first.
                        for _ in start..i {
                            out.pop();
                        }
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

    /// The example in `stem.md` under `dir`. Fails the build on a hidden block it can't place.
    fn parse_example(rust: &str, dir: &std::path::Path, stem: &str) -> Example {
        Example::parse(rust).unwrap_or_else(|e| panic!("{}: {e}", dir.join(format!("{stem}.md")).display()))
    }

    /// ` meta="collapse={1-3, 9-10}"` for the hidden line ranges of a tab, or nothing.
    fn collapse_attr(ranges: &[(usize, usize)]) -> String {
        if ranges.is_empty() {
            return String::new();
        }
        let ranges: Vec<String> = ranges.iter().map(|(start, end)| format!("{start}-{end}")).collect();
        format!(" meta=\"collapse={{{}}}\"", ranges.join(", "))
    }

    fn extract_first_code_block(md: &str) -> Option<(String, String)> {
        let mut in_code = false;
        let mut lang = String::new();
        let mut code = String::new();
        for line in md.lines() {
            let trimmed = line.trim_start();
            if !in_code && trimmed.starts_with("```") {
                in_code = true;
                lang = trimmed
                    .trim_start_matches("```")
                    .trim()
                    .to_ascii_lowercase();
                continue;
            }
            if in_code && trimmed.starts_with("```") {
                return Some((lang, code.trim_end().to_string()));
            }
            if in_code {
                code.push_str(line);
                code.push('\n');
            }
        }
        None
    }

    fn is_placeholder_example(code: &str) -> bool {
        let text = code.trim().to_ascii_lowercase();
        text.is_empty()
            || text.contains("hidden draft")
            || text.contains("hidden file")
            || text.contains("no public example")
    }

    /// `Texture` → `texture`, `WindowTarget` → `window_target`,
    /// `read_texture` → `read_texture` (already snake). Used by
    /// `read_lang_override` to fall back to the source-file naming
    /// convention when the lookup uses the CamelCase struct name as
    /// `file_stem`.
    fn camel_to_snake(s: &str) -> String {
        let mut out = String::with_capacity(s.len() + 4);
        for (i, ch) in s.chars().enumerate() {
            if ch.is_ascii_uppercase() {
                if i > 0 {
                    out.push('_');
                }
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
        }
        out
    }

    /// The example in `lang`, with its hidden line ranges:
    /// a hand-written page when one exists, the transpiled Rust example otherwise.
    fn lang_code(
        example: &Example,
        lang: Lang,
        root: &std::path::Path,
        cat_rel: &str,
        obj_dir_slug: &str,
        file_stem: &str,
    ) -> (String, Vec<(usize, usize)>) {
        read_lang_override(root, cat_rel, obj_dir_slug, file_stem, lang)
            .unwrap_or_else(|| transpile(example, lang))
    }

    /// `example`'s Rust lines in `lang`, between that language's hidden lines.
    fn transpile(example: &Example, lang: Lang) -> (String, Vec<(usize, usize)>) {
        let items = &example.rust;
        let body = match lang {
            Lang::Js => crate::convert::to_js(items),
            Lang::Python => crate::convert::to_py(items),
            Lang::Swift => crate::convert::to_swift(items),
            Lang::Kotlin => crate::convert::to_kotlin(items),
        };
        example.harness(lang).wrap(&body)
    }

    fn read_lang_override(
        root: &std::path::Path,
        cat_rel: &str,
        obj_dir_slug: &str,
        file_stem: &str,
        lang: Lang,
    ) -> Option<(String, Vec<(usize, usize)>)> {
        let lang_suffix = match lang {
            Lang::Js => "js",
            Lang::Python => "py",
            Lang::Swift => "swift",
            Lang::Kotlin => "kotlin",
        };
        // Class entry-point examples use the CamelCase struct name as the
        // `file_stem` (e.g. `Texture`, `Mipmap`), so the natural
        // override path is `Texture_py.md`. The earlier convention put the
        // override at the snake_cased source-file stem (`texture_py.md`) —
        // both happen to coexist in `git ls-files` because macOS HFS+ is
        // case-insensitive. On Linux (case-sensitive) the lookup misses,
        // the override is silently ignored, and the fall-through Rust→
        // target conversion produces wrong API shape (e.g. CI's
        // `Texture.py` ends up with `create_texture((pixels, [1, 1]))`
        // tuple form instead of the kwargs the override carries).
        //
        // Try the exact stem first, then the snake_cased form, so both
        // override naming conventions resolve identically across OSes.
        let dir = if cat_rel.is_empty() {
            root.join("docs/api").join(obj_dir_slug).join("hidden")
        } else {
            root.join("docs/api")
                .join(cat_rel)
                .join(obj_dir_slug)
                .join("hidden")
        };
        let candidates = [
            dir.join(format!("{}_{}.md", file_stem, lang_suffix)),
            dir.join(format!(
                "{}_{}.md",
                camel_to_snake(file_stem),
                lang_suffix
            )),
        ];
        let hidden = candidates.into_iter().find(|p| p.exists())?;
        let md = std::fs::read_to_string(&hidden).ok()?;
        let (fence, code) = extract_first_code_block(&md)?;
        if is_placeholder_example(&code) {
            return None;
        }
        let is_rust = fence.is_empty() || fence == "rust" || fence.starts_with("rust,");
        if !is_rust {
            return Some((code, Vec::new()));
        }
        let example = Example::parse(&code).unwrap_or_else(|e| panic!("{}: {e}", hidden.display()));
        Some(transpile(&example, lang))
    }

    // Build the Tabs for an example and write each language's healthcheck file.
    // Nine args is more than clippy's default, but each one is load-bearing,
    // and the four `ex_*` sets carry distinct per-language state.
    #[allow(clippy::too_many_arguments)]
    fn build_tabs_for_example(
        example: &Example,
        cat_rel: &str,
        obj_dir_slug: &str,
        file_stem: &str,
        root: &std::path::Path,
        ex_js: &mut std::collections::HashSet<String>,
        ex_py: &mut std::collections::HashSet<String>,
        ex_swift: &mut std::collections::HashSet<String>,
        ex_kotlin: &mut std::collections::HashSet<String>,
    ) -> String {
        // Collapse ranges (1-based inclusive) over the hidden runs
        let items = &example.rust;
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

        let mut tabs = String::new();
        tabs.push_str("<Tabs>\n\n");
        push_tab(&mut tabs, "Rust", "rust", &rust_processed, &ranges);

        // Each language's example is both a tab and a healthcheck file.
        let outputs = [
            (Lang::Js, "JavaScript", "js", "js", "platforms/web/examples", ex_js),
            (Lang::Python, "Python", "python", "py", "platforms/python/examples", ex_py),
            (Lang::Swift, "Swift", "swift", "swift", "platforms/swift/examples", ex_swift),
            (Lang::Kotlin, "Kotlin", "kotlin", "kt", "platforms/kotlin/examples", ex_kotlin),
        ];
        for (lang, label, fence, ext, dir, written) in outputs {
            let (code, hidden) = lang_code(example, lang, root, cat_rel, obj_dir_slug, file_stem);
            let rel = if cat_rel.is_empty() {
                format!("{}/{}.{}", obj_dir_slug, file_stem, ext)
            } else {
                format!("{}/{}/{}.{}", cat_rel, obj_dir_slug, file_stem, ext)
            };
            let abs = root.join(dir).join(&rel);
            if let Some(parent) = abs.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = super::meta::write_if_changed(&abs, &code);
            written.insert(rel);
            push_tab(&mut tabs, label, fence, &code, &hidden);
        }

        tabs.push_str("</Tabs>\n\n");
        tabs
    }

    /// One `<TabItem>` holding `code`, with its hidden lines collapsed.
    fn push_tab(tabs: &mut String, label: &str, fence: &str, code: &str, hidden: &[(usize, usize)]) {
        tabs.push_str(&format!("<TabItem label=\"{label}\">\n"));
        tabs.push_str("<Code\n");
        tabs.push_str("code={`\n");
        // Escape backticks so a comment like `// see `Foo` ` doesn't close the template literal early.
        tabs.push_str(&sanitize_for_template(code));
        tabs.push_str("\n`}\n");
        tabs.push_str(&format!("lang=\"{fence}\""));
        // The Code component numbers lines from the first one with text.
        let blank = code.lines().take_while(|line| line.trim_end().is_empty()).count();
        let hidden: Vec<(usize, usize)> = hidden
            .iter()
            .filter(|(_, end)| *end > blank)
            .map(|(start, end)| (start.saturating_sub(blank).max(1), end - blank))
            .collect();
        tabs.push_str(&collapse_attr(&hidden));
        tabs.push_str("\n/>\n\n</TabItem>\n\n");
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

    /// A heading on a site page, and the id the export sets on it.
    struct Heading {
        text: String,
        anchor: Option<String>,
    }

    impl Heading {
        fn new(text: &str) -> Self {
            Self {
                text: text.to_string(),
                anchor: None,
            }
        }
    }

    /// The headings of a docs/api page, with `anchor` on the first `#` heading, its title.
    fn list_headings(md: &str, anchor: Option<&str>) -> Vec<Heading> {
        let mut anchor = anchor.map(String::from);
        super::links::read_headings(md)
            .into_iter()
            .map(|(level, text)| {
                let anchor = if level == 1 { anchor.take() } else { None };
                Heading { text, anchor }
            })
            .collect()
    }

    /// Ends the method title, the first `###` heading, with `\{#anchor\}`.
    ///
    /// The site's heading id plugin makes the braces the heading's id,
    /// and the backslashes keep MDX from reading them as an expression.
    fn anchor_title(mdx: &str, anchor: &str) -> String {
        let mut fenced = false;
        let mut tagged = false;
        mdx.lines()
            .map(|line| {
                if line.trim_start().starts_with("```") {
                    fenced = !fenced;
                }
                if fenced || tagged || !line.starts_with("### ") {
                    return line.to_string();
                }
                tagged = true;
                format!("{} \\{{#{anchor}\\}}", line.trim_end())
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Fails the build when two headings on the page at `path` get the same id.
    ///
    /// A heading with an anchor gets it as its id. Starlight gives every other
    /// heading github-slugger's id for its text, with `-1`, `-2` on repeats, and
    /// doesn't count the anchors as repeats, so an anchor can take a slug's id.
    fn check_heading_ids(path: &std::path::Path, headings: &[Heading]) {
        let mut slugs = HashMap::new();
        let mut owners: HashMap<String, &str> = HashMap::new();
        for heading in headings {
            let id = match &heading.anchor {
                Some(anchor) => anchor.clone(),
                None => next_slug(&mut slugs, &heading.text),
            };
            if let Some(other) = owners.insert(id.clone(), &heading.text) {
                panic!(
                    "{}: the headings `{}` and `{}` both get the id `{}`. Rename one of them in docs/api.",
                    path.display(),
                    other,
                    heading.text,
                    id
                );
            }
        }
    }

    /// The id github-slugger gives `text`, after the ids in `seen`.
    fn next_slug(seen: &mut HashMap<String, u32>, text: &str) -> String {
        let base = super::links::slugify(text);
        let mut slug = base.clone();
        while seen.contains_key(&slug) {
            let count = seen.entry(base.clone()).or_default();
            *count += 1;
            slug = format!("{base}-{count}");
        }
        seen.insert(slug.clone(), 0);
        slug
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
            // Hidden lines lose their '#', and other languages' hidden lines leave the Rust code
            let code: String = body.iter().map(|line| format!("{line}\n")).collect();
            let example =
                Example::parse(&code).unwrap_or_else(|e| panic!("in a Rust code fence: {e}\n{code}"));
            let items: Vec<LineItem> = example
                .rust
                .into_iter()
                .map(|(text, hidden)| LineItem { text, hidden })
                .collect();

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
