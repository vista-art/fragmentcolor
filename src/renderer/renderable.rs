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

/// Uniffi-marshallable union of every type that implements [`Renderable`].
/// Mobile bindings carry a concrete enum because uniffi can't marshal
/// `&impl Renderable`. The Rust core stays generic; this enum exists so
/// Swift / Kotlin can call `renderer.render(shader, target)` (or `pass`,
/// `mesh`, list-of-passes) through a single mobile `render` entry point —
/// the Swift / Kotlin extension files supply natural overloads that wrap
/// the concrete handle into the matching variant invisibly.
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum RenderableHandle {
    Shader(Arc<crate::Shader>),
    Pass(Arc<crate::Pass>),
    Mesh(Arc<crate::Mesh>),
    /// Iterable of `Pass` instances — emits passes in order.
    Passes(Vec<Arc<crate::Pass>>),
}

#[cfg(mobile)]
impl Renderable for RenderableHandle {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        match self {
            Self::Shader(s) => s.passes(),
            Self::Pass(p) => p.passes(),
            Self::Mesh(m) => m.passes(),
            Self::Passes(ps) => {
                let mut all: Vec<Arc<PassObject>> = Vec::new();
                for p in ps {
                    all.extend_from_slice(&p.passes());
                }
                all.into()
            }
        }
    }
}

/// Uniffi-marshallable union of every type that implements [`crate::Target`].
/// Same rationale as [`RenderableHandle`] — uniffi can't marshal
/// `&impl Target`, so the mobile `render` entry takes this enum and the
/// implementation matches and dispatches to the typed `Renderer::render`
/// underneath.
///
/// Both variants are the mobile wrapper types (`MobileWindowTarget` /
/// `MobileTextureTarget`) that carry a `Mutex` / `RwLock` inside, allowing
/// `resize()` to be called on a shared `Arc` from Swift / Kotlin.
#[cfg(mobile)]
#[derive(Debug, Clone, uniffi::Enum)]
pub enum TargetHandle {
    Window(Arc<crate::MobileWindowTarget>),
    Texture(Arc<crate::MobileTextureTarget>),
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
