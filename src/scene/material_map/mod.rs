//! `MaterialMap`: one texture map of a material, and the UV set and transform it samples through.
//!
//! Every map samples one vertex UV set through its own `KHR_texture_transform`.
//! The PBR shader keeps them in `material.uv`, an array indexed in `MapSlot` order.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use crate::core::UniformInput;
use crate::scene::check_finite;
use crate::scene::material::{skip_missing, write_values};
use crate::texture::{TextureData, TextureMeta, TextureSource};
use crate::{MaterialError, Shader, Texture, TextureInput, UniformData};

mod platform;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Debug, Clone)]
#[lsp_doc("docs/api/scene/material_map/material_map.md")]
pub struct MaterialMap {
    shader: Shader,
    slot: MapSlot,
    /// The type of the material this map belongs to, as errors name the call.
    owner: &'static str,
}

crate::impl_fc_kind!(MaterialMap, "MaterialMap");

impl MaterialMap {
    /// The handle to the map in `slot` of the `owner` material that draws with `shader`.
    pub(crate) fn new(shader: Shader, slot: MapSlot, owner: &'static str) -> Self {
        Self {
            shader,
            slot,
            owner,
        }
    }

    #[lsp_doc("docs/api/scene/material_map/texture.md")]
    pub fn texture(&self, source: impl Into<TextureInput>) -> Result<Self, MaterialError> {
        let input = source.into();
        let call = self.name_call("texture");
        input.check_options(&call)?;
        match &input.data {
            TextureData::CloneOf(texture) => self.bind_texture(texture)?,
            _ => {
                let source = TextureSource::try_from(input).map_err(|error| error.name(&call))?;
                self.set_map(source)?;
            }
        }
        Ok(self.clone())
    }

    #[lsp_doc("docs/api/scene/material_map/uv.md")]
    pub fn uv(&self, set: u32) -> Result<Self, MaterialError> {
        self.check_uv("uv")?;
        let set = UvSet::try_from(set).map_err(|_| MaterialError::UvSet {
            call: self.name_call("uv"),
            set,
        })?;
        write_values(&self.shader, vec![self.slot.build_set(set)])?;
        Ok(self.clone())
    }

    #[lsp_doc("docs/api/scene/material_map/transform.md")]
    pub fn transform(
        &self,
        offset: [f32; 2],
        scale: [f32; 2],
        rotation: f32,
    ) -> Result<Self, MaterialError> {
        self.check_uv("transform")?;
        let transform = UvTransform::new(offset, scale, rotation);
        let transform = transform.check(&self.name_call("transform"))?;
        write_values(&self.shader, self.slot.build_transform(transform).into())?;
        Ok(self.clone())
    }
}

impl MaterialMap {
    /// The slot this map fills.
    pub(crate) fn slot(&self) -> MapSlot {
        self.slot
    }

    /// The name of `method` on this map, as errors name the call.
    fn name_call(&self, method: &str) -> String {
        format!("{}::{}().{method}", self.owner, self.slot.pick_key())
    }

    /// Refuses `method`, `uv` or `transform`, on a map that samples no mesh UVs.
    fn check_uv(&self, method: &str) -> Result<(), MaterialError> {
        if self.slot.reads_uv() {
            return Ok(());
        }
        Err(MaterialError::NoUv {
            call: self.name_call(method),
        })
    }

    /// Sets the UV set and the transform this map samples with, or neither.
    pub(crate) fn set_uv(&self, set: UvSet, transform: UvTransform) -> Result<(), MaterialError> {
        let mut values = vec![self.slot.build_set(set)];
        values.extend(self.slot.build_transform(transform));
        write_values(&self.shader, values)
    }

    /// Binds `texture` to this map as it is, so the slot holds its id and no source.
    fn bind_texture(&self, texture: &Texture) -> Result<(), MaterialError> {
        let key = self.slot.pick_key();
        let meta = TextureMeta::from(*texture.id());
        skip_missing(key, self.shader.object.set(key, UniformData::Texture(meta)))
    }

    /// Sets this map to `source`, which takes the slot's format as a hint when it picked none.
    /// A Mipmap or KTX2 container whose bytes don't read in it, such as a float one, keeps its own.
    pub(crate) fn set_map(&self, mut source: TextureSource) -> Result<(), MaterialError> {
        source.take_hint(self.slot.pick_format());
        let key = self.slot.pick_key();
        skip_missing(key, self.shader.object.set_map(key, source))
    }
}

