use std::process::Command;

fn main() {
    println!("🚀 Running xtask...");

    compile_crate("plrender", "⭕ Building PLRender...", true);

    // @TODO bump version in Cargo.toml and documentation

    generate_api_map(
        "../../plrender/src/api.rs",
        "../../generated/api_map.rs",
        "🗺️ Generating API map...",
    );

    compile_crate(
        "plrender-macros",
        "🧙‍♂️ Building API wrapper macros...",
        true,
    );

    compile_crate("plrender-py", "🐍 Building Python module...", false);

    compile_crate("plrender-wasm", "🌎 Building JS/Wasm module...", false);

    // @TODO update /pkg for running the JS examples

    println!("✅ All done!");
}

fn compile_crate(crate_name: &str, message: &str, required: bool) {
    println!("{}", message);
    let status = Command::new("cargo")
        .args(&["build", "--package", crate_name])
        .status()
        .expect(&format!("Failed to compile {}", crate_name));
    if !status.success() {
        match required {
            true => panic!("🛑 Compilation of {} failed!", crate_name),
            false => println!("⚠️ Compilation of {} failed!", crate_name),
        }
    }
}

fn generate_api_map(from: &str, to: &str, message: &str) {
    println!("{}", message);

    // Here, you'd have the logic to parse the `api.rs` file in the `plrender` crate's root
    // and generate the API map. This might involve reading the file, parsing it with `syn`,
    // and generating the map using `quote` and `phf-codegen`.

    // [Your code to generate the API map goes here]
}
