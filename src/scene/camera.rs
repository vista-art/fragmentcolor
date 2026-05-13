//! Camera — projection + view + cached eye position.
//!
//! Wraps the `proj` + `view` matrix pair every 3D render needs into one
//! object, plus the world-space eye position for shaders that consume it
//! directly (specular highlights, fresnel). Bind into a Shader with
//! [`Camera::bind`] — the typical target is a Material's shader, which
//! declares `camera.view_proj` + `camera.position` as part of the default
//! PBR uniform surface.

use glam::{Mat4, Vec3};
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
#[lsp_doc("docs/api/scene/camera/camera.md")]
pub struct Camera {
    pub(crate) view: Mat4,
    pub(crate) proj: Mat4,
    pub(crate) position: Vec3,
}

crate::impl_fc_kind!(Camera, "Camera");

impl Camera {
    /// Construct a Camera with a perspective projection. `fovy_radians` is
    /// the vertical FOV; `aspect` is width / height. View defaults to
    /// identity (eye at origin, looking down -Z, +Y up).
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            view: Mat4::IDENTITY,
            proj: Mat4::perspective_rh(fovy_radians, aspect, near, far),
            position: Vec3::ZERO,
        }
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
        Self {
            view: Mat4::IDENTITY,
            proj: Mat4::orthographic_rh(left, right, bottom, top, near, far),
            position: Vec3::ZERO,
        }
    }

    /// Chainable: position the camera in world space. `eye` is where the
    /// camera is, `target` is the point it aims at, `up` is the world-up
    /// vector that orients the roll (typically `[0, 1, 0]`).
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at(mut self, eye: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        let eye_v = Vec3::from(eye);
        self.view = Mat4::look_at_rh(eye_v, Vec3::from(target), Vec3::from(up));
        self.position = eye_v;
        self
    }

    /// Read the combined `proj * view` matrix as a column-major 4×4.
    /// Matches WGSL's `mat4x4<f32>` storage and glam's `to_cols_array_2d()`.
    #[lsp_doc("docs/api/scene/camera/view_proj.md")]
    pub fn view_proj(&self) -> [[f32; 4]; 4] {
        (self.proj * self.view).to_cols_array_2d()
    }

    /// Read the world-space eye position as `[x, y, z]`.
    #[lsp_doc("docs/api/scene/camera/position.md")]
    pub fn position(&self) -> [f32; 3] {
        self.position.to_array()
    }

    /// Write `camera.view_proj` and `camera.position` into a Shader. The
    /// call is best-effort: if the shader doesn't declare those uniforms
    /// the underlying `Shader::set` error is silently demoted to a
    /// `log::debug!`.
    #[lsp_doc("docs/api/scene/camera/bind.md")]
    pub fn bind(&self, shader: &Shader) {
        let vp = self.view_proj();
        if let Err(e) = shader.set("camera.view_proj", vp) {
            log::debug!("Camera::bind 'camera.view_proj' did not apply: {e}");
        }
        if let Err(e) = shader.set("camera.position", self.position()) {
            log::debug!("Camera::bind 'camera.position' did not apply: {e}");
        }
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
        let camera = camera.look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let after = camera.view_proj();
        assert!(before != after, "look_at must change view_proj");
        assert_eq!(camera.position(), [0.0, 0.0, 5.0]);
    }

    #[test]
    fn bind_writes_view_proj_and_position_to_material_shader() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .look_at([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let renderer = crate::Renderer::new();
        let material = pollster::block_on(Material::pbr(&renderer)).expect("pbr");
        camera.bind(material.shader());

        let m: [[f32; 4]; 4] = material
            .shader()
            .get("camera.view_proj")
            .expect("camera.view_proj");
        assert_eq!(m, camera.view_proj());

        let p: [f32; 3] = material.shader().get("camera.position").expect("camera.position");
        assert_eq!(p, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn bind_silently_noops_when_uniform_missing() {
        // A shader with no camera.* uniforms.
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
        // Should not panic; missing uniforms are demoted to debug logs.
        camera.bind(&shader);
    }
}
