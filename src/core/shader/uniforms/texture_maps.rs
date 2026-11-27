//! Texture uniforms a shader fills from sources it keeps, so every Renderer can draw them.
//!
//! A map keeps its source on the CPU and one id for its key. Each Renderer that loads
//! it uploads a copy of its own into its registry under that id, so the uniform names
//! the right texture in every Renderer, and many Renderers hold a copy at once.
//! Slots of several shaders may share one map's texture, and each Renderer then holds
//! one copy for all of them.

use crate::RenderContext;
use crate::core::shader::{ShaderError, ShaderObject, UniformData};
use crate::texture::{TextureId, TextureMeta, TextureObject, TextureSource};
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::mem::replace;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

/// The ids of the map textures alive now. A texture's copies go when the last slot
/// that holds it lets go, so a slot that doesn't hold it refuses its id (`check_bind`).
static LIVE: Mutex<BTreeSet<u64>> = parking_lot::const_mutex(BTreeSet::new());

/// A texture uniform filled from a source the shader keeps.
#[derive(Debug)]
pub(crate) struct TextureMap {
    pub(crate) key: String,
    /// The texture the slot samples, which the slots of other maps may share.
    pub(crate) texture: Arc<MapTexture>,
}

/// The source of one or more map slots, and the copy each Renderer that loaded it holds.
///
/// Every slot that samples it holds it, under one id, so a Renderer uploads one copy
/// for all of them. The last slot to let go frees every copy.
#[derive(Debug)]
pub(crate) struct MapTexture {
    /// Taken when the texture is made and kept for its life,
    /// so every slot that holds it names the same copy in every Renderer.
    pub(crate) id: TextureId,
    state: Mutex<MapState>,
}

/// What a map texture uploads from, and the contexts that hold a copy of it.
#[derive(Debug)]
struct MapState {
    source: TextureSource,
    /// The setter call that gave `source`. A later call takes a newer one.
    stamp: u64,
    /// The contexts that hold a copy, each with the stamp it came from.
    uploads: Vec<Upload>,
}

/// One context's copy of a map texture.
#[derive(Debug)]
struct Upload {
    /// `RenderContext::id`, as a mesh tags its buffers.
    owner: u64,
    context: Weak<RenderContext>,
    stamp: u64,
}

/// What a new map lets go of: the source it replaced, or the texture the slot held,
/// which the caller drops after its locks.
type Replaced = (Option<TextureSource>, Option<Arc<MapTexture>>);

impl Drop for MapTexture {
    /// Each context still alive drops its copy.
    fn drop(&mut self) {
        #[cfg(test)]
        crate::testing::race_hooks::run("map dropped");
        LIVE.lock().remove(&self.id.id);
        for upload in &self.state.get_mut().uploads {
            if let Some(context) = upload.context.upgrade() {
                context.release_texture(&self.id);
            }
        }
    }
}

impl MapTexture {
    /// A texture of `source` under a new id, which no context holds a copy of yet.
    fn create(source: TextureSource, stamp: u64) -> Arc<Self> {
        let id = TextureId::create();
        LIVE.lock().insert(id.id);
        let state = MapState {
            source,
            stamp,
            uploads: Vec::new(),
        };
        Arc::new(Self {
            id,
            state: Mutex::new(state),
        })
    }

    /// A texture of `source` under a new id. Each context that holds a copy of this one
    /// draws that copy under the new id too, until a load replaces it.
    fn branch(&self, source: TextureSource, stamp: u64) -> Arc<Self> {
        let texture = Self::create(source, stamp);
        let held = self.state.lock();
        let carried = held.uploads.iter().filter_map(|upload| {
            let context = upload.context.upgrade()?;
            let copy = context.get_texture(&self.id)?;
            drop(context.insert_texture(texture.id, copy));
            Some(Upload {
                owner: upload.owner,
                context: upload.context.clone(),
                stamp: upload.stamp,
            })
        });
        let carried: Vec<Upload> = carried.collect();
        drop(held);
        texture.state.lock().uploads = carried;
        texture
    }

    /// The source the texture uploads from now.
    pub(crate) fn read_source(&self) -> TextureSource {
        self.state.lock().source.clone()
    }
}

impl MapState {
    /// Whether `context` holds a copy of the latest source under `id`, still in its registry.
    /// A copy that `unregister_texture` removed uploads again.
    fn is_loaded(&self, id: TextureId, context: &RenderContext) -> bool {
        let current = self
            .uploads
            .iter()
            .any(|upload| upload.owner == context.id && upload.stamp == self.stamp);
        current && context.has_texture(&id)
    }

