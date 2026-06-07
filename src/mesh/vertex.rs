use super::builtins::VertexPosition;
use lsp_doc::lsp_doc;
use std::collections::HashMap;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Clone, Debug)]
#[lsp_doc("docs/api/geometry/vertex/vertex.md")]
pub struct Vertex {
    pub(crate) position: VertexPosition,
    pub(crate) dimensions: u8,
    pub(crate) properties: HashMap<String, VertexValue>,
    pub(crate) prop_locations: HashMap<String, u32>,
    pub(crate) next_location: u32,
}

pub(crate) trait IntoVertexPositionFull {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8);
}

impl IntoVertexPositionFull for [f32; 2] {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self[0], self[1], 0.0, 1.0], 2)
    }
}
impl IntoVertexPositionFull for [f32; 3] {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self[0], self[1], self[2], 1.0], 3)
    }
}
impl IntoVertexPositionFull for [f32; 4] {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        (self, 4)
    }
}
impl IntoVertexPositionFull for (f32, f32) {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self.0, self.1, 0.0, 1.0], 2)
    }
}
impl IntoVertexPositionFull for (f32, f32, f32) {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self.0, self.1, self.2, 1.0], 3)
    }
}
impl IntoVertexPositionFull for (f32, f32, f32, f32) {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self.0, self.1, self.2, self.3], 4)
    }
}
impl IntoVertexPositionFull for f32 {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self, 0.0, 0.0, 1.0], 1)
    }
}
impl IntoVertexPositionFull for (u32, u32) {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self.0 as f32, self.1 as f32, 0.0, 1.0], 2)
    }
}
impl IntoVertexPositionFull for (u32, u32, u32) {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self.0 as f32, self.1 as f32, self.2 as f32, 1.0], 3)
    }
}
impl IntoVertexPositionFull for [u32; 2] {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self[0] as f32, self[1] as f32, 0.0, 1.0], 2)
    }
}
impl IntoVertexPositionFull for [u32; 3] {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self[0] as f32, self[1] as f32, self[2] as f32, 1.0], 3)
    }
}

impl Vertex {
    /// Canonical attribute keys for the most common per-vertex channels. Pass
    /// these to [`Vertex::set`] (or equivalently, use the bare lowercase
    /// string) so the loader, the shader, and any later glTF import agree on
    /// names without bikeshedding.
    ///
    /// Use the numbered forms (`UV0`, `UV1`, `COLOR0`, `COLOR1`) when a mesh
    /// carries multiple sets — typical for glTF imports with secondary UV
    /// layers or vertex colour palettes. For meshes with a single texcoord
    /// and a single colour, plain string `"uv"` / `"color"` literals remain
    /// valid (FC matches by string).
    pub const NORMAL: &'static str = "normal";
    pub const TANGENT: &'static str = "tangent";
    pub const UV0: &'static str = "uv0";
    pub const UV1: &'static str = "uv1";
    pub const COLOR0: &'static str = "color0";
    pub const COLOR1: &'static str = "color1";

    #[lsp_doc("docs/api/geometry/vertex/new.md")]
    #[allow(private_bounds)]
    pub fn new<P: IntoVertexPositionFull>(position: P) -> Self {
        let (v4, dimensions) = position.into_v4_and_dimensions();
        Self {
            position: VertexPosition(glam::Vec4::from(v4)),
            dimensions,
            properties: HashMap::new(),
            prop_locations: HashMap::new(),
            next_location: 1,
        }
    }

    #[lsp_doc("docs/api/geometry/vertex/pbr.md")]
    #[allow(private_bounds)]
    pub fn pbr<P: IntoVertexPositionFull>(position: P) -> Self {
        // Seed every attribute `Material::pbr` reads in its vertex shader
        // with a neutral identity value. Chain `.set(...)` afterwards to
        // override the slots a real mesh actually has data for. The
        // defaults match the loader's fallbacks for missing glTF
        // accessors, so a vertex built via `Vertex::pbr(pos)` alone
        // renders the same way the loader renders a glTF primitive that
        // carries only POSITION.
        Self::new(position)
            .set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
            .set(Self::UV0, [0.0_f32, 0.0])
            .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
            .set(Self::UV1, [0.0_f32, 0.0])
            .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0])
    }

    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    pub fn set<V: Into<VertexValue>>(mut self, key: &str, v: V) -> Self {
        let k = key.to_string();
        if !self.prop_locations.contains_key(&k) {
            self.prop_locations.insert(k.clone(), self.next_location);
            self.next_location = self.next_location.saturating_add(1);
        }
        self.properties.insert(k, v.into());
        self
    }

    #[lsp_doc("docs/api/geometry/vertex/create_instance.md")]
    pub fn create_instance(&self) -> Instance {
        // Instances do not implicitly copy position; only explicit per-instance properties are carried.
        Instance {
            properties: self.properties.clone(),
            prop_locations: self.prop_locations.clone(),
            next_location: self.next_location,
        }
    }
}

