mod parity {
    //! API parity audit.
    //!
    //! Cross-references the canonical API surface (`docs/api/**/*.md`) against
    //! actual platform bindings discovered in `src/`. Every documented
    //! method/object must be reachable from every supported platform — JS,
    //! Python, Swift, Kotlin — unless explicitly waived in `docs/api/PARITY`.
    //!
    //! ## How a binding gets credited
    //!
    //! Every platform-bound method in `src/` carries an `#[lsp_doc("docs/api/.../X.md")]`
    //! attribute that ties the binding back to its documentation page. The audit
    //! walks every `impl` block, classifies its platform context (any one of
    //! `#[wasm_bindgen]`, `#[pymethods]`, `#[uniffi::export]`, or a file-level
    //! `#![cfg(<platform>)]` gate), and records the lsp_doc paths of methods
    //! inside.
    //!
    //! ## Hidden overrides
    //!
    //! When a platform's signature diverges from the canonical doc (e.g. uniffi
    //! cannot marshal `impl Into<T>` so the mobile constructor takes a concrete
    //! type), the binding's `lsp_doc` points at a `hidden/<method>_<platform>.md`
    //! file. The audit treats those overrides as parity for the canonical
    //! `<method>.md` for the corresponding platform.
    //!
    //! Recognized suffixes in `hidden/`:
    //!   - `_js`     → Web
    //!   - `_py`     → Python
    //!   - `_mobile` → Swift AND Kotlin
    //!   - `_swift`  → Swift only
    //!   - `_kotlin` → Kotlin only
    //!
    //! ## Waivers
    //!
    //! Some divergence is intentional and load-bearing. JS's `Shader.fetch`
    //! exists because WASM constructors cannot be async; the same need does
    //! not exist on platforms whose `Shader::new(URL)` can block on I/O.
    //! These cases are listed in `docs/api/PARITY` with a written reason.
    //!
    //! ## Mode
    //!
    //! The audit currently runs in **warn mode** — gaps are printed but do
    //! NOT fail the build. The intent is to converge to zero gaps via Phase 3
    //! (uniffi exports for Mesh/Vertex/Pass/etc.) and then flip to fail mode.

    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::{Path, PathBuf};
    use syn::{Attribute, ImplItem, Item, ItemImpl, Meta, parse_file};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Platform {
        Web,
        Python,
        Swift,
        Kotlin,
    }

    impl Platform {
        fn label(self) -> &'static str {
            match self {
                Platform::Web => "web",
                Platform::Python => "python",
                Platform::Swift => "swift",
                Platform::Kotlin => "kotlin",
            }
        }

