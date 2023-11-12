use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
    }
    built::write_built_file().expect("Failed to acquire build-time data");
}
