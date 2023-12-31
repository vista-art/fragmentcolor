use crate::{api_mapper, meta};
use phf::phf_map;
use std::process::Command;

pub static ICON: phf::Map<&str, &str> = phf_map! {
    "fragmentcolor" => "⭕",
    "fragmentcolor-codegen" => "🧙‍♂️",
    "fragmentcolor-wasm" => "🌎",
    "fragmentcolor-py" => "🐍",
};

pub static BUILDER: phf::Map<&str, &str> = phf_map! {
    "fragmentcolor" => "cargo",
    "fragmentcolor-codegen" => "cargo",
    "fragmentcolor-wasm" => "wasm",
    "fragmentcolor-py" => "py",
};

pub fn build_all() {
    println!();
    println!("🚀 Building all workspace crates...");

    build("fragmentcolor");
    api_mapper::map_public_api("fragmentcolor");
    //build("fragmentcolor-codegen");
    build_cargo("fragmentcolor-wasm");
    build_cargo("fragmentcolor-py");

    println!("🎉 All done! 🎉");
    println!();
}

pub fn build(crate_name: &str) {
    if crate_name == "all" {
        build_all()
    }

    let builder = BUILDER.get(crate_name).unwrap_or(&"cargo");
    let icon = ICON.get(crate_name).unwrap_or(&"📦");
    println!("\n{} Building {}...", icon, crate_name);

    let status = compile_crate(crate_name, builder);

    if !status.success() {
        panic!("🛑 Compilation of {} failed!\n", crate_name);
    } else {
        println!("✅ Compilation successful!\n");
    };
}

fn compile_crate(crate_name: &str, builder: &str) -> std::process::ExitStatus {
    match builder {
        "cargo" => build_cargo(crate_name),
        "wasm" => build_wasm(crate_name),
        "py" => build_py(crate_name),
        _ => panic!("Unknown builder: {}", builder),
    }
}

fn build_cargo(crate_name: &str) -> std::process::ExitStatus {
    Command::new("cargo")
        .args(["build", "--package", crate_name])
        .status()
        .unwrap_or_else(|_| panic!("Failed to run build command for {}", crate_name))
}

fn build_wasm(crate_name: &str) -> std::process::ExitStatus {
    let crate_root = meta::crate_root(crate_name);
    Command::new("wasm-pack")
        .args(["build"])
        .current_dir(crate_root)
        .status()
        .expect("Failed to run wasm-pack build command")
}

fn build_py(crate_name: &str) -> std::process::ExitStatus {
    let crate_root = meta::crate_root(crate_name);

    Command::new("maturin")
        .args(["build", "--release"])
        .current_dir(crate_root)
        .status()
        .expect("Failed to run maturin build command")
}
