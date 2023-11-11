use winit::event::*;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

#[derive(Clone, Debug)]
pub enum Projection {
    Orthographic {
        /// The center of the projection.
        center: cgmath::Vector2<f32>,
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

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: super::NodeId,
    pub target: super::NodeId,
    pub up: cgmath::Vector3<f32>,
    pub projection: Projection,
    pub aspect: f32,
    pub near: f32,
    pub far: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            eye: super::NodeId::root(),

            projection: Projection::Orthographic {
                center: cgmath::Vector2 { x: 0.0, y: 0.0 },
                extent_y: 1.0,
            },
            depth: 0.0..1.0,
            node: super::NodeId::root(),
        }
    }
}

impl Camera {
    pub fn projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let matrix = match self.projection {
            Projection::Orthographic { center, extent_y } => {
                let extent_x = self.aspect * extent_y;
                cgmath::ortho(
                    center.x - extent_x,
                    center.x + extent_x,
                    center.y - extent_y,
                    center.y + extent_y,
                    self.near,
                    self.far,
                )
            }
            Projection::Perspective { fov_y } => {
                let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

                let projection =
                    cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

                return OPENGL_TO_WGPU_MATRIX * projection * view;
            }
        };
    }
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);

        let projection =
            cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        return OPENGL_TO_WGPU_MATRIX * projection * view;
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
