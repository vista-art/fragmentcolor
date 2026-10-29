//! The UV source of each texture slot in the built-in PBR shader.
//!
//! Every slot samples one vertex UV set through its own `KHR_texture_transform`.
//! The shader keeps them in `material.uv`, an array indexed in `TextureSlot` order.

use lsp_doc::lsp_doc;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

use super::{Material, set_or_warn};

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(eq, eq_int, from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[lsp_doc("docs/api/scene/texture_slot/texture_slot.md")]
pub enum TextureSlot {
    /// The base color map, set by `base_color_texture`.
    BaseColor,
    /// The metallic-roughness map, set by `metallic_roughness_texture`.
    MetallicRoughness,
    /// The normal map, set by `normal_texture`.
    Normal,
    /// The occlusion map, set by `occlusion_texture`.
    Occlusion,
    /// The emissive map, set by `emissive_texture`.
    Emissive,
}

impl TextureSlot {
    pub(crate) const ALL: [Self; 5] = [
        Self::BaseColor,
        Self::MetallicRoughness,
        Self::Normal,
        Self::Occlusion,
        Self::Emissive,
    ];

    /// The Material setter that fills this slot, as an error names it.
    pub(crate) fn name_setter(self) -> &'static str {
        match self {
            Self::BaseColor => "Material::base_color_texture",
            Self::MetallicRoughness => "Material::metallic_roughness_texture",
            Self::Normal => "Material::normal_texture",
            Self::Occlusion => "Material::occlusion_texture",
            Self::Emissive => "Material::emissive_texture",
        }
    }

    /// The uniform the built-in PBR shader reads this slot's map from.
    pub(crate) fn pick_key(self) -> &'static str {
        match self {
            Self::BaseColor => "base_color_map",
            Self::MetallicRoughness => "metallic_roughness_map",
            Self::Normal => "normal_map",
            Self::Occlusion => "occlusion_map",
            Self::Emissive => "emissive_map",
        }
    }

    /// The format a map in this slot takes when the caller picks none,
    /// sRGB for the two color maps and linear for the three that hold data.
    pub(crate) fn pick_format(self) -> crate::TextureFormat {
        match self {
            Self::BaseColor | Self::Emissive => crate::TextureFormat::Rgba8UnormSrgb,
            Self::MetallicRoughness | Self::Normal | Self::Occlusion => {
                crate::TextureFormat::Rgba8Unorm
            }
        }
    }

    /// The uniform key of one field of this slot's `material.uv` entry.
    fn format_key(self, field: &str) -> String {
        format!("material.uv[{}].{field}", self as usize)
    }
}

/// Errors from Material setters that take a value the material can't use.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum MaterialError {
    #[error("Material::texture_uv takes UV set 0 or 1, got {0}")]
    UvSet(u32),
}

/// The vertex UV set a texture slot samples: `Vertex::UV0` or `Vertex::UV1`.
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

/// The set glTF numbers `index`, which `TEXCOORD_<index>` names.
impl TryFrom<u32> for UvSet {
    type Error = MaterialError;

    fn try_from(index: u32) -> Result<Self, MaterialError> {
        match index {
            0 => Ok(Self::First),
            1 => Ok(Self::Second),
            other => Err(MaterialError::UvSet(other)),
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

impl Default for UvTransform {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            scale: [1.0, 1.0],
            rotation: 0.0,
        }
    }
}

impl Material {
    /// Sets the UV set and the transform one texture slot samples with.
    pub(crate) fn set_slot_uv(&self, slot: TextureSlot, set: UvSet, transform: UvTransform) {
        self.set_slot_set(slot, set);
        self.set_slot_transform(slot, transform);
    }

    /// Sets the UV set one texture slot samples.
    pub(crate) fn set_slot_set(&self, slot: TextureSlot, set: UvSet) {
        set_or_warn(&self.shader, &slot.format_key("uv_set"), u32::from(set));
    }

    /// Sets the transform one texture slot applies to its UVs.
    pub(crate) fn set_slot_transform(&self, slot: TextureSlot, transform: UvTransform) {
        set_or_warn(&self.shader, &slot.format_key("offset"), transform.offset);
        set_or_warn(&self.shader, &slot.format_key("scale"), transform.scale);
        set_or_warn(
            &self.shader,
            &slot.format_key("rotation"),
            transform.rotation,
        );
    }

