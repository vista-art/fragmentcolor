use crate::{
    math::geometry::Quad,
    scene::{macros::spatial_object, node::NodeId, SceneObject},
};
use serde::{Deserialize, Serialize};

/// The Projection type (orthographic or perspective).
///
/// The projection is used to convert the Scene world
/// coordinates into 2D screen normalized coordinates.
///
/// Scene coordinates are arbitrary and depend on what
/// the user wants to represent. In a typical 2D scene,
/// it normally means Pixels, but it could also be any
/// other unit, like meters, milimeters, parsecs, etc.
///
/// GPU coordinates are normalized on both axes, with
/// the origin (0.0, 0.0) at the center of the screen.
#[derive(Clone, Debug, Copy, Serialize, Deserialize)]
pub enum Projection {
    /// Holds Orthographic projection data.
    Orthographic {
        /// The center of the projection in Scene coordinates.
        ///
        /// For a typical 2D scene, this normally means
        /// the center of the Window / Canvas in pixels.
        center: mint::Vector2<f32>,

        /// Vertical extent from the center point.
        ///
        /// The height is double the extent.
        /// The width is derived from the height
        /// based on the current target aspect ratio.
        ///
        /// ````
        /// _____________________________
        /// | extent_y -> |             |
        /// |             |             |
        /// |             |             |
        /// |             x (center)    |
        /// |                           |
        /// |                           |
        /// |___________________________|
        /// ````
        extent_y: f32,
    },
    Perspective {
        /// Vertical field of View, in degrees.
        ///
        /// The horizontal FOV is computed based on the Target's aspect ratio.
        fov_y: f32,
    },
}

/// Options for creating a new Projection.
///
/// This format is convenient for the Python and Javascript wrappers.
/// Rust users can instantiate the Projection directly instead.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProjectionOptions {
    /// The projection type: "orthographic" (default) or "perspective".
    ///
    /// Type aliases:
    /// - "ortho" or "2d" for orthographic
    /// - "3d" for perspective
    projection: String,

    /// The size of the screen in pixels.
    target_size: Quad,

    /// Vertical field of View, in degrees (perspective only).
    /// This value is ignored for orthographic projections.
    ///
    /// The horizontal FOV is computed based on the Target's aspect ratio.
    vertical_fov: f32,
}

impl Projection {
    /// Creates a new Projection from options.
    pub fn new(options: ProjectionOptions) -> Self {
        match options.projection.to_lowercase().as_str() {
            "orthographic" | "ortho" | "2d" => Self::ortographic_from_quad(options.target_size),
            "perspective" | "3d" => Self::perspective(options.vertical_fov),
            _ => {
                log::warn!("Unknown projection type: {}", options.projection.as_str());
                log::warn!("Defaulting to orthographic projection.");
                Self::ortographic_from_quad(options.target_size)
            }
        }
    }

    /// Creates a new Orthographic projection from a Quad.
    pub fn ortographic_from_quad(quad: Quad) -> Self {
        let center = mint::Vector2 {
            x: quad.width() as f32 / 2.0,
            y: quad.height() as f32 / 2.0,
        };
        let extent_y = quad.height() as f32 / 2.0;
        Self::Orthographic { center, extent_y }
    }

    /// Creates a new Orthographic projection.
    pub fn orthographic(center: mint::Vector2<f32>, extent_y: f32) -> Self {
        Self::Orthographic { center, extent_y }
    }

    /// Creates a new Perspective projection.
    pub fn perspective(fov_y: f32) -> Self {
        Self::Perspective { fov_y }
    }
}

/// A Camera is the link between the Scene and the Renderer.
///
/// It contains the inputs for building a projection matrix,
/// the near and far clip distances in Scene units, and the
/// reference for the Scene's Node that owns the camera.
///
/// The Scene's Node contains the camera's position and
/// orientation in the Scene space.
#[derive(Clone, Debug, Copy)]
pub struct Camera {
    /// The projection type (orthographic or perspective).
    pub projection: Projection,

    /// Specify the depth range as seen by the camera.
    /// `z_near` maps to 0.0, and .z_far` maps to 1.0.
    pub z_near: f32,

