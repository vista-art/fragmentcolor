mod validation {
    use crate::codegen::ApiMap;

    use super::*;
    use std::fs;
    use std::path::Path;

    fn to_snake_case(s: &str) -> String {
        // Basic snake_case converter for method names (already snake in Rust)
        s.to_string()
    }

    pub fn object_dir_name(object: &str) -> String {
        // Convert CamelCase to snake_case for directory names
        let mut out = String::new();
        for (i, ch) in object.chars().enumerate() {
            if ch.is_uppercase() {
                if i != 0 {
                    out.push('_');
                }
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
        }
        out
    }

    fn ensure_object_md_ok(object: &str, path: &Path, problems: &mut Vec<String>) {
        if !path.exists() {
            problems.push(format!("Missing object file: {}", path.display()));
            return;
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.trim().is_empty() {
            problems.push(format!("Empty object file: {}", path.display()));
        }
        if !content.lines().any(|l| l.trim() == format!("# {}", object)) {
            problems.push(format!("{}: H1 '# {}' not found", path.display(), object));
        }
        if !content.contains("## Example") {
            problems.push(format!("{}: '## Example' section missing", path.display()));
        }
        if content.contains("fragmentcolor.com") {
            problems.push(format!("{}: contains fragmentcolor.com", path.display()));
        }
        // BLOCK build on legacy winit API usage in public examples
        if content.contains("WindowBuilder") || content.contains("EventLoop") {
            problems.push(format!(
                "{}: legacy winit API (WindowBuilder/EventLoop) detected in docs — replace with RenderCanvas/HTMLCanvas",
                path.display()
            ));
        }
    }

    fn ensure_method_md_ok(object: &str, method: &str, path: &Path, problems: &mut Vec<String>) {
        if !path.exists() {
            problems.push(format!(
                "Missing method file for {}.{}: {}",
                object,
                method,
                path.display()
            ));
            return;
        }
        let content = fs::read_to_string(path).unwrap_or_default();
        if content.trim().is_empty() {
            problems.push(format!("Empty method file: {}", path.display()));
        }
        if !content.lines().any(|l| l.trim().starts_with('#')) {
            problems.push(format!("{}: H1 heading missing", path.display()));
        } else {
            // Enforce method title starts with "<Object>::"
            if let Some(head) = content.lines().find(|l| l.trim_start().starts_with('#')) {
                let mut t = head.trim_start();
                while t.starts_with('#') {
                    t = t[1..].trim_start();
                }
                let expected_prefix = format!("{}::", object);
                if !t.starts_with(&expected_prefix) {
                    problems.push(format!(
                        "{}: method title must start with '{}' (e.g., '{}{}(...)')",
                        path.display(),
                        expected_prefix,
                        expected_prefix,
                        method
                    ));
                }
            }
        }
        if !content.contains("## Example") {
            problems.push(format!("{}: '## Example' section missing", path.display()));
        }
        if content.contains("fragmentcolor.com") {
            problems.push(format!("{}: contains fragmentcolor.com", path.display()));
        }
    }

    /// Locate a method doc within any nested `hidden/` subdirectory under the given object directory.
    /// Returns Some(path) if a file named `<file_stem>.md` exists below a path that contains a
    /// directory segment named `hidden`; otherwise None.
    fn find_hidden_method_doc(
        obj_dir: &std::path::Path,
        file_stem: &str,
    ) -> Option<std::path::PathBuf> {
        fn contains_hidden(mut p: &std::path::Path) -> bool {
            while let Some(parent) = p.parent() {
                if parent.file_name().and_then(|s| s.to_str()) == Some("hidden") {
                    return true;
                }
                p = parent;
            }
            false
        }
        fn walk(dir: &std::path::Path, file_stem: &str, out: &mut Option<std::path::PathBuf>) {
            if out.is_some() {
                return;
            }
            if let Ok(rd) = std::fs::read_dir(dir) {
                for entry in rd.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        walk(&p, file_stem, out);
                        if out.is_some() {
                            return;
                        }
                    } else if p.extension().and_then(|s| s.to_str()) == Some("md")
                        && p.file_stem().and_then(|s| s.to_str()) == Some(file_stem)
                        && contains_hidden(&p)
                    {
                        *out = Some(p);
                        return;
                    }
                }
            }
        }
        let mut found = None;
        walk(obj_dir, file_stem, &mut found);
        found
    }

    pub fn validate_docs(api_map: &ApiMap) {
        let mut problems = Vec::new();
        let mut warnings = Vec::new();
        let root = meta::workspace_root();
        let docs_root = root.join("docs/api");

        // Helper: find the directory for an object recursively (must contain <dir>/<dir>.md)
        fn find_object_dir(
            docs_root: &std::path::Path,
            dir_name: &str,
        ) -> Option<std::path::PathBuf> {
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

        // Helper: recursively enumerate all object dirs found under docs_root
        fn scan_docs_objects(docs_root: &std::path::Path) -> Vec<(String, std::path::PathBuf)> {
            fn walk(
                dir: &std::path::Path,
                _root: &std::path::Path,
                out: &mut Vec<(String, std::path::PathBuf)>,
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
                                let object = dir_to_object_name(name);
                                out.push((object, p.clone()));
                                // Do not descend into this object dir further
                                continue;
                            }
                        }
                        walk(&p, _root, out);
                    }
                }
            }
            let mut out = Vec::new();
            walk(docs_root, docs_root, &mut out);
            out
        }

        // Discover all objects and their categories under docs/api (recursively)
        fn scan_docs_objects_with_cat(
            docs_root: &std::path::Path,
        ) -> Vec<(
            String,             /*object name*/
            std::path::PathBuf, /*dir*/
            String,             /*cat_rel*/
            String,             /*dir_slug*/
        )> {
            fn walk(
                dir: &std::path::Path,
                root: &std::path::Path,
                out: &mut Vec<(String, std::path::PathBuf, String, String)>,
            ) {
                if !dir.is_dir() {
                    return;
                }
                if let Ok(rd) = std::fs::read_dir(dir) {
                    for entry in rd.flatten() {
                        let p = entry.path();
                        if p.is_dir() {
                            if let Some(dir_name) = p.file_name().and_then(|s| s.to_str()) {
                                let md = p.join(format!("{}.md", dir_name));
                                if md.exists() {
                                    let object = dir_to_object_name(dir_name);
                                    let parent = p.parent().unwrap_or(root);
                                    let cat_rel = if let Ok(rel) = parent.strip_prefix(root) {
                                        rel.to_string_lossy().replace('\\', "/")
                                    } else {
                                        String::new()
                                    };
                                    out.push((object, p.clone(), cat_rel, dir_name.to_string()));
                                    // Do not descend further into an object dir
                                    continue;
                                }
                            }
                            walk(&p, root, out);
                        }
                    }
                }
            }
            let mut out = Vec::new();
            walk(docs_root, docs_root, &mut out);
            out
        }

        fn canonical_url(cat_rel: &str, object_lower: &str) -> String {
            let mut url = String::from("https://fragmentcolor.org/api/");
            if !cat_rel.is_empty() {
                url.push_str(cat_rel);
                url.push('/');
            }
            url.push_str(object_lower);
            url
        }

        // Fix incorrect absolute links and auto-link unlinked object names in source docs
        fn rewrite_source_links_and_autolink(docs_root: &std::path::Path) {
            use std::collections::HashMap;

            let objects = scan_docs_objects_with_cat(docs_root);

            // Build maps: by object name (raw), by lowercased object slug, by folder dir slug
            let mut by_name: HashMap<String, (String /*cat*/, String /*obj_lower*/)> =
                HashMap::new();
            let mut by_obj_slug: HashMap<String, (String, String)> = HashMap::new();
            let mut by_dir_slug: HashMap<String, (String, String)> = HashMap::new();
            for (name, _dir, cat, dir_slug) in &objects {
                let obj_lower = name.to_lowercase();
                by_name.insert(name.clone(), (cat.clone(), obj_lower.clone()));
                by_obj_slug.insert(obj_lower.clone(), (cat.clone(), obj_lower.clone()));
                by_dir_slug.insert(dir_slug.clone(), (cat.clone(), obj_lower.clone()));
            }

            fn is_word_boundary(ch: Option<char>) -> bool {
                match ch {
                    None => true,
                    Some(c) => !c.is_alphanumeric() && c != '_',
                }
            }

            fn process_file(
                path: &std::path::Path,
                by_name: &std::collections::HashMap<String, (String, String)>,
                by_obj_slug: &std::collections::HashMap<String, (String, String)>,
                by_dir_slug: &std::collections::HashMap<String, (String, String)>,
            ) {
                let Ok(src) = std::fs::read_to_string(path) else {
                    return;
                };
                let mut changed = false;

                // Pass 1: fix absolute links
                let mut out = String::new();
                let bytes = src.as_bytes();
                let mut i = 0usize;

                let needle_https = "](https://fragmentcolor.org/api/".as_bytes();
                let needle_http = "](http://fragmentcolor.org/api/".as_bytes();
                let needle_www_https = "](https://www.fragmentcolor.org/api/".as_bytes();
                let needle_www_http = "](http://www.fragmentcolor.org/api/".as_bytes();

                while i < bytes.len() {
                    let mut matched = None::<&[u8]>;
                    if i + needle_https.len() <= bytes.len()
                        && &bytes[i..i + needle_https.len()] == needle_https
                    {
                        matched = Some(needle_https);
                    } else if i + needle_http.len() <= bytes.len()
                        && &bytes[i..i + needle_http.len()] == needle_http
                    {
                        matched = Some(needle_http);
                    } else if i + needle_www_https.len() <= bytes.len()
                        && &bytes[i..i + needle_www_https.len()] == needle_www_https
                    {
                        matched = Some(needle_www_https);
                    } else if i + needle_www_http.len() <= bytes.len()
                        && &bytes[i..i + needle_www_http.len()] == needle_www_http
                    {
                        matched = Some(needle_www_http);
                    }

                    if let Some(m) = matched {
                        out.push_str("](");
                        i += m.len();
                        // Capture until ')'
                        let start = i;
                        while i < bytes.len() && bytes[i] != b')' {
                            i += 1;
                        }
                        let tail = &src[start..i];
                        let mut parts: Vec<&str> =
                            tail.split('/').filter(|s| !s.is_empty()).collect();
                        if let Some(last) = parts.pop() {
                            let key = last.to_string();
                            let best = by_obj_slug
                                .get(&key)
                                .or_else(|| by_dir_slug.get(&key))
                                .cloned();
                            if let Some((cat, obj_lower)) = best {
                                let canon = canonical_url(&cat, &obj_lower);
                                out.push_str(&canon);
                                changed = true;
                            } else {
                                out.push_str("https://fragmentcolor.org/api/");
                                out.push_str(tail);
                            }
                        } else {
                            out.push_str("https://fragmentcolor.org/api/");
                            out.push_str(tail);
                        }
                    } else {
                        out.push(bytes[i] as char);
                        i += 1;
                    }
                }

                let content = if out.is_empty() { src.clone() } else { out };
                if content != src {
                    changed = true;
                }

                // Pass 2: auto-link unlinked names (skip code fences, inline code, and headings)
                let mut names: Vec<&String> = by_name.keys().collect();
                names.sort_by_key(|s| std::cmp::Reverse(s.len()));

                let mut result = String::new();
                let mut in_code_block = false;
                for line in content.lines() {
                    let trimmed = line.trim_start();
                    if trimmed.starts_with("```") {
                        in_code_block = !in_code_block;
                        result.push_str(line);
                        result.push('\n');
                        continue;
                    }
                    if trimmed.starts_with('#') || in_code_block {
                        result.push_str(line);
                        result.push('\n');
                        continue;
                    }

                    let mut i = 0usize;
                    let chars: Vec<char> = line.chars().collect();
                    let mut in_inline = false;
                    while i < chars.len() {
                        let c = chars[i];
                        if c == '`' {
                            in_inline = !in_inline;
                            result.push(c);
                            i += 1;
                            continue;
                        }
                        if !in_inline && c == '[' {
                            // Copy existing link intact
                            let mut j = i + 1;
                            while j < chars.len() && chars[j] != ']' {
                                j += 1;
                            }
                            if j < chars.len() && j + 1 < chars.len() && chars[j + 1] == '(' {
                                let mut k = j + 2;
                                while k < chars.len() && chars[k] != ')' {
                                    k += 1;
                                }
                                let upper = k.min(chars.len().saturating_sub(1));
                                for ch in chars.iter().take(upper + 1).skip(i) {
                                    result.push(*ch);
                                }
                                i = k.min(chars.len());
                                if i < chars.len() && chars[i] == ')' {
                                    i += 1;
                                }
                                continue;
                            }
                        }

                        if in_inline {
                            result.push(c);
                            i += 1;
                            continue;
                        }

                        let mut matched_any = false;
                        for name in &names {
                            let name_chars: Vec<char> = name.chars().collect();
                            let end = i + name_chars.len();
                            if end <= chars.len()
                                && chars[i..end].iter().copied().eq(name_chars.iter().copied())
                                && is_word_boundary(
                                    i.checked_sub(1).and_then(|p| chars.get(p)).copied(),
                                )
                                && is_word_boundary(chars.get(end).copied())
                            {
                                let (cat, obj_lower) = by_name.get(*name).cloned().unwrap();
                                let url = canonical_url(&cat, &obj_lower);
                                result.push('[');
                                result.push_str(name);
                                result.push_str("](");
                                result.push_str(&url);
                                result.push(')');
                                i = end;
                                matched_any = true;
                                changed = true;
                                break;
                            }
                        }
                        if !matched_any {
                            result.push(c);
                            i += 1;
                        }
                    }
                    if !line.ends_with('\n') {
                        result.push('\n');
                    }
                }

                // Read-only: do not write back to source docs during validation.
                let _ = changed; // intentionally unused; this pass only reports potential changes
            }

            fn walk_files(dir: &std::path::Path, cb: &dyn Fn(&std::path::Path)) {
                if let Ok(rd) = std::fs::read_dir(dir) {
                    for e in rd.flatten() {
                        let p = e.path();
                        if p.is_dir() {
                            walk_files(&p, cb);
                        } else if p.extension().and_then(|s| s.to_str()) == Some("md") {
                            cb(&p);
                        }
                    }
                }
            }

            walk_files(docs_root, &|p| {
                process_file(p, &by_name, &by_obj_slug, &by_dir_slug)
            });
        }

        rewrite_source_links_and_autolink(&docs_root);

        // Enforce documentation for ALL public objects (including wrappers)
        let objects = public_structs_excluding_hidden();
        let _all_objects = objects.clone();

        // Validate objects and their methods
        for object in objects.iter() {
            let methods_vec = api_map.get(object).cloned().unwrap_or_default();
            let dir = object_dir_name(object);
            let obj_dir = find_object_dir(&docs_root, &dir).unwrap_or(docs_root.join(&dir));
            let object_md = obj_dir.join(format!("{}.md", dir));
            ensure_object_md_ok(object, &object_md, &mut problems);
            // Link enforcement disabled; links are auto-rewritten during export

            for m in &methods_vec {
                if let Some(fun) = &m.function {
                    let name = &fun.name;

                    let file = to_snake_case(name);
                    let path = obj_dir.join(format!("{}.md", file));
                    if path.exists() {
                        ensure_method_md_ok(object, name, &path, &mut problems);
                    } else if let Some(hidden_path) = find_hidden_method_doc(&obj_dir, &file) {
                        warnings.push(format!(
                            "Hidden method: {}.{} ({})",
                            object,
                            name,
                            hidden_path.display()
                        ));
                    } else {
                        ensure_method_md_ok(object, name, &path, &mut problems);
                    }
                    // Link enforcement disabled; links are auto-rewritten during export
                }
            }
        }

        // Also validate any docs-only objects under docs/api not present in allowed (recursively)
        for (object, obj_dir) in scan_docs_objects(&docs_root) {
            if objects.iter().any(|o| o == &object) {
                continue;
            }
            let dir_name = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let object_md = obj_dir.join(format!("{}.md", dir_name));
            ensure_object_md_ok(&object, &object_md, &mut problems);
            // Link enforcement disabled; links are auto-rewritten during export
        }

        // NEW: Validate method titles across all docs recursively (non-hidden), independent of API map
        for (object, obj_dir) in scan_docs_objects(&docs_root) {
            let dir_name = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if let Ok(read_dir) = std::fs::read_dir(&obj_dir) {
                for e in read_dir.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        // Skip hidden subfolders
                        if p.file_name().and_then(|s| s.to_str()) == Some("hidden") {
                            continue;
                        }
                        continue;
                    }
                    if p.extension().and_then(|s| s.to_str()) == Some("md")
                        && let Some(stem) = p.file_stem().and_then(|s| s.to_str())
                        && stem != dir_name
                    {
                        ensure_method_md_ok(&object, stem, &p, &mut problems);
                    }
                }
            }
        }

        if !warnings.is_empty() {
            eprintln!("\nDocumentation warnings:\n");
            for w in &warnings {
                eprintln!("- {}", w);
            }
        }

        if !problems.is_empty() {
            eprintln!("\nDocumentation validation failed with the following issues:\n");
            for p in &problems {
                eprintln!("- {}", p);
            }
            panic!("documentation incomplete");
        }

        // Enforce that all public methods referenced in the API map have #[lsp_doc]
        enforce_lsp_doc_coverage(&objects, api_map, &mut problems);

        // If we reach here, validation passed.
    }

    // Website export steps are invoked directly from generate_docs()

    pub fn dir_to_object_name(dir: &str) -> String {
        let mut out = String::new();
        let mut capitalize = true;
        for ch in dir.chars() {
            if ch == '_' {
                capitalize = true;
                continue;
            }
            if capitalize {
                out.push(ch.to_ascii_uppercase());
            } else {
                out.push(ch);
            }
            capitalize = false;
        }
        out
    }

    pub fn public_structs_excluding_hidden() -> Vec<String> {
        use syn::{Item, Visibility};
        let entry = super::codegen::parse_lib_entry_point(&meta::workspace_root());
        fn is_doc_hidden(attrs: &[syn::Attribute]) -> bool {
            use quote::ToTokens;
            attrs.iter().any(|a| {
                a.to_token_stream().to_string().contains("doc")
                    && a.to_token_stream().to_string().contains("hidden")
            })
        }
        fn walk(path: &std::path::Path, items: Vec<Item>, out: &mut Vec<String>) {
            for item in items {
                match item {
                    Item::Mod(m) => {
                        if let Visibility::Public(_) = m.vis {
                            let (mod_path, mod_items) = super::codegen::parse_module(path, &m);
                            walk(&mod_path, mod_items, out);
                        }
                    }
                    Item::Struct(s) => {
                        if let Visibility::Public(_) = s.vis
                            && !is_doc_hidden(&s.attrs)
                            && has_lsp_doc(&s.attrs)
                        {
                            let name = s.ident.to_string();
                            if !out.contains(&name) {
                                out.push(name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(entry.0.as_path(), entry.1.items, &mut out);
        out.sort();
        out
    }

    pub fn collect_public_structs_info() -> Vec<(String, Vec<syn::Attribute>, Vec<String>)> {
        use syn::{Item, Visibility};
        let entry = codegen::parse_lib_entry_point(&meta::workspace_root());
        fn is_doc_hidden(attrs: &[syn::Attribute]) -> bool {
            use quote::ToTokens;
            attrs.iter().any(|a| {
                a.to_token_stream().to_string().contains("doc")
                    && a.to_token_stream().to_string().contains("hidden")
            })
        }
        fn collect_type_idents(ty: &syn::Type, out: &mut Vec<String>) {
            use syn::{GenericArgument, PathArguments, Type};
            match ty {
                Type::Path(tp) => {
                    if let Some(seg) = tp.path.segments.last() {
                        out.push(seg.ident.to_string());
                    }
                    for seg in &tp.path.segments {
                        if let PathArguments::AngleBracketed(ab) = &seg.arguments {
                            for arg in &ab.args {
                                if let GenericArgument::Type(inner) = arg {
                                    collect_type_idents(inner, out);
                                }
                            }
                        }
                    }
                }
                Type::Reference(r) => collect_type_idents(&r.elem, out),
                Type::Paren(p) => collect_type_idents(&p.elem, out),
                Type::Group(g) => collect_type_idents(&g.elem, out),
                Type::Tuple(t) => {
                    for elem in &t.elems {
                        collect_type_idents(elem, out);
                    }
                }
                Type::Array(a) => collect_type_idents(&a.elem, out),
                _ => {}
            }
        }
        fn walk(
            path: &std::path::Path,
            items: Vec<Item>,
            out: &mut Vec<(String, Vec<syn::Attribute>, Vec<String>)>,
        ) {
            for item in items {
                match item {
                    Item::Mod(m) => {
                        if let Visibility::Public(_) = m.vis {
                            let (mod_path, mod_items) = super::codegen::parse_module(path, &m);
                            walk(&mod_path, mod_items, out);
                        }
                    }
                    Item::Struct(s) => {
                        if let Visibility::Public(_) = s.vis
                            && !is_doc_hidden(&s.attrs)
                        {
                            let name = s.ident.to_string();
                            let mut inner_types = Vec::new();
                            match &s.fields {
                                syn::Fields::Unnamed(unnamed) => {
                                    if unnamed.unnamed.len() == 1 {
                                        collect_type_idents(
                                            &unnamed.unnamed[0].ty,
                                            &mut inner_types,
                                        );
                                    }
                                }
                                syn::Fields::Named(named) => {
                                    for f in named.named.iter() {
                                        collect_type_idents(&f.ty, &mut inner_types);
                                    }
                                }
                                syn::Fields::Unit => {}
                            }
                            // Dedup to reduce noise like [Option, WindowTarget, WindowTarget]
                            inner_types.sort();
                            inner_types.dedup();
                            out.push((name, s.attrs.clone(), inner_types));
                        }
                    }
                    _ => {}
                }
            }
        }
        let mut out = Vec::new();
        walk(entry.0.as_path(), entry.1.items, &mut out);
        out
    }

    pub fn base_public_objects() -> Vec<String> {
        let info = collect_public_structs_info();
        // Consider only documented types as canonical API objects.
        let documented: std::collections::HashSet<String> = info
            .iter()
            .filter(|(_, attrs, _)| has_lsp_doc(attrs))
            .map(|(n, _, _)| n.clone())
            .collect();

        // A struct is a wrapper if:
        // - It is a tuple newtype around a documented type, OR
        // - It contains a field that references a documented type (possibly nested in generics like Option<T>, Arc<T>, etc.)
        // We detect by checking whether any collected inner type idents match a documented name.
        let wrapper_names: std::collections::HashSet<String> = info
            .iter()
            .filter(|(n, _attrs, inner)| {
                // Prevent self-matching in degenerate cases
                inner.iter().any(|t| documented.contains(t) && t != n)
            })
            .map(|(n, _, _)| n.clone())
            .collect();

        // Base objects are documented types that are not wrappers
        let base: Vec<String> = info
            .iter()
            .filter(|(n, attrs, _)| {
                documented.contains(n) && !wrapper_names.contains(n) && has_lsp_doc(attrs)
            })
            .map(|(n, _, _)| n.clone())
            .collect();
        base
    }

    fn has_lsp_doc(attrs: &[syn::Attribute]) -> bool {
        attrs.iter().any(|a| a.path().is_ident("lsp_doc"))
    }

    /// Enforce that all public methods referenced in the API map have #[lsp_doc].
    ///
    /// Strategy:
    /// - Walk the crate's AST and collect (Type, method) pairs that are public and annotated with #[lsp_doc].
    /// - For each object and its methods in the provided ApiMap, verify the (Type, method) exists in the collected set.
    /// - Report a problem for any missing annotation.
    pub fn enforce_lsp_doc_coverage(
        objects: &[String],
        api_map: &ApiMap,
        problems: &mut Vec<String>,
    ) {
        use quote::ToTokens;
        use syn::{ImplItem, Item, Visibility};

        let entry = super::codegen::parse_lib_entry_point(&meta::workspace_root());

        fn is_doc_hidden(attrs: &[syn::Attribute]) -> bool {
            attrs.iter().any(|a| {
                a.to_token_stream().to_string().contains("doc")
                    && a.to_token_stream().to_string().contains("hidden")
            })
        }

        // Collect documented structs and methods
        use std::collections::HashSet;
        let mut doc_structs: HashSet<String> = HashSet::new();
        let mut doc_methods: HashSet<(String, String)> = HashSet::new();

        fn walk(
            path: &std::path::Path,
            items: Vec<Item>,
            doc_structs: &mut HashSet<String>,
            doc_methods: &mut HashSet<(String, String)>,
        ) {
            for item in items {
                match item {
                    Item::Mod(m) => {
                        if let Visibility::Public(_) = m.vis {
                            let (mod_path, mod_items) = super::codegen::parse_module(path, &m);
                            walk(&mod_path, mod_items, doc_structs, doc_methods);
                        }
                    }
                    Item::Struct(s) => {
                        if let Visibility::Public(_) = s.vis
                            && !is_doc_hidden(&s.attrs)
                            && super::validation::has_lsp_doc(&s.attrs)
                        {
                            doc_structs.insert(s.ident.to_string());
                        }
                    }
                    Item::Impl(item_impl) => {
                        // Only track inherent impls for types
                        if let syn::Type::Path(type_path) = *item_impl.self_ty {
                            let type_name =
                                type_path.path.segments.last().unwrap().ident.to_string();
                            for impl_item in item_impl.items {
                                if let ImplItem::Fn(method) = impl_item
                                    && matches!(method.vis, Visibility::Public(_))
                                    && !is_doc_hidden(&method.attrs)
                                    && super::validation::has_lsp_doc(&method.attrs)
                                {
                                    let name = method.sig.ident.to_string();
                                    doc_methods.insert((type_name.clone(), name));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        walk(
            entry.0.as_path(),
            entry.1.items,
            &mut doc_structs,
            &mut doc_methods,
        );

        // Objects were pre-filtered to those with #[lsp_doc], so struct-level checks are typically satisfied.
        // Check method-level coverage using the in-memory ApiMap.
        for o in objects {
            if let Some(list) = api_map.get(o) {
                for prop in list {
                    if let Some(fun) = &prop.function {
                        let name = &fun.name;
                        if !doc_methods.contains(&(o.clone(), name.clone())) {
                            problems.push(format!("Missing #[lsp_doc] on method {}::{}", o, name));
                        }
                    }
                }
            }
        }
    }
}
