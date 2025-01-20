pub mod transform {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct TransformId(pub usize);

    impl TransformId {
        pub fn root() -> Self {
            Self(0)
        }
    }

    pub struct Transform {
        pub position: glam::Vec3,
        pub rotation: glam::Quat,
        pub scale: glam::Vec3,
    }
}

pub type ObjectId = hecs::Entity;

pub struct Scene {
    pub objects: hecs::World,
    pub nodes: Vec<transform::TransformId>,
}