/// The maps of the built-in shaders. The PBR maps come in the order the `material.uv` arrays
/// keep them, and the unlit and toon shaders have the first one. The toon shader's ramp,
/// last, reads the light level in place of the mesh's UVs, so it has no `material.uv` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum MapSlot {
    BaseColor,
    MetallicRoughness,
    Normal,
    Occlusion,
    Emissive,
    Ramp,
}

impl MapSlot {
    /// The five maps of the PBR shader.
    pub(crate) const ALL: [Self; 5] = [
        Self::BaseColor,
        Self::MetallicRoughness,
        Self::Normal,
        Self::Occlusion,
        Self::Emissive,
    ];

    /// The uniform the built-in shaders read this slot's map from,
    /// which is also the name of the material call that returns the map.
    pub(crate) fn pick_key(self) -> &'static str {
        match self {
            Self::BaseColor => "base_color_map",
            Self::MetallicRoughness => "metallic_roughness_map",
            Self::Normal => "normal_map",
            Self::Occlusion => "occlusion_map",
            Self::Emissive => "emissive_map",
            Self::Ramp => "ramp_map",
        }
    }

    /// Whether the map samples the mesh's UVs, which every map but the toon ramp does.
    fn reads_uv(self) -> bool {
        self != Self::Ramp
    }

    /// The format a map in this slot takes when the caller picks none,
    /// sRGB for the three color maps and linear for the three that hold data.
    pub(crate) fn pick_format(self) -> crate::TextureFormat {
        match self {
            Self::BaseColor | Self::Emissive | Self::Ramp => crate::TextureFormat::Rgba8UnormSrgb,
            Self::MetallicRoughness | Self::Normal | Self::Occlusion => {
                crate::TextureFormat::Rgba8Unorm
            }
        }
    }

    /// The uniform key of one field of this slot's `material.uv` entry.
    fn format_key(self, field: &str) -> String {
        format!("material.uv[{}].{field}", self as usize)
    }

    /// The three writes of `transform` into this slot's `material.uv` entry.
    pub(crate) fn build_transform(self, transform: UvTransform) -> [(String, UniformInput); 3] {
        [
            (
                self.format_key("offset"),
                UniformInput::from(transform.offset.as_slice()),
            ),
            (
                self.format_key("scale"),
                UniformInput::from(transform.scale.as_slice()),
            ),
            (
                self.format_key("rotation"),
                UniformInput::from([transform.rotation].as_slice()),
            ),
        ]
    }

    /// The write of `set` into this slot's `material.uv` entry.
    fn build_set(self, set: UvSet) -> (String, UniformInput) {
        let number = UniformInput::Number(u32::from(set).into());
        (self.format_key("uv_set"), number)
    }
}

/// The vertex UV set a map samples: `Vertex::UV0` or `Vertex::UV1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum UvSet {
    #[default]
    First,
    Second,
}

/// The number glTF gives the set, the `n` of `TEXCOORD_<n>`.
impl From<UvSet> for u32 {
    fn from(set: UvSet) -> Self {
        match set {
            UvSet::First => 0,
            UvSet::Second => 1,
        }
    }
}

/// The set glTF numbers `index`, which `TEXCOORD_<index>` names. Any other number is
/// returned as the error, for the caller to name.
impl TryFrom<u32> for UvSet {
    type Error = u32;

    fn try_from(index: u32) -> Result<Self, u32> {
        match index {
            0 => Ok(Self::First),
            1 => Ok(Self::Second),
            other => Err(other),
        }
    }
}

/// A `KHR_texture_transform`, which scales a UV, then rotates it, then offsets it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct UvTransform {
    pub(crate) offset: [f32; 2],
    pub(crate) scale: [f32; 2],
    pub(crate) rotation: f32,
}

impl UvTransform {
    /// A transform from its three parts, which `check` refuses unless each number is finite.
    pub(crate) fn new(offset: [f32; 2], scale: [f32; 2], rotation: f32) -> Self {
        Self {
            offset,
            scale,
            rotation,
        }
    }

    /// Refuses a transform with a number that isn't finite, naming the part after `call`.
    pub(crate) fn check(self, call: &str) -> Result<Self, MaterialError> {
        check_finite(call, Some("offset"), &self.offset)?;
        check_finite(call, Some("scale"), &self.scale)?;
        check_finite(call, Some("rotation"), &[self.rotation])?;
        Ok(self)
    }
}

impl Default for UvTransform {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PbrMaterial;
    use crate::scene::Model;
    use crate::texture::SamplerOptions;
    use crate::{Mesh, Vertex};
    use crate::{Pass, Renderer, Size, Target, Texture};

