use crate::scene::{macros::spatial_object, transform::TransformId, Object};

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty {
    transform_id: TransformId,
}

spatial_object!(Empty);

impl Empty {
    pub fn new() -> Object<Self> {
        Object::new(Self::default())
    }
}
