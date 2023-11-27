/// Handy macro that implements the SpatialObject trait for the given type
macro_rules! api_object {
    ($type:ty) => {
        impl crate::scene::APIObject for $type {
            // fn transform_id(&self) -> &crate::scene::transform::TransformId {
            //     &self.transform_id
            // }
            // fn set_transform_id(&mut self, transform_id: &crate::scene::transform::TransformId) {
            //     &self.transform_id = transform_id;
            // }
        }
    };
}

pub(crate) use api_object;
