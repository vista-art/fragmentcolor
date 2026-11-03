//! `Renderable`: anything `Renderer::render` takes.

use crate::PassObject;
use std::collections::HashSet;
use std::sync::Arc;

pub trait Renderable {
    /// The passes a render of this runs, in order. Each pass comes once,
    /// after every pass it requires, as `deduplicate` leaves a list.
    fn passes(&self) -> Arc<[Arc<PassObject>]>;
    /// The passes `Pass::require` links when this is a dependency. By default, `passes`.
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        self.passes()
    }
    /// The passes `passes` returns, without the setup a renderable runs first,
    /// such as a Scene's default camera and light. The renderer checks each render
    /// against these before drawing, so a render it refuses leaves the renderable as it was.
    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        self.passes()
    }
}

/// Each pass of `passes` once, at its first place. A list of renderables whose passes
/// each come after the passes they require keeps that order, with no pass run twice.
pub(crate) fn deduplicate(
    passes: impl IntoIterator<Item = Arc<PassObject>>,
) -> Arc<[Arc<PassObject>]> {
    let mut seen = HashSet::new();
    passes
        .into_iter()
        .filter(|pass| seen.insert(pass.id))
        .collect()
}

/// A list renders its items in order, each pass once and after the passes it requires.
impl Renderable for Vec<Box<dyn Renderable>> {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        deduplicate(self.iter().flat_map(|item| item.passes().to_vec()))
    }

    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        let mut roots: Vec<Arc<PassObject>> = Vec::new();
        for item in self {
            roots.extend_from_slice(&item.roots());
        }
        roots.into()
    }

    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        deduplicate(self.iter().flat_map(|item| item.peek_passes().to_vec()))
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

    // Story: the require.md graph builds through a list of renderables, because a
    // list links each item's own pass and not the passes that item requires.
    #[test]
    fn require_links_roots_of_a_list() {
        let color = Pass::new("color");
        let blur_x = Pass::new("blur_x");
        blur_x.require(&color).expect("blur_x requires color");
        let blur_y = Pass::new("blur_y");
        blur_y.require(&blur_x).expect("blur_y requires blur_x");

        let compose = Pass::new("compose");
        compose.require(&color).expect("compose requires color");
        let list: Vec<Box<dyn Renderable>> = vec![Box::new(blur_y.clone())];
        assert_eq!(list.roots().len(), 1);
        compose.require(&list).expect("compose requires blur_y");

        let fan_in = Pass::new("fan_in");
        let both: Vec<Box<dyn Renderable>> = vec![Box::new(color), Box::new(blur_y)];
        fan_in
            .require(&both)
            .expect("fan_in requires color and blur_y");
    }
}
