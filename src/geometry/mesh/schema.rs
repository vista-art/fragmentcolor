//! The fields a mesh derives from its first vertex and instance,
//! and the types shaders give the attributes that hold plain numbers.

use parking_lot::RwLock;
use std::collections::HashMap;

use crate::ShaderError;
use crate::geometry::{Instance, POSITION, Vertex, VertexData, describe};

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct VertexSchema {
    pub(crate) stride: u64,
    pub(crate) fields: Vec<Field>,
}

impl VertexSchema {
    pub(super) fn new(fields: Vec<Field>) -> Self {
        let stride = fields.iter().map(|field| field.size).sum();
        Self { stride, fields }
    }

    /// Whether this is the instance layout of a `Model`, without building one to compare.
    pub(super) fn is_model(&self) -> bool {
        let columns = self
            .fields
            .iter()
            .map(|field| (field.name.as_str(), field.fmt));
        columns.eq(MODEL_COLUMNS.map(|name| (name, wgpu::VertexFormat::Float32x4)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) fmt: wgpu::VertexFormat,
    pub(crate) size: u64,
}

impl Field {
    pub(super) fn new(name: &str, fmt: wgpu::VertexFormat) -> Self {
        Self {
            name: name.to_string(),
            fmt,
            size: fmt.size(),
        }
    }
}

/// The format a shader gave each untyped attribute, by stream and name.
#[derive(Clone, Debug, Default)]
pub(crate) struct AttributeTypes {
    pub(crate) vertex: HashMap<String, wgpu::VertexFormat>,
    pub(crate) instance: HashMap<String, wgpu::VertexFormat>,
}

impl AttributeTypes {
    pub(super) fn is_empty(&self) -> bool {
        self.vertex.is_empty() && self.instance.is_empty()
    }
}

/// How a shader reads one attribute: the format the mesh packs it in,
/// the WGSL type the shader declares, such as `vec2<f16>`, and whether it is `f16`,
/// whose range every value of the attribute must stay in after rounding, later ones too.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Read {
    pub(crate) format: wgpu::VertexFormat,
    pub(crate) declared: &'static str,
    pub(crate) half: bool,
}

/// What a shader reads from a mesh: each attribute by stream and name, and the position.
#[derive(Clone, Debug, Default)]
pub(crate) struct Reads {
    pub(crate) vertex: HashMap<String, Read>,
    pub(crate) instance: HashMap<String, Read>,
    pub(crate) position: Option<Read>,
}

impl Reads {
    /// Adds `other`'s reads. Where both read one attribute, an `f16` read wins,
    /// so the `f16` rule holds while any shader that reads it as `f16` holds the mesh.
    pub(super) fn merge(&mut self, other: &Reads) {
        merge_reads(&mut self.vertex, &other.vertex);
        merge_reads(&mut self.instance, &other.instance);
        if let Some(read) = other.position {
            self.position = Some(pick_stricter(self.position, read));
        }
    }
}

/// Adds each of `other`'s reads to `reads`, keeping the stricter of two reads of one name.
fn merge_reads(reads: &mut HashMap<String, Read>, other: &HashMap<String, Read>) {
    for (name, read) in other {
        let kept = reads.get(name).copied();
        reads.insert(name.clone(), pick_stricter(kept, *read));
    }
}

/// `read`, unless `kept` reads the attribute as `f16` and `read` doesn't.
fn pick_stricter(kept: Option<Read>, read: Read) -> Read {
    match kept {
        Some(kept) if kept.half && !read.half => kept,
        _ => read,
    }
}

/// What a shader's attach records on a mesh: the types it gives
/// untyped attributes, and the format and type it reads each attribute in.
/// `columns` is set when the check assumed the Model columns as the instance layout.
#[derive(Clone, Debug, Default)]
pub(crate) struct Binding {
    pub(crate) types: AttributeTypes,
    pub(crate) read: Reads,
    pub(crate) columns: bool,
}

impl Binding {
    /// The same binding with no reads, for a shader that checks the mesh itself before it draws.
    pub(crate) fn without_reads(self) -> Self {
        Self {
            read: Reads::default(),
            ..self
        }
    }
}

/// The names of the four `vec4<f32>` columns of a `Model`'s transform.
pub(super) const MODEL_COLUMNS: [&str; 4] = ["model_0", "model_1", "model_2", "model_3"];

/// The instance layout of a `Model`: the four `vec4<f32>` columns of its transform.
pub(crate) fn build_columns() -> VertexSchema {
    let fields = MODEL_COLUMNS
        .into_iter()
        .map(|name| Field::new(name, wgpu::VertexFormat::Float32x4))
        .collect();
    VertexSchema::new(fields)
}

pub(super) fn derive_vertex_schema(
    vertex: &Vertex,
    types: &HashMap<String, wgpu::VertexFormat>,
) -> VertexSchema {
    let position = match vertex.dimensions {
        0..=2 => wgpu::VertexFormat::Float32x2,
        _ => wgpu::VertexFormat::Float32x3,
    };
    let mut fields = vec![Field::new(POSITION, position)];
    fields.extend(derive_fields(&vertex.properties, types));
    VertexSchema::new(fields)
}

pub(super) fn derive_instance_schema(
    instance: &Instance,
    types: &HashMap<String, wgpu::VertexFormat>,
) -> VertexSchema {
    VertexSchema::new(derive_fields(&instance.properties, types))
}

/// One field per value in key order, in the format each value resolves to.
fn derive_fields(
    properties: &HashMap<String, VertexData>,
    types: &HashMap<String, wgpu::VertexFormat>,
) -> Vec<Field> {
    let mut values: Vec<(&String, &VertexData)> = properties.iter().collect();
    values.sort_by(|left, right| left.0.cmp(right.0));
    values
        .into_iter()
        .map(|(name, value)| Field::new(name, value.resolve(types.get(name).copied())))
        .collect()
}

/// Fails on the first attribute `added` types another way than `known` does.
pub(super) fn check_conflicts(
    known: &HashMap<String, wgpu::VertexFormat>,
    added: &HashMap<String, wgpu::VertexFormat>,
) -> Result<(), ShaderError> {
    for (name, format) in added {
        if let Some(fixed) = known.get(name)
            && fixed != format
        {
            return Err(ShaderError::AttributeConflict {
                name: name.clone(),
                fixed: describe(*fixed).to_string(),
                declared: describe(*format).to_string(),
            });
        }
    }
    Ok(())
}

/// Drops a schema that lists one of `names`, so the next pack derives it with their types.
pub(super) fn reset_schema(
    schema: &RwLock<Option<VertexSchema>>,
    names: &HashMap<String, wgpu::VertexFormat>,
) {
    let mut schema = schema.write();
    let listed = schema.as_ref().is_some_and(|schema| {
        schema
            .fields
            .iter()
            .any(|field| names.contains_key(&field.name))
    });
    if listed {
        *schema = None;
    }
}