    const SIZE: u32 = 16;

    const RED: [u8; 4] = [255, 0, 0, 255];
    const GREEN: [u8; 4] = [0, 255, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];
    const WHITE: [u8; 4] = [255, 255, 255, 255];
    const BLACK: [u8; 4] = [0, 0, 0, 255];

    /// A quad over the whole target. UV0 maps its top left corner to `[0, 0]`
    /// and its bottom right corner to `[1, 1]`, and `uv1` turns each UV0 into UV1.
    fn quad(uv1: impl Fn([f32; 2]) -> [f32; 2]) -> Mesh {
        let top_left = ([-1.0, 1.0, 0.0], [0.0, 0.0]);
        let top_right = ([1.0, 1.0, 0.0], [1.0, 0.0]);
        let bottom_left = ([-1.0, -1.0, 0.0], [0.0, 1.0]);
        let bottom_right = ([1.0, -1.0, 0.0], [1.0, 1.0]);
        let mesh = Mesh::new();
        for (position, uv) in [
            top_left,
            bottom_left,
            bottom_right,
            top_left,
            bottom_right,
            top_right,
        ] {
            let vertex = Vertex::pbr(position)
                .set(Vertex::UV0, uv)
                .set(Vertex::UV1, uv1(uv));
            mesh.add_vertex(vertex);
        }
        mesh
    }

    /// A 2×2 texture, its texels in rows from the top, that repeats and
    /// samples the nearest texel. The PBR shader samples every map with
    /// the options of the first texture set, in map order, so each test
    /// gives every texture the same options.
    async fn texture(renderer: &Renderer, texels: [[u8; 4]; 4]) -> Texture {
        let bytes = texels.concat();
        let texture = renderer
            .create_texture((&bytes[..], Size::from((2u32, 2u32))))
            .await
            .expect("create a 2×2 texture");
        texture.set_sampler_options(SamplerOptions {
            repeat_x: true,
            repeat_y: true,
            smooth: false,
            compare: None,
        });
        texture
    }

    /// Draw `mesh` with `material` lit by a white ambient term alone, so each
    /// pixel is base color times occlusion. Returns the RGBA pixels.
    async fn draw(renderer: &Renderer, material: PbrMaterial, mesh: Mesh) -> Vec<u8> {
        let material = material.double_sided(true);
        let shader = material.shader();
        shader
            .set("lights.ambient", [1.0f32, 1.0, 1.0])
            .expect("ambient light");
        let target = renderer
            .create_texture_target([SIZE, SIZE])
            .await
            .expect("texture target");
        let pass = Pass::new("texture uv");
        pass.add(&Model::new(mesh, material))
            .expect("add the model");
        // A black light takes the place of the placeholder light.
        pass.add(&crate::Light::directional([0.0, -1.0, 0.0], [0.0; 3]).expect("light"))
            .expect("add a black light");
        renderer.render(&pass, &target).expect("render");
        target.get_image().await.expect("read back")
    }

    /// The RGB of the pixel at `row` and `column`.
    fn read_pixel(image: &[u8], row: u32, column: u32) -> [u8; 3] {
        let start = ((row * SIZE + column) * 4) as usize;
        [image[start], image[start + 1], image[start + 2]]
    }

    /// Read one field of a map's `material.uv` entry.
    fn read_field<T: From<crate::UniformData>>(
        material: &PbrMaterial,
        slot: MapSlot,
        field: &str,
    ) -> T {
        material
            .shader()
            .get(&slot.format_key(field))
            .unwrap_or_else(|error| panic!("read {field} of {slot:?}: {error:?}"))
    }

    #[test]
    fn pbr_starts_every_map_on_set_zero_untransformed() {
        let material = PbrMaterial::new();
        for slot in MapSlot::ALL {
            let set: u32 = read_field(&material, slot, "uv_set");
            let offset: [f32; 2] = read_field(&material, slot, "offset");
            let scale: [f32; 2] = read_field(&material, slot, "scale");
            let rotation: f32 = read_field(&material, slot, "rotation");
            assert_eq!(
                (set, offset, scale, rotation),
                (0, [0.0, 0.0], [1.0, 1.0], 0.0)
            );
        }
    }