    /// Sets the same transform on all five texture slots.
    pub(crate) fn set_transforms(&self, transform: UvTransform) {
        for slot in TextureSlot::ALL {
            self.set_slot_transform(slot, transform);
        }
    }
}

#[cfg(any(python, wasm, test))]
impl crate::error::Classify for MaterialError {
    fn classify(&self) -> crate::error::ErrorClass {
        crate::error::ErrorClass::Material
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{Mesh, Vertex};
    use crate::scene::Model;
    use crate::texture::SamplerOptions;
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
    /// samples the nearest texel. The PBR shader samples every slot with
    /// the options of the first texture set, in slot order, so each test
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
    async fn draw(renderer: &Renderer, material: Material, mesh: Mesh) -> Vec<u8> {
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
        pass.add(&crate::Light::directional([0.0, -1.0, 0.0], [0.0; 3]))
            .expect("add a black light");
        renderer.render(&pass, &target).expect("render");
        target.get_image().await.expect("read back")
    }

    /// The RGB of the pixel at `row` and `column`.
    fn read_pixel(image: &[u8], row: u32, column: u32) -> [u8; 3] {
        let start = ((row * SIZE + column) * 4) as usize;
        [image[start], image[start + 1], image[start + 2]]
    }

    /// Read one field of a slot's `material.uv` entry.
    fn read_field<T: From<crate::UniformData>>(
        material: &Material,
        slot: TextureSlot,
        field: &str,
    ) -> T {
        material
            .shader()
            .get(&slot.format_key(field))
            .unwrap_or_else(|error| panic!("read {field} of {slot:?}: {error:?}"))
    }

    #[test]
    fn pbr_starts_every_slot_on_set_zero_untransformed() {
        let material = Material::pbr();
        for slot in TextureSlot::ALL {
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
    fn uv_transform_sets_all_five_slots() {
        let material = Material::pbr().uv_transform([0.5, 0.25], [2.0, 3.0], 0.5);
        for slot in TextureSlot::ALL {
            let offset: [f32; 2] = read_field(&material, slot, "offset");
            let scale: [f32; 2] = read_field(&material, slot, "scale");
            let rotation: f32 = read_field(&material, slot, "rotation");
            assert_eq!((offset, scale, rotation), ([0.5, 0.25], [2.0, 3.0], 0.5));
        }
    }

    #[test]
    fn texture_transform_after_uv_transform_replaces_one_slot() {
        let material = Material::pbr()
            .uv_transform([0.5, 0.0], [2.0, 2.0], 0.0)
            .texture_transform(TextureSlot::Normal, [0.0, 0.25], [8.0, 8.0], 1.0);
        let normal: [f32; 2] = read_field(&material, TextureSlot::Normal, "scale");
        let base: [f32; 2] = read_field(&material, TextureSlot::BaseColor, "scale");
        let rotation: f32 = read_field(&material, TextureSlot::Normal, "rotation");
        assert_eq!((normal, base, rotation), ([8.0, 8.0], [2.0, 2.0], 1.0));
    }

    #[test]
    fn texture_uv_refuses_a_third_set_and_keeps_the_slot() {
        let material = Material::pbr()
            .texture_uv(TextureSlot::Occlusion, 1)
            .expect("set 1");
        let error = material
            .texture_uv(TextureSlot::Occlusion, 2)
            .expect_err("set 2 doesn't exist");
        assert_eq!(error, MaterialError::UvSet(2));
        let set: u32 = read_field(&material, TextureSlot::Occlusion, "uv_set");
        assert_eq!(set, 1, "the slot keeps its set");
    }

    // Story: occlusion moves by its own transform while base color keeps its own.
    #[test]
    fn slot_transform_moves_only_its_slot() {
        pollster::block_on(async {
            // Arrange: a red left half and a green right half for base color,
            // and for occlusion a white left half and a black right half,
            // moved by half the texture so the black half lands on the left.
            let renderer = Renderer::new();
            let colors = texture(&renderer, [RED, GREEN, RED, GREEN]).await;
            let shade = texture(&renderer, [WHITE, BLACK, WHITE, BLACK]).await;
            let material = Material::pbr()
                .base_color_texture(&colors)
                .occlusion_texture(&shade)
                .texture_transform(TextureSlot::Occlusion, [0.5, 0.0], [1.0, 1.0], 0.0);

            // Act
            let image = draw(&renderer, material, quad(|uv| uv)).await;

            // Assert: the left half is occluded, and the right half shows
            // base color's own green, untouched by the occlusion transform.
            assert_eq!(read_pixel(&image, 8, 2), [0, 0, 0], "left half");
            assert_eq!(read_pixel(&image, 8, 13), [0, 255, 0], "right half");
        });
    }

    // Story: a slot on UV set 1 samples UV1, and the others keep UV0.
    #[test]
    fn slot_samples_its_uv_set() {
        pollster::block_on(async {
            // Arrange: UV0 spans the texture, and UV1 stays on its left half.
            let renderer = Renderer::new();
            let colors = texture(&renderer, [RED, GREEN, RED, GREEN]).await;
            let shade = texture(&renderer, [WHITE, BLACK, WHITE, BLACK]).await;
            let material = Material::pbr()
                .base_color_texture(&colors)
                .occlusion_texture(&shade)
                .texture_uv(TextureSlot::Occlusion, 1)
                .expect("set 1");
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
            let material = Material::pbr().base_color_texture(&colors).uv_transform(
                [0.0, 1.0],
                [0.5, 0.5],
                std::f32::consts::FRAC_PI_2,
            );

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
