use crate::PassObject;
use std::sync::Arc;

pub trait Renderable {
    // Return an Arc to a slice of Arc<PassObject> for fast, lock-free iteration.
    fn passes(&self) -> Arc<[Arc<PassObject>]>;
    // Root node(s) of this renderable when used as a dependency. By default, same as passes().
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.passes()
    }
}

impl Renderable for Vec<Box<dyn Renderable>> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        let mut all_passes: Vec<Arc<PassObject>> = Vec::new();
        for r in self {
            all_passes.extend_from_slice(&r.passes());
        }
        all_passes.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pass;

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
