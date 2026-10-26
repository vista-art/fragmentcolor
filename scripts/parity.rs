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
    //! ## Unlabeled-export check
    //!
    //! A second check catches the opposite blind spot: platform-bound `pub fn`
    //! items that exist but carry no `#[lsp_doc(...)]`. When a binding agent
    //! forgets to add `#[lsp_doc]`, the gap check silently considers the API
    //! "missing" on that platform — the binding exists but is uncredited.
    //!
    //! Scope: **platform-only** — only `pub fn` methods inside impl blocks
    //! decorated with `#[wasm_bindgen]`, `#[pymethods]`, or `#[uniffi::export]`
    //! are checked. Free functions and non-platform impl blocks are out of scope.
    //!
    //! Exemption: add `#[doc(hidden)]` to any platform-bound method that is
    //! intentionally internal (JS compat shims, internal duck-typed interfaces,
    //! helper type constructors, etc.). The check skips methods annotated with
    //! `#[doc(hidden)]`.
    //!
    //! ## Mode
    //!
    //! The audit runs in strict mode by default and fails the build
    //! on any gap, unresolved binding, or unlabeled export missing from
    //! `docs/api/PARITY_BASELINE`. `FC_PARITY_LENIENT=1` prints the report
    //! without failing, and `FC_PARITY_REWRITE_BASELINE=1` writes the current
    //! findings into the baseline, as the build does when the file is missing.

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

    /// Output of `classify_gaps`: three live-vs-baseline buckets per gap.
    /// 0: acknowledged (subset of baseline), 1: regressed (with baseline
    /// missing-set for the diff), 2: new (not in baseline).
    type GapClassification<'a> = (
        Vec<&'a Gap>,
        Vec<(&'a Gap, BTreeSet<Platform>)>,
        Vec<&'a Gap>,
    );

    /// Output of `classify_unresolved`: same partition shape as
    /// `GapClassification`, keyed on the unresolved-binding tuple.
    type UnresolvedClassification<'a> = (
        Vec<&'a (String, BTreeSet<Platform>)>,
        Vec<(&'a String, &'a BTreeSet<Platform>, BTreeSet<Platform>)>,
        Vec<&'a (String, BTreeSet<Platform>)>,
    );

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

    /// A platform-bound `pub fn` that has neither an `#[lsp_doc(...)]` nor an
    /// `#[doc(hidden)]` annotation. This is the "forgotten lsp_doc" blind spot:
    /// the binding exists but is uncredited to the parity audit.
    ///
    /// Fields:
    ///   - `file`        — workspace-relative path of the source file
    ///   - `fn_name`     — the Rust function name (e.g. `texture_js`)
    ///   - `platform`    — which binding context it was found in
    #[derive(Debug, Clone)]
    pub struct UnlabeledExport {
        pub file: String,
        pub fn_name: String,
        pub platform: Platform,
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
        /// Platform-bound `pub fn` items that have neither `#[lsp_doc(...)]`
        /// nor `#[doc(hidden)]`. These are "uncredited exports" — the binding
        /// exists but the parity audit cannot credit it because the annotation
        /// is missing. Fix by adding `#[lsp_doc(...)]` or `#[doc(hidden)]`.
        pub unlabeled_exports: Vec<UnlabeledExport>,
        pub bound_doc_count: usize,
        pub doc_count: usize,
    }

    #[derive(Debug)]
    pub struct Gap {
        pub doc_path: DocPath,
        pub missing: BTreeSet<Platform>,
    }

    /// Audit operating mode. Selected by environment in `build.rs`:
    ///
    ///   - default                          → `Strict`. Phase 3 forcing function.
    ///   - `FC_PARITY_LENIENT=1`            → `Warn`. Local opt-out.
    ///   - `FC_PARITY_REWRITE_BASELINE=1`   → `RewriteBaseline`. Snapshot the
    ///     current state into PARITY_BASELINE, then exit cleanly. Used after
    ///     a Phase 3 batch closes gaps so the ratchet tightens.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Mode {
        Warn,
        Strict,
        RewriteBaseline,
    }

    /// Currently-acknowledged gaps + unresolved bindings + unlabeled exports,
    /// loaded from PARITY_BASELINE. Strict mode panics on anything in the live
    /// audit that is NOT a subset of these baselines.
    #[derive(Default, Debug)]
    struct Baseline {
        /// Per-doc acknowledged missing platforms. Live gap is acceptable
        /// when its missing-set is a subset of the baseline's.
        gaps: BTreeMap<DocPath, BTreeSet<Platform>>,
        /// Per-lsp-doc-path acknowledged credited platforms. Same subset rule.
        unresolved: BTreeMap<String, BTreeSet<Platform>>,
        /// Set of "<file>:<fn_name>" pairs acknowledged as unlabeled exports.
        /// Each entry is an uncredited platform export that the team has not
        /// yet fixed. Strict mode panics on any NEW entry not in this set.
        unlabeled: BTreeSet<String>,
    }

    /// Public entry point. Walks docs/api/ and src/ once each, applies
    /// waivers and aliases, and returns the resulting report.
    pub fn audit(workspace_root: &Path) -> ParityReport {
        let raw_bindings = scan_bindings(&workspace_root.join("src"));
        let docs = scan_docs(&workspace_root.join("docs/api"));
        let waivers = load_waivers(&workspace_root.join("docs/api/PARITY"));
        let unlabeled_exports = scan_unlabeled_exports(&workspace_root.join("src"), workspace_root);

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
            unlabeled_exports,
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

    /// Print the report and (in Strict mode) panic if any drift is detected
    /// outside the acknowledged baseline. `baseline_path` points to
    /// `docs/api/PARITY_BASELINE` — see `Baseline` for format.
    pub fn print_report(report: &ParityReport, mode: Mode, baseline_path: &Path) {
        let total = report.doc_count;
        let bound = report.bound_doc_count;
        let gap_count = report.gaps.len();
        let unresolved_count = report.unresolved_bindings.len();
        let unlabeled_count = report.unlabeled_exports.len();
        let waivers = report.waiver_count;
        let aliases = report.alias_count;

        if matches!(mode, Mode::RewriteBaseline) {
            match write_baseline(baseline_path, report) {
                Ok(_) => println!(
                    "✅ Baseline rewritten: {} acknowledged gap(s), {} acknowledged unresolved binding(s), {} acknowledged unlabeled export(s) → {}",
                    gap_count,
                    unresolved_count,
                    unlabeled_count,
                    baseline_path.display()
                ),
                Err(e) => panic!(
                    "Failed to write baseline at {}: {}",
                    baseline_path.display(),
                    e
                ),
            }
            return;
        }

        if report.gaps.is_empty() && report.unresolved_bindings.is_empty() && report.unlabeled_exports.is_empty() {
            println!(
                "✅ API parity: {} documented entries, {} bound on every supported platform, {} waiver(s), {} alias(es).",
                total, bound, waivers, aliases
            );
            return;
        }

        let baseline = if matches!(mode, Mode::Strict) {
            load_baseline(baseline_path)
        } else {
            Baseline::default()
        };

        // Partition gaps and unresolved entries by their relationship to the baseline.
        let (acknowledged_gaps, regressed_gaps, new_gaps) = classify_gaps(&report.gaps, &baseline);
        let (acknowledged_unres, regressed_unres, new_unres) =
            classify_unresolved(&report.unresolved_bindings, &baseline);
        let (acknowledged_unlabeled, new_unlabeled) =
            classify_unlabeled(&report.unlabeled_exports, &baseline);
        let closed_gap_count = baseline.gaps.len() - (acknowledged_gaps.len() + regressed_gaps.len());
        let closed_unres_count =
            baseline.unresolved.len() - (acknowledged_unres.len() + regressed_unres.len());
        let closed_unlabeled_count =
            baseline.unlabeled.len().saturating_sub(acknowledged_unlabeled.len());

        let has_strict_failure = matches!(mode, Mode::Strict)
            && (!new_gaps.is_empty()
                || !regressed_gaps.is_empty()
                || !new_unres.is_empty()
                || !regressed_unres.is_empty()
                || !new_unlabeled.is_empty());

        let header_emoji = match mode {
            Mode::Warn => "⚠️",
            Mode::Strict if has_strict_failure => "❌",
            Mode::Strict => "✅",
            Mode::RewriteBaseline => unreachable!(),
        };
        let header_mode = match mode {
            Mode::Warn => "WARN",
            Mode::Strict => "STRICT",
            Mode::RewriteBaseline => unreachable!(),
        };

        println!(
            "\n{} API parity {}: {} gap(s) over {} documented entries; {} unresolved lsp_doc link(s); {} unlabeled export(s).",
            header_emoji, header_mode, gap_count, total, unresolved_count, unlabeled_count
        );
        println!(
            "    bound: {}  waivers: {}  aliases: {}  baseline: {} gap + {} unresolved + {} unlabeled",
            bound, waivers, aliases, baseline.gaps.len(), baseline.unresolved.len(), baseline.unlabeled.len()
        );

        // ALWAYS print the live state grouped by missing-platform-set, so the
        // human reader sees the full picture regardless of mode.
        print_live_groups(&report.gaps, &report.unresolved_bindings, &report.unlabeled_exports);

        // Strict mode follow-up: list anything that violates the baseline.
        if matches!(mode, Mode::Strict) {
            if !new_gaps.is_empty() {
                println!(
                    "\n  ❌ NEW gaps not acknowledged in baseline ({} doc(s)):",
                    new_gaps.len()
                );
                for g in &new_gaps {
                    println!(
                        "    - {} missing on [{}]",
                        g.doc_path,
                        g.missing.iter().map(|p| p.label()).collect::<Vec<_>>().join(", ")
                    );
                }
            }
            if !regressed_gaps.is_empty() {
                println!(
                    "\n  ❌ REGRESSED gaps (now missing on more platforms than baseline acknowledged) ({} doc(s)):",
                    regressed_gaps.len()
                );
                for (g, baseline_set) in &regressed_gaps {
                    let live: Vec<&str> = g.missing.iter().map(|p| p.label()).collect();
                    let baseline_str: Vec<&str> = baseline_set.iter().map(|p| p.label()).collect();
                    println!(
                        "    - {} now missing on [{}], baseline acknowledged [{}]",
                        g.doc_path,
                        live.join(", "),
                        baseline_str.join(", ")
                    );
                }
            }
            if !new_unres.is_empty() {
                println!(
                    "\n  ❌ NEW unresolved lsp_doc paths ({} binding(s)):",
                    new_unres.len()
                );
                for (path, plats) in &new_unres {
                    let labels: Vec<&str> = plats.iter().map(|p| p.label()).collect();
                    println!("    - {}  [credited: {}]", path, labels.join(", "));
                }
            }
            if !regressed_unres.is_empty() {
                println!(
                    "\n  ❌ REGRESSED unresolved bindings ({} binding(s) now credited on more platforms than baseline):",
                    regressed_unres.len()
                );
                for (path, plats, baseline_set) in &regressed_unres {
                    let live: Vec<&str> = plats.iter().map(|p| p.label()).collect();
                    let baseline_str: Vec<&str> = baseline_set.iter().map(|p| p.label()).collect();
                    println!(
                        "    - {}  live: [{}]  baseline: [{}]",
                        path,
                        live.join(", "),
                        baseline_str.join(", ")
                    );
                }
            }
            if !new_unlabeled.is_empty() {
                println!(
                    "\n  ❌ NEW unlabeled platform exports ({} fn(s)) — add #[lsp_doc(...)] or #[doc(hidden)]:",
                    new_unlabeled.len()
                );
                for exp in &new_unlabeled {
                    println!(
                        "    - {}  fn {}  [{}]",
                        exp.file,
                        exp.fn_name,
                        exp.platform.label()
                    );
                }
            }
            let closed_any = closed_gap_count > 0 || closed_unres_count > 0 || closed_unlabeled_count > 0;
            if closed_any {
                println!(
                    "\n  ✅ Closed since last baseline rewrite: {} gap(s), {} unresolved binding(s), {} unlabeled export(s).",
                    closed_gap_count, closed_unres_count, closed_unlabeled_count
                );
                println!(
                    "     Run `FC_PARITY_REWRITE_BASELINE=1 cargo build --lib` to tighten the ratchet."
                );
            }
        }

        let footer = match mode {
            Mode::Warn => "\n  ℹ️  Warn mode (FC_PARITY_LENIENT=1). Strict mode is the default; set FC_PARITY_LENIENT=1 to suppress.\n",
            Mode::Strict => "\n",
            Mode::RewriteBaseline => unreachable!(),
        };
        println!("{}", footer);

        if has_strict_failure {
            panic!(
                "API parity STRICT failed: {} new gap(s), {} regressed gap(s), {} new unresolved binding(s), {} regressed unresolved binding(s), {} new unlabeled export(s). Fix by: (a) adding #[lsp_doc(\"docs/api/...\")] to credit the binding, (b) adding #[doc(hidden)] if the export is intentionally internal, or (c) extend the baseline (FC_PARITY_REWRITE_BASELINE=1).",
                new_gaps.len(),
                regressed_gaps.len(),
                new_unres.len(),
                regressed_unres.len(),
                new_unlabeled.len()
            );
        }
    }

    /// Print the live gap grouping (regardless of mode) — easier to read
    /// than a flat list and stays useful for human consumption even when
    /// the audit doesn't fail.
    fn print_live_groups(
        gaps: &[Gap],
        unresolved: &[(String, BTreeSet<Platform>)],
        unlabeled: &[UnlabeledExport],
    ) {
        let mut by_missing: BTreeMap<BTreeSet<Platform>, Vec<&Gap>> = BTreeMap::new();
        for g in gaps {
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

        if !unresolved.is_empty() {
            println!(
                "\n  Unresolved lsp_doc paths ({} binding(s)):",
                unresolved.len()
            );
            let mut sorted = unresolved.to_vec();
            sorted.sort_by(|a, b| a.0.cmp(&b.0));
            for (path, plats) in sorted {
                let labels: Vec<&str> = plats.iter().map(|p| p.label()).collect();
                println!("    - {}  [credited: {}]", path, labels.join(", "));
            }
        }

        if !unlabeled.is_empty() {
            println!(
                "\n  Unlabeled platform exports ({} fn(s)) — missing #[lsp_doc] or #[doc(hidden)]:",
                unlabeled.len()
            );
            let mut sorted = unlabeled.to_vec();
            sorted.sort_by(|a, b| a.file.cmp(&b.file).then(a.fn_name.cmp(&b.fn_name)));
            for exp in sorted {
                println!("    - {}  fn {}  [{}]", exp.file, exp.fn_name, exp.platform.label());
            }
        }
    }

    /// Partition live gaps by relation to the baseline:
    ///   - acknowledged: live missing-set is a subset of baseline missing-set
    ///   - regressed:    live missing-set has at least one platform not in baseline
    ///   - new:          doc path is not in baseline at all
    fn classify_gaps<'a>(live: &'a [Gap], baseline: &Baseline) -> GapClassification<'a> {
        let mut acknowledged = Vec::new();
        let mut regressed = Vec::new();
        let mut new_gaps = Vec::new();
        for g in live {
            match baseline.gaps.get(&g.doc_path) {
                None => new_gaps.push(g),
                Some(baseline_set) => {
                    if g.missing.is_subset(baseline_set) {
                        acknowledged.push(g);
                    } else {
                        regressed.push((g, baseline_set.clone()));
                    }
                }
            }
        }
        (acknowledged, regressed, new_gaps)
    }

    fn classify_unresolved<'a>(
        live: &'a [(String, BTreeSet<Platform>)],
        baseline: &Baseline,
    ) -> UnresolvedClassification<'a> {
        let mut acknowledged = Vec::new();
        let mut regressed = Vec::new();
        let mut new_unres = Vec::new();
        for entry in live {
            let (path, plats) = entry;
            match baseline.unresolved.get(path) {
                None => new_unres.push(entry),
                Some(baseline_set) => {
                    if plats.is_subset(baseline_set) {
                        acknowledged.push(entry);
                    } else {
                        regressed.push((path, plats, baseline_set.clone()));
                    }
                }
            }
        }
        (acknowledged, regressed, new_unres)
    }

    /// Partition unlabeled exports into acknowledged (in baseline) vs new (not in baseline).
    /// The key is `"<file>:<fn_name>"`, matching the format written by `write_baseline`.
    fn classify_unlabeled<'a>(
        live: &'a [UnlabeledExport],
        baseline: &Baseline,
    ) -> (Vec<&'a UnlabeledExport>, Vec<&'a UnlabeledExport>) {
        let mut acknowledged = Vec::new();
        let mut new_items = Vec::new();
        for exp in live {
            let key = format!("{}:{}", exp.file, exp.fn_name);
            if baseline.unlabeled.contains(&key) {
                acknowledged.push(exp);
            } else {
                new_items.push(exp);
            }
        }
        (acknowledged, new_items)
    }

    fn load_baseline(file: &Path) -> Baseline {
        let mut b = Baseline::default();
        let text = match fs::read_to_string(file) {
            Ok(t) => t,
            Err(_) => return b,
        };
        for (lineno, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut parts = line.splitn(3, ':');
            let kind = match parts.next() {
                Some(s) => s.trim(),
                None => continue,
            };
            let key = match parts.next() {
                Some(s) => s.trim().to_string(),
                None => {
                    eprintln!(
                        "docs/api/PARITY_BASELINE:{}: malformed line: {}",
                        lineno + 1,
                        line
                    );
                    continue;
                }
            };
            let plats_str = parts.next().map(str::trim).unwrap_or("");
            let mut plats = BTreeSet::new();
            for p in plats_str.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                if let Some(parsed) = parse_platform(p) {
                    plats.insert(parsed);
                }
            }
            match kind {
                "gap" => {
                    b.gaps.insert(key, plats);
                }
                "unresolved" => {
                    b.unresolved.insert(key, plats);
                }
                "unlabeled" => {
                    // Key format: "<workspace-relative-file>:<fn_name>"
                    // plats_str is unused for unlabeled entries (no platform set to track).
                    b.unlabeled.insert(key);
                }
                other => {
                    eprintln!(
                        "docs/api/PARITY_BASELINE:{}: unknown kind '{}'",
                        lineno + 1,
                        other
                    );
                }
            }
        }
        b
    }

    fn write_baseline(file: &Path, report: &ParityReport) -> std::io::Result<()> {
        let mut out = String::new();
        out.push_str("# Auto-generated by build.rs (scripts/parity.rs).\n");
        out.push_str("# Lists currently-acknowledged parity gaps. The audit fails on any\n");
        out.push_str("# gap or unresolved binding that is NOT a subset of the entries here.\n");
        out.push_str("# Trim entries as Phase 3 closes them; regenerate with:\n");
        out.push_str("#   FC_PARITY_REWRITE_BASELINE=1 cargo build --lib\n");
        out.push_str("#\n");
        out.push_str("# Line format:\n");
        out.push_str("#   gap:<canonical_doc>:<comma_separated_missing_platforms>\n");
        out.push_str("#   unresolved:<lsp_doc_path>:<comma_separated_credited_platforms>\n");
        out.push_str("#   unlabeled:<workspace-relative-file>:<fn_name>\n\n");
        let mut gaps: Vec<&Gap> = report.gaps.iter().collect();
        gaps.sort_by(|a, b| a.doc_path.cmp(&b.doc_path));
        for g in &gaps {
            let plats: Vec<&str> = g.missing.iter().map(|p| p.label()).collect();
            out.push_str(&format!("gap:{}:{}\n", g.doc_path, plats.join(",")));
        }
        if !gaps.is_empty() && !report.unresolved_bindings.is_empty() {
            out.push('\n');
        }
        let mut unres: Vec<&(String, BTreeSet<Platform>)> =
            report.unresolved_bindings.iter().collect();
        unres.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, plats) in &unres {
            let plats: Vec<&str> = plats.iter().map(|p| p.label()).collect();
            out.push_str(&format!("unresolved:{}:{}\n", path, plats.join(",")));
        }
        if !report.unlabeled_exports.is_empty() {
            if !gaps.is_empty() || !report.unresolved_bindings.is_empty() {
                out.push('\n');
            }
            let mut unlabeled: Vec<&UnlabeledExport> = report.unlabeled_exports.iter().collect();
            unlabeled.sort_by(|a, b| a.file.cmp(&b.file).then(a.fn_name.cmp(&b.fn_name)));
            for exp in unlabeled {
                out.push_str(&format!("unlabeled:{}:{}\n", exp.file, exp.fn_name));
            }
        }
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(file, out)
    }

    fn parse_platform(s: &str) -> Option<Platform> {
        match s {
            "web" => Some(Platform::Web),
            "python" => Some(Platform::Python),
            "swift" => Some(Platform::Swift),
            "kotlin" => Some(Platform::Kotlin),
            _ => None,
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
            if name == "README.md" || name == "_index.md" || name == "AGENTS.md" {
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

    // -- unlabeled-export scan -------------------------------------------

    /// Walk `src/` and collect all `pub fn` methods inside platform-bound
    /// impl blocks (`#[wasm_bindgen]`, `#[pymethods]`, `#[uniffi::export]`)
    /// that carry neither `#[lsp_doc(...)]` nor `#[doc(hidden)]`.
    ///
    /// These "unlabeled exports" are the blind spot the audit was designed to
    /// catch: the binding exists and is accessible from the platform, but no
    /// `#[lsp_doc]` annotation credits it against any docs/api entry. Fix by
    /// adding `#[lsp_doc("docs/api/...")]` (to credit it) or `#[doc(hidden)]`
    /// (to declare it intentionally internal).
    fn scan_unlabeled_exports(src_root: &Path, workspace_root: &Path) -> Vec<UnlabeledExport> {
        let mut out = Vec::new();
        if !src_root.exists() {
            return out;
        }
        walk_src_unlabeled(src_root, workspace_root, &mut out);
        out.sort_by(|a, b| a.file.cmp(&b.file).then(a.fn_name.cmp(&b.fn_name)));
        out
    }

    fn walk_src_unlabeled(dir: &Path, workspace_root: &Path, out: &mut Vec<UnlabeledExport>) {
        let read = match fs::read_dir(dir) {
            Ok(r) => r,
            Err(_) => return,
        };
        for entry in read.flatten() {
            let p = entry.path();
            if p.is_dir() {
                walk_src_unlabeled(&p, workspace_root, out);
                continue;
            }
            if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            process_file_unlabeled(&p, workspace_root, out);
        }
    }

    fn process_file_unlabeled(
        file: &Path,
        workspace_root: &Path,
        out: &mut Vec<UnlabeledExport>,
    ) {
        let text = match fs::read_to_string(file) {
            Ok(t) => t,
            Err(_) => return,
        };
        let parsed = match syn::parse_file(&text) {
            Ok(p) => p,
            Err(_) => return,
        };
        // Compute the workspace-relative path once for all items in this file.
        let rel_path = match file.strip_prefix(workspace_root) {
            Ok(r) => r.to_string_lossy().replace('\\', "/"),
            Err(_) => file.to_string_lossy().into_owned(),
        };
        for item in &parsed.items {
            collect_unlabeled_item(item, &rel_path, out);
        }
    }

    /// Recursively walk items looking for impl blocks decorated with a platform
    /// binding attribute. For each such impl block, inspect every `pub fn`
    /// method: if it has neither `#[lsp_doc(...)]` nor `#[doc(hidden)]`, record
    /// it as an unlabeled export.
    fn collect_unlabeled_item(item: &Item, rel_path: &str, out: &mut Vec<UnlabeledExport>) {
        match item {
            Item::Impl(it) => collect_unlabeled_impl(it, rel_path, out),
            Item::Mod(it) => {
                if let Some((_, items)) = &it.content {
                    for inner in items {
                        collect_unlabeled_item(inner, rel_path, out);
                    }
                }
            }
            _ => {}
        }
    }

    /// Determine the platform context of an impl block from its attributes.
    /// Only considers *direct* platform binding attributes:
    ///   - `#[wasm_bindgen]`   → Web
    ///   - `#[pymethods]`      → Python
    ///   - `#[uniffi::export]` → Swift + Kotlin
    ///
    /// Intentionally does NOT consider file-level `#![cfg(...)]` gates here:
    /// those gates select which platform COMPILES the file, but they don't
    /// make every un-annotated `pub fn` in the file a platform binding.
    /// Only explicit binding attributes on the impl block itself are meaningful
    /// for the unlabeled-export check.
    fn impl_platform(it: &ItemImpl) -> Option<Platform> {
        for a in &it.attrs {
            if attr_is(a, "wasm_bindgen") {
                return Some(Platform::Web);
            }
            if attr_is(a, "pymethods") {
                return Some(Platform::Python);
            }
            if is_uniffi_export(a) {
                return Some(Platform::Swift); // Swift+Kotlin; we return Swift as the marker.
            }
        }
        None
    }

    fn collect_unlabeled_impl(it: &ItemImpl, rel_path: &str, out: &mut Vec<UnlabeledExport>) {
        let platform = match impl_platform(it) {
            Some(p) => p,
            None => return, // not a platform impl block — skip entirely
        };

        for impl_item in &it.items {
            let method = match impl_item {
                ImplItem::Fn(m) => m,
                _ => continue,
            };

            // Only care about `pub fn` — private helpers are intentionally untracked.
            if !matches!(method.vis, syn::Visibility::Public(_)) {
                continue;
            }

            let has_lsp_doc = lsp_doc_path(&method.attrs).is_some();
            let has_doc_hidden = is_doc_hidden(&method.attrs);

            if !has_lsp_doc && !has_doc_hidden {
                out.push(UnlabeledExport {
                    file: rel_path.to_string(),
                    fn_name: method.sig.ident.to_string(),
                    platform,
                });
            }
        }
    }

    /// Returns `true` when attrs contains `#[doc(hidden)]`.
    fn is_doc_hidden(attrs: &[Attribute]) -> bool {
        for a in attrs {
            if !a.path().is_ident("doc") {
                continue;
            }
            // `#[doc(hidden)]` — Meta::List form with token `hidden`
            if let Meta::List(list) = &a.meta {
                let toks = list.tokens.to_string();
                if toks.trim() == "hidden" {
                    return true;
                }
            }
            // `#[doc = "hidden"]` — NameValue form (rare but valid)
            if let Meta::NameValue(nv) = &a.meta
                && let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                    && s.value().trim() == "hidden" {
                        return true;
                    }
        }
        false
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

    /// Recognize platform identifiers inside a cfg/cfg_attr token stream.
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
    /// of the recognized platform tokens. Pure string transform — no
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
    /// Kotlin only; `_mobile` covers both. Returns `None` for unrecognized
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
            // `type` entries waive a binding-only class name (scripts/binding_checks.rs).
            if line.is_empty() || line.starts_with('#') || line.starts_with("type ") {
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

    // -- shapes ------------------------------------------------------------

    pub mod shapes {
        //! Shape fingerprints.
        //!
        //! For every binding the audit credits to a doc page, this records a
        //! fingerprint of its signature and compares it with the Rust method
        //! that page documents:
        //!
        //! - the arity, with the parameters a default covers;
        //! - each parameter's class, such as `size`, `floats(4)`, `bytes`,
        //!   `renderable`, `matrix`, `text` or a crate type by name;
        //! - whether it returns a `Result` and whether it is `async`;
        //! - the class of what it returns.
        //!
        //! Classes are normalized before they are compared. A binding type
        //! counts under the name its binding exports it as, a parameter that
        //! takes any value (`JsValue`, `Py<PyAny>`) matches every class, and a
        //! type that converts into the Rust parameter's `impl Into<X>` matches
        //! it. Idioms each language forces don't count: JS has one number type
        //! and no fixed-length arrays, uniffi has no fixed-length arrays, and
        //! Python calls block instead of returning futures.
        //!
        //! Known differences live in `docs/api/PARITY_SHAPES`. A difference
        //! missing from it, or one that changed, prints a cargo warning.

        use std::collections::{BTreeMap, BTreeSet};
        use std::fs;
        use std::path::Path;

        use quote::ToTokens;
        use syn::{FnArg, GenericArgument, PathArguments, ReturnType, Type, TypeParamBound};

        use super::{
            Mode, Platform, load_waivers, override_platforms, resolve_canonical, scan_docs,
        };
        use crate::binding_checks::rust;
        use crate::reflect::{self, Build, FnDef, ImplDef, Reflection};

        /// What a parameter or a return value holds, after normalization.
        #[derive(Clone, Debug, PartialEq, Eq)]
        enum Class {
            Unit,
            Bool,
            Int,
            Float,
            Text,
            Bytes,
            /// Floats: `[f32; N]` with a length, `Vec<f32>` without one.
            Floats(Option<usize>),
            Ints(Option<usize>),
            /// Nested float arrays, such as `[[f32; 4]; 4]`.
            Matrix,
            List(Box<Class>),
            Optional(Box<Class>),
            Map,
            /// The type that declares the method.
            This,
            /// A crate type, by the name its binding exports it as.
            Named(String),
            /// `impl Into<X>`, `impl TryInto<X>`, or a generic bound by either.
            Into(String),
            /// `&impl X`, `&dyn X`, or a generic bound by the trait `X`.
            Trait(String),
            /// A value any class fits: `JsValue`, `Py<PyAny>` and the like.
            Any,
            Other(String),
        }

        impl Class {
            fn label(&self) -> String {
                match self {
                    Class::Unit => "unit".into(),
                    Class::Bool => "bool".into(),
                    Class::Int => "int".into(),
                    Class::Float => "float".into(),
                    Class::Text => "text".into(),
                    Class::Bytes => "bytes".into(),
                    Class::Floats(Some(n)) => format!("floats({n})"),
                    Class::Floats(None) => "floats".into(),
                    Class::Ints(Some(n)) => format!("ints({n})"),
                    Class::Ints(None) => "ints".into(),
                    Class::Matrix => "matrix".into(),
                    Class::List(inner) => format!("list({})", inner.label()),
                    Class::Optional(inner) => format!("option({})", inner.label()),
                    Class::Map => "map".into(),
                    Class::This => "self".into(),
                    Class::Named(name) => name.clone(),
                    Class::Into(name) => format!("into {name}"),
                    Class::Trait(name) => format!("impl {name}"),
                    Class::Any => "any".into(),
                    Class::Other(text) => format!("`{text}`"),
                }
            }
        }

        /// A signature reduced to what the comparison reads.
        #[derive(Clone, Debug)]
        struct Shape {
            /// Each parameter's class and the raw type it came from.
            params: Vec<(Class, Option<String>)>,
            /// How many leading parameters a caller must pass.
            required: usize,
            /// Whether more arguments than `params` are accepted, as `**kwargs` allows.
            open: bool,
            throws: bool,
            is_async: bool,
            returns: Class,
            /// The type the return value names under its wrappers.
            returns_raw: Option<String>,
        }

        impl Shape {
            fn render(&self) -> String {
                let params: Vec<String> = self
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, (class, _))| {
                        let optional = if i >= self.required { "?" } else { "" };
                        format!("{}{optional}", class.label())
                    })
                    .collect();
                let mut out = format!("({}) -> {}", params.join(", "), self.returns.label());
                if self.throws {
                    out.push_str(" throws");
                }
                if self.is_async {
                    out.push_str(" async");
                }
                out
            }
        }

        /// A function and the context its types read in.
        struct Site<'model> {
            imp: &'model ImplDef,
            f: &'model FnDef,
            /// `None` for the Rust API.
            build: Option<Build>,
        }

        /// One binding whose shape differs from the Rust method its page documents.
        #[derive(Debug, Clone)]
        pub struct Mismatch {
            pub doc: String,
            pub platform: Platform,
            /// The function as the binding exports it, such as `Pass.add`.
            pub name: String,
            pub binding: String,
            pub rust: String,
        }

        impl Mismatch {
            fn key(&self) -> String {
                format!("{} {} {}", self.doc, self.platform.label(), self.name)
            }

            fn value(&self) -> String {
                format!("{} | rust {}", self.binding, self.rust)
            }
        }

        const BINDINGS: [(Build, Platform, &str); 4] = [
            (Build::Wasm, Platform::Web, "wasm_bindgen"),
            (Build::Python, Platform::Python, "pymethods"),
            (Build::Ios, Platform::Swift, "export"),
            (Build::Android, Platform::Kotlin, "export"),
        ];

        /// What [`audit`] found.
        pub struct Shapes {
            pub mismatches: Vec<Mismatch>,
            /// How many credited bindings had a Rust method to compare with.
            pub compared: usize,
            /// Doc pages with bindings but no Rust method to compare them with.
            pub unmatched: BTreeSet<String>,
        }

        /// Compare every credited binding with its Rust method.
        pub fn audit(root: &Path, model: &Reflection) -> Shapes {
            let docs = scan_docs(&root.join("docs/api"));
            let docs: BTreeSet<&str> = docs.iter().map(String::as_str).collect();
            let aliases = load_waivers(&root.join("docs/api/PARITY")).aliases;

            let mut canonical: BTreeMap<String, Vec<Site>> = BTreeMap::new();
            let mut by_name: BTreeMap<(String, String), Vec<Site>> = BTreeMap::new();
            for imp in model.impls.iter().filter(|imp| is_rust_api(imp)) {
                for f in imp.fns.iter().filter(|f| rust::holds(&f.cfg)) {
                    let site = || Site {
                        imp,
                        f,
                        build: None,
                    };
                    if let Some(doc) = reflect::lsp_doc(&f.attrs) {
                        canonical.entry(doc).or_default().push(site());
                    }
                    if f.is_pub || imp.is_trait_impl {
                        by_name
                            .entry((imp.self_ident.clone(), f.sig.ident.to_string()))
                            .or_default()
                            .push(site());
                    }
                }
            }

            let mut out = Vec::new();
            let mut compared = 0;
            let mut unmatched = BTreeSet::new();
            for (build, platform, attr) in BINDINGS {
                for imp in model.binding_impls(build, attr) {
                    let owner = name_owner(model, imp, build);
                    for f in imp
                        .fns
                        .iter()
                        .filter(|f| f.cfg.holds(build) && !is_hidden(f, build))
                    {
                        let Some(lsp_doc) = reflect::lsp_doc(&f.attrs) else {
                            continue;
                        };
                        if override_platforms(&lsp_doc).is_some_and(|ps| !ps.contains(&platform)) {
                            continue;
                        }
                        let Some(doc) = resolve_canonical(&lsp_doc, &docs, &aliases) else {
                            continue;
                        };
                        let stem = Path::new(&doc)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or_default()
                            .to_string();
                        let rust = match canonical.get(&doc) {
                            Some(sites) => sites,
                            None => match by_name.get(&(owner.clone(), stem)) {
                                Some(sites) => sites,
                                None => {
                                    unmatched.insert(doc);
                                    continue;
                                }
                            },
                        };
                        compared += 1;
                        let site = Site {
                            imp,
                            f,
                            build: Some(build),
                        };
                        let binding = fingerprint(model, &site);
                        let candidates: Vec<Shape> =
                            rust.iter().map(|s| fingerprint(model, s)).collect();
                        if candidates
                            .iter()
                            .any(|r| conforms(model, &binding, r, &site, platform))
                        {
                            continue;
                        }
                        out.push(Mismatch {
                            doc: doc.clone(),
                            platform,
                            name: format!("{owner}.{}", export_fn_name(f, build)),
                            binding: binding.render(),
                            rust: candidates[0].render(),
                        });
                    }
                }
            }
            out.sort_by_key(Mismatch::key);
            out.dedup_by(|a, b| a.key() == b.key() && a.value() == b.value());
            Shapes {
                mismatches: out,
                compared,
                unmatched,
            }
        }

        /// Print what changed against `docs/api/PARITY_SHAPES`, or rewrite it.
        /// Only [`Mode::Strict`] fails the build.
        pub fn report(shapes: &Shapes, mode: Mode, baseline_path: &Path) {
            let live = &shapes.mismatches;
            println!(
                "    compared {} binding(s) with their Rust method; {} page(s) have no Rust method: {}",
                shapes.compared,
                shapes.unmatched.len(),
                shapes
                    .unmatched
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            if matches!(mode, Mode::RewriteBaseline) {
                match write_baseline(baseline_path, live) {
                    Ok(()) => println!(
                        "✅ Shape baseline rewritten: {} acknowledged difference(s) → {}",
                        live.len(),
                        baseline_path.display()
                    ),
                    Err(e) => panic!("Failed to write {}: {e}", baseline_path.display()),
                }
                return;
            }
            let baseline = load_baseline(baseline_path);
            let mut drift = Vec::new();
            for m in live {
                match baseline.get(&m.key()) {
                    Some(known) if *known == m.value() => {}
                    Some(known) => drift.push(format!(
                        "{} changed: {} (baseline: {known})",
                        m.key(),
                        m.value()
                    )),
                    None => drift.push(format!("{} is new: {}", m.key(), m.value())),
                }
            }
            let live_keys: BTreeSet<String> = live.iter().map(Mismatch::key).collect();
            let closed = baseline.keys().filter(|k| !live_keys.contains(*k)).count();
            println!(
                "{} API shapes: {} known difference(s), {} new or changed, {} closed since the baseline.",
                if drift.is_empty() { "✅" } else { "⚠️" },
                live.len() - drift.len(),
                drift.len(),
                closed
            );
            for line in &drift {
                println!("cargo::warning=parity shape: {line}");
            }
            if closed > 0 {
                println!(
                    "    Run `FC_PARITY_REWRITE_BASELINE=1 cargo build --lib` to drop the closed entries."
                );
            }
            if matches!(mode, Mode::Strict) && !drift.is_empty() {
                panic!(
                    "{} binding shape(s) differ from the Rust API outside docs/api/PARITY_SHAPES",
                    drift.len()
                );
            }
        }

        fn load_baseline(file: &Path) -> BTreeMap<String, String> {
            let Ok(text) = fs::read_to_string(file) else {
                return BTreeMap::new();
            };
            text.lines()
                .map(str::trim)
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .filter_map(|line| {
                    let (key, value) = line.split_once(": ")?;
                    Some((key.to_string(), value.to_string()))
                })
                .collect()
        }

        fn write_baseline(file: &Path, live: &[Mismatch]) -> std::io::Result<()> {
            let mut out = String::from(
                "# Auto-generated by build.rs (scripts/parity.rs).\n\
                 # Bindings whose signature shape differs from the Rust method their doc page\n\
                 # documents. The build warns on a difference missing here or changed.\n\
                 # Regenerate with:\n\
                 #   FC_PARITY_REWRITE_BASELINE=1 cargo build --lib\n\
                 #\n\
                 # Line format:\n\
                 #   <doc> <platform> <Type.method>: <binding shape> | rust <rust shape>\n\
                 # A `?` marks a parameter a caller may leave out.\n\n",
            );
            for m in live {
                out.push_str(&format!("{}: {}\n", m.key(), m.value()));
            }
            fs::write(file, out)
        }

        // -- which functions ------------------------------------------------

        /// A block of the Rust API: no binding attribute of its own, and a
        /// `cfg` that holds in a plain Rust build.
        fn is_rust_api(imp: &ImplDef) -> bool {
            let binding = imp.attrs.iter().any(|a| {
                let path = a.path();
                path.is_ident("pymethods")
                    || path.is_ident("wasm_bindgen")
                    || path.segments.last().is_some_and(|s| s.ident == "export")
            });
            !binding && rust::holds(&imp.cfg)
        }

        fn is_hidden(f: &FnDef, build: Build) -> bool {
            reflect::metas(&f.attrs, build)
                .iter()
                .any(|m| reflect::meta_ident(m) == "doc" && reflect::meta_args(m) == "hidden")
        }

        /// The name the binding exports the impl's type as.
        fn name_owner(model: &Reflection, imp: &ImplDef, build: Build) -> String {
            export_name(model, &imp.self_ident, build)
        }

        fn export_name(model: &Reflection, ident: &str, build: Build) -> String {
            model
                .type_named(ident, build)
                .and_then(|t| reflect::exported_type_name(&t.attrs, &t.ident, build))
                .unwrap_or_else(|| ident.to_string())
        }

        fn export_fn_name(f: &FnDef, build: Build) -> String {
            let key = if build == Build::Wasm {
                "js_name"
            } else {
                "name"
            };
            reflect::metas(&f.attrs, build)
                .iter()
                .filter(|m| {
                    matches!(
                        reflect::meta_ident(m).as_str(),
                        "wasm_bindgen" | "pyo3" | "method" | "constructor"
                    )
                })
                .find_map(|m| reflect::arg_value(&reflect::meta_args(m), key))
                .unwrap_or_else(|| f.sig.ident.to_string())
        }

        // -- fingerprints ---------------------------------------------------

        fn fingerprint(model: &Reflection, site: &Site) -> Shape {
            let sig = &site.f.sig;
            let inputs: Vec<&syn::PatType> = sig
                .inputs
                .iter()
                .filter_map(|arg| match arg {
                    FnArg::Typed(t) => Some(t),
                    FnArg::Receiver(_) => None,
                })
                .filter(|t| !is_receiver(t))
                .collect();
            let params: Vec<(Class, Option<String>)> = inputs
                .iter()
                .map(|t| (classify(model, &t.ty, site), read_core(&t.ty)))
                .collect();
            let names: Vec<String> = inputs.iter().map(|t| read_name(&t.pat)).collect();
            let (required, open) = match site.build {
                None => (params.len(), false),
                Some(Build::Python) => read_signature(site.f, &names),
                Some(Build::Wasm) => {
                    let trailing = inputs.iter().rev().take_while(|t| is_option(&t.ty)).count();
                    (params.len() - trailing, false)
                }
                Some(build) => (params.len() - count_defaults(site.f, build, &names), false),
            };
            // A JS promise resolves to the value and rejects with the error,
            // so a function that returns one reads as an async call that throws.
            let promise = matches!(&sig.output,
                ReturnType::Type(_, ty) if read_ident(ty).as_deref() == Some("Promise"));
            let (returns, returns_raw, throws) = match &sig.output {
                _ if promise => (Class::Any, None, true),
                ReturnType::Default => (Class::Unit, None, false),
                ReturnType::Type(_, ty) => match unwrap_result(ty) {
                    Some(ok) => (classify(model, ok, site), read_core(ok), true),
                    None => (classify(model, ty, site), read_core(ty), false),
                },
            };
            let params = params
                .into_iter()
                .map(|(class, raw)| match class {
                    Class::Optional(inner) => (*inner, raw),
                    other => (other, raw),
                })
                .collect();
            Shape {
                params,
                required,
                open,
                throws,
                is_async: sig.asyncness.is_some() || promise,
                returns,
                returns_raw,
            }
        }

        /// `slf: PyRef<Self>`, `self: Arc<Self>` and `py: Python` stand in for the receiver.
        fn is_receiver(arg: &syn::PatType) -> bool {
            let name = read_name(&arg.pat);
            let ident = read_ident(&arg.ty).unwrap_or_default();
            name == "self" || name == "slf" || ident == "Python"
        }

        fn read_name(pat: &syn::Pat) -> String {
            match pat {
                syn::Pat::Ident(p) => p.ident.to_string(),
                _ => "_".to_string(),
            }
        }

        /// Required parameter count from `#[pyo3(signature = ...)]`, and
        /// whether it takes `**kwargs`.
        fn read_signature(f: &FnDef, names: &[String]) -> (usize, bool) {
            let signature = reflect::metas(&f.attrs, Build::Python)
                .iter()
                .filter(|m| reflect::meta_ident(m) == "pyo3")
                .find_map(|m| reflect::arg_value(&reflect::meta_args(m), "signature"));
            let Some(signature) = signature else {
                return (names.len(), false);
            };
            let parts = reflect::split_top_level(&signature);
            let open = parts.iter().any(|p| p.starts_with("**"));
            let defaulted: BTreeSet<String> = parts
                .iter()
                .filter_map(|p| p.split_once('=').map(|(name, _)| name.trim().to_string()))
                .collect();
            let required = names.iter().take_while(|n| !defaulted.contains(*n)).count();
            (required, open)
        }

        /// Trailing parameters a uniffi `default(...)` argument covers.
        fn count_defaults(f: &FnDef, build: Build, names: &[String]) -> usize {
            let named: BTreeSet<String> = reflect::metas(&f.attrs, build)
                .iter()
                .filter(|m| matches!(reflect::meta_ident(m).as_str(), "method" | "constructor"))
                .flat_map(|m| reflect::split_top_level(&reflect::meta_args(m)))
                .filter_map(|part| {
                    let inner = part.strip_prefix("default")?.trim();
                    Some(inner.strip_prefix('(')?.strip_suffix(')')?.to_string())
                })
                .flat_map(|inner| reflect::split_top_level(&inner))
                .filter_map(|pair| Some(pair.split_once('=')?.0.trim().to_string()))
                .collect();
            names
                .iter()
                .rev()
                .take_while(|n| named.contains(*n))
                .count()
        }

        fn is_option(ty: &Type) -> bool {
            read_ident(ty).as_deref() == Some("Option")
        }

        /// `T` of `Result<T, E>` or `PyResult<T>`.
        fn unwrap_result(ty: &Type) -> Option<&Type> {
            let ident = read_ident(ty)?;
            matches!(ident.as_str(), "Result" | "PyResult")
                .then(|| read_arg(ty))
                .flatten()
        }

        fn read_segment(ty: &Type) -> Option<&syn::PathSegment> {
            match reflect::strip_type(ty) {
                Type::Path(p) => p.path.segments.last(),
                _ => None,
            }
        }

        /// The type a value names under its `&`, `Option`, `Arc`, `Py` and `Vec` wrappers.
        fn read_core(ty: &Type) -> Option<String> {
            match reflect::strip_type(ty) {
                Type::Slice(slice) => return read_core(&slice.elem),
                Type::Array(array) => return read_core(&array.elem),
                _ => {}
            }
            let ident = read_ident(ty)?;
            match ident.as_str() {
                "Option" | "Arc" | "Rc" | "Box" | "Py" | "PyRef" | "PyRefMut" | "Bound" | "Vec" => {
                    read_arg(ty).and_then(read_core)
                }
                _ => Some(ident),
            }
        }

        fn read_ident(ty: &Type) -> Option<String> {
            read_segment(ty).map(|s| s.ident.to_string())
        }

        fn read_args(ty: &Type) -> Vec<&Type> {
            let Some(segment) = read_segment(ty) else {
                return Vec::new();
            };
            let PathArguments::AngleBracketed(args) = &segment.arguments else {
                return Vec::new();
            };
            args.args
                .iter()
                .filter_map(|a| match a {
                    GenericArgument::Type(t) => Some(t),
                    _ => None,
                })
                .collect()
        }

        fn read_arg(ty: &Type) -> Option<&Type> {
            read_args(ty).into_iter().next()
        }

        /// The class of `ty` as `site` declares it.
        fn classify(model: &Reflection, ty: &Type, site: &Site) -> Class {
            match reflect::strip_type(ty) {
                Type::Array(array) => {
                    classify_array(model, &array.elem, site, read_len(&array.len))
                }
                Type::Slice(slice) => classify_array(model, &slice.elem, site, None),
                Type::Tuple(tuple) if tuple.elems.is_empty() => Class::Unit,
                Type::Tuple(tuple) => {
                    let classes: Vec<Class> = tuple
                        .elems
                        .iter()
                        .map(|t| classify(model, t, site))
                        .collect();
                    let n = Some(classes.len());
                    if classes.iter().all(|c| *c == Class::Float) {
                        Class::Floats(n)
                    } else if classes.iter().all(|c| *c == Class::Int) {
                        Class::Ints(n)
                    } else {
                        Class::Other(ty.to_token_stream().to_string())
                    }
                }
                Type::ImplTrait(it) => classify_bounds(model, it.bounds.iter(), site),
                Type::TraitObject(to) => classify_bounds(model, to.bounds.iter(), site),
                Type::Path(_) => classify_path(model, ty, site),
                other => Class::Other(other.to_token_stream().to_string()),
            }
        }

        fn read_len(len: &syn::Expr) -> Option<usize> {
            match len {
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Int(n),
                    ..
                }) => n.base10_parse().ok(),
                _ => None,
            }
        }

        fn classify_array(
            model: &Reflection,
            elem: &Type,
            site: &Site,
            len: Option<usize>,
        ) -> Class {
            match classify(model, elem, site) {
                Class::Float => Class::Floats(len),
                Class::Int if read_ident(elem).as_deref() == Some("u8") => Class::Bytes,
                Class::Int => Class::Ints(len),
                Class::Floats(_) => Class::Matrix,
                other => Class::List(Box::new(other)),
            }
        }

        fn classify_bounds<'bounds>(
            model: &Reflection,
            mut bounds: impl Iterator<Item = &'bounds TypeParamBound>,
            site: &Site,
        ) -> Class {
            let trait_path = bounds.find_map(|b| match b {
                TypeParamBound::Trait(t) => Some(&t.path),
                _ => None,
            });
            let Some(path) = trait_path else {
                return Class::Any;
            };
            let Some(segment) = path.segments.last() else {
                return Class::Any;
            };
            let name = segment.ident.to_string();
            let arg = match &segment.arguments {
                PathArguments::AngleBracketed(args) => args.args.iter().find_map(|a| match a {
                    GenericArgument::Type(t) => Some(t),
                    GenericArgument::AssocType(assoc) => Some(&assoc.ty),
                    _ => None,
                }),
                _ => None,
            };
            match (name.as_str(), arg) {
                ("Into" | "TryInto" | "From" | "TryFrom", Some(target)) => {
                    match classify(model, target, site) {
                        Class::Named(name) => Class::Into(name),
                        other => other,
                    }
                }
                ("AsRef", Some(target)) => classify(model, target, site),
                ("IntoIterator", Some(item)) => match classify(model, item, site) {
                    Class::Float => Class::Floats(None),
                    Class::Int if read_ident(item).as_deref() == Some("u8") => Class::Bytes,
                    Class::Int => Class::Ints(None),
                    other => Class::List(Box::new(other)),
                },
                _ => Class::Trait(name),
            }
        }

        fn classify_path(model: &Reflection, ty: &Type, site: &Site) -> Class {
            let Some(ident) = read_ident(ty) else {
                return Class::Other(ty.to_token_stream().to_string());
            };
            let inner = || read_arg(ty).map(|t| classify(model, t, site));
            match ident.as_str() {
                "bool" => Class::Bool,
                "f32" | "f64" => Class::Float,
                "u8" | "u16" | "u32" | "u64" | "usize" | "i8" | "i16" | "i32" | "i64" | "isize" => {
                    Class::Int
                }
                "String" | "str" | "Cow" | "Path" | "PathBuf" | "PyString" => Class::Text,
                "Uint8Array" | "Uint8ClampedArray" | "PyBytes" | "PyByteArray" | "Bytes" => {
                    Class::Bytes
                }
                // A uint8 ndarray is Python's container for image bytes.
                "PyArray1" | "PyArray2" | "PyArray3" | "PyArrayDyn"
                    if read_arg(ty).and_then(read_ident).as_deref() == Some("u8") =>
                {
                    Class::Bytes
                }
                "Float32Array" | "Float64Array" => Class::Floats(None),
                "Uint32Array" | "Int32Array" => Class::Ints(None),
                "JsValue" | "PyAny" | "PyObject" | "Object" | "PyTuple" => Class::Any,
                "PyList" | "Array" => Class::List(Box::new(Class::Any)),
                "PyDict" | "HashMap" | "BTreeMap" => Class::Map,
                "Self" => Class::This,
                "Vec" | "VecDeque" | "Box" if inner().is_some() => {
                    let elem = read_arg(ty).map_or(Class::Any, |t| classify(model, t, site));
                    let len = None;
                    match elem {
                        Class::Float => Class::Floats(len),
                        Class::Int
                            if read_arg(ty).and_then(read_ident).as_deref() == Some("u8") =>
                        {
                            Class::Bytes
                        }
                        Class::Int => Class::Ints(len),
                        Class::Floats(_) if ident != "Box" => Class::Matrix,
                        other if ident == "Box" => other,
                        other => Class::List(Box::new(other)),
                    }
                }
                "Option" => Class::Optional(Box::new(inner().unwrap_or(Class::Any))),
                "Arc" | "Rc" | "Py" | "PyRef" | "PyRefMut" | "Bound" | "RwLock" | "Mutex" => {
                    inner().unwrap_or(Class::Any)
                }
                "Result" | "PyResult" => inner().unwrap_or(Class::Unit),
                _ => classify_named(model, &ident, site),
            }
        }

        /// A crate type, a generic parameter, or a handle that stands in for a trait.
        fn classify_named(model: &Reflection, ident: &str, site: &Site) -> Class {
            if let Some(bound) = find_bound(model, site, ident) {
                return bound;
            }
            let name = match site.build {
                Some(build) => export_name(model, ident, build),
                None => ident.to_string(),
            };
            let owner = match site.build {
                Some(build) => name_owner(model, site.imp, build),
                None => site.imp.self_ident.clone(),
            };
            if name == owner {
                return Class::This;
            }
            for (prefix, suffix) in [("", "Handle"), ("Py", "")] {
                if let Some(stand_in) = name
                    .strip_prefix(prefix)
                    .and_then(|n| n.strip_suffix(suffix))
                    .filter(|n| !n.is_empty() && *n != name)
                    && matches!(stand_in, "Renderable" | "Target" | "SceneObject")
                {
                    return Class::Trait(stand_in.to_string());
                }
            }
            Class::Named(name)
        }

        /// The class a generic parameter of the function or its impl stands for.
        fn find_bound(model: &Reflection, site: &Site, ident: &str) -> Option<Class> {
            let params = site
                .f
                .sig
                .generics
                .params
                .iter()
                .chain(site.imp.generics.params.iter());
            let mut bounds: Vec<TypeParamBound> = Vec::new();
            for param in params {
                if let syn::GenericParam::Type(tp) = param
                    && tp.ident == ident
                {
                    bounds.extend(tp.bounds.iter().cloned());
                }
            }
            let clauses = site
                .f
                .sig
                .generics
                .where_clause
                .iter()
                .chain(site.imp.generics.where_clause.iter());
            for clause in clauses {
                for predicate in &clause.predicates {
                    if let syn::WherePredicate::Type(pt) = predicate
                        && read_ident(&pt.bounded_ty).as_deref() == Some(ident)
                    {
                        bounds.extend(pt.bounds.iter().cloned());
                    }
                }
            }
            if bounds.is_empty() {
                return None;
            }
            Some(classify_bounds(model, bounds.iter(), site))
        }

        // -- comparison -----------------------------------------------------

        /// What a comparison reads besides the two classes.
        struct Check<'model> {
            model: &'model Reflection,
            site: &'model Site<'model>,
            platform: Platform,
        }

        /// How deep a comparison follows conversions and enum variants.
        const DEPTH: usize = 3;

        fn conforms(
            model: &Reflection,
            binding: &Shape,
            rust: &Shape,
            site: &Site,
            platform: Platform,
        ) -> bool {
            let check = Check {
                model,
                site,
                platform,
            };
            let total = rust.params.len();
            let arity =
                binding.required <= total && (binding.open || total <= binding.params.len());
            if !arity {
                return false;
            }
            let pairs: Vec<_> = binding.params.iter().zip(&rust.params).collect();
            let params = pairs
                .iter()
                .all(|((b, raw), (r, _))| check.fits(b, raw.as_deref(), r, DEPTH));
            if !params {
                return false;
            }
            let returns = check.returns(
                &binding.returns,
                binding.returns_raw.as_deref(),
                &rust.returns,
            );
            // A binding that reads a looser value than Rust takes may reject it.
            let loosened = pairs.iter().any(|((b, _), (r, _))| b != r);
            let throws = binding.throws == rust.throws || (binding.throws && loosened);
            let is_async = platform == Platform::Python || binding.is_async == rust.is_async;
            returns && throws && is_async
        }

        impl Check<'_> {
            fn build(&self) -> Option<Build> {
                self.site.build
            }

            /// Whether a binding argument of class `b` takes the place of a Rust parameter of class `r`.
            fn fits(&self, b: &Class, raw: Option<&str>, r: &Class, depth: usize) -> bool {
                if b == r || *b == Class::Any || self.loosely(b, r) {
                    return true;
                }
                if depth == 0 {
                    return false;
                }
                let fits = match (b, r) {
                    (Class::Optional(b), Class::Optional(r)) | (Class::List(b), Class::List(r)) => {
                        self.fits(b, raw, r, depth - 1)
                    }
                    (_, Class::Trait(name)) => self.implements(b, raw, name, depth - 1),
                    (_, Class::Into(target) | Class::Named(target)) => {
                        self.converts(b, raw, target, depth - 1)
                    }
                    _ => false,
                };
                fits || self
                    .variants(raw)
                    .iter()
                    .any(|v| self.fits(v, None, r, depth - 1))
            }

            /// Whether a binding return value of class `b` stands for a Rust return value of class `r`.
            fn returns(&self, b: &Class, raw: Option<&str>, r: &Class) -> bool {
                if b == r || *b == Class::Any || self.loosely(b, r) {
                    return true;
                }
                match (b, r) {
                    (Class::Optional(b), Class::Optional(r)) | (Class::List(b), Class::List(r)) => {
                        self.returns(b, raw, r)
                    }
                    (Class::Named(name), Class::Into(target) | Class::Named(target)) => {
                        let Some(build) = self.build() else {
                            return false;
                        };
                        let binding = raw.unwrap_or(name);
                        let from_rust = self
                            .model
                            .conversions_into(binding, "From", build)
                            .iter()
                            .any(|(_, source)| read_core(source).as_deref() == Some(target));
                        from_rust || self.converts(b, raw, target, DEPTH)
                    }
                    _ => false,
                }
            }

            /// Classes that differ only by an idiom of the binding's language.
            fn loosely(&self, b: &Class, r: &Class) -> bool {
                let web = self.platform == Platform::Web;
                let no_fixed_arrays = matches!(
                    self.platform,
                    Platform::Web | Platform::Swift | Platform::Kotlin
                );
                match (b, r) {
                    (Class::Int, Class::Float) | (Class::Float, Class::Int) => web,
                    (Class::Floats(_), Class::Floats(_)) | (Class::Ints(_), Class::Ints(_)) => {
                        no_fixed_arrays
                    }
                    (Class::Floats(_), Class::Ints(_)) | (Class::Ints(_), Class::Floats(_)) => web,
                    _ => false,
                }
            }

            /// Whether a binding value meets a Rust `impl Trait` parameter: it names a type that
            /// implements the trait, or a list of them where lists implement it, or has the
            /// class of one of its implementors.
            fn implements(&self, b: &Class, raw: Option<&str>, name: &str, depth: usize) -> bool {
                let build = self.build().unwrap_or(Build::Python);
                let impls: Vec<&ImplDef> = self.model.trait_impls(name, build).collect();
                let names = |ident: Option<&str>| {
                    ident.is_some_and(|ident| {
                        impls.iter().any(|imp| {
                            imp.self_ident == ident
                                || export_name(self.model, &imp.self_ident, build) == ident
                        })
                    })
                };
                if let Class::Named(named) = b
                    && (names(raw) || names(Some(named)))
                {
                    return true;
                }
                if let Class::List(item) = b {
                    let lists = impls.iter().any(|imp| {
                        matches!(read_ident(&imp.self_ty).as_deref(), Some("Vec"))
                            || matches!(
                                reflect::strip_type(&imp.self_ty),
                                Type::Slice(_) | Type::Array(_)
                            )
                    });
                    return lists && self.fits(item, None, &Class::Trait(name.to_string()), depth);
                }
                let rust_site = Site {
                    imp: self.site.imp,
                    f: self.site.f,
                    build: None,
                };
                impls.iter().any(|imp| {
                    let class = classify(self.model, &imp.self_ty, &rust_site);
                    !matches!(class, Class::Trait(_) | Class::Named(_) | Class::This)
                        && self.fits(b, raw, &class, depth)
                })
            }

            /// Whether a `From` or `TryFrom` impl turns the binding's value into `target`.
            fn converts(&self, b: &Class, raw: Option<&str>, target: &str, depth: usize) -> bool {
                if *b == Class::Named(target.to_string()) {
                    return true;
                }
                let Some(build) = self.build() else {
                    return false;
                };
                ["From", "TryFrom"].iter().any(|conversion| {
                    self.model
                        .conversions_into(target, conversion, build)
                        .iter()
                        .any(|(_, source)| {
                            let source_raw = read_core(source);
                            if raw.is_some() && source_raw.as_deref() == raw {
                                return true;
                            }
                            let class = classify(self.model, source, self.site);
                            class == *b
                                || (!matches!(class, Class::Named(_) | Class::Into(_))
                                    && self.fits(b, raw, &class, depth))
                        })
                })
            }

            /// The classes of the one-field variants of the binding enum `raw`,
            /// which a binding reads as a union, such as a `derive(FromPyObject)` enum.
            fn variants(&self, raw: Option<&str>) -> Vec<Class> {
                let (Some(raw), Some(build)) = (raw, self.build()) else {
                    return Vec::new();
                };
                let Some(ty) = self.model.type_named(raw, build) else {
                    return Vec::new();
                };
                ty.variants
                    .iter()
                    .filter(|v| v.cfg.holds(build))
                    .filter_map(|v| match v.fields.as_slice() {
                        [only] => Some(classify(self.model, only, self.site)),
                        _ => None,
                    })
                    .collect()
            }
        }
    }
}
