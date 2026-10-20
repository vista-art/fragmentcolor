use cfg_aliases::cfg_aliases;

fn main() {
    configure_aliases();
    let checkout = is_repo_checkout();
    set_build_env(checkout);
    embedded_shaders::generate();
    if checkout {
        generate_docs();
        // Generate language-specific READMEs from templates + Rust examples
        println!("cargo::rerun-if-changed=README.md");
        println!("cargo::rerun-if-changed=platforms/web/README.tpl.md");
        println!("cargo::rerun-if-changed=platforms/python/README.tpl.md");
        readme::generate_readmes();
    }
}

/// True in a FragmentColor repository checkout.
///
/// Only a checkout runs the docs pipeline, which writes into the source tree.
/// Packaged sources (crates.io, docs.rs, a vendored tree) can be read-only, so
/// their builds skip the pipeline and write nothing outside OUT_DIR. A worktree
/// keeps `.git` as a file and a clone as a directory, and both count as checkouts.
fn is_repo_checkout() -> bool {
    println!("cargo::rerun-if-env-changed=DOCS_RS");
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    std::env::var_os("DOCS_RS").is_none() && manifest_dir.join(".git").exists()
}

/// Configures custom cfg aliases for conditional compilation
fn configure_aliases() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        ios: { target_os = "ios" },
        macos: { target_os = "macos" },
        apple: { any(target_os = "macos", target_os = "ios") },
        android: { target_os = "android" },
        mobile: { any(android, ios) },
        desktop: { not(any(wasm, mobile)) },
        python: { all(desktop, feature="python") },
        dev: { all(desktop, feature="uniffi/cli") },
    }
    println!("cargo::rustc-check-cfg=cfg(wasm)");
    println!("cargo::rustc-check-cfg=cfg(ios)");
    println!("cargo::rustc-check-cfg=cfg(macos)");
    println!("cargo::rustc-check-cfg=cfg(apple)");
    println!("cargo::rustc-check-cfg=cfg(android)");
    println!("cargo::rustc-check-cfg=cfg(mobile)");
    println!("cargo::rustc-check-cfg=cfg(desktop)");
    println!("cargo::rustc-check-cfg=cfg(dev)");
}

/// Capture build metadata for runtime diagnostics (e.g., GPU error hook)
fn set_build_env(checkout: bool) {
    // Git commit hash (short). A packaged crate reads the commit `cargo package`
    // recorded, so a vendored copy never reports the hash of the repo around it.
    let git_hash = if checkout {
        std::process::Command::new("git")
            .args(["rev-parse", "--short=12", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    } else {
        packaged_git_hash()
    }
    .unwrap_or_else(|| "unknown".to_string());
    println!("cargo::rustc-env=FC_GIT_HASH={}", git_hash);

    // Build timestamp (unix epoch seconds)
    let build_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    println!("cargo::rustc-env=FC_BUILD_TIME={}", build_time);

    // Rerun when HEAD changes
    if checkout {
        println!("cargo::rerun-if-changed=.git/HEAD");
    }
}

/// Short commit hash from the `.cargo_vcs_info.json` that `cargo package` writes.
fn packaged_git_hash() -> Option<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".cargo_vcs_info.json");
    let info = std::fs::read_to_string(path).ok()?;
    let (_, rest) = info.split_once("\"sha1\"")?;
    let sha = rest.split('"').nth(1)?;
    Some(sha.chars().take(12).collect())
}

