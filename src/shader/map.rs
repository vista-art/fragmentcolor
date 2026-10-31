//! Texture uniforms a shader fills from sources it keeps, so every Renderer can draw them.
//!
//! A map keeps its source on the CPU and one id for its key. Each Renderer that loads
//! it uploads a copy of its own into its registry under that id, so the uniform names
//! the right texture in every Renderer, and many Renderers hold a copy at once.

use super::{ShaderError, ShaderObject, UniformData};
use crate::RenderContext;
use crate::texture::{TextureId, TextureMeta, TextureObject, TextureSource};
use parking_lot::Mutex;
use std::collections::BTreeSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Weak};

/// The ids of the maps alive now. A map's copies go when the map does,
/// so a slot that doesn't hold the map refuses its id (`check_bind`).
static LIVE: Mutex<BTreeSet<u64>> = parking_lot::const_mutex(BTreeSet::new());

/// A texture uniform filled from a source the shader keeps.
/// Each Renderer that loads it holds a copy under `id` in its own registry.
#[derive(Debug)]
pub(crate) struct TextureMap {
    pub(crate) key: String,
    /// Taken when the key becomes a map and kept while it stays one,
    /// so the uniform names the same copy in every Renderer.
    pub(crate) id: TextureId,
    pub(crate) source: TextureSource,
    /// The setter call that gave `source`. A later call takes a newer one.
    pub(crate) stamp: u64,
    /// The contexts that hold a copy, each with the stamp it came from.
    uploads: Vec<Upload>,
}

/// One context's copy of a map.
#[derive(Debug)]
struct Upload {
    /// `RenderContext::id`, as a mesh tags its buffers.
    owner: u64,
    context: Weak<RenderContext>,
    stamp: u64,
}

impl Drop for TextureMap {
    /// Each context still alive drops its copy.
    fn drop(&mut self) {
        #[cfg(test)]
        crate::hooks::run("map dropped");
        LIVE.lock().remove(&self.id.id);
        for upload in &self.uploads {
            if let Some(context) = upload.context.upgrade() {
                context.release_texture(&self.id);
            }
        }
    }
}

impl TextureMap {
    /// Whether `context` holds a copy of the latest source, still in its registry.
    /// A copy that `unregister_texture` removed uploads again.
    fn is_loaded(&self, context: &RenderContext) -> bool {
        let current = self
            .uploads
            .iter()
            .any(|upload| upload.owner == context.id && upload.stamp == self.stamp);
        current && context.has_texture(&self.id)
    }