    #[test]
    fn uv_transform_sets_all_five_maps() {
        let material = PbrMaterial::new()
            .uv_transform([0.5, 0.25], [2.0, 3.0], 0.5)
            .expect("uv transform");
        for slot in MapSlot::ALL {
            let offset: [f32; 2] = read_field(&material, slot, "offset");
            let scale: [f32; 2] = read_field(&material, slot, "scale");
            let rotation: f32 = read_field(&material, slot, "rotation");
            assert_eq!((offset, scale, rotation), ([0.5, 0.25], [2.0, 3.0], 0.5));
        }
    }

    #[test]
    fn transform_after_uv_transform_replaces_one_map() {
        let material = PbrMaterial::new()
            .uv_transform([0.5, 0.0], [2.0, 2.0], 0.0)
            .expect("uv transform");
        let map = material.normal_map();
        map.transform([0.0, 0.25], [8.0, 8.0], 1.0)
            .expect("transform");
        let normal: [f32; 2] = read_field(&material, MapSlot::Normal, "scale");
        let base: [f32; 2] = read_field(&material, MapSlot::BaseColor, "scale");
        let rotation: f32 = read_field(&material, MapSlot::Normal, "rotation");
        assert_eq!((normal, base, rotation), ([8.0, 8.0], [2.0, 2.0], 1.0));
    }

    #[test]
    fn uv_refuses_a_third_set_and_keeps_the_map() {
        let material = PbrMaterial::new();
        let map = material.occlusion_map().uv(1).expect("set 1");
        let error = map.uv(2).expect_err("set 2 doesn't exist");
        assert!(
            matches!(error, MaterialError::UvSet { set: 2, .. }),
            "{error:?}"
        );
        assert_eq!(
            error.to_string(),
            "PbrMaterial::occlusion_map().uv takes UV set 0 or 1, got 2"
        );
        let set: u32 = read_field(&material, MapSlot::Occlusion, "uv_set");
        assert_eq!(set, 1, "the slot keeps its set");
    }

    // Story: a transform with one number that isn't finite is refused whole, naming the
    // call and the part, so no map takes its offset or scale while its rotation fails.
    #[test]
    fn a_refused_transform_changes_no_map() {
        let material = PbrMaterial::new()
            .uv_transform([0.5, 0.0], [2.0, 2.0], 0.25)
            .expect("uv transform");

        let rotation = material.uv_transform([0.0, 0.0], [4.0, 4.0], f32::NAN);
        let normal = material.normal_map();
        let offset = normal.transform([0.0, f32::INFINITY], [8.0, 8.0], 1.0);

        let message = rotation.expect_err("NaN rotation").to_string();
        assert_eq!(
            message,
            "PbrMaterial::uv_transform: rotation must be finite, got NaN"
        );
        let message = offset.expect_err("infinite offset").to_string();
        assert_eq!(
            message,
            "PbrMaterial::normal_map().transform: offset: item 1 must be finite, got inf"
        );
        for slot in MapSlot::ALL {
            let offset: [f32; 2] = read_field(&material, slot, "offset");
            let scale: [f32; 2] = read_field(&material, slot, "scale");
            let rotation: f32 = read_field(&material, slot, "rotation");
            assert_eq!((offset, scale, rotation), ([0.5, 0.0], [2.0, 2.0], 0.25));
        }
    }

    // Story: a material whose shader holds rotation in an f16 refuses a rotation past its
    // range before any map changes, so the offsets and scales of the call stay out too.
    #[test]
    fn a_transform_the_shader_refuses_changes_no_map() {
        let shader = crate::Shader::new(
            "enable f16;
struct TextureUv { offset: vec2<f32>, scale: vec2<f32>, rotation: f16, uv_set: u32, pad: vec2<u32> }
struct Factors { uv: array<TextureUv, 5> }
@group(0) @binding(0) var<uniform> material: Factors;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
  return vec4<f32>(material.uv[4].offset, f32(material.uv[4].rotation), 1.0);
}",
        )
        .expect("shader");
        let material = PbrMaterial::wrap(shader);

        let refused = material.uv_transform([0.5, 0.5], [2.0, 2.0], 70000.0);

