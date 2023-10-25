use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::Command,
};
use xtask::api_mapper;

pub const API_MAP_KEYWORD: &str = "API_MAP";
pub const API_MAP_FILE: &str = "generated/api_map.rs";

fn main() {
    println!("🚀 Running xtask...");

    compile_crate("plrender", "⭕ Building PLRender...", true);

    // @TODO bump version in documentation from project's Cargo.toml manifest

    let crates = [crate_root("plrender")];
    let api_map_file = workspace_root().join(API_MAP_FILE);
    generate_api_map(&crates, &api_map_file, "🗺️ Generating API map...");

    compile_crate("plrender-macros", "🧙‍♂️ Building API wrapper...", true);

    compile_crate("plrender-py", "🐍 Building Python module...", false);

    compile_crate("plrender-wasm", "🌎 Building JS/Wasm module...", false);

    // @TODO update /pkg for running the JS examples

    println!("🎉 All done! 🎉");

    // @TODO inform the user about next steps and link to docs
}

fn compile_crate(crate_name: &str, message: &str, required: bool) {
    println!("\n{}", message);

    let status = Command::new("cargo")
        .args(&["build", "--package", crate_name])
        .status()
        .expect(&format!("Failed to run build command for {}", crate_name));

    if !status.success() {
        match required {
            true => panic!("🛑 Compilation of required crate {} failed!\n", crate_name),
            false => println!("⚠️ Compilation of optional crate {} failed!\n", crate_name),
        };
    } else {
        println!("✅ compilation successful!\n");
    };
}

fn generate_api_map(crate_roots: &[PathBuf], target_file: &Path, message: &str) {
    println!();
    println!("{}", message);

    let mut api_map: api_mapper::ApiMap = api_mapper::ApiMap::new();
    for crate_root in crate_roots {
        let crate_api_map = api_mapper::extract_public_functions(crate_root);
        api_map.extend(crate_api_map);
    }

    export_api_map(api_map, target_file);

    println!("✅ API map successfully generated!");
    println!();
}

fn export_api_map(api_map: api_mapper::ApiMap, target_file: &Path) {
    let mut static_map_builder = phf_codegen::Map::new();
    let mut target_file = File::create(&target_file).unwrap();
    let mut writer = BufWriter::new(&mut target_file);

    for (struct_name, functions) in api_map {
        static_map_builder.entry(
            struct_name.clone(),
            &format!(
                "&[{}]",
                functions
                    .iter()
                    .map(|function| {
                        format!("{:?}, ", function).replace("parameters: [", "parameters: &[")
                    })
                    .collect::<String>()
            ),
        );
    }

    write!(
        &mut writer,
        "{}\n\nstatic {}: phf::Map<&'static str, &[FunctionSignature]> = {};\n",
        api_mapper::FUNCTION_SIGNATURE_STRUCT_DEFINITION,
        API_MAP_KEYWORD,
        static_map_builder.build()
    )
    .unwrap();
}

fn workspace_root() -> PathBuf {
    let output = Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

fn crate_root(crate_name: &str) -> PathBuf {
    workspace_root().join(format!("crates/{}", crate_name))
}
