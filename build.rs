use cfg_aliases::cfg_aliases;

fn main() {
    configure_aliases();
    set_build_env();
    embedded_shaders::generate();
    generate_docs();
    // Generate language-specific READMEs from templates + Rust examples
    println!("cargo::rerun-if-changed=README.md");
    println!("cargo::rerun-if-changed=README_JS.tpl.md");
    println!("cargo::rerun-if-changed=README_PY.tpl.md");
    readme::generate_readmes();
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
fn set_build_env() {
    // Git commit hash (short)
    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo::rustc-env=FC_GIT_HASH={}", git_hash);

    // Build timestamp (unix epoch seconds)
    let build_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());
    println!("cargo::rustc-env=FC_BUILD_TIME={}", build_time);

    // Rerun when HEAD changes
    println!("cargo::rerun-if-changed=.git/HEAD");
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
    let api_map = codegen::scan_api();
    codegen::export_api_map(&api_map);
    codegen::export_api_objects();
    println!("✅ API map successfully generated!\n");

    println!("🔎 Validating documentation...");
    validation::validate_docs(&api_map);
    println!("✅ Docs validated!\n");

    println!("🧭 Auditing API parity across platforms...");
    let parity_report = parity::audit(&meta::workspace_root());
    let fail_on_gaps = std::env::var("FC_PARITY_STRICT").is_ok();
    parity::print_report(&parity_report, fail_on_gaps);
    println!("==> docs/api/PARITY drives intentional divergence (web-only Shader.fetch, etc.).\n");

    println!("🌎 Exporting website (examples + pages)...");

    println!("==> website::export_examples_and_pages()");
    let outcome = website::export_examples_and_pages(&api_map);

    println!("==> website::cleanup_site()");
    website::cleanup_site(&outcome.expected);

    println!("==> website::write_healthcheck_aggregators()");
    website::write_healthcheck_aggregators(
        &outcome.ex_js,
        &outcome.ex_py,
        &outcome.ex_swift,
        &outcome.ex_kotlin,
    );

    println!("✅ Website export done!\n");

    println!("📚 Building tutorials manifest...");
    tutorials::build();
    println!("✅ Tutorials manifest written.\n");
}

include!("scripts/no_panics.rs");
include!("scripts/codegen.rs");
include!("scripts/convert.rs");
include!("scripts/swift.rs");
include!("scripts/kotlin.rs");
include!("scripts/validation.rs");
include!("scripts/parity.rs");
include!("scripts/website.rs");
include!("scripts/tutorials.rs");
include!("scripts/meta.rs");
include!("scripts/readme.rs");
include!("scripts/embedded_shaders.rs");
