#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::Scene;

// One triangle with an `_ID` vertex attribute and extras on its node.
const APPLICATION_DATA: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/assets/application_data.gltf"
);

// Story: A glTF file that carries application data still loads its geometry.
#[test]
fn load_skips_application_attributes_and_extras() {
    // Act
    let scene = Scene::load(APPLICATION_DATA).expect("load application_data.gltf");

    // Assert
    assert_eq!(scene.models().len(), 1, "the triangle loads as one model");
}
