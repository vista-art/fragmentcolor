//! Camera — projection + view + cached eye position.
//!
//! Wraps the `proj` + `view` matrix pair every 3D render needs into one
//! object, plus the world-space eye position for shaders that consume it
//! directly (specular highlights, fresnel). A Camera holds Arc-shared state,
//! so the same value can be absorbed by multiple Passes with `pass.add(&camera)`;
//! later mutations (`camera.look_at(...)`) propagate to every shader the
//! Camera has been wired into.

use glam::camera::rh::{proj::directx, view};
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

/// Errors from camera parameters that can't build a finite matrix.
#[derive(Debug, thiserror::Error)]
pub enum CameraError {
    #[error(
        "Camera::perspective needs finite values with 0 < fovy < π, a nonzero aspect, a positive near and far, and near != far, got fovy {fovy}, aspect {aspect}, near {near}, far {far}"
    )]
    Perspective {
        fovy: f32,
        aspect: f32,
        near: f32,
        far: f32,
    },
    #[error(
        "Camera::orthographic needs finite planes with left != right, bottom != top and near != far, got left {left}, right {right}, bottom {bottom}, top {top}, near {near}, far {far}"
    )]
    Orthographic {
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    },
    #[error("Camera::set_aspect needs a finite, nonzero aspect, got {0}")]
    Aspect(f32),
    #[error(
        "Camera::look_at needs finite vectors, a target apart from the position, and an up that isn't parallel to the view, got position {position:?}, target {target:?}, up {up:?}"
    )]
    View {
        position: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
    },
}

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

