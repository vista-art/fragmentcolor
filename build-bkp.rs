use cfg_aliases::cfg_aliases;

fn main() {
    configure_aliases();
    generate_docs();
}

/// Configures custom cfg aliases for conditional compilation
fn configure_aliases() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        ios: { target_os = "ios" },
        android: { target_os = "android" },
        mobile: { any(android, ios) },
        desktop: { not(any(wasm, mobile)) },
        python: { all(desktop, feature="python") },
        dev: { all(desktop, feature="uniffi/cli") },
    }
    println!("cargo::rustc-check-cfg=cfg(wasm)");
    println!("cargo::rustc-check-cfg=cfg(ios)");
    println!("cargo::rustc-check-cfg=cfg(android)");
    println!("cargo::rustc-check-cfg=cfg(mobile)");
    println!("cargo::rustc-check-cfg=cfg(desktop)");
    println!("cargo::rustc-check-cfg=cfg(dev)");
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
    println!("\n🗺️ Generating API map...");
    let api_map = codegen::scan_api();
    codegen::export_api_map(&api_map);
    println!("✅ API map successfully generated!\n");

    println!("🔎 Validating documentation...");
    validation::validate_docs(&api_map);
    println!("✅ Docs validated!\n");

    println!("🌎 Exporting website (examples + pages)...");
    println!("==> website::update_version_badge()");
    website::update_version_badge();

    println!("==> website::export_examples_and_pages()");
    let outcome = website::export_examples_and_pages(&api_map);

    println!("==> website::cleanup_site()");
    website::cleanup_site(&outcome.expected);

    println!("==> website::write_healthcheck_aggregators()");
    website::write_healthcheck_aggregators(&outcome.ex_js, &outcome.ex_py);

    println!("✅ Website export done!\n");
}

// Split modules: include their exact previous definitions from build/
include!("build/codegen.rs");
include!("build/convert.rs");
include!("build/validation.rs");
include!("build/website.rs");
include!("build/meta.rs");