impl From<[f32; 2]> for Vertex {
    fn from(v: [f32; 2]) -> Self {
        Vertex::new(v)
    }
}
impl From<[f32; 3]> for Vertex {
    fn from(v: [f32; 3]) -> Self {
        Vertex::new(v)
    }
}
impl From<([f32; 3], [f32; 2])> for Vertex {
    fn from((p, uv): ([f32; 3], [f32; 2])) -> Self {
        Vertex::new(p).set("uv", uv)
    }
}
impl From<([f32; 3], [f32; 2], [f32; 4])> for Vertex {
    fn from((p, uv, c): ([f32; 3], [f32; 2], [f32; 4])) -> Self {
        Vertex::new(p).set("uv", uv).set("color", c)
    }
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.properties == other.properties
    }
}
impl Eq for Vertex {}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Clone, Debug, Default)]
pub struct Instance {
    pub(crate) properties: HashMap<String, VertexValue>,
    pub(crate) prop_locations: HashMap<String, u32>,
    pub(crate) next_location: u32,
}

impl Instance {
    /// Create an empty `Instance`. Chain `.set("key", value)` calls to populate
    /// per-instance attributes.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a per-instance attribute. Locations auto-increment in insertion order
    /// (instance attributes have their own location namespace, separate from vertex).
    pub fn set<V: Into<VertexValue>>(mut self, key: &str, v: V) -> Self {
        let k = key.to_string();
        if !self.prop_locations.contains_key(&k) {
            self.prop_locations.insert(k.clone(), self.next_location);
            self.next_location = self.next_location.saturating_add(1);
        }
        self.properties.insert(k, v.into());
        self
    }
}

impl From<Vertex> for Instance {
    fn from(v: Vertex) -> Self {
        v.create_instance()
    }
}
impl From<&Vertex> for Instance {
    fn from(v: &Vertex) -> Self {
        v.create_instance()
    }
}

#[cfg_attr(python, derive(FromPyObject, IntoPyObject))]
#[cfg_attr(mobile, derive(uniffi::Enum))]
#[derive(Clone, Debug, PartialEq)]
pub enum VertexValue {
    F32(f32),
    #[cfg(not(mobile))]
    F32x2([f32; 2]),
    #[cfg(mobile)]
    F32x2(Vec<f32>),
    #[cfg(not(mobile))]
    F32x3([f32; 3]),
    #[cfg(mobile)]
    F32x3(Vec<f32>),
    #[cfg(not(mobile))]
    F32x4([f32; 4]),
    #[cfg(mobile)]
    F32x4(Vec<f32>),
    U32(u32),
    #[cfg(not(mobile))]
    U32x2([u32; 2]),
    #[cfg(mobile)]
    U32x2(Vec<u32>),
    #[cfg(not(mobile))]
    U32x3([u32; 3]),
    #[cfg(mobile)]
    U32x3(Vec<u32>),
    #[cfg(not(mobile))]
    U32x4([u32; 4]),
    #[cfg(mobile)]
    U32x4(Vec<u32>),
    I32(i32),
    #[cfg(not(mobile))]
    I32x2([i32; 2]),
    #[cfg(mobile)]
    I32x2(Vec<i32>),
    #[cfg(not(mobile))]
    I32x3([i32; 3]),
    #[cfg(mobile)]
    I32x3(Vec<i32>),
    #[cfg(not(mobile))]
    I32x4([i32; 4]),
    #[cfg(mobile)]
    I32x4(Vec<i32>),
}