        fn all() -> [Platform; 4] {
            [
                Platform::Web,
                Platform::Python,
                Platform::Swift,
                Platform::Kotlin,
            ]
        }
    }

    /// Doc paths used as keys throughout this module are workspace-relative
    /// (e.g. `docs/api/core/shader/set.md`). They match the strings written
    /// in `#[lsp_doc(...)]` attributes verbatim.
    type DocPath = String;

    #[derive(Default, Debug)]
    struct Bindings {
        per_doc: BTreeMap<DocPath, BTreeSet<Platform>>,
    }

    /// One waiver entry from `docs/api/PARITY`.
    #[derive(Debug, Clone)]
    struct Waiver {
        platforms_present: BTreeSet<Platform>,
        /// Free-form rationale captured for human review of the manifest;
        /// not consumed by the audit logic itself, but kept on the struct
        /// so unrecognized future use sites have it without re-parsing.
        #[allow(dead_code)]
        reason: String,
    }

    #[derive(Default, Debug)]
    struct Waivers {
        // Keyed by doc path relative to workspace (e.g. "docs/api/core/shader/fetch.md").
        entries: BTreeMap<DocPath, Waiver>,
        /// Hidden-override → canonical aliases. Used when the hidden filename
        /// does not match the canonical via simple suffix-strip — e.g. uniffi
        /// splits `Renderer::render` into two concrete methods (`renderShader`
        /// and `renderShaderToTexture`) whose hidden docs are
        /// `render_shader_mobile.md` and `render_shader_texture_mobile.md`,
        /// neither of which strips back to `render.md`. The alias declares
        /// the connection explicitly.
        aliases: BTreeMap<DocPath, DocPath>,
    }

    #[derive(Debug)]
    pub struct ParityReport {
        pub gaps: Vec<Gap>,
        pub waiver_count: usize,
        pub alias_count: usize,
        /// Bindings whose `#[lsp_doc(...)]` path doesn't resolve to any
        /// canonical docs/api entry (after waiver/alias/suffix-strip
        /// resolution). Each is a real binding pointing at a dangling docs
        /// link — surfaced as a warning so the offending file can be fixed.
        pub unresolved_bindings: Vec<(String, BTreeSet<Platform>)>,
        pub bound_doc_count: usize,
        pub doc_count: usize,
    }

    #[derive(Debug)]
    pub struct Gap {
        pub doc_path: DocPath,
        pub missing: BTreeSet<Platform>,
    }

    /// Public entry point. Walks docs/api/ and src/ once each, applies
    /// waivers and aliases, and returns the resulting report.
    pub fn audit(workspace_root: &Path) -> ParityReport {
        let raw_bindings = scan_bindings(&workspace_root.join("src"));
        let docs = scan_docs(&workspace_root.join("docs/api"));
        let waivers = load_waivers(&workspace_root.join("docs/api/PARITY"));

        // Resolve every raw binding's lsp_doc string to a canonical doc path.
        // Strategy:
        //   1. If the lsp_doc IS a canonical doc, use it.
        //   2. If an alias is declared in PARITY, use the aliased canonical.
        //   3. If the lsp_doc points inside `hidden/` and the suffix-strip
        //      yields an existing canonical, use that.
        //   4. Otherwise: skip and report the unresolved lsp_doc as a warning
        //      (the binding is real but its docs link is dangling).
        let docs_set: BTreeSet<&str> = docs.iter().map(String::as_str).collect();
        let mut resolved: BTreeMap<DocPath, BTreeSet<Platform>> = BTreeMap::new();
        let mut unresolved: Vec<(String, BTreeSet<Platform>)> = Vec::new();
        for (lsp_doc, platforms) in &raw_bindings.per_doc {
            let canonical = resolve_canonical(lsp_doc, &docs_set, &waivers.aliases);
            match canonical {
                Some(c) => {
                    resolved.entry(c).or_default().extend(platforms.iter().copied());
                }
                None => unresolved.push((lsp_doc.clone(), platforms.clone())),
            }
        }

        let mut gaps = Vec::new();
        let mut bound_doc_count = 0;
        for doc in &docs {
            let bound = resolved.get(doc).cloned().unwrap_or_default();
            if !bound.is_empty() {
                bound_doc_count += 1;
            }
            let mut missing: BTreeSet<Platform> = Platform::all().into_iter().collect();
            for p in &bound {
                missing.remove(p);
            }
            if let Some(w) = waivers.entries.get(doc) {
                for p in &w.platforms_present {
                    missing.remove(p);
                }
                // Anything NOT in platforms_present is intentionally absent.
                let waived: BTreeSet<Platform> = Platform::all()
                    .into_iter()
                    .filter(|p| !w.platforms_present.contains(p))
                    .collect();
                for p in &waived {
                    missing.remove(p);
                }
            }
            if !missing.is_empty() {
                gaps.push(Gap {
                    doc_path: doc.clone(),
                    missing,
                });
            }
        }

        ParityReport {
            gaps,
            waiver_count: waivers.entries.len(),
            alias_count: waivers.aliases.len(),
            unresolved_bindings: unresolved,
            bound_doc_count,
            doc_count: docs.len(),
        }
    }

    /// Resolve a raw `lsp_doc` string (as it appears verbatim in `#[lsp_doc(...)]`)
    /// to a canonical docs/api entry, if one exists. See the top of `audit()`
    /// for the resolution strategy.
    fn resolve_canonical(
        lsp_doc: &str,
        docs: &BTreeSet<&str>,
        aliases: &BTreeMap<DocPath, DocPath>,
    ) -> Option<DocPath> {
        if docs.contains(lsp_doc) {
            return Some(lsp_doc.to_string());
        }
        if let Some(target) = aliases.get(lsp_doc) {
            return Some(target.clone());
        }
        // Hidden-override fallback: strip the platform suffix.
        let stripped = canonicalize_via_strip(lsp_doc);
        if let Some(s) = stripped
            && docs.contains(s.as_str())
        {
            return Some(s);
        }
        None
    }

    /// Print the report. `fail_build` panics the build script with a summary
    /// when there are gaps; otherwise gaps are printed as warnings.
    pub fn print_report(report: &ParityReport, fail_build: bool) {
        let total = report.doc_count;
        let bound = report.bound_doc_count;
        let gaps = report.gaps.len();
        let waivers = report.waiver_count;
        let aliases = report.alias_count;
        let unresolved = report.unresolved_bindings.len();

        if report.gaps.is_empty() && report.unresolved_bindings.is_empty() {
            println!(
                "✅ API parity: {} documented entries, {} bound on every supported platform, {} explicit waiver(s), {} alias(es).",
                total, bound, waivers, aliases
            );
            return;
        }

        let header_emoji = if fail_build { "❌" } else { "⚠️" };
        let header_mode = if fail_build { "FAIL" } else { "WARN" };
        println!(
            "\n{} API parity {}: {} of {} documented entries lack bindings on one or more platforms; {} unresolved lsp_doc link(s) ({} bound somewhere; {} waiver(s); {} alias(es)).",
            header_emoji, header_mode, gaps, total, unresolved, bound, waivers, aliases
        );

        // Group gaps by which set of platforms is missing — easier to read than
        // a flat list.
        let mut by_missing: BTreeMap<BTreeSet<Platform>, Vec<&Gap>> = BTreeMap::new();
        for g in &report.gaps {
            by_missing.entry(g.missing.clone()).or_default().push(g);
        }
        let mut groups: Vec<(BTreeSet<Platform>, Vec<&Gap>)> = by_missing.into_iter().collect();
        groups.sort_by_key(|(_, gs)| std::cmp::Reverse(gs.len()));

        for (missing, gs) in groups {
            let labels: Vec<&str> = missing.iter().map(|p| p.label()).collect();
            println!("\n  Missing on [{}] ({} docs):", labels.join(", "), gs.len());
            for g in gs {
                println!("    - {}", g.doc_path);
            }
        }

        if !report.unresolved_bindings.is_empty() {
            println!(
                "\n  Unresolved lsp_doc paths ({} bindings point at a docs file that no longer exists or doesn't strip to a real canonical):",
                unresolved
            );
            let mut sorted = report.unresolved_bindings.clone();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (path, plats) in sorted {
                let labels: Vec<&str> = plats.iter().map(|p| p.label()).collect();
                println!("    - {}  [credited: {}]", path, labels.join(", "));
            }
        }

        let footer = if fail_build {
            "\n\nBuild failed because API parity is enforced. Add a binding or list a waiver/alias in docs/api/PARITY.\n"
        } else {
            "\n\nThis is a warning. Convergence to zero gaps is tracked under the API-parity initiative; the audit will be flipped to fail-build mode (set FC_PARITY_STRICT=1 to opt in early) once the contract is reached.\n"
        };
        println!("{}", footer);

        if fail_build {
            panic!(
                "API parity audit failed: {} unwaived gap(s), {} unresolved lsp_doc link(s).",
                gaps, unresolved
            );
        }
    }

    // -- docs scan -------------------------------------------------------

    /// Walk docs/api/ recursively and collect all canonical method/object
    /// MD files. Skips:
    ///   - hidden/ subdirectories (those are platform-specific overrides;
    ///     the audit credits bindings pointing at hidden files toward the
    ///     canonical doc, but the hidden files themselves don't need
    ///     bindings of their own).
    ///   - top-level docs/api/README.md and category _index.md files.
    fn scan_docs(api_root: &Path) -> Vec<DocPath> {
        let mut out = Vec::new();
        if !api_root.exists() {
            return out;
        }
        walk_docs(api_root, api_root, &mut out);
        out.sort();
        out
    }

    fn walk_docs(api_root: &Path, dir: &Path, out: &mut Vec<DocPath>) {
        let read = match fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        for entry in read.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if p.file_name().and_then(|s| s.to_str()) == Some("hidden") {
                    continue;
                }
                walk_docs(api_root, &p, out);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or_default();
            if name == "README.md" || name == "_index.md" {
                continue;
            }
            // Skip object-level docs (e.g. `core/shader/shader.md`). Those describe
            // the type itself; parity for the type is enforced by the per-method
            // entries that live alongside.
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or_default();
            let parent_name = p
                .parent()
                .and_then(|d| d.file_name())
                .and_then(|s| s.to_str())
                .unwrap_or_default();
            if !stem.is_empty() && stem == parent_name {
                continue;
            }
            // Workspace-relative path with forward slashes — same shape as
            // the strings inside `#[lsp_doc(...)]`.
            let workspace = api_root
                .parent() // docs/
                .and_then(Path::parent) // workspace root
                .unwrap_or(api_root);
            let rel = match p.strip_prefix(workspace) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let s = rel.to_string_lossy().replace('\\', "/");
            out.push(s);
        }
    }

    // -- bindings scan ---------------------------------------------------

    fn scan_bindings(src_root: &Path) -> Bindings {
        let mut b = Bindings::default();
        if !src_root.exists() {
            return b;
        }
        walk_src(src_root, &mut b);
        b
    }

    fn walk_src(dir: &Path, b: &mut Bindings) {
        let read = match fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        for entry in read.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk_src(&p, b);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            process_file(&p, b);
        }
    }

    fn process_file(file: &Path, b: &mut Bindings) {
        let text = match fs::read_to_string(file) {
            Ok(t) => t,
            Err(_) => return,
        };
        let parsed = match parse_file(&text) {
            Ok(p) => p,
            Err(_) => return,
        };
        let file_platforms = file_level_platforms(&parsed.attrs);
        for item in &parsed.items {
            visit_item(item, b, &file_platforms);
        }
    }

    fn visit_item(item: &Item, b: &mut Bindings, file_platforms: &[Platform]) {
        match item {
            Item::Impl(it) => visit_impl(it, b, file_platforms),
            Item::Fn(it) => visit_free_fn(&it.attrs, b, file_platforms),
            Item::Mod(it) => {
                if let Some((_, items)) = &it.content {
                    let nested_platforms = combine_platforms(file_platforms, &it.attrs);
                    for inner in items {
                        visit_item(inner, b, &nested_platforms);
                    }
                }
            }
            _ => {}
        }
    }

    /// Free functions (top-level `pub fn`) can carry a binding marker too —
    /// `#[uniffi::export]` for mobile (e.g. `set_shader_registry` in
    /// src/shader/platform/mobile.rs), `#[wasm_bindgen]` directly, or
    /// `#[pyfunction]` for Python. If the function carries an `#[lsp_doc(...)]`
    /// annotation, credit the matching platforms.
    fn visit_free_fn(attrs: &[Attribute], b: &mut Bindings, file_platforms: &[Platform]) {
        let doc = match lsp_doc_path(attrs) {
            Some(d) => d,
            None => return,
        };
        let mut platforms: Vec<Platform> = file_platforms.to_vec();
        for a in attrs {
            if attr_is(a, "wasm_bindgen") {
                push_unique(&mut platforms, Platform::Web);
            }
            if attr_is(a, "pyfunction") {
                push_unique(&mut platforms, Platform::Python);
            }
            if is_uniffi_export(a) {
                push_unique(&mut platforms, Platform::Swift);
                push_unique(&mut platforms, Platform::Kotlin);
            }
            if let Meta::List(list) = &a.meta {
                let path = path_string(&list.path);
                if path == "cfg" || path == "cfg_attr" {
                    let toks = list.tokens.to_string();
                    apply_platform_tokens(&toks, &mut platforms);
                }
            }
        }
        if platforms.is_empty() {
            return;
        }
        credit_binding(b, &doc, &platforms);
    }

    fn visit_impl(it: &ItemImpl, b: &mut Bindings, file_platforms: &[Platform]) {
        let impl_platforms = combine_platforms(file_platforms, &it.attrs);
        if impl_platforms.is_empty() {
            return;
        }
        for impl_item in &it.items {
            if let ImplItem::Fn(m) = impl_item
                && let Some(doc) = lsp_doc_path(&m.attrs)
            {
                credit_binding(b, &doc, &impl_platforms);
            }
        }
    }

    /// A binding's `#[lsp_doc(...)]` value is recorded verbatim. The
    /// canonical resolution (suffix-strip / aliases / existence-check)
    /// happens later in `audit()` once we have the full docs catalog and
    /// PARITY map loaded. The platforms credited are the intersection of
    /// the binding's host context (the impl/file platform set) and any
    /// platform suffix the lsp_doc carries — e.g. `_mobile` on a hidden
    /// override caps the credit to Swift+Kotlin even if the impl block
    /// were wider.
    fn credit_binding(b: &mut Bindings, lsp_doc: &str, host_platforms: &[Platform]) {
        let target = override_platforms(lsp_doc).unwrap_or_else(|| host_platforms.to_vec());
        let entry = b.per_doc.entry(lsp_doc.to_string()).or_default();
        for p in &target {
            if host_platforms.contains(p) {
                entry.insert(*p);
            }
        }
    }

    /// Merge file-level platform context with attributes on a mod or impl
    /// block. We accept any platform indicator: a direct `#[wasm_bindgen]`
    /// attribute, a `#[cfg(wasm)]` gate, or a `#[cfg_attr(wasm, ...)]` form.
    fn combine_platforms(parent: &[Platform], attrs: &[Attribute]) -> Vec<Platform> {
        let mut out: Vec<Platform> = parent.to_vec();
        for a in attrs {
            // Direct binding markers
            if attr_is(a, "wasm_bindgen") {
                push_unique(&mut out, Platform::Web);
            }
            if attr_is(a, "pymethods") {
                push_unique(&mut out, Platform::Python);
            }
            if is_uniffi_export(a) {
                push_unique(&mut out, Platform::Swift);
                push_unique(&mut out, Platform::Kotlin);
            }
            // cfg(...) / cfg_attr(...) gates
            if let Meta::List(list) = &a.meta {
                let path = path_string(&list.path);
                if path == "cfg" || path == "cfg_attr" {
                    let toks = list.tokens.to_string();
                    apply_platform_tokens(&toks, &mut out);
                }
            }
        }
        out
    }

    /// Inspect file-level attributes (`#![cfg(...)]`) for platform context.
    fn file_level_platforms(attrs: &[Attribute]) -> Vec<Platform> {
        let mut out = Vec::new();
        for a in attrs {
            if let Meta::List(list) = &a.meta {
                let path = path_string(&list.path);
                if path == "cfg" || path == "cfg_attr" {
                    let toks = list.tokens.to_string();
                    apply_platform_tokens(&toks, &mut out);
                }
            }
        }
        out
    }

    /// Recognise platform identifiers inside a cfg/cfg_attr token stream.
    /// `wasm`, `python`, `mobile`, `ios`, `android`, `apple` all expand to
    /// one or more platforms in this codebase's cfg_aliases.rs setup.
    fn apply_platform_tokens(toks: &str, out: &mut Vec<Platform>) {
        // Coarse: word-match against known alias identifiers. False positives
        // would be detected only if a non-cfg identifier collided with one
        // of these keywords; the cost of a miss here is a false negative in
        // the audit, not a bad binding.
        for word in toks.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
            match word {
                "wasm" => push_unique(out, Platform::Web),
                "python" => push_unique(out, Platform::Python),
                "mobile" | "ios" | "android" | "apple" => {
                    push_unique(out, Platform::Swift);
                    push_unique(out, Platform::Kotlin);
                }
                _ => {}
            }
        }
    }

    fn push_unique(v: &mut Vec<Platform>, p: Platform) {
        if !v.contains(&p) {
            v.push(p);
        }
    }

    fn attr_is(a: &Attribute, ident: &str) -> bool {
        a.path().is_ident(ident)
    }

    /// `#[uniffi::export]` and `#[uniffi::method(...)]` and the like.
    /// Ignore the constructor sub-form because that's per-method, not
    /// per-impl-block.
    fn is_uniffi_export(a: &Attribute) -> bool {
        let p = path_string(a.path());
        p == "uniffi::export"
    }

    fn path_string(p: &syn::Path) -> String {
        p.segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }

    fn lsp_doc_path(attrs: &[Attribute]) -> Option<String> {
        for a in attrs {
            if !a.path().is_ident("lsp_doc") {
                continue;
            }
            if let Meta::List(list) = &a.meta {
                let toks = list.tokens.to_string();
                // Tokens look like: `"docs/api/core/shader/set.md"` (with the quotes).
                let trimmed = toks.trim();
                if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
                    return Some(trimmed[1..trimmed.len() - 1].to_string());
                }
            }
        }
        None
    }

    /// Suffix-strip rule: given a path that points at
    /// `<obj>/hidden/<method>_<platform>.md`, return the candidate
    /// canonical sibling `<obj>/<method>.md`. Returns `None` if the path
    /// does not match the hidden-override shape, or the suffix isn't one
    /// of the recognised platform tokens. Pure string transform — no
    /// filesystem check; the caller (`resolve_canonical`) decides whether
    /// the candidate is real.
    fn canonicalize_via_strip(p: &str) -> Option<String> {
        let normalized = p.replace('\\', "/");
        let pb = PathBuf::from(&normalized);
        let parent = pb.parent()?;
        if parent.file_name().and_then(|s| s.to_str()) != Some("hidden") {
            return None;
        }
        let file_stem = pb.file_stem().and_then(|s| s.to_str())?;
        let canonical_stem = strip_platform_suffix(file_stem)?;
        let object_dir = parent.parent()?;
        let canonical = object_dir.join(format!("{}.md", canonical_stem));
        Some(canonical.to_string_lossy().replace('\\', "/"))
    }

    /// Map a hidden-override platform suffix to the platforms it credits.
    /// `_ios` and `_macos` count for Swift only (those are the Apple
    /// targets the Swift binding ships against); `_android` counts for
    /// Kotlin only; `_mobile` covers both. Returns `None` for unrecognised
    /// suffixes — those become unresolved bindings reported in the audit.
    fn platforms_for_suffix(suffix: &str) -> Option<Vec<Platform>> {
        match suffix {
            "js" => Some(vec![Platform::Web]),
            "py" => Some(vec![Platform::Python]),
            "mobile" => Some(vec![Platform::Swift, Platform::Kotlin]),
            "swift" | "ios" | "macos" => Some(vec![Platform::Swift]),
            "kotlin" | "android" => Some(vec![Platform::Kotlin]),
            _ => None,
        }
    }

    /// If the lsp_doc points at a hidden override, return the platforms
    /// that override applies to. Returns None for canonical paths.
    fn override_platforms(p: &str) -> Option<Vec<Platform>> {
        let normalized = p.replace('\\', "/");
        let pb = PathBuf::from(&normalized);
        let parent_name = pb.parent()?.file_name().and_then(|s| s.to_str())?;
        if parent_name != "hidden" {
            return None;
        }
        let file_stem = pb.file_stem().and_then(|s| s.to_str())?;
        let underscore = file_stem.rfind('_')?;
        let suffix = &file_stem[underscore + 1..];
        platforms_for_suffix(suffix)
    }

    fn strip_platform_suffix(stem: &str) -> Option<String> {
        let underscore = stem.rfind('_')?;
        let suffix = &stem[underscore + 1..];
        if platforms_for_suffix(suffix).is_some() {
            Some(stem[..underscore].to_string())
        } else {
            None
        }
    }

    // -- waivers ---------------------------------------------------------

    /// `docs/api/PARITY` — two entry types share the file:
    ///
    /// **Waiver** (parity intentionally restricted to a platform subset):
    ///
    /// ```text
    /// <doc_path>:<platforms_present>:<reason>
    /// ```
    ///
    /// **Alias** (hidden override credits a canonical doc; needed when the
    /// override's filename does not strip to the canonical via the `_<platform>`
    /// suffix rule — e.g. uniffi splits `render` into `renderShader` and
    /// `renderShaderToTexture`):
    ///
    /// ```text
    /// <hidden_doc_path>=><canonical_doc_path>:<reason>
    /// ```
    ///
    /// Lines starting with `#` are comments; blank lines are ignored. The
    /// `=>` is detected before the first `:` to disambiguate the two forms.
    fn load_waivers(file: &Path) -> Waivers {
        let mut w = Waivers::default();
        let text = match fs::read_to_string(file) {
            Ok(t) => t,
            Err(_) => return w,
        };
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(arrow) = line.find("=>") {
                // Alias entry: `<hidden>=><canonical>:<reason>`.
                let hidden = line[..arrow].trim().to_string();
                let rest = &line[arrow + 2..];
                let mut tail = rest.splitn(2, ':');
                let canonical = match tail.next() {
                    Some(s) => s.trim().to_string(),
                    None => {
                        eprintln!(
                            "docs/api/PARITY:{}: malformed alias (missing canonical): {}",
                            lineno + 1,
                            line
                        );
                        continue;
                    }
                };
                // Reason is captured but not consumed; reading it as a sanity
                // check that the manifest author wrote one.
                let _reason = tail.next().map(str::trim).unwrap_or("");
                w.aliases.insert(hidden, canonical);
                continue;
            }
            // Waiver entry: `<doc>:<platforms_present>:<reason>`.
            // Split on the first two `:` separators so reasons may contain colons.
            let mut parts = line.splitn(3, ':');
            let doc = match parts.next() {
                Some(s) => s.trim(),
                None => continue,
            };
            let platforms_str = match parts.next() {
                Some(s) => s.trim(),
                None => {
                    eprintln!(
                        "docs/api/PARITY:{}: malformed entry (missing platforms): {}",
                        lineno + 1,
                        line
                    );
                    continue;
                }
            };
            let reason = parts.next().map(str::trim).unwrap_or("").to_string();
            let mut platforms_present = BTreeSet::new();
            for p in platforms_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                match p {
                    "web" => {
                        platforms_present.insert(Platform::Web);
                    }
                    "python" => {
                        platforms_present.insert(Platform::Python);
                    }
                    "swift" => {
                        platforms_present.insert(Platform::Swift);
                    }
                    "kotlin" => {
                        platforms_present.insert(Platform::Kotlin);
                    }
                    other => {
                        eprintln!(
                            "docs/api/PARITY:{}: unknown platform '{}'",
                            lineno + 1,
                            other
                        );
                    }
                }
            }
            w.entries.insert(
                doc.to_string(),
                Waiver {
                    platforms_present,
                    reason,
                },
            );
        }
        w
    }
}
