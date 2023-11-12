use std::env;
use xtask::build;

type Error = Box<dyn std::error::Error>;

fn main() {
    if let Err(error) = run() {
        eprintln!("{}", error);
        std::process::exit(-1);
    }
}

fn run() -> Result<(), Error> {
    let task = env::args().nth(1);
    let arg = env::args().nth(2);

    match task.as_deref() {
        Some("help") => print_help(&arg.unwrap_or("".to_string())),
        Some("build") => build::build(&arg.expect("Please provide the crate name")),
        _ => build::build_all(),
    }

    Ok(())
}

pub fn print_help(command: &str) {
    match command {
        "build" => print_build_help(),
        "map" => print_api_map_help(),
        _ => print_general_help(),
    }
}

// @TODO it would be nicer to use `clap` here
fn print_general_help() {
    println!("Usage: cargo xtask [COMMAND] [CRATE]");
    println!();
    println!("Commands:");
    println!("  build all           Builds all crates");
    println!("  build [CRATE]       Builds the specified crate");
    println!("  map   [CRATE]       Generates the API map of a crate");
    println!("  help  [COMMAND]     Prints help about a command");
    println!("  help                Prints this message");
}

fn print_build_help() {
    println!("Usage: cargo xtask build [CRATE]");
    println!();
    println!("Subcommands:");
    println!("  all                 Builds all crates");
    println!("  plrender            Builds the plrender crate");
    println!("  plrender-codegen    Builds the plrender-codegen crate");
    println!("  plrender-py         Builds the plrender-py crate");
    println!("  plrender-wasm       Builds the plrender-wasm crate");
    println!();
    println!("Example:");
    println!("    cargo xtask build plrender");
}

fn print_api_map_help() {
    println!("Usage: cargo xtask map [CRATE]");
    println!();
    println!("Crates:");
    println!("  all                 Maps all crates");
    println!("  plrender            Maps the plrender crate");
    println!("  plrender-codegen    Maps the plrender-codegen crate");
    println!("  plrender-py         Maps the plrender-py crate");
    println!("  plrender-wasm       Maps the plrender-wasm crate");
    println!();
    println!("Example:");
    println!("    cargo xtask map plrender");
}
