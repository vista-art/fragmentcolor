/// Handy macro that implements the SpatialObject trait for the given type
macro_rules! spatial_object {
    ($type:ty) => {
        impl crate::scene::SceneObject for $type {
            fn transform_id(&self) -> crate::scene::transform::TransformId {
                self.transform_id
            }
            fn set_transform_id(&mut self, transform_id: crate::scene::transform::TransformId) {
                self.transform_id = transform_id;
            }
        }
    };
}

pub(crate) use spatial_object;
