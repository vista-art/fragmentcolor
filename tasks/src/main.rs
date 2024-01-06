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

// it would be nicer to use `clap` here
fn print_general_help() {
    println!("Usage: cargo task [COMMAND] [CRATE]");
    println!();
    println!("Commands:");
    println!("  build all           Builds all crates");
    println!("  build [CRATE]       Builds the specified crate");
    println!("  map   [CRATE]       Generates the API map of a crate");
    println!("  help  [COMMAND]     Prints help about a command");
    println!("  help                Prints this message");
}

fn print_build_help() {
    println!("Usage: cargo task build [CRATE]");
    println!();
    println!("Subcommands:");
    println!("  all                 Builds all crates");
    println!("  fragmentcolor            Builds the fragmentcolor crate");
    println!("  fragmentcolor-codegen    Builds the fragmentcolor-codegen crate");
    println!("  fragmentcolor-py         Builds the fragmentcolor-py crate");
    println!("  fragmentcolor-wasm       Builds the fragmentcolor-wasm crate");
    println!();
    println!("Example:");
    println!("    cargo task build fragmentcolor");
}

fn print_api_map_help() {
    println!("Usage: cargo task map [CRATE]");
    println!();
    println!("Crates:");
    println!("  all                 Maps all crates");
    println!("  fragmentcolor            Maps the fragmentcolor crate");
    println!("  fragmentcolor-codegen    Maps the fragmentcolor-codegen crate");
    println!("  fragmentcolor-py         Maps the fragmentcolor-py crate");
    println!("  fragmentcolor-wasm       Maps the fragmentcolor-wasm crate");
    println!();
    println!("Example:");
    println!("    cargo xtask map fragmentcolor");
}
