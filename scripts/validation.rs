mod validation {
    use crate::codegen::ApiMap;

    use super::*;
    use std::fs;
    use std::path::Path;

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

    pub fn validate_docs(catalog: &super::codegen::ApiCatalog, api_map: &ApiMap) {
        let mut problems = Vec::new();
        let mut warnings = Vec::new();
        let root = meta::workspace_root();
        let docs_root = root.join("docs/api");

        // Enforce documentation for ALL public objects (including wrappers)
        let objects = catalog.public_structs_excluding_hidden();

        // Validate objects and their methods
        for object in objects.iter() {
            let methods_vec = api_map.get(object).cloned().unwrap_or_default();
            let dir = super::docs::object_dir_name(object);
            let obj_dir = super::docs::find_object_dir(&docs_root, &dir)
                .unwrap_or(docs_root.join(&dir));
            let object_md = obj_dir.join(format!("{}.md", dir));
            ensure_object_md_ok(object, &object_md, &mut problems);

            for m in &methods_vec {
                if let Some(fun) = &m.function {
                    let name = &fun.name;

                    let file = name.clone();
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
                }
            }
        }

        // Also validate any docs-only objects under docs/api not present in allowed (recursively)
        for (object, obj_dir, _cat) in super::docs::scan_docs_objects(&docs_root) {
            if objects.iter().any(|o| o == &object) {
                continue;
            }
            let dir_name = obj_dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let object_md = obj_dir.join(format!("{}.md", dir_name));
            ensure_object_md_ok(&object, &object_md, &mut problems);
        }

        // NEW: Validate method titles across all docs recursively (non-hidden), independent of API map
        for (object, obj_dir, _cat) in super::docs::scan_docs_objects(&docs_root) {
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
        enforce_lsp_doc_coverage(catalog, &objects, api_map, &mut problems);

        // If we reach here, validation passed.
    }

    /// Enforce that all public methods referenced in the API map have `#[lsp_doc]`.
    ///
    /// Reads `(Type, method)` pairs from the catalog (filtered to public,
    /// non-hidden, lsp-doc'd methods) and verifies every method listed under
    /// each object in `api_map` is present.
    pub fn enforce_lsp_doc_coverage(
        catalog: &super::codegen::ApiCatalog,
        objects: &[String],
        api_map: &ApiMap,
        problems: &mut Vec<String>,
    ) {
        use std::collections::HashSet;

        let doc_methods: HashSet<(String, String)> = catalog
            .methods
            .iter()
            .filter(|m| !m.is_doc_hidden && m.has_lsp_doc)
            .map(|m| (m.type_name.clone(), m.method_name.clone()))
            .collect();

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
