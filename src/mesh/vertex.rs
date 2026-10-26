use super::builtins::VertexPosition;
use super::data::VertexData;
use lsp_doc::lsp_doc;
use std::collections::HashMap;

#[cfg(python)]
use pyo3::prelude::*;
#[cfg(wasm)]
use wasm_bindgen::prelude::*;

/// The built-in position attribute, a name no property takes.
pub(crate) const POSITION: &str = "position";

/// Why vertex `index`, which sets a property named `position`, is refused.
pub(crate) fn explain_position(index: usize) -> String {
    format!(
        "vertex {index} sets a property named '{POSITION}', and a vertex takes its position from Vertex::new"
    )
}

#[cfg_attr(wasm, wasm_bindgen)]
#[cfg_attr(python, pyclass(from_py_object))]
#[cfg_attr(mobile, derive(uniffi::Object))]
#[derive(Clone, Debug)]
#[lsp_doc("docs/api/geometry/vertex/vertex.md")]
pub struct Vertex {
    pub(crate) position: VertexPosition,
    pub(crate) dimensions: u8,
    pub(crate) properties: HashMap<String, VertexData>,
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
impl IntoVertexPositionFull for [f32; 1] {
    fn into_v4_and_dimensions(self) -> ([f32; 4], u8) {
        ([self[0], 0.0, 0.0, 1.0], 1)
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
    /// layers or vertex color palettes. For meshes with a single texcoord
    /// and a single color, plain string `"uv"` / `"color"` literals remain
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
        // override the slots a real mesh actually has data for. These are
        // the loader's fallbacks for missing glTF accessors, with two
        // exceptions: the loader computes a missing NORMAL from the faces,
        // and it copies TEXCOORD_0 into a missing TEXCOORD_1, where UV1
        // here stays at [0, 0] after UV0 is set.
        Self::new(position)
            .set(Self::NORMAL, [0.0_f32, 0.0, 1.0])
            .set(Self::UV0, [0.0_f32, 0.0])
            .set(Self::COLOR0, [1.0_f32, 1.0, 1.0, 1.0])
            .set(Self::UV1, [0.0_f32, 0.0])
            .set(Self::TANGENT, [1.0_f32, 0.0, 0.0, 1.0])
    }

