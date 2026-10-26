//! Clustered lights. Each pass groups its lit shaders into light sets that
//! share one light list, packs each set's lights into its own buffers, and
//! bins the bounded ones into a grid of clusters with a compute pass. A
//! device without storage buffers in fragment shaders takes the lights
//! through a uniform block instead, and its shaders loop over every light.

use glam::{Mat4, Vec3};
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use super::{RenderContext, RendererError};
use crate::PassObject;
use crate::pass::PassError;
use crate::scene::{GpuLight, current_epoch};
use crate::shader::ShaderObject;
use crate::shader::light_roots::{ARRAY_HEADER, LIGHT_SIZE, LIGHTS_HEADER, LightRoots, LightShape};

mod grid;
use grid::{Frustum, GRID_CELLS, Grid, Sphere};

#[cfg(test)]
mod tests;

/// Renders a light set may sit unused before the context frees it.
const IDLE_RENDERS: u64 = 8;
/// The size of the `Bins` header in the binning shader.
const BINS_HEADER: u64 = 128;
/// The size of one `Bin`.
const BIN_SIZE: u64 = 32;
/// The fewest lights a light buffer holds.
const MIN_LIGHTS: u64 = 64;
/// The fewest words a `light_grid` buffer holds.
const MIN_WORDS: u64 = 4096;
/// Threads per binning workgroup, one per cluster.
const WORKGROUP: u32 = 64;
/// The binning shader, which composes after `src/material/light.wgsl`.
const BINNING: &str = include_str!("light_binning.wgsl");

/// How a context hands lights to its shaders, picked once per context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LightPath {
    /// Storage roots, with a compute pass that bins the lights. `limit` is
    /// the largest storage buffer a binding takes.
    Clustered { limit: u64 },
    /// A uniform block of `count` lights, which every fragment loops over.
    Uniform { count: u32 },
}

impl LightPath {
    /// Picks the path `device` supports. The clustered path needs compute
    /// shaders, storage buffers in fragment shaders and a backend other than GL.
    pub(crate) fn pick(adapter: &wgpu::Adapter, device: &wgpu::Device) -> Self {
        let limits = device.limits();
        let flags = adapter.get_downlevel_capabilities().flags;
        let limit = limits
            .max_storage_buffer_binding_size
            .min(limits.max_buffer_size);
        let clustered = adapter.get_info().backend != wgpu::Backend::Gl
            && flags.contains(
                wgpu::DownlevelFlags::COMPUTE_SHADERS | wgpu::DownlevelFlags::FRAGMENT_STORAGE,
            )
            && limits.max_storage_buffers_per_shader_stage >= 2
            && limits.max_compute_workgroup_size_x >= WORKGROUP
            && limits.max_compute_invocations_per_workgroup >= WORKGROUP
            && limits.max_compute_workgroups_per_dimension >= GRID_CELLS / WORKGROUP
            && limits.max_compute_workgroup_storage_size >= 4096
            && limit >= 1 << 20;
        match clustered {
            true => LightPath::Clustered { limit },
            false => LightPath::Uniform {
                count: uniform_count(limits.max_uniform_buffer_binding_size),
            },
        }
    }

    /// The most lights one shader takes on this path.
    pub(crate) fn cap(&self) -> u32 {
        match *self {
            LightPath::Clustered { limit } => {
                let lights = limit.saturating_sub(u64::from(LIGHTS_HEADER)) / u64::from(LIGHT_SIZE);
                u32::try_from(lights).unwrap_or(u32::MAX)
            }
            LightPath::Uniform { count } => count,
        }
    }

    /// The lights a storage `lights` root lowers to a uniform block of, on the uniform path.
    pub(crate) fn uniform_lights(&self) -> Option<u32> {
        match *self {
            LightPath::Clustered { .. } => None,
            LightPath::Uniform { count } => Some(count),
        }
    }
}

/// The lights a uniform block of at most `size` bytes holds, capped at 64 KiB
/// so a driver that reports a huge block doesn't make a huge shader.
fn uniform_count(size: u64) -> u32 {
    let lights = size.min(65536).saturating_sub(u64::from(LIGHTS_HEADER)) / u64::from(LIGHT_SIZE);
    u32::try_from(lights).unwrap_or(u32::MAX)
}