    /// Whether an upload of the source from `stamp` should be `context`'s copy: it is the latest,
    /// or the context holds no copy, or only one from an older set. Stamps only grow,
    /// so an older upload never replaces a newer copy that another load kept first.
    fn takes(&self, stamp: u64, context: &RenderContext) -> bool {
        if stamp == self.stamp || !context.has_texture(&self.id) {
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
    pub(crate) shader: Arc<ShaderObject>,
    pub(crate) id: TextureId,
    pub(crate) stamp: u64,
    pub(crate) source: TextureSource,
}

/// Takes a stamp newer than every stamp before it.
fn take_stamp() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

impl ShaderObject {
    /// Makes `key` a map of `source`, which each Renderer that draws the shader uploads.
    ///
    /// A key that already is a map keeps its id, so each Renderer draws its old copy
    /// until a load replaces it. The map is in place before the slot names its id,
    /// so a plan that reads the id finds the map. An explicit texture the key held is released.
    pub(crate) fn set_map(&self, key: &str, source: TextureSource) -> Result<(), ShaderError> {
        self.check_key(key)?;
        let writing = self.texture_writes.lock();
        let (id, replaced) = self.keep_source(key, source);
        let written = self.write_slot(key, UniformData::Texture(TextureMeta::with_id_only(id)));
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

    /// Gives `key`'s map `source` and a new stamp, or makes the key a map with a new id.
    /// Returns the map's id and the source it replaced, which the caller drops after its locks.
    fn keep_source(&self, key: &str, source: TextureSource) -> (TextureId, Option<TextureSource>) {
        let stamp = take_stamp();
        let mut maps = self.maps.write();
        if let Some(map) = maps.iter_mut().find(|map| map.key == key) {
            map.stamp = stamp;
            return (map.id, Some(std::mem::replace(&mut map.source, source)));
        }
        let id = TextureId::create();
        LIVE.lock().insert(id.id);
        maps.push(TextureMap {
            key: key.to_string(),
            id,
            source,
            stamp,
            uploads: Vec::new(),
        });
        (id, None)
    }

    /// Refuses `id` for `key` when it names a live map that `key` doesn't hold.
    /// The map's copies go when the map does, so no other slot can keep them.
    pub(crate) fn check_bind(&self, key: &str, id: TextureId) -> Result<(), ShaderError> {
        let mapped = LIVE.lock().contains(&id.id);
        let owned = || {
            self.maps
                .read()
                .iter()
                .any(|map| map.key == key && map.id == id)
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
    /// Returns the map, which the caller drops after its locks, and its copies go with it.
    pub(crate) fn remove_map(&self, key: &str, id: TextureId) -> Option<TextureMap> {
        let mut maps = self.maps.write();
        let index = maps.iter().position(|map| map.key == key && map.id != id);
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

    /// Adds a job to `jobs` for each map that `context` lacks or holds from an older set.
    /// A map that another job already names, from this shader in another pass, is skipped.
    pub(crate) fn find_jobs(self: &Arc<Self>, context: &RenderContext, jobs: &mut Vec<MapJob>) {
        let maps = self.maps.read();
        for map in maps.iter() {
            if map.is_loaded(context) || jobs.iter().any(|job| job.id == map.id) {
                continue;
            }
            jobs.push(MapJob {
                shader: self.clone(),
                id: map.id,
                stamp: map.stamp,
                source: map.source.clone(),
            });
        }
    }

    /// Keeps `object`, which `job` uploaded, as `context`'s copy of the map.
    ///
    /// It refuses the copy when the map ended or the context holds a newer one.
    /// An upload that a later set overtook stands in until the next load uploads
    /// the latest, so a stream of sets can't leave a Renderer without a copy. It
    /// returns whether it kept the copy, and takes only `maps.write()`. A copy it replaces drops after that lock.
    pub(crate) fn keep_upload(
        &self,
        job: &MapJob,
        context: &Arc<RenderContext>,
        object: TextureObject,
    ) -> bool {
        let replaced = {
            let mut maps = self.maps.write();
            let found = maps.iter_mut().find(|map| map.id == job.id);
            let Some(map) = found.filter(|map| map.takes(job.stamp, context)) else {
                return false;
            };
            let replaced = context.insert_texture(map.id, Arc::new(object));
            let alive = |upload: &Upload| upload.context.strong_count() > 0;
            map.uploads
                .retain(|upload| upload.owner != context.id && alive(upload));
            map.uploads.push(Upload {
                owner: context.id,
                context: Arc::downgrade(context),
                stamp: job.stamp,
            });
            replaced
        };
        drop(replaced);
        true
    }

    /// Whether `job` still uploads the latest source of its map.
    pub(crate) fn is_current(&self, job: &MapJob) -> bool {
        let maps = self.maps.read();
        maps.iter()
            .any(|map| map.id == job.id && map.stamp == job.stamp)
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
                object.keep_upload(&older, &context, uploaded),
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
            assert!(
                object.keep_upload(&latest, &context, newer),
                "the latest upload"
            );
            let kept = context.get_texture(&latest.id).expect("a copy");
            let stale = upload(&older, &context).await;

            assert!(
                !object.keep_upload(&older, &context, stale),
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
                !object.keep_upload(&older, &context, uploaded),
                "the upload of the ended map was kept"
            );
            assert!(!object.is_current(&older), "the ended map's job is current");
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
        crate::hooks::once("map dropped", move || {
            let free = object.texture_writes.try_lock().is_some();
            assert!(free, "the map dropped under texture_writes");
        });

        let input = crate::shader::UniformInput::Texture(TextureId { id: 9 });
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
            let unset = || UniformData::Texture(TextureMeta::with_id_only(TextureId { id: 0 }));

            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            object.set("image", unset()).expect("unsets");
            assert!(!object.has_map("image"), "set kept the map");
            assert_eq!(read_image(object), 0, "set kept the map's id");

            object
                .set_map("image", build_source([255, 0, 0, 255]))
                .expect("a map");
            let input = crate::shader::UniformInput::Texture(TextureId { id: 0 });
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

        let input = crate::shader::UniformInput::Texture(id);
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

        let meta = UniformData::Texture(TextureMeta::with_id_only(id));
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
        assert!(Arc::ptr_eq(&map.source.data, &data), "{:?}", map.source);
        assert!(
            matches!(&value, UniformData::Texture(meta) if meta.id == map.id),
            "{value:?}"
        );
    }
}