// Convenient conversions so callers can use plain arrays/scalars
impl From<f32> for VertexValue {
    fn from(v: f32) -> Self {
        VertexValue::F32(v)
    }
}
impl From<[f32; 2]> for VertexValue {
    fn from(v: [f32; 2]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::F32x2(v);
        #[cfg(mobile)]
        return VertexValue::F32x2(v.to_vec());
    }
}
impl From<[f32; 3]> for VertexValue {
    fn from(v: [f32; 3]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::F32x3(v);
        #[cfg(mobile)]
        return VertexValue::F32x3(v.to_vec());
    }
}
impl From<[f32; 4]> for VertexValue {
    fn from(v: [f32; 4]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::F32x4(v);
        #[cfg(mobile)]
        return VertexValue::F32x4(v.to_vec());
    }
}
impl From<u32> for VertexValue {
    fn from(v: u32) -> Self {
        VertexValue::U32(v)
    }
}
impl From<[u32; 2]> for VertexValue {
    fn from(v: [u32; 2]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::U32x2(v);
        #[cfg(mobile)]
        return VertexValue::U32x2(v.to_vec());
    }
}
impl From<[u32; 3]> for VertexValue {
    fn from(v: [u32; 3]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::U32x3(v);
        #[cfg(mobile)]
        return VertexValue::U32x3(v.to_vec());
    }
}
impl From<[u32; 4]> for VertexValue {
    fn from(v: [u32; 4]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::U32x4(v);
        #[cfg(mobile)]
        return VertexValue::U32x4(v.to_vec());
    }
}
impl From<i32> for VertexValue {
    fn from(v: i32) -> Self {
        VertexValue::I32(v)
    }
}
impl From<[i32; 2]> for VertexValue {
    fn from(v: [i32; 2]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::I32x2(v);
        #[cfg(mobile)]
        return VertexValue::I32x2(v.to_vec());
    }
}
impl From<[i32; 3]> for VertexValue {
    fn from(v: [i32; 3]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::I32x3(v);
        #[cfg(mobile)]
        return VertexValue::I32x3(v.to_vec());
    }
}
impl From<[i32; 4]> for VertexValue {
    fn from(v: [i32; 4]) -> Self {
        #[cfg(not(mobile))]
        return VertexValue::I32x4(v);
        #[cfg(mobile)]
        return VertexValue::I32x4(v.to_vec());
    }
}

