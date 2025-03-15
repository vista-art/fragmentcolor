use cfg_aliases::cfg_aliases;

fn main() {
    cfg_aliases! {
        wasm: { target_arch = "wasm32" },
        ios: { target_os = "ios" },
        android: { target_os = "android" },
        mobile: { any(android, ios) },
        generic: { not(any(wasm, mobile)) },
        desktop: { all(generic, not(feature="python")) },
        python: { all(generic, feature="python") },
        dev: { all(generic, feature="uniffi/cli") },
    }
    println!("cargo::rustc-check-cfg=cfg(wasm)");
    println!("cargo::rustc-check-cfg=cfg(ios)");
    println!("cargo::rustc-check-cfg=cfg(android)");
    println!("cargo::rustc-check-cfg=cfg(mobile)");
    println!("cargo::rustc-check-cfg=cfg(desktop)");
    println!("cargo::rustc-check-cfg=cfg(dev)");
}
