#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Pass, Renderer, Shader};

const COUNT: &str = r#"
@group(0) @binding(0) var<storage, read_write> atomic_counts: array<atomic<u32>, 4>;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    atomicAdd(&atomic_counts[gid.x % 4u], 1u);
}
"#;

const SUM: &str = r#"
@group(0) @binding(0) var<storage, read> atomic_counts: array<u32, 4>;
@group(0) @binding(1) var<storage, read_write> atomic_total: u32;

@compute @workgroup_size(1)
fn cs_main() {
    atomic_total = atomic_counts[0] + atomic_counts[1] + atomic_counts[2] + atomic_counts[3];
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

// Story: A compute pass counts with atomics, and a second shader reads the same buffer by name.
#[test]
fn atomic_counts_are_shared_with_a_plain_view() {
    pollster::block_on(async move {
        // Arrange
        let renderer = Renderer::new();
        let probe = renderer
            .create_texture_target([8, 8])
            .await
            .expect("create_texture_target failed");

        let count = Shader::new(COUNT).expect("atomic shader failed");
        let counting = Pass::from_shader("count", &count);
        counting.set_compute_dispatch(2, 1, 1);

        let sum = Shader::new(SUM).expect("plain view shader failed");
        let summing = Pass::from_shader("sum", &sum);

        // Act
        renderer
            .render(&vec![&counting, &summing], &probe)
            .expect("render failed");

        // Assert
        let counted = renderer
            .read_storage(&count, "atomic_counts")
            .await
            .expect("read_storage failed");
        let viewed = renderer
            .read_storage(&sum, "atomic_counts")
            .await
            .expect("read_storage failed");
        let total = renderer
            .read_storage(&sum, "atomic_total")
            .await
            .expect("read_storage failed");
        assert_eq!(integers(&counted), [32, 32, 32, 32], "atomic counts");
        assert_eq!(
            integers(&viewed),
            [32, 32, 32, 32],
            "plain view of the counts"
        );
        assert_eq!(integers(&total), [128], "sum read through the plain view");
    });
}
