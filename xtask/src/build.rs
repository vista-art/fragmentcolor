use crate::api_mapper;
use phf::phf_map;
use std::process::Command;

pub static ICON: phf::Map<&str, &str> = phf_map! {
    "plrender" => "⭕",
    "plrender-codegen" => "🧙‍♂️",
    "plrender-py" => "🐍",
    "plrender-wasm" => "🌎",
};

pub fn build_all() {
    println!();
    println!("🚀 Building all workspace crates...");

    build("plrender");
    api_mapper::map_public_api("plrender");

    // @TODO Get version info

    build("plrender-codegen");
    build_optional("plrender-py");
    build_optional("plrender-wasm");

    // @TODO update /pkg for running the JS examples

    // @TODO build the docs

    // @TODO publish JS examples to /public

    println!("🎉 All done! 🎉");

    // @TODO inform the user about next steps and link to docs
    println!();
}

pub fn build(crate_name: &str) {
    if crate_name == "all" {
        build_all()
    }

    compile_crate(crate_name, true)
}

fn build_optional(crate_name: &str) {
    compile_crate(crate_name, false)
}

fn compile_crate(crate_name: &str, required: bool) {
    let icon = ICON.get(crate_name).unwrap_or(&"📦");
    println!("\n{} Building {}...", icon, crate_name);

    let status = Command::new("cargo")
        .args(&["build", "--package", crate_name])
        .status()
        .expect(&format!("Failed to run build command for {}", crate_name));

    if !status.success() {
        match required {
            true => panic!("🛑 Compilation of {} failed!\n", crate_name),
            false => println!("⚠️ Compilation of optional crate {} failed!\n", crate_name),
        };
    } else {
        println!("✅ Compilation successful!\n");
    };
}
