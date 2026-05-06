mod locks {
    //! Build-time scanner for `<Lock id="...">` regions in MDX/MD content.
    //!
    //! Walks `docs/website/src/content/docs/**/*.{mdx,md}`, finds every
    //! `<Lock id="..." description="..." comments="...">...</Lock>` block,
    //! validates that every open tag has a matching close (and that locks
    //! don't nest), hashes the inner content with SipHash-1-3 (std), and
    //! tracks per-block version history in `.claude/locks/locks.json`.
    //! The store file is chmod 600'd after every write — a convention-level
    //! "agents shouldn't poke at this directly" signal more than a security
    //! boundary (any process running as the user can override).
    //!
    //! Failures (unpaired tag, nested lock, missing `id` attribute) print a
    //! diagnostic and exit the build. Hash drift does NOT fail the build —
    //! it just appends a new history entry. Restoration / strict drift
    //! enforcement is a follow-up tier.
    //!
    //! Companion CLI: `cargo run --release -p fce --example locks -- ...`
    //! reads the store and surfaces history + diffs.
    //!
    //! Re-run trigger: only the docs content directory; this scan is cheap
    //! (string scan + tiny JSON write) so re-running on every doc edit is
    //! fine.

    use std::collections::hash_map::DefaultHasher;
    use std::fs;
    use std::hash::{Hash, Hasher};
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use serde::{Deserialize, Serialize};

    const DOCS_DIR: &str = "docs/website/src/content/docs";
    const STORE_REL: &str = ".claude/locks/locks.json";

    #[derive(Default, Serialize, Deserialize)]
    struct Store {
        blocks: Vec<Block>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    struct Block {
        post_id: String,
        lock_id: String,
        current_version: u64,
        current_hash: String,
        current_content: String,
        description: Option<String>,
        comments: Option<String>,
        updated_at: u64, // unix seconds
        history: Vec<Snapshot>,
    }

    #[derive(Serialize, Deserialize, Clone)]
    struct Snapshot {
        version: u64,
        hash: String,
        content: String,
        saved_at: u64, // unix seconds
        description: Option<String>,
        comments: Option<String>,
    }

    pub fn run() {
        let workspace = super::meta::workspace_root();
        let docs_dir = workspace.join(DOCS_DIR);
        if !docs_dir.exists() {
            return; // no docs in this checkout — nothing to scan
        }

        let store_path = workspace.join(STORE_REL);
        if let Some(parent) = store_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let mut store = read_store(&store_path);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        // Walk and parse every MDX/MD under DOCS_DIR.
        let mut errors: Vec<String> = Vec::new();
        let mut seen: std::collections::HashSet<(String, String)> = std::collections::HashSet::new();

        for path in walk_docs(&docs_dir) {
            let rel = path
                .strip_prefix(&workspace)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let text = match fs::read_to_string(&path) {
                Ok(t) => t,
                Err(e) => {
                    errors.push(format!("read {}: {}", rel, e));
                    continue;
                }
            };
            let blocks = match parse_locks(&text) {
                Ok(b) => b,
                Err(e) => {
                    errors.push(format!("{}: {}", rel, e));
                    continue;
                }
            };
            for b in blocks {
                seen.insert((rel.clone(), b.id.clone()));
                ingest(&mut store, &rel, &b, now);
            }
        }

        if !errors.is_empty() {
            for e in &errors {
                eprintln!("locks: {}", e);
            }
            // Fail the build: unpaired or malformed Lock blocks are bugs.
            panic!("scripts/locks.rs: {} error(s) — see above", errors.len());
        }

        write_store(&store_path, &store);

        // chmod 600 on unix — convention-level deterrent.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = fs::metadata(&store_path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o600);
                let _ = fs::set_permissions(&store_path, perms);
            }
        }

        println!("cargo::rerun-if-changed={}", DOCS_DIR);
    }

    fn ingest(store: &mut Store, post_id: &str, parsed: &ParsedBlock, now: u64) {
        let hash = hash_content(&parsed.content);
        let pos = store
            .blocks
            .iter()
            .position(|b| b.post_id == post_id && b.lock_id == parsed.id);
        match pos {
            None => {
                let snap = Snapshot {
                    version: 1,
                    hash: hash.clone(),
                    content: parsed.content.clone(),
                    saved_at: now,
                    description: parsed.description.clone(),
                    comments: parsed.comments.clone(),
                };
                store.blocks.push(Block {
                    post_id: post_id.to_string(),
                    lock_id: parsed.id.clone(),
                    current_version: 1,
                    current_hash: hash,
                    current_content: parsed.content.clone(),
                    description: parsed.description.clone(),
                    comments: parsed.comments.clone(),
                    updated_at: now,
                    history: vec![snap],
                });
            }
            Some(i) => {
                let block = &mut store.blocks[i];
                if block.current_hash == hash {
                    // Content unchanged. Refresh metadata if attrs changed.
                    if block.description != parsed.description {
                        block.description = parsed.description.clone();
                    }
                    if block.comments != parsed.comments {
                        block.comments = parsed.comments.clone();
                    }
                    return;
                }
                let next_version = block.current_version + 1;
                let snap = Snapshot {
                    version: next_version,
                    hash: hash.clone(),
                    content: parsed.content.clone(),
                    saved_at: now,
                    description: parsed.description.clone(),
                    comments: parsed.comments.clone(),
                };
                block.current_version = next_version;
                block.current_hash = hash;
                block.current_content = parsed.content.clone();
                block.description = parsed.description.clone();
                block.comments = parsed.comments.clone();
                block.updated_at = now;
                block.history.push(snap);
            }
        }
    }

    fn read_store(path: &Path) -> Store {
        match fs::read_to_string(path) {
            Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
            Err(_) => Store::default(),
        }
    }

    fn write_store(path: &Path, store: &Store) {
        match serde_json::to_string_pretty(store) {
            Ok(s) => {
                if let Err(e) = fs::write(path, s) {
                    eprintln!("locks: write {} failed: {}", path.display(), e);
                }
            }
            Err(e) => eprintln!("locks: serialize failed: {}", e),
        }
    }

    fn walk_docs(dir: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() {
                    out.extend(walk_docs(&p));
                } else if let Some(ext) = p.extension() {
                    if ext == "mdx" || ext == "md" {
                        out.push(p);
                    }
                }
            }
        }
        out
    }

    fn hash_content(content: &str) -> String {
        let mut h = DefaultHasher::new();
        content.hash(&mut h);
        format!("{:016x}", h.finish())
    }

    struct ParsedBlock {
        id: String,
        description: Option<String>,
        comments: Option<String>,
        content: String,
    }

    fn parse_locks(text: &str) -> Result<Vec<ParsedBlock>, String> {
        let bytes = text.as_bytes();
        let len = bytes.len();
        let mut out = Vec::new();
        let mut i = 0;

        while i < len {
            let rel = match find_from(bytes, i, b"<Lock") {
                Some(n) => n,
                None => break,
            };
            // Confirm `<Lock` is followed by whitespace, `>`, or `/` — not
            // another identifier character (so we don't match `<Lockable`).
            let after = rel + 5;
            if after >= len {
                return Err("file ends mid-Lock tag".into());
            }
            let next = bytes[after];
            if !(next.is_ascii_whitespace() || next == b'>' || next == b'/') {
                i = rel + 1;
                continue;
            }

            // Find end of the open tag.
            let close_open = find_from(bytes, after, b">")
                .ok_or_else(|| format!("unclosed `<Lock` near byte {}", rel))?;

            // Self-closing `<Lock ... />` — locks must wrap content; reject.
            if close_open > rel && bytes[close_open - 1] == b'/' {
                return Err(format!(
                    "`<Lock />` is self-closing at byte {} — Lock must wrap content",
                    rel
                ));
            }

            let attrs = std::str::from_utf8(&bytes[after..close_open])
                .map_err(|e| format!("non-utf8 in attrs near byte {}: {}", rel, e))?;
            let content_start = close_open + 1;

            // Find the matching `</Lock>`. Disallow nesting: any `<Lock`
            // before the close fails the parse.
            let close_rel = find_from(bytes, content_start, b"</Lock>")
                .ok_or_else(|| format!("`<Lock>` opened at byte {} never closed", rel))?;
            if let Some(nested) = find_from(bytes, content_start, b"<Lock") {
                if nested < close_rel {
                    return Err(format!(
                        "nested Lock blocks not supported (outer opens at byte {}, inner at {})",
                        rel, nested
                    ));
                }
            }

            let content = std::str::from_utf8(&bytes[content_start..close_rel])
                .map_err(|e| format!("non-utf8 content near byte {}: {}", rel, e))?;

            let id = parse_attr(attrs, "id")
                .ok_or_else(|| format!("`<Lock>` near byte {} missing required `id` attribute", rel))?;
            let description = parse_attr(attrs, "description");
            let comments = parse_attr(attrs, "comments");

            out.push(ParsedBlock {
                id,
                description,
                comments,
                content: content.to_string(),
            });

            i = close_rel + b"</Lock>".len();
        }

        Ok(out)
    }

    /// Find `needle` in `haystack[from..]` and return the absolute index, or None.
    fn find_from(haystack: &[u8], from: usize, needle: &[u8]) -> Option<usize> {
        if from >= haystack.len() {
            return None;
        }
        haystack[from..]
            .windows(needle.len())
            .position(|w| w == needle)
            .map(|n| from + n)
    }

    /// Parse a double-quoted attribute value out of an attribute string.
    /// Accepts `name="..."`. Does NOT support `name={...}` JSX-expression
    /// syntax for the first iteration — the build script will surface a
    /// clear error if `id` is missing.
    fn parse_attr(attrs: &str, name: &str) -> Option<String> {
        let target = format!("{}=", name);
        let mut start = 0;
        while let Some(rel) = attrs[start..].find(&target) {
            let pos = start + rel;
            // Make sure the character before is a word boundary (start of
            // the attrs or whitespace) so `id` doesn't match e.g. `pid`.
            let before_ok = pos == 0
                || attrs.as_bytes()[pos - 1].is_ascii_whitespace();
            if !before_ok {
                start = pos + target.len();
                continue;
            }
            let after = &attrs[pos + target.len()..];
            let after = after.trim_start();
            if let Some(rest) = after.strip_prefix('"') {
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
            // `name={...}` is unsupported — bail this match, try next.
            start = pos + target.len();
        }
        None
    }
}