        assert!(
            matches!(refused, Err(MaterialError::Shader(_))),
            "{refused:?}"
        );
        for slot in MapSlot::ALL {
            let offset: [f32; 2] = read_field(&material, slot, "offset");
            assert_eq!(offset, [0.0, 0.0], "{slot:?} took the offset");
        }
    }

    // Story: a material whose shader has fewer UV entries than the PBR shader takes the
    // transform in the entries it declares and skips the rest, as it skips a factor.
    #[test]
    fn a_transform_skips_the_maps_a_shader_lacks() {
        let shader = crate::Shader::new(
            "
struct TextureUv { offset: vec2<f32>, scale: vec2<f32>, rotation: f32, uv_set: u32, pad: vec2<u32> }
struct Factors { uv: array<TextureUv, 2> }
@group(0) @binding(0) var<uniform> material: Factors;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
  return vec4<f32>(f32(i), 0.0, 0.0, 1.0);
}
@fragment fn fs_main() -> @location(0) vec4<f32> {
  return vec4<f32>(material.uv[1].offset, material.uv[1].rotation, 1.0);
}",
        )
        .expect("shader");
        let material = PbrMaterial::wrap(shader);

        let result = material.uv_transform([0.5, 0.25], [2.0, 2.0], 0.5);

        assert!(result.is_ok(), "{result:?}");
        for slot in [MapSlot::BaseColor, MapSlot::MetallicRoughness] {
            let offset: [f32; 2] = read_field(&material, slot, "offset");
            assert_eq!(offset, [0.5, 0.25], "{slot:?}");
        }
    }

    // Story: occlusion moves by its own transform while base color keeps its own.
    #[test]
    fn map_transform_moves_only_its_map() {
        pollster::block_on(async {
            // Arrange: a red left half and a green right half for base color,
            // and for occlusion a white left half and a black right half,
            // moved by half the texture so the black half lands on the left.
            let renderer = Renderer::new();
            let colors = texture(&renderer, [RED, GREEN, RED, GREEN]).await;
            let shade = texture(&renderer, [WHITE, BLACK, WHITE, BLACK]).await;
            let material = PbrMaterial::new();
            material
                .base_color_map()
                .texture(&colors)
                .expect("base color map");
            let occlusion = material.occlusion_map().texture(&shade);
            let occlusion = occlusion.expect("occlusion map");
            occlusion
                .transform([0.5, 0.0], [1.0, 1.0], 0.0)
                .expect("transform");

            // Act
            let image = draw(&renderer, material, quad(|uv| uv)).await;

            // Assert: the left half is occluded, and the right half shows
            // base color's own green, untouched by the occlusion transform.
            assert_eq!(read_pixel(&image, 8, 2), [0, 0, 0], "left half");
            assert_eq!(read_pixel(&image, 8, 13), [0, 255, 0], "right half");
        });
    }

    // Story: a map on UV set 1 samples UV1, and the others keep UV0.
    #[test]
    fn map_samples_its_uv_set() {
        pollster::block_on(async {
            // Arrange: UV0 spans the texture, and UV1 stays on its left half.
            let renderer = Renderer::new();
            let colors = texture(&renderer, [RED, GREEN, RED, GREEN]).await;
            let shade = texture(&renderer, [WHITE, BLACK, WHITE, BLACK]).await;
            let material = PbrMaterial::new();
            material
                .base_color_map()
                .texture(&colors)
                .expect("base color map");
            let occlusion = material.occlusion_map().texture(&shade);
            occlusion.expect("occlusion map").uv(1).expect("set 1");
            let mesh = quad(|uv| [uv[0] * 0.25, uv[1]]);

            // Act
            let image = draw(&renderer, material, mesh).await;

            // Assert: occlusion reads white everywhere through UV1, and base
            // color still spans both halves through UV0.
            assert_eq!(read_pixel(&image, 8, 2), [255, 0, 0], "left half");
            assert_eq!(read_pixel(&image, 8, 13), [0, 255, 0], "right half");
        });
    }

    // Story: the transform turns UVs the way KHR_texture_transform's example does.
    #[test]
    fn rotation_follows_the_extension_example() {
        pollster::block_on(async {
            // Arrange: four colors, red and green on top, blue and white below.
            // The extension's example maps the whole UV square onto the lower
            // left quarter, here the blue texel. A rotation the other way
            // lands on the upper right one, which is green.
            let renderer = Renderer::new();
            let colors = texture(&renderer, [RED, GREEN, BLUE, WHITE]).await;
            let material = PbrMaterial::new();
            material
                .base_color_map()
                .texture(&colors)
                .expect("base color map");
            let material = material
                .uv_transform([0.0, 1.0], [0.5, 0.5], std::f32::consts::FRAC_PI_2)
                .expect("uv transform");

            // Act
            let image = draw(&renderer, material, quad(|uv| uv)).await;

            // Assert
            for (row, column) in [(1, 1), (1, 14), (14, 1), (14, 14), (8, 8)] {
                let pixel = read_pixel(&image, row, column);
                assert_eq!(pixel, [0, 0, 255], "pixel at row {row}, column {column}");
            }
        });
    }
}