/// The layout entries of a shader's light roots on `path`, by group.
pub(crate) fn layout_entries(
    roots: &LightRoots,
    path: LightPath,
) -> Vec<(u32, wgpu::BindGroupLayoutEntry)> {
    let entry = |binding: u32, ty: wgpu::BufferBindingType, size: u64| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: roots.stages,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: false,
            min_binding_size: std::num::NonZeroU64::new(size),
        },
        count: None,
    };
    let storage = wgpu::BufferBindingType::Storage { read_only: true };
    let uniform = wgpu::BufferBindingType::Uniform;
    let mut entries = Vec::new();
    let lights = roots.lights;
    let header = u64::from(LIGHTS_HEADER);
    let light = u64::from(LIGHT_SIZE);
    match (roots.shape, path) {
        (LightShape::Storage, LightPath::Clustered { .. }) => {
            entries.push((lights.group, entry(lights.binding, storage, header + light)));
            if let Some(grid) = roots.grid {
                entries.push((grid.group, entry(grid.binding, storage, 4)));
            }
        }
        (LightShape::Storage, LightPath::Uniform { count }) => {
            let size = header + light * u64::from(count);
            entries.push((lights.group, entry(lights.binding, uniform, size)));
        }
        (LightShape::Array { count }, _) => {
            let size = u64::from(ARRAY_HEADER) + light * u64::from(count);
            entries.push((lights.group, entry(lights.binding, uniform, size)));
        }
    }
    entries
}

/// The buffers one lit shader binds.
#[derive(Debug, Clone)]
pub(crate) struct LightBuffers {
    lights: wgpu::Buffer,
    grid: Option<wgpu::Buffer>,
}

impl LightBuffers {
    /// Each buffer with the group and binding of the root it fills.
    pub(crate) fn bindings(&self, roots: &LightRoots) -> Vec<(u32, u32, wgpu::Buffer)> {
        let lights = (
            roots.lights.group,
            roots.lights.binding,
            self.lights.clone(),
        );
        let grid = roots
            .grid
            .zip(self.grid.clone())
            .map(|(root, buffer)| (root.group, root.binding, buffer));
        std::iter::once(lights).chain(grid).collect()
    }
}

/// What a light set's buffers hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SetShape {
    /// Storage roots with a grid.
    Clustered,
    /// A storage root lowered to a uniform block of `count` lights.
    Uniform { count: u32 },
    /// The older uniform `LightArray` of `count` lights.
    Array { count: u32 },
}

impl SetShape {
    fn new(roots: &LightRoots, path: LightPath) -> Self {
        match (roots.shape, path) {
            (LightShape::Storage, LightPath::Clustered { .. }) => SetShape::Clustered,
            (LightShape::Storage, LightPath::Uniform { count }) => SetShape::Uniform { count },
            (LightShape::Array { count }, _) => SetShape::Array { count },
        }
    }
}

/// A set's lights packed for the GPU: the lights without a range first.
#[derive(Debug, Default, PartialEq)]
struct Packed {
    array: Vec<u8>,
    count: u32,
    global_count: u32,
    /// The bounded lights, by their index in `array`.
    spheres: Vec<(u32, Sphere)>,
}

impl Packed {
    fn new(lights: &[GpuLight]) -> Self {
        let mut packed = Packed::default();
        let bounds: Vec<Option<Sphere>> = lights.iter().map(Sphere::around).collect();
        for (light, _) in lights
            .iter()
            .zip(&bounds)
            .filter(|(_, sphere)| sphere.is_none())
        {
            light.write(&mut packed.array);
        }
        packed.global_count = (packed.array.len() / GpuLight::SIZE) as u32;
        for (light, sphere) in lights.iter().zip(&bounds) {
            if let Some(sphere) = sphere {
                let index = (packed.array.len() / GpuLight::SIZE) as u32;
                packed.spheres.push((index, *sphere));
                light.write(&mut packed.array);
            }
        }
        packed.count = (packed.array.len() / GpuLight::SIZE) as u32;
        packed
    }
}

