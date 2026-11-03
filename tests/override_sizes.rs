#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Shader};

// Two overrides give the workgroup size, 8 by 4 at their defaults, and each thread
// writes its own lane, so a workgroup of one thread would write only the first.
const LANES: &str = r#"
override width: u32 = 8;
override height: u32 = width / 2;
@group(0) @binding(0) var<storage, read_write> override_lanes: array<u32, 32>;

@compute @workgroup_size(width, height)
fn cs_main(@builtin(local_invocation_index) index: u32) {
    override_lanes[index] = index + 1u;
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

// Story: a compute shader whose workgroup size names overrides runs as many threads
// as their defaults give, so one workgroup fills every lane.
#[test]
fn override_workgroup_sizes_run_every_thread() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([1, 1])
            .await
            .expect("create_texture_target failed");
        let shader = Shader::new(LANES).expect("shader failed");
        let pass = Pass::from_shader("lanes", &shader);

        // Act
        renderer.render(&pass, &target).expect("render failed");

        // Assert
        let bytes = renderer
            .read_storage(&shader, "override_lanes")
            .await
            .expect("read_storage failed");
        let expected: Vec<u32> = (1..=32).collect();
        assert_eq!(integers(&bytes), expected);
    });
}
