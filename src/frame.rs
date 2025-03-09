// Reference https://blog.mecheye.net/2023/09/how-to-write-a-renderer-for-modern-apis

use crate::pass::Pass;
use crate::Renderable;

#[derive(Debug, Default)]
/// A Frame represents a graph of Passes that are executed in sequence.
pub struct Frame {
    passes: Vec<Pass>,
    _dependencies: Vec<(usize, usize)>, // @TODO implement directed acyclic graph
}

impl Frame {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            _dependencies: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &Pass> {
        &self.passes
    }
}