    /// Whether an upload of the source from `stamp` should be `context`'s copy under `id`:
    /// it is the latest, or the context holds no copy, or only one from an older set.
    /// Stamps only grow, so an older upload never replaces a newer copy that another load kept first.
    fn takes(&self, id: TextureId, stamp: u64, context: &RenderContext) -> bool {
        if stamp == self.stamp || !context.has_texture(&id) {
            return true;
        }
        let held = self
            .uploads
            .iter()
            .find(|upload| upload.owner == context.id);
        held.is_none_or(|upload| upload.stamp < stamp)
    }
}

/// An upload that `load` runs, read under the lock and run outside it.
pub(crate) struct MapJob {
    /// The texture to keep the upload in, while a slot still holds it.
    texture: Weak<MapTexture>,
    pub(crate) id: TextureId,
    pub(crate) stamp: u64,
    pub(crate) source: TextureSource,
}

impl MapJob {
    /// Keeps `object`, which this job uploaded, as `context`'s copy of the map texture.
    ///
    /// It refuses the copy when no slot holds the texture or the context holds a newer
    /// one. An upload that a later set overtook stands in until the next load uploads
    /// the latest, so a stream of sets can't leave a Renderer without a copy. It
    /// returns whether it kept the copy. A copy it replaces drops after the texture's lock.
    pub(crate) fn keep_upload(&self, context: &Arc<RenderContext>, object: TextureObject) -> bool {
        let Some(texture) = self.texture.upgrade() else {
            return false;
        };
        let replaced = {
            let mut state = texture.state.lock();
            if !state.takes(texture.id, self.stamp, context) {
                return false;
            }
            let replaced = context.insert_texture(texture.id, Arc::new(object));
            let alive = |upload: &Upload| upload.context.strong_count() > 0;
            state
                .uploads
                .retain(|upload| upload.owner != context.id && alive(upload));
            state.uploads.push(Upload {
                owner: context.id,
                context: Arc::downgrade(context),
                stamp: self.stamp,
            });
            replaced
        };
        drop(replaced);
        true
    }

    /// Whether this job still uploads the latest source of its texture.
    pub(crate) fn is_current(&self) -> bool {
        let Some(texture) = self.texture.upgrade() else {
            return false;
        };
        let stamp = texture.state.lock().stamp;
        stamp == self.stamp
    }
}

/// Takes a stamp newer than every stamp before it.
fn take_stamp() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

/// Gives `key`'s map `source` and a new stamp, or makes the key a map with a new id.
///
/// A texture that only this slot holds keeps its id, so each Renderer draws its old copy
/// until a load replaces it. A texture another slot shares stays as it is for that slot,
/// and this one takes a new texture, which draws the old copy until then.
fn keep_source(
    maps: &mut Vec<TextureMap>,
    key: &str,
    source: TextureSource,
) -> (TextureId, Replaced) {
    let stamp = take_stamp();
    let Some(map) = maps.iter_mut().find(|map| map.key == key) else {
        let texture = MapTexture::create(source, stamp);
        let id = texture.id;
        maps.push(TextureMap {
            key: key.to_string(),
            texture,
        });
        return (id, (None, None));
    };
    if Arc::strong_count(&map.texture) > 1 {
        let texture = map.texture.branch(source, stamp);
        let id = texture.id;
        return (id, (None, Some(replace(&mut map.texture, texture))));
    }
    let mut state = map.texture.state.lock();
    state.stamp = stamp;
    let replaced = replace(&mut state.source, source);
    (map.texture.id, (Some(replaced), None))
}

/// Makes `key` a map of `texture`, in place of the texture the key held.
fn hold_texture(
    maps: &mut Vec<TextureMap>,
    key: &str,
    texture: Arc<MapTexture>,
) -> (TextureId, Replaced) {
    let id = texture.id;
    let Some(map) = maps.iter_mut().find(|map| map.key == key) else {
        maps.push(TextureMap {
            key: key.to_string(),
            texture,
        });
        return (id, (None, None));
    };
    (id, (None, Some(replace(&mut map.texture, texture))))
}

impl ShaderObject {
    /// Makes `key` a map of `source`, which each Renderer that draws the shader uploads.
    /// The key keeps its id when it already is a map that no other slot shares (`keep_source`).
    pub(crate) fn set_map(&self, key: &str, source: TextureSource) -> Result<(), ShaderError> {
        self.write_map(key, |maps| keep_source(maps, key, source))
    }

