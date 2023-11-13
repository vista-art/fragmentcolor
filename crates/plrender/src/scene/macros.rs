/// Handy macro that implements the HasNodeId trait for the given type
macro_rules! has_node_id {
    ($type:ty) => {
        impl crate::scene::node::HasNodeId for $type {
            fn node_id(&self) -> crate::scene::node::NodeId {
                self.node_id
            }
            fn set_node_id(&mut self, node_id: crate::scene::node::NodeId) {
                self.node_id = node_id;
            }
        }
    };
}

pub(crate) use has_node_id;
