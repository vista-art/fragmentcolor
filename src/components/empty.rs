use crate::transform::TransformId;

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty {
    pub transform_id: TransformId,
}

impl Empty {
    pub fn new() -> Self {
        Self::default()
    }
}
