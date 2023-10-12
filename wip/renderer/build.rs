use anyhow::*;
use cfg_aliases::cfg_aliases;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
    // This tells cargo to rerun this script if something in /res/ changes.
    println!("cargo:rerun-if-changed=assets/*");

    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
    }

    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("assets/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    Ok(())
}