    /// Makes `key` a map of `texture`, another map's, so both slots name one id and each
    /// Renderer holds one copy for them. A Renderer that loaded the texture draws it at once.
    pub(crate) fn share_map(&self, key: &str, texture: Arc<MapTexture>) -> Result<(), ShaderError> {
        self.write_map(key, |maps| hold_texture(maps, key, texture))
    }

    /// Makes `key` a map through `place`, which returns the map's id.
    ///
    /// The map is in place before the slot names its id, so a plan that reads the id
    /// finds the map. An explicit texture the key held is released. What the map
    /// replaced drops after the locks.
    fn write_map(
        &self,
        key: &str,
        place: impl FnOnce(&mut Vec<TextureMap>) -> (TextureId, Replaced),
    ) -> Result<(), ShaderError> {
        self.check_key(key)?;
        let writing = self.texture_writes.lock();
        let (id, replaced) = place(&mut self.maps.write());
        let written = self.write_slot(key, UniformData::Texture(TextureMeta::from(id)));
        let released = self.bound_textures.remove(key);
        drop(writing);
        drop((replaced, released));
        written
    }

    /// Refuses a `key` the shader doesn't declare as a texture. It reads the storage before
    /// the caller takes `texture_writes`, since a flush holds the storage while it waits for it.
    fn check_key(&self, key: &str) -> Result<(), ShaderError> {
        match self.storage.read().shape(key)? {
            UniformData::Texture(_) => Ok(()),
            _ => Err(ShaderError::TypeMismatch(key.into())),
        }
    }

    /// The texture `key`'s map samples, when the key is a map.
    pub(crate) fn find_map(&self, key: &str) -> Option<Arc<MapTexture>> {
        let maps = self.maps.read();
        let map = maps.iter().find(|map| map.key == key);
        map.map(|map| map.texture.clone())
    }

    /// Refuses `id` for `key` when it names a live map texture that `key` doesn't hold.
    /// The texture's copies go when the last slot that holds it lets go, so no other slot can keep them.
    pub(crate) fn check_bind(&self, key: &str, id: TextureId) -> Result<(), ShaderError> {
        let mapped = LIVE.lock().contains(&id.id);
        let owned = || {
            self.maps
                .read()
                .iter()
                .any(|map| map.key == key && map.texture.id == id)
        };
        match !mapped || owned() {
            true => Ok(()),
            false => Err(ShaderError::ForeignMap {
                key: key.into(),
                id: id.id,
            }),
        }
    }

    /// Ends `key`'s map, unless `id` is the map's own, as when a `get` comes back through `set`.
    /// Returns the map, which the caller drops after its locks, and its copies go with it
    /// unless another slot shares its texture.
    pub(crate) fn remove_map(&self, key: &str, id: TextureId) -> Option<TextureMap> {
        let mut maps = self.maps.write();
        let index = maps
            .iter()
            .position(|map| map.key == key && map.texture.id != id);
        index.map(|index| maps.swap_remove(index))
    }

    /// Whether `key` is a map, which a Renderer that hasn't loaded it holds no copy of.
    pub(crate) fn has_map(&self, key: &str) -> bool {
        self.maps.read().iter().any(|map| map.key == key)
    }

    /// Whether the shader has any map, so a load needs a device to upload into.
    pub(crate) fn has_maps(&self) -> bool {
        !self.maps.read().is_empty()
    }