/// This function validates all the API documentation and breaks
/// the build if ANY public item from code is missing documentation.
///
/// The canonical documentation is located in `docs/api`.
///
/// It is used in lieu of doc-comments in code
/// to appear in the hover tooltips in IDEs.
///
/// ## What this script does:
///
/// - Converts `docs/api` from Rust to Javascript and Python,
///   and generate all the examples in the `platforms/`` folder.
///
/// - The CI will run a healthcheck in the generated examples and
///   only allow a build if all examples in all languages work.
///
/// - Finally, the generated examples and the api/docs are used
///   to generate the website contents in docs/website from the
///   Rust, JS and Python code.
///
/// This process ensures that anything published in the website actually
/// works in practice, and that the documentation is always up to date.
fn generate_docs() {
    if let Err(message) = enforce_no_panic_policy() {
        panic!("{}", message);
    }

    println!("\n🗺️ Generating API map...");
    let catalog = codegen::build_catalog();
    let api_map = codegen::scan_api(&catalog);
    codegen::export_api_map(&api_map);
    println!("✅ API map successfully generated!\n");

    println!("🔎 Validating documentation...");
    validation::validate_docs(&catalog, &api_map);
    println!("✅ Docs validated!\n");

    println!("🧭 Auditing API parity across platforms...");
    let workspace = meta::workspace_root();
    let parity_report = parity::audit(&workspace);
    let baseline_path = workspace.join("docs/api/PARITY_BASELINE");
    // Mode selection:
    //   FC_PARITY_REWRITE_BASELINE=1 → snapshot the current state into PARITY_BASELINE
    //                                  and exit cleanly. Use after a Phase 3 batch
    //                                  closes gaps so the ratchet tightens.
    //   FC_PARITY_LENIENT=1          → Warn (print only). Local opt-out.
    //   default + baseline missing   → bootstrap: snapshot once, continue. Lets a
    //                                  fresh checkout build before any baseline file
    //                                  exists (e.g. first time this lands on `main`).
    //   default + baseline present   → Strict (fail on drift outside baseline).
    let mode = if std::env::var("FC_PARITY_REWRITE_BASELINE").is_ok() {
        parity::Mode::RewriteBaseline
    } else if std::env::var("FC_PARITY_LENIENT").is_ok() {
        parity::Mode::Warn
    } else if !baseline_path.exists() {
        println!("==> PARITY_BASELINE missing; bootstrapping it from the current audit state.");
        parity::Mode::RewriteBaseline
    } else {
        parity::Mode::Strict
    };
    println!("cargo::rerun-if-env-changed=FC_PARITY_LENIENT");
    println!("cargo::rerun-if-env-changed=FC_PARITY_REWRITE_BASELINE");
    println!("cargo::rerun-if-changed=docs/api/PARITY");
    println!("cargo::rerun-if-changed=docs/api/PARITY_BASELINE");
    // Rerun the audit (and the rest of generate_docs) whenever src/ or
    // docs/api/ change. Without these directives cargo would only watch the
    // explicit paths we declare elsewhere — meaning a new uniffi binding in
    // src/ would not retrigger the audit, and drift would slip through until
    // an unrelated file in the rerun list changed.
    println!("cargo::rerun-if-changed=src");
    println!("cargo::rerun-if-changed=docs/api");
    parity::print_report(&parity_report, mode, &baseline_path);
    println!(
        "==> docs/api/PARITY drives intentional divergence; PARITY_BASELINE tracks Phase-3 backlog.\n"
    );

    println!("🌎 Exporting website (examples + pages)...");
    convert::install(&catalog.reflection);
    swift::install(&catalog.reflection);
    kotlin::install(&catalog.reflection);

    println!("==> website::export_examples_and_pages()");
    let outcome = website::export_examples_and_pages(&catalog, &api_map);

    println!("==> website::cleanup_site()");
    website::cleanup_site(&outcome.expected);

    println!("==> website::cleanup_examples()");
    website::cleanup_examples(&outcome);

    println!("==> website::write_healthcheck_aggregators()");
    website::write_healthcheck_aggregators(
        &outcome.ex_js,
        &outcome.ex_py,
        &outcome.ex_swift,
        &outcome.ex_kotlin,
    );

    println!("✅ Website export done!\n");

    println!("🐍 Generating Python type stubs...");
    python_stubs::generate(&workspace, &catalog.reflection);
    println!("✅ Python stubs written.\n");

    println!("📚 Building tutorials manifest...");
    tutorials::build();
    println!("✅ Tutorials manifest written.\n");
}

include!("scripts/no_panics.rs");
include!("scripts/reflect.rs");
include!("scripts/codegen.rs");
include!("scripts/convert.rs");
include!("scripts/example.rs");
include!("scripts/source_text.rs");
include!("scripts/swift.rs");
include!("scripts/kotlin.rs");
include!("scripts/docs.rs");
include!("scripts/validation.rs");
include!("scripts/parity.rs");
include!("scripts/website.rs");
include!("scripts/tutorials.rs");
include!("scripts/meta.rs");
include!("scripts/readme.rs");
include!("scripts/embedded_shaders.rs");
include!("scripts/python_stubs.rs");