    /// Specify the depth range as seen by the camera.
    /// `z_near` maps to 0.0, and .z_far` maps to 1.0.
    pub z_far: f32,

    /// A reference to the Node that owns this camera,
    /// containing its position and orientation in the
    /// Scene space. Set to NodeId::root() by default,
    /// which means the camera is at the origin.
    pub node_id: NodeId,
}

spatial_object!(Camera);

impl Default for Camera {
    /// Creates a 2D Camera with default options.
    fn default() -> Self {
        Self {
            projection: Projection::Orthographic {
                center: mint::Vector2 { x: 0.0, y: 0.0 },
                extent_y: 1.0,
            },
            z_near: 0.0,
            z_far: 1.0,
            node_id: NodeId::root(),
        }
    }
}

/// Options for creating a new Camera.
///
/// This version is convenient for the Python and Javascript wrappers.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CameraOptions {
    /// The projection type (orthographic or perspective).
    pub projection: Projection,
    /// Specify the depth range as seen by the camera.
    /// `z_near` maps to 0.0, and .z_far` maps to 1.0.
    pub z_near: f32,
    /// Specify the depth range as seen by the camera.
    /// `z_near` maps to 0.0, and .z_far` maps to 1.0.
    pub z_far: f32,
}

impl Camera {
    /// Creates a new Camera from options.
    pub fn new(options: CameraOptions) -> SceneObject<Self> {
        SceneObject::new(Camera {
            projection: options.projection,
            z_near: options.z_near,
            z_far: options.z_far,
            node_id: NodeId::root(),
        })
    }

    pub fn new_perspective(fov_y: f32) -> SceneObject<Self> {
        SceneObject::new(Camera {
            projection: Projection::Perspective { fov_y },
            z_near: 0.0,
            z_far: 1.0,
            node_id: NodeId::root(),
        })
    }

    /// Creates a new 2D Camera from the Target's size.
    pub fn from_target_size(quad: Quad) -> SceneObject<Self> {
        let projection = Projection::ortographic_from_quad(quad);
        SceneObject::new(Camera {
            projection,
            z_near: 0.0,
            z_far: 1.0,
            node_id: NodeId::root(),
        })
    }

    /// Sets the camera's projection type.
    pub fn set_projection(&mut self, projection: Projection) -> &mut Self {
        self.projection = projection;
        self
    }

    /// Sets the camera's near clip distances.
    pub fn set_near_plane(&mut self, z_near: f32) -> &mut Self {
        self.z_near = z_near;
        self
    }

    /// Sets the camera's far clip distance.
    pub fn set_far_plane(&mut self, z_far: f32) -> &mut Self {
        self.z_far = z_far;
        self
    }

    /// This function is used by the RenderPass
    /// to get the camera's projection matrix.
    pub(crate) fn projection_matrix(&self, aspect: f32) -> mint::ColumnMatrix4<f32> {
        let matrix = match self.projection {
            Projection::Orthographic { center, extent_y } => {
                let extent_x = aspect * extent_y;

                glam::Mat4::orthographic_rh(
                    center.x - extent_x,
                    center.x + extent_x,
                    center.y - extent_y,
                    center.y + extent_y,
                    self.z_near,
                    self.z_far,
                )
            }
            Projection::Perspective { fov_y } => {
                let fov = fov_y.to_radians();

                if self.z_far == f32::INFINITY && self.z_near == f32::INFINITY {
                    log::warn!(
                        "{} {}",
                        "Camera z_near and z_far are both infinite.",
                        "Returning a default orthographic projection matrix."
                    );
                    glam::Mat4::orthographic_rh(-1.0, 1.0, -1.0, 1.0, self.z_near, self.z_far)
                } else if self.z_near == f32::INFINITY {
                    glam::Mat4::perspective_infinite_rh(fov, aspect, self.z_near)
                } else if self.z_near == f32::INFINITY {
                    glam::Mat4::perspective_infinite_reverse_rh(fov, aspect, self.z_far)
                } else {
                    glam::Mat4::perspective_rh(fov, aspect, self.z_near, self.z_far)
                }
            }
        };
        matrix.into()
    }
}
