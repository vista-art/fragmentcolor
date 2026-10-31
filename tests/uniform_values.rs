#![cfg(not(target_arch = "wasm32"))]

use fragmentcolor::{Shader, ShaderError, UniformData};

/// A fullscreen shader that draws its `tint`.
const TINTED: &str = r#"
@group(0) @binding(0) var<uniform> tint: vec4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    return vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return tint;
}
"#;

/// Sets `value` under `key` on every shader in `shaders`.
fn set_all(
    shaders: &[&Shader],
    key: &str,
    value: impl Into<UniformData>,
) -> Result<(), ShaderError> {
    let value = value.into();
    for shader in shaders {
        shader.set(key, value.clone())?;
    }
    Ok(())
}

// Story: a helper outside the crate names UniformData to forward one value to several shaders.
#[test]
fn a_helper_forwards_one_value_to_several_shaders() {
    let first = Shader::new(TINTED).expect("a shader");
    let second = Shader::new(TINTED).expect("a shader");

    set_all(&[&first, &second], "tint", [1.0f32, 0.5, 0.25, 1.0]).expect("sets");

    for shader in [&first, &second] {
        let tint: [f32; 4] = shader.get("tint").expect("a tint");
        assert_eq!(tint, [1.0, 0.5, 0.25, 1.0]);
    }
}
