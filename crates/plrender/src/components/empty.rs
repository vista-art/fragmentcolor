use crate::scene::{macros::spatial_object, node::NodeId, SceneObject};

#[derive(Debug, Default, Clone, Copy)]
pub struct Empty {
    node_id: NodeId,
}

spatial_object!(Empty);

impl Empty {
    pub fn new() -> SceneObject<Self> {
        SceneObject::new(Self::default())
    }
}
