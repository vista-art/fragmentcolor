#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Renderer, Shader};

const FULLSCREEN: &str = r#"
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var p = array<vec2<f32>, 3>(vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    return vec4<f32>(p[i], 0.0, 1.0);
}
"#;

// Story: a fragment entry point counts its pixels into a storage buffer, and the pass draws.
#[test]
fn fragment_writes_a_storage_buffer() {
    pollster::block_on(async move {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([4, 4])
            .await
            .expect("target");
        let source = format!(
            "{FULLSCREEN}
@group(0) @binding(0) var<storage, read_write> hits: array<atomic<u32>, 1>;
@fragment fn fs_main() -> @location(0) vec4<f32> {{
    atomicAdd(&hits[0], 1u);
    return vec4<f32>(1.0);
}}"
        );
        let shader = Shader::new(source.as_str()).expect("shader");

        renderer.render(&shader, &target).expect("render");
        let bytes = renderer
            .read_storage(&shader, "hits")
            .await
            .expect("read hits");

        let count = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert!(count >= 16, "16 pixels counted {count} hits");
    });
}

// Story: no device FragmentColor opens takes writable storage in the vertex stage, so Shader::new refuses it.
#[test]
fn vertex_writable_storage_fails_at_new() {
    let buffer = r#"
@group(0) @binding(0) var<storage, read_write> seen: array<u32, 4>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    seen[i % 4u] = 1u;
    return vec4<f32>(0.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    let texture = r#"
@group(0) @binding(0) var marks: texture_storage_2d<rgba8unorm, write>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    textureStore(marks, vec2<u32>(i, 0u), vec4<f32>(1.0));
    return vec4<f32>(0.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(1.0); }
"#;
    for (source, name) in [(buffer, "seen"), (texture, "marks")] {
        let err = Shader::new(source).unwrap_err().to_string();
        assert!(
            err.contains(&format!("`{name}` is writable storage")),
            "{err}"
        );
    }
}
