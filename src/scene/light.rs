//! Light — directional lighting primitive.
//!
//! MVP: a single directional light (parallel beam from a fixed world-space
//! direction, with a linear-RGB color). This is the shape `Material::pbr`
//! expects out of the box. Point and spot lights are a follow-up — the
//! type name `Light` reserves the abstraction either way.

use glam::Vec3;
use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::Shader;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/light/light.md")]
pub struct Light {
    pub(crate) direction: Vec3,
    pub(crate) color: Vec3,
}

crate::impl_fc_kind!(Light, "Light");

impl Light {
    /// Construct a directional light. `direction` is the world-space
    /// direction the light *travels in* (so `[0, -1, 0]` is "noon sun
    /// pointing straight down"); `color` is linear RGB intensity.
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional(direction: [f32; 3], color: [f32; 3]) -> Self {
        Self {
            direction: Vec3::from(direction),
            color: Vec3::from(color),
        }
    }

    /// Read the world-space travel direction as `[x, y, z]`.
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction(&self) -> [f32; 3] {
        self.direction.to_array()
    }

    /// Read the linear-RGB color / intensity as `[r, g, b]`.
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color(&self) -> [f32; 3] {
        self.color.to_array()
    }

    /// Write `light.direction` and `light.color` into a Shader. The call is
    /// best-effort: if the shader doesn't declare those uniforms the
    /// underlying `Shader::set` error is silently demoted to a `log::debug!`.
    #[lsp_doc("docs/api/scene/light/bind.md")]
    pub fn bind(&self, shader: &Shader) {
        if let Err(e) = shader.set("light.direction", self.direction()) {
            log::debug!("Light::bind 'light.direction' did not apply: {e}");
        }
        if let Err(e) = shader.set("light.color", self.color()) {
            log::debug!("Light::bind 'light.color' did not apply: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Material;

    #[test]
    fn directional_round_trips_direction_and_color() {
        let light = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
        assert_eq!(light.direction(), [0.3, -1.0, -0.4]);
        assert_eq!(light.color(), [1.0, 0.95, 0.9]);
    }

    #[test]
    fn bind_writes_direction_and_color_to_material_shader() {
        let light = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
        let renderer = crate::Renderer::new();
        let material = pollster::block_on(Material::pbr(&renderer)).expect("pbr");
        light.bind(material.shader());

        let dir: [f32; 3] = material
            .shader()
            .get("light.direction")
            .expect("light.direction");
        assert_eq!(dir, [0.3, -1.0, -0.4]);

        let col: [f32; 3] = material.shader().get("light.color").expect("light.color");
        assert_eq!(col, [1.0, 0.95, 0.9]);
    }

    #[test]
    fn bind_silently_noops_when_uniform_missing() {
        let shader = crate::Shader::new(
            r#"
            @vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
                let p = array<vec2<f32>, 3>(vec2f(-1.0,-1.0), vec2f(3.0,-1.0), vec2f(-1.0,3.0));
                return vec4<f32>(p[i], 0.0, 1.0);
            }
            @fragment fn fs_main() -> @location(0) vec4<f32> {
                return vec4<f32>(1.0);
            }
            "#,
        )
        .expect("compile");
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        light.bind(&shader);
    }
}
