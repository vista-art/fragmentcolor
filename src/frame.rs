// Reference https://blog.mecheye.net/2023/09/how-to-write-a-renderer-for-modern-apis

use crate::pass::Pass;
use crate::{Renderable, Target};

/// A Frame is a collection of passes that are executed in sequence.
pub struct Frame {
    pub(crate) passes: Vec<Pass>,
    pub(crate) targets: Vec<Target>,
}

impl Frame {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            targets: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, pass: Pass) {
        self.passes.push(pass);
    }

    pub fn add_target(&mut self, target: Target) {
        self.targets.push(target);
    }
}

impl Renderable for Frame {
    fn passes(&self) -> impl IntoIterator<Item = &Pass> {
        &self.passes
    }

    fn targets(&self) -> impl IntoIterator<Item = &Target> {
        &self.targets
    }
}
