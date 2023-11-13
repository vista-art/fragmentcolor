use crate::scene::node::NodeId;
use std::ops;

#[derive(Clone, Debug)]
pub enum Projection {
    Orthographic {
        /// The center of the projection.
        center: mint::Vector2<f32>,
        /// Vertical extent from the center point. The height is double the extent.
        /// The width is derived from the height based on the current aspect ratio.
        extent_y: f32,
    },
    Perspective {
        /// Vertical field of view, in degrees.
        /// Note: the horizontal FOV is computed based on the aspect.
        fov_y: f32,
    },
}

#[derive(Clone, Debug)]
pub struct Camera {
    pub projection: Projection,
    /// Specify the depth range as seen by the camera.
    /// `depth.start` maps to 0.0, and `depth.end` maps to 1.0.
    pub depth: ops::Range<f32>,
    pub node_id: NodeId,
    pub background: crate::Color,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            projection: Projection::Orthographic {
                center: mint::Vector2 { x: 0.0, y: 0.0 },
                extent_y: 1.0,
            },
            depth: 0.0..1.0,
            node_id: NodeId::root(),
            background: crate::Color::default(),
        }
    }
}

impl Camera {
    pub fn projection_matrix(&self, aspect: f32) -> mint::ColumnMatrix4<f32> {
        let matrix = match self.projection {
            Projection::Orthographic { center, extent_y } => {
                let extent_x = aspect * extent_y;
                glam::Mat4::orthographic_rh(
                    center.x - extent_x,
                    center.x + extent_x,
                    center.y - extent_y,
                    center.y + extent_y,
                    self.depth.start,
                    self.depth.end,
                )
            }
            Projection::Perspective { fov_y } => {
                let fov = fov_y.to_radians();
                if self.depth.end == f32::INFINITY {
                    assert!(self.depth.start.is_finite());
                    glam::Mat4::perspective_infinite_rh(fov, aspect, self.depth.start)
                } else if self.depth.start == f32::INFINITY {
                    glam::Mat4::perspective_infinite_reverse_rh(fov, aspect, self.depth.end)
                } else {
                    glam::Mat4::perspective_rh(fov, aspect, self.depth.start, self.depth.end)
                }
            }
        };
        matrix.into()
    }
}