impl Projection {
    /// Rebuild this projection for a new aspect, with its matrix.
    fn with_aspect(self, aspect: f32) -> Result<(Self, Mat4), CameraError> {
        if !aspect.is_finite() || aspect == 0.0 {
            return Err(CameraError::Aspect(aspect));
        }
        match self {
            Projection::Perspective {
                fovy_radians,
                near,
                far,
            } => Ok((self, build_perspective(fovy_radians, aspect, near, far)?)),
            Projection::Orthographic {
                left,
                right,
                bottom,
                top,
                near,
                far,
            } => {
                let half_width = 0.5 * aspect * (top - bottom);
                let middle = 0.5 * (left + right);
                let (left, right) = (middle - half_width, middle + half_width);
                let proj = build_orthographic(left, right, bottom, top, near, far)?;
                let projection = Projection::Orthographic {
                    left,
                    right,
                    bottom,
                    top,
                    near,
                    far,
                };
                Ok((projection, proj))
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct CameraState {
    projection: Projection,
    view: Mat4,
    proj: Mat4,
    position: Vec3,
}

impl CameraState {
    /// A state with `projection` and its matrix, viewed from the origin.
    fn new(projection: Projection, proj: Mat4) -> Self {
        Self {
            projection,
            view: Mat4::IDENTITY,
            proj,
            position: Vec3::ZERO,
        }
    }
}

/// Return `matrix` when every entry is finite, or `error`.
fn require_finite(matrix: Mat4, error: CameraError) -> Result<Mat4, CameraError> {
    if matrix.is_finite() {
        Ok(matrix)
    } else {
        Err(error)
    }
}

/// Build a perspective matrix, refusing parameters that give no finite one.
/// `near > far` is allowed, since it gives a reverse-Z projection.
fn build_perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Result<Mat4, CameraError> {
    let error = CameraError::Perspective {
        fovy,
        aspect,
        near,
        far,
    };
    let finite = [fovy, aspect, near, far]
        .iter()
        .all(|value| value.is_finite());
    let valid = finite
        && fovy > 0.0
        && fovy < std::f32::consts::PI
        && aspect != 0.0
        && near > 0.0
        && far > 0.0
        && near != far;
    if !valid {
        return Err(error);
    }
    require_finite(directx::perspective(fovy, aspect, near, far), error)
}

/// Build an orthographic matrix, refusing planes that give no finite one.
fn build_orthographic(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Result<Mat4, CameraError> {
    let error = CameraError::Orthographic {
        left,
        right,
        bottom,
        top,
        near,
        far,
    };
    let planes = [left, right, bottom, top, near, far];
    if !planes.iter().all(|plane| plane.is_finite())
        || left == right
        || bottom == top
        || near == far
    {
        return Err(error);
    }
    require_finite(
        directx::orthographic(left, right, bottom, top, near, far),
        error,
    )
}

/// Build a view matrix, refusing vectors that give no finite one.
/// The finite check on the result covers a target on the position, a zero
/// up, and an up parallel to the view, since each one normalizes a zero vector.
fn build_view(position: [f32; 3], target: [f32; 3], up: [f32; 3]) -> Result<Mat4, CameraError> {
    let error = CameraError::View {
        position,
        target,
        up,
    };
    let finite = position
        .iter()
        .chain(&target)
        .chain(&up)
        .all(|value| value.is_finite());
    if !finite {
        return Err(error);
    }
    let view = view::look_at_mat4(Vec3::from(position), Vec3::from(target), Vec3::from(up));
    require_finite(view, error)
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
    pub fn perspective(
        fovy_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Result<Self, CameraError> {
        Self::create_perspective(fovy_radians, aspect, near, far)
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
    ) -> Result<Self, CameraError> {
        Self::create_orthographic(left, right, bottom, top, near, far)
    }

    fn create_perspective(
        fovy_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Result<Self, CameraError> {
        let proj = build_perspective(fovy_radians, aspect, near, far)?;
        let projection = Projection::Perspective {
            fovy_radians,
            near,
            far,
        };
        Ok(Self::from_state(CameraState::new(projection, proj)))
    }

    fn create_orthographic(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> Result<Self, CameraError> {
        let proj = build_orthographic(left, right, bottom, top, near, far)?;
        let projection = Projection::Orthographic {
            left,
            right,
            bottom,
            top,
            near,
            far,
        };
        Ok(Self::from_state(CameraState::new(projection, proj)))
    }

    /// The camera a Scene injects when it has none: a 60° perspective with
    /// a square aspect over [0.1, 100], five units back from the origin and
    /// looking at it. Its constants build finite matrices, so it skips the checks.
    pub(crate) fn stock() -> Self {
        let projection = Projection::Perspective {
            fovy_radians: std::f32::consts::FRAC_PI_3,
            near: 0.1,
            far: 100.0,
        };
        let proj = directx::perspective(std::f32::consts::FRAC_PI_3, 1.0, 0.1, 100.0);
        let position = Vec3::new(0.0, 0.0, 5.0);
        Self::from_state(CameraState {
            projection,
            view: view::look_at_mat4(position, Vec3::ZERO, Vec3::Y),
            proj,
            position,
        })
    }

    /// Update the camera's aspect ratio. The typical caller is a resize
    /// handler: hand the new `width / height` after the window resizes
    /// and the projection matrix recomputes in place, propagating to every
    /// shader (and the Pass camera snapshot used by the transparency sort)
    /// without dropping the Camera handle.
    ///
    /// Behavior by projection kind:
    /// - **Perspective**: rebuilds from `fovy_radians / near / far` with
    ///   the new aspect.
    /// - **Orthographic**: keeps the current vertical extent and rescales
    ///   the horizontal extents so `(right - left) / (top - bottom)`
    ///   matches the new aspect, centered on the existing horizontal
    ///   midpoint. Frustum height stays put.
    ///
    /// Returns a handle to the same Camera (Arc-shared) for chaining. An
    /// aspect that can't build a finite projection leaves the Camera as it was.
    #[lsp_doc("docs/api/scene/camera/set_aspect.md")]
    pub fn set_aspect(&self, aspect: f32) -> Result<Self, CameraError> {
        self.apply_aspect(aspect)
    }

    /// Position the camera in world space. `position` is where the camera
    /// sits, `target` is the point it aims at, `up` is the world-up vector
    /// that orients the roll (typically `[0, 1, 0]`). Matches Light's
    /// `position` vocabulary so both scene types describe "where this
    /// object is in the world" the same way. Returns a handle to the same
    /// Camera (Arc-shared backing) for chaining. Vectors that can't build a
    /// finite view leave the Camera as it was.
    #[lsp_doc("docs/api/scene/camera/look_at.md")]
    pub fn look_at(
        &self,
        position: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
    ) -> Result<Self, CameraError> {
        self.apply_view(position, target, up)
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

    /// Compute the projection for `aspect` before taking the write lock, so a
    /// refused aspect never reaches the state.
    fn apply_aspect(&self, aspect: f32) -> Result<Self, CameraError> {
        let projection = self.object.state.read().projection;
        let (projection, proj) = projection.with_aspect(aspect)?;
        {
            let mut state = self.object.state.write();
            state.projection = projection;
            state.proj = proj;
        }
        self.propagate();
        Ok(self.clone())
    }

    /// Compute the view before taking the write lock, so refused vectors
    /// never reach the state.
    fn apply_view(
        &self,
        position: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
    ) -> Result<Self, CameraError> {
        let view = build_view(position, target, up)?;
        {
            let mut state = self.object.state.write();
            state.view = view;
            state.position = Vec3::from(position);
        }
        self.propagate();
        Ok(self.clone())
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
        let camera =
            Camera::perspective(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 100.0).expect("camera");
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
        let camera = Camera::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0).expect("camera");
        let m = camera.view_proj();
        // Orthographic keeps the bottom-right [3][3] at 1 (no w divide).
        assert!((m[3][3] - 1.0).abs() < 1.0e-5, "got {m:?}");
        assert!(m != identity());
    }

    #[test]
    fn perspective_refuses_parameters_without_a_finite_matrix() {
        for (fovy, aspect, near, far) in [
            (0.0, 1.0, 0.1, 100.0),
            (std::f32::consts::PI, 1.0, 0.1, 100.0),
            (1.0, 0.0, 0.1, 100.0),
            (1.0, f32::NAN, 0.1, 100.0),
            (1.0, 1.0, 0.0, 100.0),
            (1.0, 1.0, 1.0, 1.0),
            (1.0, 1.0, 0.1, f32::INFINITY),
        ] {
            assert!(
                matches!(
                    Camera::perspective(fovy, aspect, near, far),
                    Err(CameraError::Perspective { .. })
                ),
                "perspective({fovy}, {aspect}, {near}, {far}) should fail"
            );
        }
    }

    #[test]
    fn perspective_takes_reverse_z() {
        let camera = Camera::perspective(1.0, 1.0, 100.0, 0.1).expect("near > far is reverse-Z");
        assert!(glam::Mat4::from_cols_array_2d(&camera.view_proj()).is_finite());
    }

    #[test]
    fn orthographic_refuses_planes_without_a_finite_matrix() {
        for [left, right, bottom, top, near, far] in [
            [-1.0, -1.0, -1.0, 1.0, 0.1, 10.0],
            [-1.0, 1.0, 1.0, 1.0, 0.1, 10.0],
            [-1.0, 1.0, -1.0, 1.0, 5.0, 5.0],
            [f32::NAN, 1.0, -1.0, 1.0, 0.1, 10.0],
            [-1.0, 1.0, -1.0, f32::INFINITY, 0.1, 10.0],
        ] {
            assert!(
                matches!(
                    Camera::orthographic(left, right, bottom, top, near, far),
                    Err(CameraError::Orthographic { .. })
                ),
                "orthographic({left}, {right}, {bottom}, {top}, {near}, {far}) should fail"
            );
        }
    }

    #[test]
    fn set_aspect_refuses_an_aspect_without_a_projection() {
        let perspective = Camera::perspective(1.0, 1.0, 0.1, 100.0).expect("camera");
        let orthographic = Camera::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 10.0).expect("camera");
        for camera in [perspective, orthographic] {
            let before = camera.view_proj();
            for aspect in [0.0, f32::NAN, f32::INFINITY] {
                assert!(
                    matches!(camera.set_aspect(aspect), Err(CameraError::Aspect(_))),
                    "set_aspect({aspect}) should fail"
                );
            }
            assert_eq!(
                camera.view_proj(),
                before,
                "a refused aspect changes nothing"
            );
        }
    }

    #[test]
    fn look_at_refuses_a_view_without_a_direction() {
        let camera = Camera::perspective(1.0, 1.0, 0.1, 100.0)
            .expect("camera")
            .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
        let before = camera.view_proj();
        for (position, target, up) in [
            ([1.0, 2.0, 3.0], [1.0, 2.0, 3.0], [0.0, 1.0, 0.0]),
            ([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
            ([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            ([f32::NAN, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
        ] {
            assert!(
                matches!(
                    camera.look_at(position, target, up),
                    Err(CameraError::View { .. })
                ),
                "look_at({position:?}, {target:?}, {up:?}) should fail"
            );
        }
        assert_eq!(camera.view_proj(), before, "a refused view changes nothing");
        assert_eq!(camera.position(), [0.0, 0.0, 5.0]);
    }

    #[test]
    fn look_at_changes_view_component() {
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).expect("camera");
        let before = camera.view_proj();
        camera
            .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
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
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .expect("camera")
            .look_at([1.0, 2.0, 3.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
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
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).expect("camera");
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());

        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

        camera
            .look_at([5.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
        let p: [f32; 3] = material.shader().get("camera.position").unwrap();
        assert_eq!(p, [5.0, 0.0, 0.0]);
    }

    #[test]
    fn pass_add_before_model_still_reaches_the_new_shader() {
        // Camera added before any models — the pass remembers it and applies
        // to each shader as `add_model` brings them in.
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .expect("camera")
            .look_at([0.0, 0.0, 7.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
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

        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0)
            .expect("camera")
            .look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
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
        camera
            .look_at([0.0, 0.0, 10.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])
            .expect("camera");
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
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).expect("camera");
        let m_before = camera.view_proj();
        camera.set_aspect(2.0).expect("camera");
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
        // current horizontal midpoint. Starting from a centered 2×2 frustum,
        // an aspect of 2.0 should land at a 4×2 frustum (left=-2, right=2).
        let camera = Camera::orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0).expect("camera");
        camera.set_aspect(2.0).expect("camera");
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
        let camera = Camera::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0).expect("camera");
        let material = Material::pbr();
        let model = crate::scene::Model::new(pbr_triangle_mesh(), material.clone());
        let pass = crate::Pass::new("scene");
        pass.add(&model).expect("add_model");
        pass.add(&camera)
            .expect("camera attach is infallible in unit tests");

        let before: [[f32; 4]; 4] = material.shader().get("camera.view_proj").unwrap();
        camera.set_aspect(2.0).expect("camera");
        let after: [[f32; 4]; 4] = material.shader().get("camera.view_proj").unwrap();
        assert!(
            before != after,
            "set_aspect should propagate a new view_proj to attached shaders"
        );
    }
}
