//! The meshes a shader holds, and the checks of each vertex and instance
//! against the inputs the shader reads, when a mesh attaches or the quick view draws.

use super::vertex_inputs::VertexInputDesc;
use super::{ShaderError, ShaderObject};
use crate::geometry::{AttributeTypes, Binding, MeshObject, Shape, VertexSchema};
use crate::geometry::{POSITION, build_columns, describe, explain_position};
use crate::{Instance, Vertex, VertexData};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;
use wgpu::VertexFormat;

/// The mesh of a quick view, the same one for the shader's lifetime,
/// and the generation it passed the checks at, or `None` while it fails them.
/// The shader then holds no mesh, and `render` checks it again to report why.
#[derive(Debug)]
pub(super) struct Preview {
    mesh: Arc<MeshObject>,
    checked: Option<u64>,
}

/// The quick view's mesh and the generation it passed its check at, which only
/// that mesh's upload is held to. Other meshes on the shader upload as any do.
pub(crate) struct Passed {
    pub(crate) mesh: Arc<MeshObject>,
    pub(crate) generation: u64,
}

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
    /// Add a mesh to this shader, giving its untyped attributes the types this shader declares.
    /// Every vertex and instance the mesh takes later must hold the attributes this shader reads.
    pub fn add_mesh(&self, mesh: Arc<MeshObject>) -> Result<(), ShaderError> {
        let binding = self.bind_mesh(&mesh)?;
        self.record(mesh, binding, false)
    }

    /// Adds the mesh of a `Model` unless the shader holds it already. For a mesh with
    /// no instances and no instance layout, the check assumes the Model columns, and
    /// the mesh takes them as its layout only once the attach succeeds.
    pub(crate) fn attach_model(&self, mesh: Arc<MeshObject>) -> Result<(), ShaderError> {
        if self.holds(&mesh) {
            return Ok(());
        }
        #[cfg(test)]
        crate::testing::race_hooks::run("model attach");
        let columns = build_columns();
        let binding = self.bind_mesh_with(&mesh, Some(&columns))?;
        self.record(mesh, binding, true)
    }

    /// Records `binding` on the mesh and holds the mesh, in one step against `remove_mesh`
    /// and `clear_meshes`, so neither can drop the reads between the two. With `once`,
    /// a mesh the shader holds already stays as it is.
    fn record(
        &self,
        mesh: Arc<MeshObject>,
        binding: Binding,
        once: bool,
    ) -> Result<(), ShaderError> {
        let mut meshes = self.meshes.write();
        if once && meshes.iter().any(|held| Arc::ptr_eq(held, &mesh)) {
            return Ok(());
        }
        mesh.declare(self.id, binding)?;
        #[cfg(test)]
        crate::testing::race_hooks::run("declared");
        meshes.push(mesh);
        Ok(())
    }

    /// Records `binding` on `mesh` and holds the mesh, unless it holds it already.
    /// Unlike `record`, it records the binding for a held mesh too, so the types
    /// a check of the quick view finds again stay, as the first check's do.
    fn hold(&self, mesh: Arc<MeshObject>, binding: Binding) -> Result<(), ShaderError> {
        let mut meshes = self.meshes.write();
        mesh.declare(self.id, binding)?;
        if !meshes.iter().any(|held| Arc::ptr_eq(held, &mesh)) {
            meshes.push(mesh);
        }
        Ok(())
    }

    /// Whether this shader holds `mesh`.
    fn holds(&self, mesh: &Arc<MeshObject>) -> bool {
        self.meshes
            .read()
            .iter()
            .any(|held| Arc::ptr_eq(held, mesh))
    }

    /// Draws `mesh` for the quick view and checks it, as `check_preview` does.
    pub(crate) fn preview(&self, mesh: &Arc<MeshObject>) {
        #[cfg(test)]
        crate::testing::race_hooks::run("preview");
        let mut preview = self.preview.lock();
        let kept = preview.get_or_insert_with(|| Preview {
            mesh: mesh.clone(),
            checked: None,
        });
        // The refusal reaches `render` through `check_preview`.
        let _ = self.review(kept);
    }

    /// Checks the quick view's mesh again when it changed since it last passed.
    /// Returns why it fails, or the mesh generation it passed at for a quick view.
    pub(crate) fn check_preview(&self) -> Result<Option<u64>, ShaderError> {
        match self.preview.lock().as_mut() {
            Some(kept) => self.review(kept).map(Some),
            None => Ok(None),
        }
    }

    /// The meshes this shader draws, read in one step with `check_preview`, so no
    /// check on another thread lets the quick view's mesh go between the two. The
    /// renderer reads them on every render of this shader, however it reached it.
    pub(crate) fn read_meshes(
        &self,
    ) -> Result<(Vec<Arc<MeshObject>>, Option<Passed>), ShaderError> {
        let mut preview = self.preview.lock();
        let passed = match preview.as_mut() {
            Some(kept) => Some(Passed {
                generation: self.review(kept)?,
                mesh: kept.mesh.clone(),
            }),
            None => None,
        };
        #[cfg(test)]
        crate::testing::race_hooks::run("preview checked");
        Ok((self.meshes.read().clone(), passed))
    }

    /// Checks the kept mesh unless it passed at its current generation.
    ///
    /// It holds the mesh while it passes and lets it go while it fails.
    /// It fixes the types it gives untyped attributes, as any attach does,
    /// but records none of its reads, so it adds nothing the vertices must hold.
    /// It returns the generation the mesh passed at, which it read before the check.
    fn review(&self, kept: &mut Preview) -> Result<u64, ShaderError> {
        let generation = kept.mesh.generation();
        if kept.checked == Some(generation) {
            return Ok(generation);
        }
        let passed = self
            .bind_mesh(&kept.mesh)
            .and_then(|binding| self.hold(kept.mesh.clone(), binding.without_reads()));
        match passed {
            Ok(()) => kept.checked = Some(generation),
            Err(_) => {
                self.remove_mesh(&kept.mesh);
                kept.checked = None;
            }
        }
        passed.map(|()| generation)
    }

    /// Check that a mesh can attach to this shader, without attaching it or recording any type.
    pub(crate) fn validate_mesh(&self, mesh: &MeshObject) -> Result<(), ShaderError> {
        self.bind_mesh(mesh).map(drop)
    }

    /// Remove a mesh from this shader. Once the shader holds the mesh no
    /// more, the mesh stops checking for the attributes this shader reads.
    pub fn remove_mesh(&self, mesh: &Arc<MeshObject>) {
        let mut v = self.meshes.write();
        if let Some(pos) = v.iter().position(|m| Arc::ptr_eq(m, mesh)) {
            v.remove(pos);
        }
        if !v.iter().any(|m| Arc::ptr_eq(m, mesh)) {
            mesh.detach(self.id);
        }
    }

    /// Remove all meshes from this shader.
    pub fn clear_meshes(&self) {
        for mesh in self.meshes.write().drain(..) {
            mesh.detach(self.id);
        }
    }

    /// Matches the mesh to this shader's vertex inputs and checks every value it holds.
    ///
    /// Returns the types this shader gives untyped attributes no shader typed before,
    /// and the attributes it reads. It records nothing: `Shader::add_mesh` hands the
    /// result to `MeshObject::declare`, so a mesh that fails here, or one only validated,
    /// keeps its types and its buffers.
    pub(crate) fn bind_mesh(&self, mesh: &MeshObject) -> Result<Binding, ShaderError> {
        self.bind_mesh_with(mesh, None)
    }

    /// Matches the mesh as `bind_mesh` does. For a mesh with no instances and no instance
    /// layout, it reads the instance stream from `assumed`, and the binding says it did.
    pub(crate) fn bind_mesh_with(
        &self,
        mesh: &MeshObject,
        assumed: Option<&VertexSchema>,
    ) -> Result<Binding, ShaderError> {
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
        check_position(&verts)?;
        let first = verts.first();
        let position = first.map(|vertex| position_format(vertex.dimensions));
        let vertex =
            first.map(|vertex| read_attributes(&vertex.properties, &vertex.prop_locations));
        // With no instances, the instance layout is a Model's columns: the ones a Model
        // attach set before, or the ones this one assumes.
        let declared = mesh.instance_schema.read().clone();
        let columns = insts.is_empty() && declared.is_none() && assumed.is_some();
        let instance = match insts.first() {
            Some(first) => Some(read_attributes(&first.properties, &first.prop_locations)),
            None => declared.as_ref().or(assumed).map(read_fields),
        };

        let mut added = AttributeTypes::default();
        // What the shader reads from each stream, in the format each input declares.
        let mut read = HashMap::new();
        for input in &inputs {
            match find_source(input, position, vertex.as_ref(), instance.as_ref())? {
                Source::Position(format) if format == input.format => {}
                Source::Position(format) => {
                    return Err(mismatch(
                        POSITION,
                        input.format,
                        format!("{} at vertex 0", describe(format)),
                    ));
                }
                Source::Attribute(stream, name, attribute) => {
                    bind_attribute(stream, name, attribute.shape, input, &known, &mut added)?;
                    read.insert((stream, name), input.format);
                }
            }
        }

        check_vertices(&verts, &known, &added, &read)?;
        check_instances(&insts, &known, &added, &read)?;
        let mut names = AttributeTypes::default();
        for ((stream, name), format) in read {
            stream
                .select_mut(&mut names)
                .insert(name.to_string(), format);
        }
        Ok(Binding {
            types: added,
            read: names,
            columns,
        })
    }
}

/// The attributes of an instance layout with no instances behind it, each in its field's format.
fn read_fields(schema: &VertexSchema) -> HashMap<String, Attribute> {
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
        && (input.name == POSITION || input.location == 0)
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

/// Refuses a vertex that sets a property named `position`. Its position comes from
/// `Vertex::new`, so the value set would never reach the shader.
fn check_position(vertices: &[Vertex]) -> Result<(), ShaderError> {
    match vertices
        .iter()
        .position(|vertex| vertex.properties.contains_key(POSITION))
    {
        Some(index) => Err(ShaderError::InvalidKey(explain_position(index))),
        None => Ok(()),
    }
}

/// Checks every vertex: its position against the first one's, each value the
/// first vertex also holds against the format that attribute takes, and that
/// it holds every attribute the shader reads.
fn check_vertices(
    verts: &[Vertex],
    known: &AttributeTypes,
    added: &AttributeTypes,
    read: &HashMap<(Stream, &str), VertexFormat>,
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
            return Err(mismatch(POSITION, position, found));
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
    read: &HashMap<(Stream, &str), VertexFormat>,
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
    read: &HashMap<(Stream, &str), VertexFormat>,
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
            None if read.contains_key(&(stream, name)) => {
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