    /// Adds a job to `jobs` for each map texture that `context` lacks or holds from an older set.
    /// A texture that another job already names, from this shader in another pass or from
    /// another shader that shares it, is skipped, so a Renderer uploads it once.
    pub(crate) fn find_jobs(&self, context: &RenderContext, jobs: &mut Vec<MapJob>) {
        let maps = self.maps.read();
        for map in maps.iter() {
            let texture = &map.texture;
            if jobs.iter().any(|job| job.id == texture.id) {
                continue;
            }
            let state = texture.state.lock();
            if state.is_loaded(texture.id, context) {
                continue;
            }
            jobs.push(MapJob {
                texture: Arc::downgrade(texture),
                id: texture.id,
                stamp: state.stamp,
                source: state.source.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Renderer, Shader, TextureInput};

    /// A shader with one texture uniform, `image`.
    fn build_shader() -> Shader {
        Shader::new(
            r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> { return textureLoad(image, vec2<i32>(0), 0); }
"#,
        )
        .expect("a shader")
    }

    /// A 2x2 source of raw `color` pixels.
    fn build_source(color: [u8; 4]) -> TextureSource {
        let input = TextureInput::from((color.repeat(4), [2u32, 2u32]));
        TextureSource::try_from(input).expect("a source")
    }

    /// The context of a new Renderer's target, which keeps that context alive.
    async fn open_context() -> (crate::TextureTarget, Arc<RenderContext>) {
        let renderer = Renderer::new();
        let target = renderer
            .create_texture_target([8, 8])
            .await
            .expect("a target");
        let context = target.context.clone();
        (target, context)
    }

    /// Takes the one job `object` has for `context`.
    fn take_job(object: &Arc<ShaderObject>, context: &RenderContext) -> MapJob {
        let mut jobs = Vec::new();
        object.find_jobs(context, &mut jobs);
        jobs.pop().expect("a job")
    }

    /// Uploads the source `job` reads into `context`.
    async fn upload(job: &MapJob, context: &Arc<RenderContext>) -> TextureObject {
        let source = job.source.clone();
        let uploaded = TextureObject::from_source(context.clone(), source, "load").await;
        uploaded.expect("an upload")
    }

    // Story: a set that lands while an upload runs takes a new stamp. The older upload
    // stands in as the context's copy, and the next load uploads the latest source.
    #[test]
    fn an_older_upload_stands_in_until_the_next_load() {
        pollster::block_on(async move {
            let (_target, context) = open_context().await;
            let shader = build_shader();
            let object = &shader.object;
            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            let older = take_job(object, &context);

            let latest = build_source([0, 0, 255, 255]);
            let data = latest.data.clone();
            object.set_map("image", latest).expect("a new source");
            let uploaded = upload(&older, &context).await;

            assert!(
                older.keep_upload(&context, uploaded),
                "the older upload left the context without a copy"
            );
            assert!(context.has_texture(&older.id), "no copy stands in");
            let job = take_job(object, &context);
            assert_eq!(job.id, older.id, "the map changed its id");
            assert!(job.stamp > older.stamp, "the job holds the older stamp");
            assert!(Arc::ptr_eq(&job.source.data, &data), "{:?}", job.source);
        });
    }

    // Story: two loads race, and the one with the latest source keeps its copy first.
    // The older upload, which finishes last, never replaces that newer copy.
    #[test]
    fn an_older_upload_never_replaces_a_newer_copy() {
        pollster::block_on(async move {
            let (_target, context) = open_context().await;
            let shader = build_shader();
            let object = &shader.object;
            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            let older = take_job(object, &context);
            object
                .set_map("image", build_source([0, 0, 255, 255]))
                .expect("a new source");
            let latest = take_job(object, &context);

            let newer = upload(&latest, &context).await;
            assert!(latest.keep_upload(&context, newer), "the latest upload");
            let kept = context.get_texture(&latest.id).expect("a copy");
            let stale = upload(&older, &context).await;

            assert!(
                !older.keep_upload(&context, stale),
                "the older upload replaced the newer copy"
            );
            let held = context.get_texture(&latest.id).expect("a copy");
            assert!(Arc::ptr_eq(&kept, &held), "the copy changed");
            let mut jobs = Vec::new();
            object.find_jobs(&context, &mut jobs);
            assert!(jobs.is_empty(), "the latest source uploads again");
        });
    }

    // Story: an explicit texture set while a map's upload runs wins. The upload keeps
    // no copy under the map's id, the slot names the texture, and no job is left.
    #[test]
    fn an_explicit_set_during_an_upload_wins() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let target = renderer
                .create_texture_target([8, 8])
                .await
                .expect("a target");
            let context = target.context.clone();
            let pixels = ([0, 0, 255, 255].repeat(4), [2u32, 2u32]);
            let texture = renderer.create_texture(pixels).await.expect("a texture");
            let shader = build_shader();
            let object = &shader.object;
            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            let older = take_job(object, &context);

            shader.set("image", &texture).expect("binds");
            let uploaded = upload(&older, &context).await;

            assert!(
                !older.keep_upload(&context, uploaded),
                "the upload of the ended map was kept"
            );
            assert!(!older.is_current(), "the ended map's job is current");
            assert!(!context.has_texture(&older.id), "a copy of the ended map");
            assert!(!object.has_map("image"), "the map stayed");
            assert!(object.bound_textures.contains_key("image"), "no binding");
            assert_eq!(read_image(object), texture.id().id);
            let mut jobs = Vec::new();
            object.find_jobs(&context, &mut jobs);
            assert!(jobs.is_empty(), "a job for the ended map");
        });
    }

