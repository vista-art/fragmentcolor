use crate::RenderContext;
use crate::texture::{TextureId, TextureObject};
use std::collections::BTreeMap;
use std::sync::{Arc, Weak};

/// A texture's entry in its renderer's registry.
///
/// Every `Texture` handle shares one, and so do the `TextureTarget` that owns
/// the texture and each shader uniform or pass target that binds it. The entry
/// leaves the registry when the last of them drops, or on `unregister_texture`.
#[derive(Debug)]
pub(crate) struct Registration {
    id: TextureId,
    context: Weak<RenderContext>,
}

type Index = BTreeMap<u64, Weak<Registration>>;

// Live registrations by texture id. A shader binding holds only an id,
// so it finds the registration to keep here. Ids are unique across renderers.
#[cfg(not(wasm))]
static LIVE: parking_lot::Mutex<Index> = parking_lot::const_mutex(BTreeMap::new());

#[cfg(wasm)]
thread_local! {
    static LIVE: std::cell::RefCell<Index> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

fn with_live<T>(action: impl FnOnce(&mut Index) -> T) -> T {
    #[cfg(not(wasm))]
    {
        action(&mut LIVE.lock())
    }
    #[cfg(wasm)]
    {
        LIVE.with(|live| action(&mut live.borrow_mut()))
    }
}

impl Registration {
    /// Adds `object` to the registry of `context` under a new id.
    pub(crate) fn create(context: &Arc<RenderContext>, object: Arc<TextureObject>) -> Arc<Self> {
        let id = context.register_texture(object);
        let registration = Arc::new(Self {
            id,
            context: Arc::downgrade(context),
        });
        with_live(|live| live.insert(id.id, Arc::downgrade(&registration)));
        registration
    }

    /// The registration of the texture `id` names, while anything holds it.
    pub(crate) fn find(id: TextureId) -> Option<Arc<Self>> {
        with_live(|live| live.get(&id.id).and_then(Weak::upgrade))
    }

    pub(crate) fn id(&self) -> &TextureId {
        &self.id
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        with_live(|live| live.remove(&self.id.id));
        if let Some(context) = self.context.upgrade() {
            context.release_texture(&self.id);
        }
    }
}
