use crate::{Pass, PassObject};
use std::sync::Arc;

pub trait Renderable {
    // Return an Arc to a slice of Arc<PassObject> for fast, lock-free iteration.
    fn passes(&self) -> Arc<[Arc<PassObject>]>;
    // Root node(s) of this renderable when used as a dependency. By default, same as passes().
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.passes()
    }
}

// Sequential lists: do not expand dependencies; return the listed passes in order.
impl Renderable for &[Pass] {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

impl Renderable for Vec<Pass> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

impl Renderable for &[&Pass] {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

impl Renderable for Vec<&Pass> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().map(|pass| pass.object.clone()).collect();
        v.into()
    }
}

// Provide convenience for direct Arc containers if needed.
impl Renderable for &[Arc<PassObject>] {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let v: Vec<Arc<PassObject>> = self.iter().cloned().collect();
        v.into()
    }
}

impl Renderable for Vec<Arc<PassObject>> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        self.clone().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Story: Different containers should expose pass Arcs consistently via Renderable.
    #[test]
    fn exposes_passes_from_various_containers() {
        // Arrange
        let pass_one = Pass::new("p1");
        let pass_two = Pass::new("p2");

        // Act / Assert: slice of Pass
        let arr = [pass_one.clone(), pass_two.clone()];
        let n = (&arr[..]).passes().len();
        assert_eq!(n, 2);

        // Act / Assert: Vec<Pass>
        let v = vec![pass_one.clone(), pass_two.clone()];
        let n = v.passes().len();
        assert_eq!(n, 2);

        // Act / Assert: Vec<&Pass>
        let v = vec![&pass_one, &pass_two];
        let n = v.passes().len();
        assert_eq!(n, 2);
    }
}
