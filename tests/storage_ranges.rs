#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Shader};

const SIMULATION: &str = r#"
struct RangedSim {
    step: u32,
    counts: array<u32, 4>,
}

@group(0) @binding(0) var<storage, read_write> ranged_sim: RangedSim;

@compute @workgroup_size(1)
fn cs_main() {
    for (var i = 0u; i < 4u; i++) {
        ranged_sim.counts[i] += ranged_sim.step;
    }
}
"#;

fn integers(bytes: &[u8]) -> Vec<u32> {
    bytes
        .as_chunks::<4>()
        .0
        .iter()
        .map(|chunk| u32::from_le_bytes(*chunk))
        .collect()
}

// Story: A compute pass keeps counts in a storage root, and the app changes one field
// of that root between frames. Only that field goes up, so the counts the pass wrote stay.
#[test]
fn setting_one_field_keeps_what_the_compute_pass_wrote() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step 1");
        renderer.render(&pass, &probe).expect("first render failed");

        // Act
        shader.set("ranged_sim.step", 10u32).expect("set step 10");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "ranged_sim")
            .await
            .expect("read_storage failed");
        assert_eq!(
            integers(&bytes),
            [10, 11, 11, 11, 11],
            "step, then counts of 1 + 10"
        );
    });
}

// Story: The app overwrites one item of an array the compute pass keeps.
// Its neighbours keep what the pass wrote.
#[test]
fn setting_one_item_keeps_its_neighbours() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step");
        renderer.render(&pass, &probe).expect("first render failed");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Act
        shader
            .set("ranged_sim.counts[2]", 100u32)
            .expect("set one count");
        renderer.render(&pass, &probe).expect("third render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "ranged_sim")
            .await
            .expect("read_storage failed");
        assert_eq!(
            integers(&bytes),
            [1, 3, 3, 101, 3],
            "step, then counts where only item 2 was set"
        );
    });
}

// Story: Raw bytes on the root still replace the whole root, what the pass wrote included.
#[test]
fn setting_the_root_replaces_every_byte() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(SIMULATION).expect("simulation shader failed");
        let pass = Pass::from_shader("simulation", &shader);
        shader.set("ranged_sim.step", 1u32).expect("set step");
        renderer.render(&pass, &probe).expect("first render failed");

        // Act
        let root: Vec<u8> = [2u32, 50, 50, 50, 50]
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        shader.set("ranged_sim", root.as_slice()).expect("set root");
        renderer
            .render(&pass, &probe)
            .expect("second render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "ranged_sim")
            .await
            .expect("read_storage failed");
        assert_eq!(
            integers(&bytes),
            [2, 52, 52, 52, 52],
            "the root's bytes, then one step of 2"
        );
    });
}
