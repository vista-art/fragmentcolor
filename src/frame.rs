// Reference https://blog.mecheye.net/2023/09/how-to-write-a-renderer-for-modern-apis

use crate::pass::Pass;
use crate::Renderable;

#[derive(Debug, Default)]
/// A Frame is a collection of passes that are executed in sequence.
pub struct Frame {
    pub(crate) passes: Vec<Pass>,
}

impl Frame {
    pub fn new() -> Self {
        Self { passes: Vec::new() }
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
