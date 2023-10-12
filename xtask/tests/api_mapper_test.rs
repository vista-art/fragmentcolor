mod helpers;

use helpers::crate_builder;
use tempfile::tempdir;
use xtask::api_mapper;

const TEMP_FILESYSTEM: &[&str] = [
    "Cargo.toml",
    "src/lib.rs",
    "src/glob.rs",
    "src/priv/mod.rs",
    "src/priv/priv.rs",
    "src/priv/reexported.rs",
    "src/group/mod.rs",
    "src/group/priv.rs",
]
.as_slice();

#[test]
fn test_extract_function_signatures_from_crate() {
    let dir = tempdir().unwrap();
    crate_builder::create_temp_files(&dir, TEMP_FILESYSTEM);

    let signatures = api_mapper::extract_public_functions(&dir.path());

    assert_eq!(signatures.len(), 2);
    assert!(signatures.contains_key("LibStruct"));
    assert!(signatures.contains_key("ModStruct"));

    let lib_methods = signatures.get("LibStruct").unwrap();
    assert_eq!(lib_methods.len(), 1);
    assert_eq!(lib_methods[0].name, "public_method");
    assert_eq!(
        lib_methods[0].parameters,
        vec![api_mapper::FunctionParameter {
            name: "arg".to_string(),
            type_name: "i32".to_string()
        }]
    );
    assert_eq!(lib_methods[0].return_type, Some("String".to_string()));

    let mod_methods = signatures.get("ModStruct").unwrap();
    assert_eq!(mod_methods.len(), 1);
    assert_eq!(mod_methods[0].name, "mod_public_method");
    assert_eq!(mod_methods[0].parameters.len(), 0);
    assert_eq!(mod_methods[0].return_type, None);
}
