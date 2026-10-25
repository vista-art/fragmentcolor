use super::{ShaderError, ShaderObject, VertexInputDesc};
use crate::mesh::{AttributeTypes, Instance, MeshObject, Shape, Vertex, VertexData, describe};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use wgpu::VertexFormat;

/// The two streams a mesh feeds a vertex shader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Stream {
    Vertex,
    Instance,
}

impl Stream {
    fn label(self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Instance => "instance",
        }
    }

    fn select(self, types: &AttributeTypes) -> &HashMap<String, VertexFormat> {
        match self {
            Self::Vertex => &types.vertex,
            Self::Instance => &types.instance,
        }
    }

    fn select_mut(self, types: &mut AttributeTypes) -> &mut HashMap<String, VertexFormat> {
        match self {
            Self::Vertex => &mut types.vertex,
            Self::Instance => &mut types.instance,
        }
    }
}

/// An attribute of a stream's first item: its location in the mesh's own buffer and its shape.
struct Attribute {
    location: Option<u32>,
    shape: Shape,
}

/// Where a shader input finds its data on the mesh.
enum Source<'mesh> {
    Position(VertexFormat),
    Attribute(Stream, &'mesh str, &'mesh Attribute),
}

impl ShaderObject {
    /// Matches the mesh to this shader's vertex inputs and checks every value it holds.
    ///
    /// Returns the types this shader gives untyped attributes no shader typed before.
    /// It records nothing: `Shader::add_mesh` hands the result to `MeshObject::declare`,
    /// so a mesh that fails here, or one only validated, keeps its types and its buffers.
    pub(crate) fn bind_mesh(&self, mesh: &MeshObject) -> Result<AttributeTypes, ShaderError> {
        let inputs = self.reflect_vertex_inputs()?;
        if inputs.is_empty() {
            return Err(ShaderError::InvalidKey(
                "
                Invalid Mesh: Shader has no vertex inputs and renders fullscreen.\n

                To create a compatible Shader, use Shader::from_vertex() or Shader::from_mesh(),
                or provide a vertex shader with at least a 'position' input.
                "
                .into(),
            ));
        }

        let known = mesh.types.read().clone();
        let verts = mesh.verts.read();
        let insts = mesh.insts.read();
        let first = verts.first();
        let position = first.map(|vertex| position_format(vertex.dimensions));
        let vertex =
            first.map(|vertex| read_attributes(&vertex.properties, &vertex.prop_locations));
        // `Model::attach` declares the model columns as a schema with no instances behind it.
        let instance = match insts.first() {
            Some(first) => Some(read_attributes(&first.properties, &first.prop_locations)),
            None => mesh.instance_schema.read().as_ref().map(|schema| {
                schema
                    .fields
                    .iter()
                    .map(|field| {
                        let attribute = Attribute {
                            location: None,
                            shape: Shape::Typed(field.fmt),
                        };
                        (field.name.clone(), attribute)
                    })
                    .collect()
            }),
        };

        let mut added = AttributeTypes::default();
        let mut read = HashSet::new();
        for input in &inputs {
            match find_source(input, position, vertex.as_ref(), instance.as_ref())? {
                Source::Position(format) if format == input.format => {}
                Source::Position(format) => {
                    return Err(mismatch(
                        "position",
                        input.format,
                        format!("{} at vertex 0", describe(format)),
                    ));
                }
                Source::Attribute(stream, name, attribute) => {
                    bind_attribute(stream, name, attribute.shape, input, &known, &mut added)?;
                    read.insert((stream, name));
                }
            }
        }

        check_vertices(&verts, &known, &added, &read)?;
        check_instances(&insts, &known, &added, &read)?;
        Ok(added)
    }
}

/// Finds the data for `input` by name (instance, then vertex), then as the position
/// at location 0, then by the location in the mesh's own buffer (instance, then vertex).
fn find_source<'mesh>(
    input: &VertexInputDesc,
    position: Option<VertexFormat>,
    vertex: Option<&'mesh HashMap<String, Attribute>>,
    instance: Option<&'mesh HashMap<String, Attribute>>,
) -> Result<Source<'mesh>, ShaderError> {
    let by_name = |stream: Stream, attributes: Option<&'mesh HashMap<String, Attribute>>| {
        attributes?
            .get_key_value(&input.name)
            .map(|(name, attribute)| Source::Attribute(stream, name.as_str(), attribute))
    };
    let by_location = |stream: Stream, attributes: Option<&'mesh HashMap<String, Attribute>>| {
        attributes?
            .iter()
            .find(|(_, attribute)| attribute.location == Some(input.location))
            .map(|(name, attribute)| Source::Attribute(stream, name.as_str(), attribute))
    };

    if let Some(source) = by_name(Stream::Instance, instance) {
        return Ok(source);
    }
    if let Some(source) = by_name(Stream::Vertex, vertex) {
        return Ok(source);
    }
    if let Some(format) = position
        && (input.name == "position" || input.location == 0)
    {
        return Ok(Source::Position(format));
    }
    if input.location == 0 {
        return Err(ShaderError::InvalidKey(
            "Mesh has no vertices to provide 'position'".into(),
        ));
    }
    if let Some(source) = by_location(Stream::Instance, instance) {
        return Ok(source);
    }
    if let Some(source) = by_location(Stream::Vertex, vertex) {
        return Ok(source);
    }
    Err(ShaderError::InvalidKey(format!(
        "Mesh attribute not found for shader input '{}' @location({})",
        input.name, input.location
    )))
}

