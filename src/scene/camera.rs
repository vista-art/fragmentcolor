//! Camera — projection + view + cached eye position.
//!
//! Wraps the `proj` + `view` matrix pair every 3D render needs into one
//! object, plus the world-space eye position for shaders that consume it
//! directly (specular highlights, fresnel). A Camera holds Arc-shared state,
//! so the same value can be absorbed by multiple Passes with `pass.add(&camera)`;
//! later mutations (`camera.look_at(...)`) propagate to every shader the
//! Camera has been wired into.

use glam::{Mat4, Vec3};
use lsp_doc::lsp_doc;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::Shader;
use crate::pass::{CameraSnapshot, PassObject};
use crate::scene::SceneObject;
use crate::shader::ShaderObject;

#[derive(Debug)]
pub(crate) struct CameraObject {
    state: RwLock<CameraState>,
    /// Shaders that have absorbed this Camera. Held weakly so a dropped
    /// shader doesn't keep the Camera-side handle alive — and so a Camera
    /// passed to many Materials doesn't grow unbounded.
    attached: RwLock<Vec<Weak<ShaderObject>>>,
    /// Passes that have absorbed this Camera. Each entry's
    /// `camera_snapshot` slot mirrors the Camera's current state (view +
    /// position) so the renderer can sort translucent draws back-to-front
    /// without going through the shader's uniform storage. Held weakly so a
    /// dropped Pass doesn't keep the Camera-side handle alive.
    attached_passes: RwLock<Vec<Weak<PassObject>>>,
}

/// Tag + rebuild parameters for the Camera's projection. `set_aspect`
/// reads these to recompute the projection matrix in place when the
/// caller hands a new aspect ratio — for `Perspective` it rebuilds from
/// `fovy_radians / near / far`; for `Orthographic` it keeps the current
/// vertical extent and rescales the horizontal so `(right - left) /
/// (top - bottom)` matches the new aspect, centered on the existing
/// horizontal midpoint. Internal-only — the public surface uses
/// `[[f32; 4]; 4]`.
#[derive(Debug, Clone, Copy)]
enum Projection {
    Perspective {
        fovy_radians: f32,
        near: f32,
        far: f32,
    },
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
}

