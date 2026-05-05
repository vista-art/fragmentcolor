mod docs {
    //! Shared helpers for walking the canonical `docs/api/` tree.
    //!
    //! Both the validator and the website exporter need to ask the same
    //! questions of `docs/api/`:
    //!   - "where does the directory for object `X` live?"
    //!   - "list every documented object — what's its dir, what category is it in?"
    //!
    //! Until this module landed, each consumer carried its own near-identical
    //! recursive walk. The helpers below are the single source of truth.

    use std::path::{Path, PathBuf};

    /// CamelCase object name → snake_case directory slug.
    /// `TextureMipChain` → `texture_mip_chain`.
    pub fn object_dir_name(object: &str) -> String {
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

    /// Inverse of `object_dir_name`.
    /// `texture_mip_chain` → `TextureMipChain`.
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

    /// Find the first directory under `docs_root` whose `file_name` is
    /// `dir_name` AND that contains a sibling `<dir_name>.md` (the convention
    /// for "object dirs" in `docs/api/`). Returns the matching path, if any.
    pub fn find_object_dir(docs_root: &Path, dir_name: &str) -> Option<PathBuf> {
        fn walk(root: &Path, target: &str) -> Option<PathBuf> {
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

    /// Path of `obj_dir`'s parent relative to `docs_root`, normalized to use
    /// forward slashes. Empty string if the parent is `docs_root` itself.
    pub fn category_rel_from(docs_root: &Path, obj_dir: &Path) -> String {
        let parent = obj_dir.parent().unwrap_or(docs_root);
        if let Ok(rel) = parent.strip_prefix(docs_root) {
            rel.to_string_lossy().replace('\\', "/")
        } else {
            String::new()
        }
    }

    /// Recursively enumerate every "object directory" under `docs_root` —
    /// a directory whose name matches an inner `<name>.md` file. The walker
    /// does not descend into an object dir once found.
    ///
    /// Returns `(object_name, object_dir, category_rel)` triples.
    pub fn scan_docs_objects(docs_root: &Path) -> Vec<(String, PathBuf, String)> {
        fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, PathBuf, String)>) {
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
                            let cat = category_rel_from(root, &p);
                            out.push((object, p.clone(), cat));
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
}