    #[lsp_doc("docs/api/geometry/vertex/set.md")]
    #[must_use = "set returns the updated Vertex"]
    pub fn set(mut self, key: &str, value: impl Into<VertexData>) -> Self {
        let k = key.to_string();
        if !self.prop_locations.contains_key(&k) {
            self.prop_locations.insert(k.clone(), self.next_location);
            self.next_location = self.next_location.saturating_add(1);
        }
        self.properties.insert(k, value.into());
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

impl From<f32> for Vertex {
    fn from(v: f32) -> Self {
        Vertex::new(v)
    }
}
impl From<[f32; 1]> for Vertex {
    fn from(v: [f32; 1]) -> Self {
        Vertex::new(v)
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
impl From<[f32; 4]> for Vertex {
    fn from(v: [f32; 4]) -> Self {
        Vertex::new(v)
    }
}
/// A vertex with a `"uv"` attribute of plain numbers, which take the shader's type.
impl From<([f32; 3], [f32; 2])> for Vertex {
    fn from((p, uv): ([f32; 3], [f32; 2])) -> Self {
        Vertex::new(p).set("uv", uv.map(f64::from))
    }
}
/// A vertex with `"uv"` and `"color"` attributes of plain numbers, which take the shader's types.
impl From<([f32; 3], [f32; 2], [f32; 4])> for Vertex {
    fn from((p, uv, c): ([f32; 3], [f32; 2], [f32; 4])) -> Self {
        Vertex::new(p)
            .set("uv", uv.map(f64::from))
            .set("color", c.map(f64::from))
    }
}

crate::impl_fc_kind!(Vertex, "Vertex");

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
#[lsp_doc("docs/api/geometry/instance/instance.md")]
pub struct Instance {
    pub(crate) properties: HashMap<String, VertexData>,
    pub(crate) prop_locations: HashMap<String, u32>,
    pub(crate) next_location: u32,
}

crate::impl_fc_kind!(Instance, "Instance");

impl Instance {
    #[lsp_doc("docs/api/geometry/instance/new.md")]
    pub fn new() -> Self {
        Self::default()
    }

    #[lsp_doc("docs/api/geometry/instance/set.md")]
    #[must_use = "set returns the updated Instance"]
    pub fn set(mut self, key: &str, value: impl Into<VertexData>) -> Self {
        let k = key.to_string();
        if !self.prop_locations.contains_key(&k) {
            self.prop_locations.insert(k.clone(), self.next_location);
            self.next_location = self.next_location.saturating_add(1);
        }
        self.properties.insert(k, value.into());
        self
    }
}

/// An instance from named values, set in key order so attribute locations stay stable.
impl From<HashMap<String, VertexData>> for Instance {
    fn from(values: HashMap<String, VertexData>) -> Self {
        let mut entries: Vec<(String, VertexData)> = values.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
            .into_iter()
            .fold(Instance::new(), |instance, (key, value)| {
                instance.set(&key, value)
            })
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

    // Story: Rust takes every position form the bindings take, and a map of
    // named values builds an instance.
    #[test]
    fn vertex_and_instance_forms() {
        assert_eq!(Vertex::from(2.0f32).dimensions, 1);
        assert_eq!(Vertex::from([2.0f32]).dimensions, 1);
        assert_eq!(Vertex::from([1.0f32, 2.0, 3.0, 4.0]).dimensions, 4);

        let mut values = HashMap::new();
        values.insert("scale".to_string(), VertexData::from(2.0f32));
        values.insert("offset".to_string(), VertexData::from([1.0f32, 0.0]));
        let instance = Instance::from(values);
        assert_eq!(instance.prop_locations.get("offset"), Some(&0));
        assert_eq!(instance.prop_locations.get("scale"), Some(&1));
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
            Some(&VertexData::F32x2([0.25, 0.75]))
        );
        assert_eq!(inst.properties.get("id"), Some(&VertexData::U32(7)));
        assert_eq!(inst.prop_locations.get("uv"), Some(&1));
        assert_eq!(inst.prop_locations.get("id"), Some(&2));
    }

    // Story: unsuffixed Rust literals, f64, i32 and the tuple forms arrive as
    // plain numbers, and only an f32 or u32 set by name keeps a type of its own.
    #[test]
    fn rust_literals_arrive_untyped() {
        let get = |vertex: &Vertex, key: &str| vertex.properties.get(key).copied();
        let id: u32 = 7;
        let vertex = Vertex::new([0.0, 0.0])
            .set("id", 7)
            .set("cell", [-1, 2])
            .set("weight", 1.0)
            .set("uv", [0.5, 0.0])
            .set("typed_id", id)
            .set("typed_uv", [0.5f32, 0.5]);
        assert_eq!(get(&vertex, "id"), Some(VertexData::Number(7.0)));
        assert_eq!(get(&vertex, "cell"), Some(VertexData::Number2([-1.0, 2.0])));
        assert_eq!(get(&vertex, "weight"), Some(VertexData::Number(1.0)));
        assert_eq!(get(&vertex, "uv"), Some(VertexData::Number2([0.5, 0.0])));
        assert_eq!(get(&vertex, "typed_id"), Some(VertexData::U32(7)));
        assert_eq!(
            get(&vertex, "typed_uv"),
            Some(VertexData::F32x2([0.5, 0.5]))
        );

        let tuple = Vertex::from(([0.0, 0.0, 0.0], [0.5, 0.25], [1.0, 0.0, 0.0, 1.0]));
        assert_eq!(get(&tuple, "uv"), Some(VertexData::Number2([0.5, 0.25])));
        assert_eq!(
            get(&tuple, "color"),
            Some(VertexData::Number4([1.0, 0.0, 0.0, 1.0]))
        );
    }
}
