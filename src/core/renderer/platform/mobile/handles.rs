//! `RenderableHandle`, `TargetHandle` and `SceneObjectHandle`.

use crate::{PassObject, Renderable};
use std::sync::Arc;

/// Uniffi-marshallable union of every type that implements [`Renderable`].
/// Mobile bindings carry a concrete enum because uniffi can't marshal
/// `&impl Renderable`. The Rust core stays generic; this enum exists so
/// Swift / Kotlin can call `renderer.render(shader, target)` (or `pass`,
/// `mesh`, list-of-passes) through a single mobile `render` entry point —
/// the Swift / Kotlin extension files supply natural overloads that wrap
/// the concrete handle into the matching variant invisibly.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum RenderableHandle {
    Shader(Arc<crate::Shader>),
    Pass(Arc<crate::Pass>),
    Mesh(Arc<crate::Mesh>),
    /// Top-level [`Scene`](crate::Scene) — emits its ordered pass graph,
    /// including the default Pass that holds the scene's `SceneObject`s.
    Scene(Arc<crate::Scene>),
    /// Iterable of `Pass` instances — emits passes in order.
    Passes(Vec<Arc<crate::Pass>>),
}

impl Renderable for RenderableHandle {
    fn passes(&self) -> Arc<[Arc<PassObject>]> {
        match self {
            Self::Shader(s) => s.passes(),
            Self::Pass(p) => p.passes(),
            Self::Mesh(m) => m.passes(),
            Self::Scene(s) => s.passes(),
            Self::Passes(ps) => {
                let mut all: Vec<Arc<PassObject>> = Vec::new();
                for p in ps {
                    all.extend_from_slice(&p.passes());
                }
                all.into()
            }
        }
    }

    fn peek_passes(&self) -> Arc<[Arc<PassObject>]> {
        match self {
            Self::Shader(s) => s.peek_passes(),
            Self::Pass(p) => p.peek_passes(),
            Self::Mesh(m) => m.peek_passes(),
            Self::Scene(s) => s.peek_passes(),
            Self::Passes(ps) => {
                let mut all: Vec<Arc<PassObject>> = Vec::new();
                for p in ps {
                    all.extend_from_slice(&p.peek_passes());
                }
                all.into()
            }
        }
    }

    /// The handle's own passes, without the passes they require,
    /// so `require` links each dependency once.
    fn roots(&self) -> Arc<[Arc<PassObject>]> {
        match self {
            Self::Shader(s) => s.roots(),
            Self::Pass(p) => p.roots(),
            Self::Mesh(m) => m.roots(),
            Self::Scene(s) => s.roots(),
            Self::Passes(ps) => {
                let mut roots: Vec<Arc<PassObject>> = Vec::new();
                for p in ps {
                    roots.extend_from_slice(&p.roots());
                }
                roots.into()
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
#[derive(Debug, Clone, uniffi::Enum)]
pub enum TargetHandle {
    Window(Arc<crate::MobileWindowTarget>),
    Texture(Arc<crate::MobileTextureTarget>),
}

/// Uniffi-marshallable union of every type that implements
/// [`crate::scene::SceneObject`]. Mobile bindings carry a concrete enum
/// because uniffi can't marshal `&impl SceneObject`. The Rust core stays
/// generic; this enum exists so Swift / Kotlin can call `scene.add(...)`
/// or `pass.add(...)` through a single mobile entry point — the Swift /
/// Kotlin extension files supply natural overloads that wrap the
/// concrete handle into the matching variant invisibly.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SceneObjectHandle {
    Model(Arc<crate::Model>),
    Camera(Arc<crate::scene::Camera>),
    Light(Arc<crate::scene::Light>),
}
