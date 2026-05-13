//! Camera — projection + view + cached eye position.
//!
//! Wraps the `proj` + `view` matrix pair every 3D render needs into one
//! object, plus the world-space eye position for shaders that consume it
//! directly (specular highlights, fresnel). A Camera holds Arc-shared state,
//! so the same value can be added to multiple Materials with
//! `material.add(&camera)`; later mutations (`camera.look_at(...)`) propagate
//! to every Material that absorbed it.

use glam::{Mat4, Vec3};
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
pub(crate) struct CameraObject {
    state: RwLock<CameraState>,
    /// Shaders that have absorbed this Camera. Held weakly so a dropped
    /// shader doesn't keep the Camera-side handle alive — and so a Camera
    /// passed to many Materials doesn't grow unbounded.
    attached: RwLock<Vec<Weak<ShaderObject>>>,
}

#[derive(Debug, Clone, Copy)]
struct CameraState {
    view: Mat4,
    proj: Mat4,
    position: Vec3,
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/camera/camera.md")]
pub struct Camera {
    pub(crate) object: Arc<CameraObject>,
}

crate::impl_fc_kind!(Camera, "Camera");

impl Camera {
    fn from_state(state: CameraState) -> Self {
        Self {
            object: Arc::new(CameraObject {
                state: RwLock::new(state),
                attached: RwLock::new(Vec::new()),
            }),
        }
    }

    /// Construct a Camera with a perspective projection. `fovy_radians` is
    /// the vertical FOV; `aspect` is width / height. View defaults to
    /// identity (eye at origin, looking down -Z, +Y up).
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::from_state(CameraState {
            view: Mat4::IDENTITY,
            proj: Mat4::perspective_rh(fovy_radians, aspect, near, far),
            position: Vec3::ZERO,
        })
    }

    /// Construct a Camera with an orthographic projection. The six args are
    /// the frustum planes in view space; pair with a depth attachment
    /// configured for wgpu's [0, 1] NDC depth range.
    #[lsp_doc("docs/api/scene/camera/orthographic.md")]
    pub fn orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::from_state(CameraState {
            view: Mat4::IDENTITY,
            proj: Mat4::orthographic_rh(left, right, bottom, top, near, far),
            position: Vec3::ZERO,
        })
    }

    /// Position the camera in world space. `eye` is where the camera is,
    /// `target` is the point it aims at, `up` is the world-up vector that
    /// orients the roll (typically `[0, 1, 0]`). Returns a handle to the
    /// same Camera (Arc-shared backing) for chaining.
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at(&self, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        let eye_v = Vec3::from(eye);
        {
            let mut state = self.object.state.write();
            state.view = Mat4::look_at_rh(eye_v, Vec3::from(target), Vec3::from(up));
            state.position = eye_v;
        }
        self.propagate();
        self.clone()
    }

    /// Read the combined `proj * view` matrix as a column-major 4×4.
    /// Matches WGSL's `mat4x4<f32>` storage and glam's `to_cols_array_2d()`.
    #[lsp_doc("docs/api/scene/camera/view_proj.md")]
    pub fn view_proj(&self) -> [[f32; 4]; 4] {
        let s = self.object.state.read();
        (s.proj * s.view).to_cols_array_2d()
    }

    /// Read the world-space eye position as `[x, y, z]`.
    #[lsp_doc("docs/api/scene/camera/position.md")]
    pub fn position(&self) -> [f32; 3] {
        self.object.state.read().position.to_array()
    }

    /// Push the current state to every shader that absorbed this Camera,
    /// dropping `Weak` entries whose `ShaderObject` has already been
    /// freed.
    fn propagate(&self) {
        let vp = self.view_proj();
        let pos = self.position();
        let mut attached = self.object.attached.write();
        attached.retain(|weak| {
            if let Some(shader) = weak.upgrade() {
                let _ = shader.set("camera.view_proj", vp);
                let _ = shader.set("camera.position", pos);
                true
            } else {
                false
            }
        });
    }
}

impl Component for Camera {
    fn apply(&self, shader: &Shader) {
        let _ = shader.set("camera.view_proj", self.view_proj());
        let _ = shader.set("camera.position", self.position());
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

    fn identity() -> [[f32; 4]; 4] {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    #[test]
    fn perspective_produces_nontrivial_mat4() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);
        let m = camera.view_proj();
        // Perspective encodes the homogeneous-w divide as a -1 in [2][3].
        assert!((m[2][3] + 1.0).abs() < 1.0e-5, "got {m:?}");
        assert!(m != identity(), "perspective(...) view_proj must not be identity");
    }

    #[test]
    fn orthographic_preserves_w() {
        let camera = Camera::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0);
        let m = camera.view_proj();
        // Orthographic keeps the bottom-right [3][3] at 1 (no w divide).
        assert!((m[3][3] - 1.0).abs() < 1.0e-5, "got {m:?}");
        assert!(m != identity());
    }

    #[test]
    fn look_at_changes_view_component() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let before = camera.view_proj();
        camera.look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let after = camera.view_proj();
        assert!(before != after, "look_at must change view_proj");
        assert_eq!(camera.position(), [0.0, 0.0, 5.0]);
    }

    #[test]
    fn add_via_material_seeds_shader_uniforms() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .look_at([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let renderer = crate::Renderer::new();
        let material = pollster::block_on(Material::pbr(&renderer)).expect("pbr");
        material.add(&camera);

        let m: [[f32; 4]; 4] = material
            .shader()
            .get("camera.view_proj")
            .expect("camera.view_proj");
        assert_eq!(m, camera.view_proj());

        let p: [f32; 3] = material
            .shader()
            .get("camera.position")
            .expect("camera.position");
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn mutations_propagate_to_absorbed_materials() {
        // Add the camera to a Material, then move it. The Material's shader
        // should see the new view_proj without a second add() call.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let renderer = crate::Renderer::new();
        let material = pollster::block_on(Material::pbr(&renderer)).expect("pbr");
        material.add(&camera);

        camera.look_at([5.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let p: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(p, [5.0, 0.0, 0.0]);
    }

    #[test]
    fn add_silently_noops_when_uniforms_missing() {
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

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let material = Material::custom(shader);
        // Should not panic; missing uniforms are demoted to debug logs.
        material.add(&camera);
    }
}