impl VertexValue {
    pub(crate) fn size(&self) -> u64 {
        match self {
            VertexValue::F32(_) | VertexValue::U32(_) | VertexValue::I32(_) => 4,
            VertexValue::F32x2(_) | VertexValue::U32x2(_) | VertexValue::I32x2(_) => 8,
            VertexValue::F32x3(_) | VertexValue::U32x3(_) | VertexValue::I32x3(_) => 12,
            VertexValue::F32x4(_) | VertexValue::U32x4(_) | VertexValue::I32x4(_) => 16,
        }
    }
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        match self {
            VertexValue::F32(v) => bytemuck::bytes_of(v).to_vec(),
            VertexValue::F32x2(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::F32x3(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::F32x4(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::U32(v) => bytemuck::bytes_of(v).to_vec(),
            VertexValue::U32x2(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::U32x3(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::U32x4(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::I32(v) => bytemuck::cast_slice(&[*v as u32]).to_vec(),
            VertexValue::I32x2(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::I32x3(v) => bytemuck::cast_slice(v).to_vec(),
            VertexValue::I32x4(v) => bytemuck::cast_slice(v).to_vec(),
        }
    }
    pub(crate) fn format(&self) -> wgpu::VertexFormat {
        match self {
            VertexValue::F32(_) => wgpu::VertexFormat::Float32,
            VertexValue::F32x2(_) => wgpu::VertexFormat::Float32x2,
            VertexValue::F32x3(_) => wgpu::VertexFormat::Float32x3,
            VertexValue::F32x4(_) => wgpu::VertexFormat::Float32x4,
            VertexValue::U32(_) => wgpu::VertexFormat::Uint32,
            VertexValue::U32x2(_) => wgpu::VertexFormat::Uint32x2,
            VertexValue::U32x3(_) => wgpu::VertexFormat::Uint32x3,
            VertexValue::U32x4(_) => wgpu::VertexFormat::Uint32x4,
            VertexValue::I32(_) => wgpu::VertexFormat::Sint32,
            VertexValue::I32x2(_) => wgpu::VertexFormat::Sint32x2,
            VertexValue::I32x3(_) => wgpu::VertexFormat::Sint32x3,
            VertexValue::I32x4(_) => wgpu::VertexFormat::Sint32x4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_position_full_dimensions_match() {
        let v2 = Vertex::new([1.0f32, 2.0]);
        assert_eq!(v2.dimensions, 2);
        assert_eq!(v2.position.0, glam::Vec4::new(1.0, 2.0, 0.0, 1.0));

        let v3 = Vertex::new([1.0f32, 2.0, 3.0]);
        assert_eq!(v3.dimensions, 3);
        assert_eq!(v3.position.0, glam::Vec4::new(1.0, 2.0, 3.0, 1.0));

        let v4 = Vertex::new([1.0f32, 2.0, 3.0, 4.0]);
        assert_eq!(v4.dimensions, 4);
        assert_eq!(v4.position.0, glam::Vec4::new(1.0, 2.0, 3.0, 4.0));
    }

    #[test]
    fn property_locations_autoincrement_and_saturate() {
        let v = Vertex::new([0.0f32, 0.0])
            .set("uv", [0.0f32, 0.0])
            .set("color", [1.0f32, 1.0, 1.0, 1.0]);
        // position is implicit at location 0
        assert_eq!(*v.prop_locations.get("uv").unwrap(), 1);
        assert_eq!(*v.prop_locations.get("color").unwrap(), 2);
        assert_eq!(v.next_location, 3);

        // Re-setting existing key does not allocate new location
        let v = v.set("uv", [0.5f32, 0.5]);
        assert_eq!(*v.prop_locations.get("uv").unwrap(), 1);
        assert_eq!(v.next_location, 3);

        // Saturating add when next_location is at u32::MAX
        let mut v = v;
        v.next_location = u32::MAX;
        let v = v.set("extra", 1u32);
        assert_eq!(*v.prop_locations.get("extra").unwrap(), u32::MAX);
        assert_eq!(v.next_location, u32::MAX);
    }

    #[test]
    fn attribute_name_constants_match_string_lookup() {
        // The Vertex::NORMAL / TANGENT / UV0 / UV1 / COLOR0 / COLOR1
        // constants are conveniences for the common attribute keys; they
        // must resolve to the same string the bare literal does so mixing
        // the two styles in a codebase doesn't drift.
        let v = Vertex::new([0.0f32, 0.0, 0.0])
            .set(Vertex::UV0, [0.5f32, 0.5])
            .set(Vertex::COLOR0, [1.0f32, 1.0, 1.0, 1.0])
            .set(Vertex::NORMAL, [0.0f32, 1.0, 0.0]);
        assert!(v.properties.contains_key("uv0"));
        assert!(v.properties.contains_key("color0"));
        assert!(v.properties.contains_key("normal"));
        // Constants are plain `&'static str`, so they're directly comparable
        // to literal strings — no PartialEq impl required.
        assert_eq!(Vertex::TANGENT, "tangent");
        assert_eq!(Vertex::UV1, "uv1");
        assert_eq!(Vertex::COLOR1, "color1");
    }

    #[test]
    fn instance_clones_properties_and_locations() {
        let v = Vertex::new([1.0f32, 2.0])
            .set("uv", [0.25f32, 0.75])
            .set("id", 7u32);
        let inst = v.create_instance();
        assert_eq!(
            inst.properties.get("uv"),
            Some(&VertexValue::F32x2([0.25, 0.75]))
        );
        assert_eq!(inst.properties.get("id"), Some(&VertexValue::U32(7)));
        assert_eq!(inst.prop_locations.get("uv"), Some(&1));
        assert_eq!(inst.prop_locations.get("id"), Some(&2));
    }

    #[test]
    fn vertex_value_size_format_and_bytes() {
        let cases = [
            (VertexValue::F32(1.5), 4usize, wgpu::VertexFormat::Float32),
            (
                VertexValue::F32x2([1.0, 2.0]),
                8,
                wgpu::VertexFormat::Float32x2,
            ),
            (
                VertexValue::F32x3([1.0, 2.0, 3.0]),
                12,
                wgpu::VertexFormat::Float32x3,
            ),
            (
                VertexValue::F32x4([1.0, 2.0, 3.0, 4.0]),
                16,
                wgpu::VertexFormat::Float32x4,
            ),
            (VertexValue::U32(9), 4, wgpu::VertexFormat::Uint32),
            (VertexValue::U32x2([9, 10]), 8, wgpu::VertexFormat::Uint32x2),
            (
                VertexValue::U32x3([9, 10, 11]),
                12,
                wgpu::VertexFormat::Uint32x3,
            ),
            (
                VertexValue::U32x4([9, 10, 11, 12]),
                16,
                wgpu::VertexFormat::Uint32x4,
            ),
            (VertexValue::I32(-7), 4, wgpu::VertexFormat::Sint32),
            (VertexValue::I32x2([-1, 2]), 8, wgpu::VertexFormat::Sint32x2),
            (
                VertexValue::I32x3([-1, 2, -3]),
                12,
                wgpu::VertexFormat::Sint32x3,
            ),
            (
                VertexValue::I32x4([-1, 2, -3, 4]),
                16,
                wgpu::VertexFormat::Sint32x4,
            ),
        ];
        for (val, expected_len, fmt) in cases {
            assert_eq!(val.size() as usize, expected_len);
            assert_eq!(val.format(), fmt);
            let bytes = val.to_bytes();
            assert_eq!(bytes.len(), expected_len);
        }

        // Specific byte layout checks for scalar ints
        let i = VertexValue::I32(-5);
        let iu = (-5i32 as u32).to_ne_bytes().to_vec();
        assert_eq!(i.to_bytes(), iu);

        let u = VertexValue::U32(123);
        let uu = 123u32.to_ne_bytes().to_vec();
        assert_eq!(u.to_bytes(), uu);
    }
}