/// The 144-byte `Lights` header.
fn lights_header(
    count: u32,
    global_count: u32,
    promoted_count: u32,
    ambient: [f32; 3],
    grid: Option<&Grid>,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(LIGHTS_HEADER as usize);
    for word in [count, global_count, promoted_count, 0] {
        out.extend_from_slice(&word.to_le_bytes());
    }
    for value in ambient {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.extend_from_slice(&[0; 4]);
    match grid {
        Some(grid) => {
            write_matrix(&grid.frustum.view_proj, &mut out);
            grid.write_tail(&mut out);
            out.extend_from_slice(&grid.reach.to_le_bytes());
            out.extend_from_slice(&[0; 4]);
        }
        None => out.resize(LIGHTS_HEADER as usize, 0),
    }
    out
}

fn write_matrix(matrix: &Mat4, out: &mut Vec<u8>) {
    for value in matrix.to_cols_array() {
        out.extend_from_slice(&value.to_le_bytes());
    }
}

/// The bits of a matrix, which compare exactly.
fn matrix_bits(matrix: &Mat4) -> [u32; 16] {
    matrix.to_cols_array().map(f32::to_bits)
}

/// A buffer with the size it was created at.
#[derive(Debug)]
struct Sized {
    buffer: wgpu::Buffer,
    size: u64,
}

impl Sized {
    fn new(context: &RenderContext, label: &str, size: u64) -> Self {
        Sized {
            buffer: create_buffer(context, label, size, wgpu::BufferUsages::STORAGE),
            size,
        }
    }
}

/// One pass's lights for the shaders that share a light list.
#[derive(Debug)]
pub(crate) struct LightSet {
    shape: SetShape,
    lights: wgpu::Buffer,
    /// The lights `lights` has room for.
    capacity: u64,
    /// The `light_grid` root of a clustered set, which binds with or without a grid.
    light_grid: Option<Sized>,
    /// The bins a clustered set's last binning read.
    bins: Option<Sized>,
    /// The light epoch at the last pack.
    epoch: u64,
    packed: Packed,
    ambient: [f32; 3],
    /// The bytes last uploaded to `lights`, from its start.
    uploaded: Vec<u8>,
    /// The binning matrix at the last bin.
    matrix: Option<[u32; 16]>,
    /// The grid and promoted count the last bin produced.
    binned: Option<(Grid, u32)>,
    /// The render that last used this set.
    used: u64,
}

impl LightSet {
    fn new(context: &RenderContext, shape: SetShape, cap: u32) -> Self {
        let (capacity, size, usage) = match shape {
            SetShape::Clustered => (
                MIN_LIGHTS.min(u64::from(cap)),
                u64::from(LIGHTS_HEADER) + u64::from(LIGHT_SIZE) * MIN_LIGHTS.min(u64::from(cap)),
                wgpu::BufferUsages::STORAGE,
            ),
            SetShape::Uniform { count } => (
                u64::from(count),
                u64::from(LIGHTS_HEADER) + u64::from(LIGHT_SIZE) * u64::from(count),
                wgpu::BufferUsages::UNIFORM,
            ),
            SetShape::Array { count } => (
                u64::from(count),
                u64::from(ARRAY_HEADER) + u64::from(LIGHT_SIZE) * u64::from(count),
                wgpu::BufferUsages::UNIFORM,
            ),
        };
        let light_grid = (shape == SetShape::Clustered)
            .then(|| Sized::new(context, "Light Grid", MIN_WORDS * 4));
        LightSet {
            shape,
            lights: create_buffer(context, "Lights", size, usage),
            capacity,
            light_grid,
            bins: None,
            epoch: 0,
            packed: Packed::default(),
            ambient: [0.0; 3],
            uploaded: Vec::new(),
            matrix: None,
            binned: None,
            used: 0,
        }
    }

    /// Forgets what the set packed, uploaded and binned, so its next prepare
    /// does all three again.
    fn forget(&mut self) {
        self.epoch = 0;
        self.packed = Packed::default();
        self.uploaded.clear();
        self.matrix = None;
        self.binned = None;
    }

    fn buffers(&self) -> LightBuffers {
        LightBuffers {
            lights: self.lights.clone(),
            grid: self.light_grid.as_ref().map(|grid| grid.buffer.clone()),
        }
    }

    /// Repacks the set when a light changed and rebins it when a sphere or the matrix did.
    fn prepare(
        &mut self,
        context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        shader: &ShaderObject,
        matrix: Option<Mat4>,
        limit: u64,
    ) -> Result<(), RendererError> {
        let epoch = current_epoch();
        let mut changed = false;
        let mut rebin = false;
        if self.epoch != epoch {
            self.epoch = epoch;
            self.ambient = shader.ambient();
            let packed = Packed::new(&shader.gpu_lights());
            rebin = packed.spheres != self.packed.spheres;
            self.packed = packed;
            changed = true;
        }
        if self.shape != SetShape::Clustered {
            if changed {
                let bytes = self.fixed_bytes();
                self.upload(context, bytes);
            }
            return Ok(());
        }
        let cap = (limit.saturating_sub(u64::from(LIGHTS_HEADER))) / u64::from(LIGHT_SIZE);
        if u64::from(self.packed.count) > self.capacity {
            self.capacity = grow(u64::from(self.packed.count), MIN_LIGHTS).min(cap);
            let size = u64::from(LIGHTS_HEADER) + u64::from(LIGHT_SIZE) * self.capacity;
            self.lights = create_buffer(context, "Lights", size, wgpu::BufferUsages::STORAGE);
            self.uploaded.clear();
            rebin = true;
        }
        let bits = matrix.map(|matrix| matrix_bits(&matrix));
        if bits != self.matrix {
            self.matrix = bits;
            rebin = true;
        }
        if rebin {
            let frustum = matrix.and_then(Frustum::from_matrix);
            self.binned = self.bin(context, encoder, frustum, limit)?;
            changed = true;
        }
        if !changed {
            return Ok(());
        }
        let (grid, promoted) = match self.binned.as_ref() {
            Some((grid, promoted)) => (Some(grid), *promoted),
            None => (None, 0),
        };
        let mut bytes = lights_header(
            self.packed.count,
            self.packed.global_count,
            promoted,
            self.ambient,
            grid,
        );
        bytes.extend_from_slice(&self.packed.array);
        self.upload(context, bytes);
        Ok(())
    }

    /// The bytes of a set without a grid: the uniform block a storage root
    /// lowers to, or the older `LightArray`.
    fn fixed_bytes(&self) -> Vec<u8> {
        let packed = &self.packed;
        let mut bytes = match self.shape {
            SetShape::Array { .. } => {
                let mut header = Vec::with_capacity(ARRAY_HEADER as usize);
                for word in [packed.count, 0, 0, 0] {
                    header.extend_from_slice(&word.to_le_bytes());
                }
                for value in self.ambient {
                    header.extend_from_slice(&value.to_le_bytes());
                }
                header.extend_from_slice(&[0; 4]);
                header
            }
            _ => lights_header(packed.count, packed.count, 0, self.ambient, None),
        };
        bytes.extend_from_slice(&packed.array);
        bytes
    }

    /// Writes the bytes of `bytes` that differ from the last upload.
    fn upload(&mut self, context: &RenderContext, bytes: Vec<u8>) {
        let header = match self.shape {
            SetShape::Array { .. } => ARRAY_HEADER as usize,
            _ => LIGHTS_HEADER as usize,
        };
        let split = header.min(bytes.len());
        let same =
            |range: std::ops::Range<usize>| self.uploaded.get(range.clone()) == bytes.get(range);
        if !same(0..split) {
            context.queue.write_buffer(&self.lights, 0, &bytes[..split]);
        }
        if bytes.len() > split && !same(split..bytes.len()) {
            context
                .queue
                .write_buffer(&self.lights, split as u64, &bytes[split..]);
        }
        self.uploaded = bytes;
    }

    /// Bins the bounded lights in `frustum`, and returns the grid with the
    /// number of lights every cluster reads. `None` means the set has no grid
    /// and every fragment loops over every light.
    fn bin(
        &mut self,
        context: &RenderContext,
        encoder: &mut wgpu::CommandEncoder,
        frustum: Option<Frustum>,
        limit: u64,
    ) -> Result<Option<(Grid, u32)>, RendererError> {
        #[cfg(test)]
        let frustum = frustum.filter(|_| !context.light_probe.no_grid());
        let spheres: Vec<Sphere> = self
            .packed
            .spheres
            .iter()
            .map(|(_, sphere)| *sphere)
            .collect();
        let Some(frustum) = frustum.filter(|_| !spheres.is_empty()) else {
            return Ok(None);
        };
        let grid = frustum.grid(frustum.reach(&spheres));
        let boxed: Vec<(u32, Sphere, grid::Cells)> = self
            .packed
            .spheres
            .iter()
            .filter_map(|(index, sphere)| grid.cells(sphere).map(|cells| (*index, *sphere, cells)))
            .collect();

        // A light whose box covers half the grid is cheaper to read everywhere.
        let (mut promoted, mut kept): (Vec<_>, Vec<_>) = boxed
            .into_iter()
            .partition(|(_, _, cells)| cells.volume() > GRID_CELLS / 2);
        let budget = limit / 4;
        let fixed = 2 * u64::from(GRID_CELLS);
        let listed = |kept: &[(u32, Sphere, grid::Cells)]| -> u64 {
            kept.iter()
                .map(|(_, _, cells)| u64::from(cells.volume()))
                .sum()
        };
        let mut words = promoted.len() as u64 + fixed + listed(&kept);
        if words > budget {
            // Promote the largest boxes until every index list fits one binding.
            kept.sort_by_key(|(index, _, cells)| (std::cmp::Reverse(cells.volume()), *index));
            while words > budget && !kept.is_empty() {
                let largest = kept.remove(0);
                words = words + 1 - u64::from(largest.2.volume());
                promoted.push(largest);
            }
            kept.sort_by_key(|(index, _, _)| *index);
        }
        promoted.sort_by_key(|(index, _, _)| *index);
        if kept.is_empty() {
            return Ok(None);
        }

        // Buffers never shrink, and a grown one comes new.
        let bins_size = BINS_HEADER + BIN_SIZE * kept.len() as u64;
        if self.bins.as_ref().is_none_or(|bins| bins.size < bins_size) {
            let size = BINS_HEADER + BIN_SIZE * grow(kept.len() as u64, MIN_LIGHTS);
            self.bins = Some(Sized::new(context, "Light Bins", size));
        }
        if self
            .light_grid
            .as_ref()
            .is_none_or(|grid| grid.size < words * 4)
        {
            let size = grow(words, MIN_WORDS).min(budget) * 4;
            self.light_grid = Some(Sized::new(context, "Light Grid", size));
        }
        let (Some(light_grid), Some(bins_buffer)) = (self.light_grid.as_ref(), self.bins.as_ref())
        else {
            return Ok(None);
        };

        if !promoted.is_empty() {
            let indices: Vec<u8> = promoted
                .iter()
                .flat_map(|(index, _, _)| index.to_le_bytes())
                .collect();
            context.queue.write_buffer(&light_grid.buffer, 0, &indices);
        }
        let list_base = promoted.len() as u32 + 2 * GRID_CELLS;
        let mut bins = Vec::with_capacity((BINS_HEADER + BIN_SIZE * kept.len() as u64) as usize);
        write_matrix(&grid.frustum.view_proj, &mut bins);
        grid.write_tail(&mut bins);
        for word in [list_base, kept.len() as u32, 0, 0, 0, 0] {
            bins.extend_from_slice(&word.to_le_bytes());
        }
        for (index, sphere, cells) in &kept {
            write_vec3(sphere.center, &mut bins);
            bins.extend_from_slice(&sphere.radius.to_le_bytes());
            for word in [*index, cells.pack(), 0, 0] {
                bins.extend_from_slice(&word.to_le_bytes());
            }
        }
        context.queue.write_buffer(&bins_buffer.buffer, 0, &bins);
        context.dispatch_binning(encoder, &bins_buffer.buffer, &light_grid.buffer)?;
        Ok(Some((grid, promoted.len() as u32)))
    }
}

fn write_vec3(value: Vec3, out: &mut Vec<u8>) {
    for component in value.to_array() {
        out.extend_from_slice(&component.to_le_bytes());
    }
}

/// The next power of two at or above `needed`, and at least `least`.
fn grow(needed: u64, least: u64) -> u64 {
    needed.max(least).next_power_of_two()
}

fn create_buffer(
    context: &RenderContext,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    context.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

/// The compute pipeline that bins lights.
#[derive(Debug)]
pub(crate) struct BinningPipeline {
    pipeline: wgpu::ComputePipeline,
    layout: wgpu::BindGroupLayout,
}

/// The binning module: the light types and constants, then the binning pass.
fn binning_module() -> Result<naga::Module, crate::ShaderError> {
    let source = format!("{}\n{}", crate::material::LIGHT, BINNING);
    Ok(naga::front::wgsl::parse_str(&source)?)
}

impl BinningPipeline {
    fn new(device: &wgpu::Device) -> Result<Self, RendererError> {
        let module = binning_module()?;
        let entry = |binding: u32, size: u64| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(size),
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Light Binning Layout"),
            entries: &[entry(0, BINS_HEADER + BIN_SIZE), entry(1, 4)],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Light Binning Pipeline Layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Light Binning"),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Light Binning"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("bin_lights"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
        Ok(BinningPipeline { pipeline, layout })
    }
}

/// Counts binning dispatches and turns grids off, for the tests.
#[cfg(test)]
#[derive(Debug, Default)]
pub(crate) struct LightProbe {
    dispatches: std::sync::atomic::AtomicU64,
    no_grid: std::sync::atomic::AtomicBool,
}

#[cfg(test)]
impl LightProbe {
    fn no_grid(&self) -> bool {
        self.no_grid.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// The key of a light set: the light ids in order, the placeholder and ambient
/// bits, the shape, and the shader when the pass has no camera to bin in.
fn set_key(shader: &ShaderObject, shape: SetShape, own_matrix: bool) -> u64 {
    let mut hasher = DefaultHasher::new();
    shader.hash_lights(&mut hasher);
    shape.hash(&mut hasher);
    if own_matrix {
        (shader as *const ShaderObject as usize).hash(&mut hasher);
    }
    hasher.finish()
}

/// The matrix a shader without a pass camera bins in: its own `camera.view_proj`.
fn shader_matrix(shader: &ShaderObject) -> Option<Mat4> {
    match shader.get_uniform_data("camera.view_proj") {
        Ok(crate::UniformData::Mat4(values)) => {
            let values: [f32; 16] = values.try_into().ok()?;
            Some(Mat4::from_cols_array(&values))
        }
        _ => None,
    }
}

impl RenderContext {
    /// The path this context hands lights to shaders on.
    pub(crate) fn light_path(&self) -> LightPath {
        *self.light_path.read()
    }

    /// Starts a render that draws, after the texture pool starts it. Frees the
    /// light sets no render used for `IDLE_RENDERS` renders, whether or not a
    /// lit pass renders now. After a failed frame, every other light set packs,
    /// uploads and bins again, since that frame's binning never ran.
    pub(crate) fn start_light_render(&self) {
        let render = self.texture_pool.read().render();
        let mut sets = self.light_sets.write();
        sets.retain(|_, set| render.saturating_sub(set.used) <= IDLE_RENDERS);
        if self
            .lights_failed
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            for set in sets.values_mut() {
                set.forget();
            }
        }
    }

    /// Packs and bins the lights of every lit shader on `pass`, at most once
    /// per render for each light set, and returns each shader's buffers by
    /// its address. A shader holding more lights than the device takes fails
    /// here, before any light buffer is touched.
    pub(crate) fn prepare_lights(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        pass: &PassObject,
    ) -> Result<HashMap<usize, LightBuffers>, RendererError> {
        let lit: Vec<Arc<ShaderObject>> = pass
            .shaders
            .read()
            .iter()
            .filter(|shader| shader.light_roots.is_some())
            .cloned()
            .collect();
        let mut prepared = HashMap::new();
        if lit.is_empty() {
            return Ok(prepared);
        }
        let path = self.light_path();
        for shader in &lit {
            if let Some(roots) = shader.light_roots.as_ref() {
                let cap = path.cap().min(roots.capacity());
                if shader.light_ids().len() as u64 > u64::from(cap) {
                    return Err(PassError::LightCapReached { cap }.into());
                }
            }
        }
        let limit = match path {
            LightPath::Clustered { limit } => limit,
            LightPath::Uniform { .. } => 0,
        };
        let render = self.texture_pool.read().render();
        let camera = pass
            .camera_snapshot
            .read()
            .map(|snapshot| Mat4::from_cols_array_2d(&snapshot.view_proj));
        let keyed: Vec<(&Arc<ShaderObject>, SetShape, (u64, u64))> = lit
            .iter()
            .filter_map(|shader| {
                let shape = SetShape::new(shader.light_roots.as_ref()?, path);
                let key = (pass.id, set_key(shader, shape, camera.is_none()));
                Some((shader, shape, key))
            })
            .collect();
        let mut sets = self.light_sets.write();
        for (shader, shape, key) in keyed.iter().copied() {
            shader.flush_pending();
            if !sets.contains_key(&key) {
                // Reuse the buffers of a set this pass left idle, which a light or
                // camera change orphaned. What it packed belongs to another key.
                let idle = sets
                    .iter()
                    .find(|(found, set)| {
                        found.0 == pass.id
                            && set.used < render
                            && set.shape == shape
                            && !keyed.iter().any(|(_, _, wanted)| wanted == *found)
                    })
                    .map(|(key, _)| *key);
                let set = match idle.and_then(|idle| sets.remove(&idle)) {
                    Some(mut set) => {
                        set.forget();
                        set
                    }
                    None => LightSet::new(self, shape, path.cap()),
                };
                sets.insert(key, set);
            }
            let Some(set) = sets.get_mut(&key) else {
                continue;
            };
            if set.used != render {
                set.used = render;
                let matrix = camera.or_else(|| shader_matrix(shader));
                set.prepare(self, encoder, shader, matrix, limit)?;
            }
            prepared.insert(Arc::as_ptr(shader) as usize, set.buffers());
        }
        Ok(prepared)
    }

    /// Records the binning compute pass for one light set.
    fn dispatch_binning(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        bins: &wgpu::Buffer,
        light_grid: &wgpu::Buffer,
    ) -> Result<(), RendererError> {
        let binning = self.binning_pipeline()?;
        let group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Light Binning"),
            layout: &binning.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bins.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_grid.as_entire_binding(),
                },
            ],
        });
        let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Light Binning"),
            timestamp_writes: None,
        });
        compute.set_pipeline(&binning.pipeline);
        compute.set_bind_group(0, &group, &[]);
        compute.dispatch_workgroups(GRID_CELLS / WORKGROUP, 1, 1);
        #[cfg(test)]
        self.light_probe
            .dispatches
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// The binning pipeline, built on first use. A build that fails isn't
    /// kept, so the next frame builds it again and reports the same error.
    /// On wasm the error lands after the insert, so the pipeline leaves then.
    fn binning_pipeline(&self) -> Result<Arc<BinningPipeline>, RendererError> {
        if let Some(pipeline) = self.binning.read().clone() {
            return Ok(pipeline);
        }
        let cache = Arc::downgrade(&self.binning);
        let failed = self.lights_failed.clone();
        let evict = move || {
            if let Some(cache) = cache.upgrade() {
                *cache.write() = None;
            }
            failed.store(true, std::sync::atomic::Ordering::Relaxed);
        };
        let pipeline = Arc::new(self.validate_with(
            "light binning pipeline",
            || BinningPipeline::new(&self.device),
            evict,
        )?);
        *self.binning.write() = Some(pipeline.clone());
        Ok(pipeline)
    }
}
