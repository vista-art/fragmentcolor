//! This file contains static copies of wgpu's Limits struct,
//! because we can't call its methods from our options' static map.
//!
//! NOTE: While this is unlikely to change, it's a good idea to check the
//! upstream implementation from time to time to make sure this is up to date.

/// Limits::downlevel_defaults(). This is a set of limits that is guaranteed
/// to work on almost all backends, including "downlevel" backends such as
/// OpenGL and D3D11, other than WebGL. For most applications we recommend
/// using these limits, assuming they are high enough for your application,
/// and you do not intent to support WebGL.
pub(crate) const DOWNLEVEL_DEFAULTS: wgpu::Limits = wgpu::Limits {
    max_texture_dimension_1d: 2048,
    max_texture_dimension_2d: 2048,
    max_texture_dimension_3d: 256,
    max_texture_array_layers: 256,
    max_bind_groups: 4,
    max_bindings_per_bind_group: 1000,
    max_dynamic_uniform_buffers_per_pipeline_layout: 8,
    max_dynamic_storage_buffers_per_pipeline_layout: 4,
    max_sampled_textures_per_shader_stage: 16,
    max_samplers_per_shader_stage: 16,
    max_storage_buffers_per_shader_stage: 4,
    max_storage_textures_per_shader_stage: 4,
    max_uniform_buffers_per_shader_stage: 12,
    max_uniform_buffer_binding_size: 16 << 10,
    max_storage_buffer_binding_size: 128 << 20,
    max_vertex_buffers: 8,
    max_vertex_attributes: 16,
    max_vertex_buffer_array_stride: 2048,
    max_push_constant_size: 0,
    min_uniform_buffer_offset_alignment: 256,
    min_storage_buffer_offset_alignment: 256,
    max_inter_stage_shader_components: 60,
    max_compute_workgroup_storage_size: 16352,
    max_compute_invocations_per_workgroup: 256,
    max_compute_workgroup_size_x: 256,
    max_compute_workgroup_size_y: 256,
    max_compute_workgroup_size_z: 64,
    max_compute_workgroups_per_dimension: 65535,
    max_buffer_size: 1 << 28,
    max_non_sampler_bindings: 1_000_000,
};

/// Limits::downlevel_webgl2_defaults() This is a set of limits that is lower
/// even than the [downlevel_defaults()], configured to be low enough to support
/// running in the browser using WebGL2.
pub(crate) const DOWNLEVEL_WEBGL2: wgpu::Limits = wgpu::Limits {
    max_texture_dimension_1d: 2048,
    max_texture_dimension_2d: 2048,
    max_texture_dimension_3d: 256,
    max_texture_array_layers: 256,
    max_bind_groups: 4,
    max_bindings_per_bind_group: 1000,
    max_dynamic_uniform_buffers_per_pipeline_layout: 8,
    max_dynamic_storage_buffers_per_pipeline_layout: 0,
    max_sampled_textures_per_shader_stage: 16,
    max_samplers_per_shader_stage: 16,
    max_storage_buffers_per_shader_stage: 0,
    max_storage_textures_per_shader_stage: 0,
    max_uniform_buffers_per_shader_stage: 12,
    max_uniform_buffer_binding_size: 16 << 10,
    max_storage_buffer_binding_size: 0,
    max_vertex_buffers: 8,
    max_vertex_attributes: 16,
    max_vertex_buffer_array_stride: 255,
    max_push_constant_size: 0,
    min_uniform_buffer_offset_alignment: 256,
    min_storage_buffer_offset_alignment: 256,
    max_inter_stage_shader_components: 60,
    max_compute_workgroup_storage_size: 0,
    max_compute_invocations_per_workgroup: 0,
    max_compute_workgroup_size_x: 0,
    max_compute_workgroup_size_y: 0,
    max_compute_workgroup_size_z: 0,
    max_compute_workgroups_per_dimension: 0,
    max_buffer_size: 1 << 28,
    max_non_sampler_bindings: 1_000_000,
};

/// Limits::default(). This is the set of limits that is guaranteed to work on
/// all modern backends and is guaranteed to be supported by WebGPU. Applications
/// needing more modern features can use this as a reasonable set of limits if
/// they are targeting only desktop and modern mobile devices.
pub(crate) const DEFAULT_LIMITS: wgpu::Limits = wgpu::Limits {
    max_texture_dimension_1d: 8192,
    max_texture_dimension_2d: 8192,
    max_texture_dimension_3d: 2048,
    max_texture_array_layers: 256,
    max_bind_groups: 4,
    max_bindings_per_bind_group: 1000,
    max_dynamic_uniform_buffers_per_pipeline_layout: 8,
    max_dynamic_storage_buffers_per_pipeline_layout: 4,
    max_sampled_textures_per_shader_stage: 16,
    max_samplers_per_shader_stage: 16,
    max_storage_buffers_per_shader_stage: 8,
    max_storage_textures_per_shader_stage: 4,
    max_uniform_buffers_per_shader_stage: 12,
    max_uniform_buffer_binding_size: 64 << 10,
    max_storage_buffer_binding_size: 128 << 20,
    max_vertex_buffers: 8,
    max_buffer_size: 1 << 28,
    max_vertex_attributes: 16,
    max_vertex_buffer_array_stride: 2048,
    min_uniform_buffer_offset_alignment: 256,
    min_storage_buffer_offset_alignment: 256,
    max_inter_stage_shader_components: 60,
    max_compute_workgroup_storage_size: 16384,
    max_compute_invocations_per_workgroup: 256,
    max_compute_workgroup_size_x: 256,
    max_compute_workgroup_size_y: 256,
    max_compute_workgroup_size_z: 64,
    max_compute_workgroups_per_dimension: 65535,
    max_push_constant_size: 0,
    max_non_sampler_bindings: 1_000_000,
};
