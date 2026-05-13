//! Light — directional lighting primitive.
//!
//! Holds a world-space travel direction and a linear-RGB color in Arc-shared
//! state, so the same Light can be absorbed by multiple Passes with
//! `pass.add(&light)`; later `set_direction` / `set_color` calls propagate
//! to every shader the Light has been wired into.
//!
//! MVP: directional only. The type name `Light` reserves the abstraction for
//! point / spot variants — coming as separate constructors on this type, or
//! as a sum-type with shared `apply` mechanics.

use glam::Vec3;
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::scene::Component;
use crate::shader::ShaderObject;
use crate::Shader;

#[derive(Debug)]
pub(crate) struct LightObject {
    state: RwLock<LightState>,
    attached: RwLock<Vec<Weak<ShaderObject>>>,
}

#[derive(Debug, Clone, Copy)]
struct LightState {
    direction: Vec3,
    color: Vec3,
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/light/light.md")]
pub struct Light {
    pub(crate) object: Arc<LightObject>,
}

crate::impl_fc_kind!(Light, "Light");

impl Light {
    fn from_state(state: LightState) -> Self {
        Self {
            object: Arc::new(LightObject {
                state: RwLock::new(state),
                attached: RwLock::new(Vec::new()),
            }),
        }
    }

    /// Construct a directional light. `direction` is the world-space
    /// direction the light *travels in* (so `[0, -1, 0]` is "noon sun
    /// pointing straight down"); `color` is linear RGB intensity.
    #[lsp_doc("docs/api/scene/light/directional.md")]
    pub fn directional(direction: [f32; 3], color: [f32; 3]) -> Self {
        Self::from_state(LightState {
            direction: Vec3::from(direction),
            color: Vec3::from(color),
        })
    }

    /// Read the world-space travel direction as `[x, y, z]`.
    #[lsp_doc("docs/api/scene/light/direction.md")]
    pub fn direction(&self) -> [f32; 3] {
        self.object.state.read().direction.to_array()
    }

    /// Read the linear-RGB color / intensity as `[r, g, b]`.
    #[lsp_doc("docs/api/scene/light/color.md")]
    pub fn color(&self) -> [f32; 3] {
        self.object.state.read().color.to_array()
    }

    /// Update the world-space travel direction and propagate the new value
    /// to every Material that absorbed this Light.
    #[lsp_doc("docs/api/scene/light/set_direction.md")]
    pub fn set_direction(&self, direction: [f32; 3]) -> Self {
        self.object.state.write().direction = Vec3::from(direction);
        self.propagate();
        self.clone()
    }

    /// Update the linear-RGB color / intensity and propagate to absorbing
    /// Materials.
    #[lsp_doc("docs/api/scene/light/set_color.md")]
    pub fn set_color(&self, color: [f32; 3]) -> Self {
        self.object.state.write().color = Vec3::from(color);
        self.propagate();
        self.clone()
    }

    fn propagate(&self) {
        let dir = self.direction();
        let col = self.color();
        let mut attached = self.object.attached.write();
        attached.retain(|weak| {
            if let Some(shader) = weak.upgrade() {
                let _ = shader.set("light.direction", dir);
                let _ = shader.set("light.color", col);
                true
            } else {
                false
            }
        });
    }
}

impl Component for Light {
    fn apply(&self, shader: &Shader) {
        let _ = shader.set("light.direction", self.direction());
        let _ = shader.set("light.color", self.color());
        self.object
            .attached
            .write()
            .push(Arc::downgrade(&shader.object));
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

    fn pbr_triangle_mesh() -> crate::Mesh {
        let mesh = crate::Mesh::new();
        mesh.add_vertex(
            crate::mesh::Vertex::new([0.0, 0.5, 0.0])
                .set(crate::mesh::Vertex::NORMAL, [0.0, 0.0, 1.0])
                .set(crate::mesh::Vertex::UV0, [0.5, 1.0]),
        );
        mesh
    }

    #[test]
    fn pass_add_seeds_shader_uniforms() {
        let light = Light::directional([0.3, -1.0, -0.4], [1.0, 0.95, 0.9]);
        let renderer = crate::Renderer::new();
        let material = pollster::block_on(Material::pbr(&renderer)).expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add_model(&model).expect("add_model");
        pass.add(&light);

        let dir: [f32; 3] = material
            .shader()
            .get("light.direction")
            .expect("light.direction");
        assert_eq!(dir, [0.3, -1.0, -0.4]);

        let col: [f32; 3] = material.shader().get("light.color").expect("light.color");
        assert_eq!(col, [1.0, 0.95, 0.9]);
    }

    #[test]
    fn set_direction_propagates_to_all_pass_shaders() {
        let light = Light::directional([0.0, -1.0, 0.0], [1.0, 1.0, 1.0]);
        let renderer = crate::Renderer::new();
        let material = pollster::block_on(Material::pbr(&renderer)).expect("pbr");
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add_model(&model).expect("add_model");
        pass.add(&light);

        light.set_direction([0.5, -0.5, 0.0]);
        let dir: [f32; 3] = material.shader().get("light.direction").unwrap();
        assert_eq!(dir, [0.5, -0.5, 0.0]);
    }
}
