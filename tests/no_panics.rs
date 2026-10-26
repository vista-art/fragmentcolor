//! The no-panic scan that build.rs runs on the library sources, tested here
//! because `cargo test` never runs a build script's own tests.
#![cfg(not(target_arch = "wasm32"))]

include!("../scripts/no_panics.rs");

// Story: the library sources pass the policy that every repository build enforces.
#[test]
fn the_library_passes_the_policy() {
    assert_eq!(enforce_no_panic_policy(), Ok(()));
}

// Story: a `tests` folder below the scan root holds test code, so the scan skips it,
// while a checkout that sits inside a folder named `tests` is still scanned.
#[test]
fn only_a_tests_folder_below_the_root_stays_out() {
    let checkout = tempfile::tempdir().expect("temp dir");
    let src = checkout.path().join("tests").join("src");
    let module = src.join("renderer").join("lights").join("tests");
    fs::create_dir_all(&module).expect("folders");
    fs::write(
        module.join("mod.rs"),
        "fn check() { None::<u8>.expect(\"test\"); }\n",
    )
    .expect("test module");
    fs::write(
        src.join("lib.rs"),
        "pub fn run() { None::<u8>.expect(\"library\"); }\n",
    )
    .expect("library");

    let found: Vec<PathBuf> = find_panics(&src)
        .into_iter()
        .map(|(path, _)| path)
        .collect();

    assert_eq!(found, vec![src.join("lib.rs")]);
}

// Story: a Windows path separates its components with backslashes, and the scan
// reads them as components, so the test module under `tests` stays out. Unix reads
// a backslash as part of a file name, so this path only parses as one on Windows.
#[cfg(windows)]
#[test]
fn a_windows_path_reads_its_components() {
    let root = Path::new(r"C:\ci\fragmentcolor\src");
    let module = Path::new(r"C:\ci\fragmentcolor\src\renderer\lights\tests\mod.rs");
    let library = Path::new(r"C:\ci\fragmentcolor\src\renderer\lights.rs");

    assert!(is_test_source(root, module));
    assert!(!is_test_source(root, library));
}