#[derive(Debug, Clone, Copy)]
struct CameraState {
    projection: Projection,
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
                attached_passes: RwLock::new(Vec::new()),
            }),
        }
    }

    /// Construct a Camera with a perspective projection. `fovy_radians` is
    /// the vertical FOV; `aspect` is width / height. View defaults to
    /// identity (eye at origin, looking down -Z, +Y up).
    #[lsp_doc("docs/api/scene/camera/perspective.md")]
    pub fn perspective(fovy_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self::from_state(CameraState {
            projection: Projection::Perspective {
                fovy_radians,
                near,
                far,
            },
            view: Mat4::IDENTITY,
            proj: Mat4::perspective_rh(fovy_radians, aspect, near, far),
            position: Vec3::ZERO,
        })
    }

    /// Construct a Camera with an orthographic projection. The six args are
    /// the frustum planes in view space; pair with a depth attachment
    /// configured for wgpu's [0, 1] NDC depth range.
    #[lsp_doc("docs/api/scene/camera/orthographic.md")]
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        Self::from_state(CameraState {
            projection: Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            },
            view: Mat4::IDENTITY,
            proj: Mat4::orthographic_rh(left, right, bottom, top, near, far),
            position: Vec3::ZERO,
        })
    }

    /// Update the camera's aspect ratio. The typical caller is a resize
    /// handler: hand the new `width / height` after the window resizes
    /// and the projection matrix recomputes in place, propagating to every
    /// shader (and the Pass camera snapshot used by the transparency sort)
    /// without dropping the Camera handle.
    ///
    /// Behaviour by projection kind:
    /// - **Perspective**: rebuilds from `fovy_radians / near / far` with
    ///   the new aspect.
    /// - **Orthographic**: keeps the current vertical extent and rescales
    ///   the horizontal extents so `(right - left) / (top - bottom)`
    ///   matches the new aspect, centred on the existing horizontal
    ///   midpoint. Frustum height stays put.
    ///
    /// Returns a handle to the same Camera (Arc-shared) for chaining.
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect(&self, aspect: f32) -> Self {
        {
            let mut state = self.object.state.write();
            match state.projection {
                Projection::Perspective {
                    fovy_radians,
                    near,
                    far,
                } => {
                    state.proj = Mat4::perspective_rh(fovy_radians, aspect, near, far);
                }
                Projection::Orthographic {
                    left,
                    right,
                    bottom,
                    top,
                    near,
                    far,
                } => {
                    let height = top - bottom;
                    let mid_x = 0.5 * (left + right);
                    let half_w = 0.5 * aspect * height;
                    let new_left = mid_x - half_w;
                    let new_right = mid_x + half_w;
                    state.projection = Projection::Orthographic {
                        left: new_left,
                        right: new_right,
                        bottom,
                        top,
                        near,
                        far,
                    };
                    state.proj = Mat4::orthographic_rh(new_left, new_right, bottom, top, near, far);
                }
            }
        }
        self.propagate();
        self.clone()
    }

    /// Position the camera in world space. `position` is where the camera
    /// sits, `target` is the point it aims at, `up` is the world-up vector
    /// that orients the roll (typically `[0, 1, 0]`). Matches Light's
    /// `position` vocabulary so both scene types describe "where this
    /// object is in the world" the same way. Returns a handle to the same
    /// Camera (Arc-shared backing) for chaining.
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at(&self, position: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Self {
        let pos_v = Vec3::from(position);
        {
            let mut state = self.object.state.write();
            state.view = Mat4::look_at_rh(pos_v, Vec3::from(target), Vec3::from(up));
            state.position = pos_v;
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
    /// freed. Also refreshes the `camera_snapshot` on every Pass that
    /// absorbed this Camera so the renderer can read world-space eye
    /// position and view matrix without going through a shader uniform.
    fn propagate(&self) {
        // One state read computes the three values every consumer needs;
        // the public getters each take their own lock, but the propagate
        // hot path stays at a single acquisition.
        let (view, view_proj, position) = {
            let s = self.object.state.read();
            (
                s.view.to_cols_array_2d(),
                (s.proj * s.view).to_cols_array_2d(),
                s.position.to_array(),
            )
        };
        let snapshot = CameraSnapshot { view };
        let mut attached = self.object.attached.write();
        attached.retain(|weak| {
            if let Some(shader) = weak.upgrade() {
                let _ = shader.set("camera.view_proj", view_proj);
                let _ = shader.set("camera.position", position);
                true
            } else {
                false
            }
        });
        let mut attached_passes = self.object.attached_passes.write();
        attached_passes.retain(|weak| {
            if let Some(pass) = weak.upgrade() {
                *pass.camera_snapshot.write() = Some(snapshot);
                true
            } else {
                false
            }
        });
    }
}

impl SceneObject for Camera {
    fn attach(&self, pass: &crate::Pass) -> Result<(), crate::PassError> {
        // One state read produces every snapshot the attach path needs
        // (view, view_proj, position). The cached locals feed both the
        // per-shader push and the Pass camera snapshot.
        let (view, view_proj, position) = {
            let s = self.object.state.read();
            (
                s.view.to_cols_array_2d(),
                (s.proj * s.view).to_cols_array_2d(),
                s.position.to_array(),
            )
        };
        // Apply the cached values to every shader already on the pass.
        let shaders: Vec<Arc<ShaderObject>> = pass.object.shaders.read().iter().cloned().collect();
        {
            let mut attached = self.object.attached.write();
            for s in shaders {
                let _ = s.set("camera.view_proj", view_proj);
                let _ = s.set("camera.position", position);
                attached.push(Arc::downgrade(&s));
            }
        }
        // Stamp the Pass's camera snapshot so the renderer can sort translucent
        // draws back-to-front without going through `Shader::get`. Register a
        // Weak<PassObject> on the Camera so subsequent `Camera::look_at` mutations
        // refresh the snapshot in step with every absorbing Pass.
        *pass.object.camera_snapshot.write() = Some(CameraSnapshot { view });
        self.object
            .attached_passes
            .write()
            .push(Arc::downgrade(&pass.object));
        // Store a handle so future shaders joining via Model::attach also pick
        // the camera state up (and so the renderer can re-invoke apply on a
        // per-shader basis).
        pass.object
            .scene_objects
            .write()
            .push(Box::new(self.clone()));
        Ok(())
    }

    fn apply_to_shader(&self, shader: &Shader) {
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
        assert!(
            m != identity(),
            "perspective(...) view_proj must not be identity"
        );
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

    fn pbr_triangle_mesh() -> crate::Mesh {
        let mesh = crate::Mesh::new();
        mesh.add_vertex(
            crate::mesh::Vertex::pbr([0.0, 0.5, 0.0]).set(crate::mesh::Vertex::UV0, [0.5, 1.0]),
        );
        mesh
    }

    #[test]
    fn pass_add_seeds_shader_uniforms() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [1.0, 2.0, 3.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

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
    fn mutations_propagate_to_all_pass_shaders() {
        // The same camera absorbed by a pass shows live updates on every
        // shader the pass wires it into, with no second `add` call.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

        camera.look_at([5.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let p: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(p, [5.0, 0.0, 0.0]);
    }

    #[test]
    fn pass_add_before_model_still_reaches_the_new_shader() {
        // Camera added before any models — the pass remembers it and applies
        // to each shader as `add_model` brings them in.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [0.0, 0.0, 7.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        let material = Material::pbr();

        let pass = crate::Pass::new("scene");
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        pass.add(&model).expect("add_model");

        let p: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(p, [0.0, 0.0, 7.0]);
    }

    #[test]
    fn pass_add_stamps_camera_snapshot() {
        // The renderer reads `pass.camera_snapshot` directly when sorting
        // translucent draws — going through `Shader::get("camera.position")`
        // would only work for shaders that happen to expose those uniforms
        // under canonical names. Camera::attach pins the snapshot on the
        // pass and Camera::look_at updates it.
        let pass = crate::Pass::new("scene");
        assert!(pass.object.camera_snapshot.read().is_none());

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).look_at(
            [0.0, 0.0, 5.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        );
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

        let snap = pass
            .object
            .camera_snapshot
            .read()
            .expect("snapshot stamped");
        // View matrix translates world by -5 along +Z (eye at +5 → world origin
        // lands at -5 in eye space). Right-handed look_at_rh: the eye sits at
        // the negation of the view matrix's third column's first three rows.
        // Verify via the translation column (column 3, rows 0..3).
        let translation = snap.view[3];
        assert!((translation[2] + 5.0).abs() < 1.0e-5, "got {translation:?}");

        // look_at should refresh the snapshot
        camera.look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
        let snap2 = pass
            .object
            .camera_snapshot
            .read()
            .expect("snapshot refreshed");
        let translation2 = snap2.view[3];
        assert!(
            (translation2[2] + 10.0).abs() < 1.0e-5,
            "got {translation2:?}"
        );
    }

    #[test]
    fn set_aspect_recomputes_projection_in_place() {
        // Same camera, different aspects → distinct view_proj outputs. The
        // perspective projection's [0][0] term is `1 / (aspect * tan(fovy/2))`
        // so doubling aspect halves the X-axis scale.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let m_before = camera.view_proj();
        camera.set_aspect(2.0);
        let m_after = camera.view_proj();
        assert!(
            (m_before[0][0] - 2.0 * m_after[0][0]).abs() < 1.0e-4,
            "set_aspect should halve X-scale when aspect doubles; before={m_before:?} after={m_after:?}"
        );
    }

    #[test]
    fn set_aspect_on_orthographic_rescales_horizontal_extent() {
        // Orthographic frustums absorb the new aspect by keeping the
        // vertical extent and rescaling the horizontal extents around the
        // current horizontal midpoint. Starting from a centred 2×2 frustum,
        // an aspect of 2.0 should land at a 4×2 frustum (left=-2, right=2).
        let camera = Camera::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
        camera.set_aspect(2.0);
        // The [0][0] term of an orthographic projection is `2 / (right - left)`.
        // For a 4-wide frustum that's 0.5; for the original 2-wide it was 1.0.
        let after = camera.view_proj();
        assert!(
            (after[0][0] - 0.5).abs() < 1.0e-5,
            "expected [0][0] ≈ 0.5 for width-4 ortho, got {}",
            after[0][0]
        );
        // The [1][1] term (vertical scale) stays at 1.0 — height unchanged.
        assert!(
            (after[1][1] - 1.0).abs() < 1.0e-5,
            "vertical scale should be unchanged, got {}",
            after[1][1]
        );
    }

    #[test]
    fn set_aspect_propagates_to_attached_shaders() {
        // After adding a Camera to a pass and rendering one frame, calling
        // set_aspect should update the attached shader's `camera.view_proj`
        // uniform without re-adding.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

        let before: [[f32; 4]; 4] = material.shader().get("camera.view_proj").unwrap();
        camera.set_aspect(2.0);
        let after: [[f32; 4]; 4] = material.shader().get("camera.view_proj").unwrap();
        assert!(
            before != after,
            "set_aspect should propagate a new view_proj to attached shaders"
        );
    }
}