/// Checks one attribute against the input it feeds, and records the type
/// the input gives it when it holds plain numbers no shader typed yet.
fn bind_attribute(
    stream: Stream,
    name: &str,
    shape: Shape,
    input: &VertexInputDesc,
    known: &AttributeTypes,
    added: &mut AttributeTypes,
) -> Result<(), ShaderError> {
    let fixed = stream.select(known).get(name).copied();
    if let Some(fixed) = fixed
        && fixed != input.format
    {
        return Err(conflict(name, fixed, input.format));
    }
    let place = format!("{} 0", stream.label());
    let count = match shape {
        Shape::Typed(format) if format == input.format => return Ok(()),
        Shape::Typed(format) => {
            let found = format!("{} at {place}", describe(format));
            return Err(mismatch(name, input.format, found));
        }
        Shape::Numbers(count) => count,
    };
    if fixed.is_some() {
        return Ok(());
    }
    if !count.fits(input.format) {
        let found = format!("{} number(s) at {place}", count.len());
        return Err(mismatch(name, input.format, found));
    }
    match stream.select_mut(added).entry(name.to_string()) {
        Entry::Occupied(entry) if *entry.get() != input.format => {
            Err(conflict(name, *entry.get(), input.format))
        }
        Entry::Occupied(_) => Ok(()),
        Entry::Vacant(entry) => {
            entry.insert(input.format);
            Ok(())
        }
    }
}

/// Checks every vertex: its position against the first one's, each value the
/// first vertex also holds against the format that attribute takes, and that
/// it holds every attribute the shader reads.
fn check_vertices(
    verts: &[Vertex],
    known: &AttributeTypes,
    added: &AttributeTypes,
    read: &HashSet<(Stream, &str)>,
) -> Result<(), ShaderError> {
    let Some(first) = verts.first() else {
        return Ok(());
    };
    let position = position_format(first.dimensions);
    let formats = resolve_formats(&first.properties, &known.vertex, &added.vertex);
    for (index, vertex) in verts.iter().enumerate() {
        let own = position_format(vertex.dimensions);
        if own != position {
            let found = format!("{} at vertex {index}", describe(own));
            return Err(mismatch("position", position, found));
        }
        check_values(&vertex.properties, &formats, Stream::Vertex, index, read)?;
    }
    Ok(())
}

/// Checks each value of every instance that the first instance also holds,
/// and that every instance holds each attribute the shader reads.
fn check_instances(
    insts: &[Instance],
    known: &AttributeTypes,
    added: &AttributeTypes,
    read: &HashSet<(Stream, &str)>,
) -> Result<(), ShaderError> {
    let Some(first) = insts.first() else {
        return Ok(());
    };
    let formats = resolve_formats(&first.properties, &known.instance, &added.instance);
    for (index, instance) in insts.iter().enumerate() {
        check_values(
            &instance.properties,
            &formats,
            Stream::Instance,
            index,
            read,
        )?;
    }
    Ok(())
}

/// The format each attribute of a first item takes, in name order.
fn resolve_formats<'item>(
    properties: &'item HashMap<String, VertexData>,
    known: &HashMap<String, VertexFormat>,
    added: &HashMap<String, VertexFormat>,
) -> Vec<(&'item str, VertexFormat)> {
    let mut formats: Vec<(&str, VertexFormat)> = properties
        .iter()
        .map(|(name, value)| {
            let declared = added.get(name).or_else(|| known.get(name)).copied();
            (name.as_str(), value.resolve(declared))
        })
        .collect();
    formats.sort_by(|left, right| left.0.cmp(right.0));
    formats
}

/// Checks one item's values. An attribute the shader reads must be there,
/// since the GPU would read zeros in its place.
fn check_values(
    properties: &HashMap<String, VertexData>,
    formats: &[(&str, VertexFormat)],
    stream: Stream,
    index: usize,
    read: &HashSet<(Stream, &str)>,
) -> Result<(), ShaderError> {
    for &(name, format) in formats {
        match properties.get(name) {
            Some(value) => {
                if let Err(misfit) = value.check(format) {
                    return Err(ShaderError::AttributeMismatch {
                        name: name.to_string(),
                        expected: misfit.expected.to_string(),
                        found: format!("{} at {} {index}", misfit.found, stream.label()),
                    });
                }
            }
            None if read.contains(&(stream, name)) => {
                let found = format!("no value at {} {index}", stream.label());
                return Err(mismatch(name, format, found));
            }
            None => {}
        }
    }
    Ok(())
}

fn read_attributes(
    properties: &HashMap<String, VertexData>,
    locations: &HashMap<String, u32>,
) -> HashMap<String, Attribute> {
    properties
        .iter()
        .map(|(name, value)| {
            let attribute = Attribute {
                location: locations.get(name).copied(),
                shape: value.shape(),
            };
            (name.clone(), attribute)
        })
        .collect()
}

/// The format a position of `dimensions` packs as.
fn position_format(dimensions: u8) -> VertexFormat {
    match dimensions {
        0..=2 => VertexFormat::Float32x2,
        _ => VertexFormat::Float32x3,
    }
}

fn mismatch(name: &str, expected: VertexFormat, found: String) -> ShaderError {
    ShaderError::AttributeMismatch {
        name: name.to_string(),
        expected: describe(expected).to_string(),
        found,
    }
}

fn conflict(name: &str, fixed: VertexFormat, declared: VertexFormat) -> ShaderError {
    ShaderError::AttributeConflict {
        name: name.to_string(),
        fixed: describe(fixed).to_string(),
        declared: describe(declared).to_string(),
    }
}
