/// Handy macro that implements the SpatialObject trait for the given type
macro_rules! spatial_object {
    ($type:ty) => {
        impl crate::scene::SpatialObject for $type {
            fn node_id(&self) -> crate::scene::node::NodeId {
                self.node_id
            }
            fn set_node_id(&mut self, node_id: crate::scene::node::NodeId) {
                self.node_id = node_id;
            }
        }
    };
}

pub(crate) use spatial_object;
