use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    process::Command,
};
use xtask::api_mapper;

pub const API_MAP_KEYWORD: &str = "API_MAP";

fn main() {
    println!("🚀 Running xtask...");

    compile_crate("plrender", "⭕ Building PLRender...", true);

    // @TODO bump version in Cargo.toml and documentation

    generate_api_map(
        &Path::new("../../crates/plrender"),
        &Path::new("../../generated/api_map.rs"),
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

    println!("🎉 All done! 🎉");

    // @TODO inform the user about next steps and link to docs
}

fn compile_crate(crate_name: &str, message: &str, required: bool) {
    println!();
    println!("{}", message);
    let status = Command::new("cargo")
        .args(&["build", "--package", crate_name])
        .status()
        .expect(&format!("Failed to compile {}", crate_name));
    if !status.success() {
        match required {
            true => panic!("🛑 Compilation of required crate {} failed!", crate_name),
            false => println!("⚠️ Compilation of optional crate {} failed!", crate_name),
        }
    }
    println!("✅ compilation successful!");
    println!();
}

fn generate_api_map(crate_root: &Path, generated_file: &Path, message: &str) {
    println!();
    println!("{}", message);
    println!("📂 Crate root: {}", crate_root.display());
    println!("📄 Generating file: {}", generated_file.display());
    println!();

    let mut static_map_builder = phf_codegen::Map::new();
    let mut generated_file = File::create(&generated_file).unwrap();
    let mut writer = BufWriter::new(&mut generated_file);
    let api_functions_map = api_mapper::extract_public_functions(crate_root);

    for (struct_name, functions) in api_functions_map {
        static_map_builder.entry(
            struct_name.clone(),
            &format!(
                "{}",
                functions
                    .iter()
                    .map(|function| {
                        format!(
                            "fn {}({}){};",
                            function.name,
                            function
                                .parameters
                                .iter()
                                .map(|param| { format!("{}: {}, ", param.name, param.type_name) })
                                .collect::<String>(),
                            function
                                .return_type
                                .as_ref()
                                .map(|return_type| { format!(" -> {}", return_type) })
                                .unwrap_or("".to_string())
                        )
                    })
                    .collect::<String>()
            ),
        );
    }

    write!(
        &mut writer,
        "static {}: phf::Map<&'static str, &'static str> = \n{};\n",
        API_MAP_KEYWORD,
        static_map_builder.build()
    )
    .unwrap();

    println!();
}