    // Story: an explicit texture that ends a map drops the map after `texture_writes`,
    // so no Renderer frees its copy while a flush on another thread waits for that lock.
    #[test]
    fn an_explicit_set_drops_the_map_after_its_lock() {
        let shader = build_shader();
        shader
            .object
            .set_map("image", build_source([255, 0, 0, 255]))
            .expect("a map");
        let object = shader.object.clone();
        crate::testing::race_hooks::once("map dropped", move || {
            let free = object.texture_writes.try_lock().is_some();
            assert!(free, "the map dropped under texture_writes");
        });

        let input = crate::core::shader::UniformInput::Texture(TextureId { id: 9 });
        shader.object.set_input("image", input).expect("a set");

        assert!(!shader.object.has_map("image"), "the map stayed");
    }

    /// The texture id `image` holds.
    fn read_image(object: &ShaderObject) -> u64 {
        match object.get_uniform_data("image") {
            Ok(UniformData::Texture(meta)) => meta.id.id,
            other => panic!("image holds no texture: {other:?}"),
        }
    }

    // Story: a texture write with id 0 unsets the slot through either write path.
    // The slot reads 0 and the key keeps neither its map nor the texture bound before.
    #[test]
    fn an_unset_texture_ends_the_map() {
        pollster::block_on(async move {
            let renderer = Renderer::new();
            let pixels = ([0, 0, 255, 255].repeat(4), [2u32, 2u32]);
            let texture = renderer.create_texture(pixels).await.expect("a texture");
            let shader = build_shader();
            let object = &shader.object;
            let unset = || UniformData::Texture(TextureMeta::from(TextureId { id: 0 }));

            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            object.set("image", unset()).expect("unsets");
            assert!(!object.has_map("image"), "set kept the map");
            assert_eq!(read_image(object), 0, "set kept the map's id");

            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            let input = crate::core::shader::UniformInput::Texture(TextureId { id: 0 });
            object.set_input("image", input).expect("unsets");
            assert!(!object.has_map("image"), "set_input kept the map");
            assert_eq!(read_image(object), 0, "set_input kept the map's id");

            shader.set("image", &texture).expect("binds");
            object.set("image", unset()).expect("unsets");
            assert!(
                !object.bound_textures.contains_key("image"),
                "set kept the texture bound before"
            );
            assert_eq!(read_image(object), 0, "set kept the texture's id");
        });
    }

    // Story: a map's id binds only the slot that holds the map, since its copies go with
    // the map. Another shader and another slot of the same shader refuse it and keep their value.
    #[test]
    fn a_map_id_binds_only_its_own_slot() {
        let shader = Shader::new(
            r#"
@group(0) @binding(0) var image: texture_2d<f32>;
@group(0) @binding(1) var other: texture_2d<f32>;
@vertex fn vs_main() -> @builtin(position) vec4<f32> { return vec4<f32>(0.0); }
@fragment fn fs_main() -> @location(0) vec4<f32> {
    return textureLoad(image, vec2<i32>(0), 0) + textureLoad(other, vec2<i32>(0), 0);
}
"#,
        )
        .expect("a shader");
        let object = &shader.object;
        object
            .set_map("image", build_source([255, 0, 0, 255]))
            .expect("a map");
        let id = match object.get_uniform_data("image") {
            Ok(UniformData::Texture(meta)) => meta.id,
            other => panic!("image holds no texture: {other:?}"),
        };
        let stranger = build_shader();
        let foreign = |error: &ShaderError| matches!(error, ShaderError::ForeignMap { id: found, .. } if *found == id.id);

        let input = crate::core::shader::UniformInput::Texture(id);
        let refused = stranger
            .object
            .set_input("image", input)
            .expect_err("another shader");
        assert!(foreign(&refused), "{refused:?}");
        assert_eq!(
            read_image(&stranger.object),
            0,
            "the refused id was written"
        );

        let meta = UniformData::Texture(TextureMeta::from(id));
        let refused = object.set("other", meta).expect_err("another slot");
        assert!(foreign(&refused), "{refused:?}");
        assert!(object.has_map("image"), "the refusal ended the map");
    }

    // Story: the id a map slot returns comes back through `set` as the map's own,
    // so the round trip keeps the map and its source.
    #[test]
    fn a_round_trip_set_keeps_the_map() {
        let shader = build_shader();
        let source = build_source([255, 0, 0, 255]);
        let data = source.data.clone();
        shader.object.set_map("image", source).expect("a map");

        let value: UniformData = shader.get("image").expect("a texture");
        shader.set("image", value.clone()).expect("a set");

        let maps = shader.object.maps.read();
        let map = maps.iter().find(|map| map.key == "image");
        let map = map.expect("the round trip ended the map");
        let source = map.texture.read_source();
        assert!(Arc::ptr_eq(&source.data, &data), "{source:?}");
        assert!(
            matches!(&value, UniformData::Texture(meta) if meta.id == map.texture.id),
            "{value:?}"
        );
    }
}
