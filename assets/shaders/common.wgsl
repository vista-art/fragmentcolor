#define_import_path common

/// Common WGSL shared among all Ruffle shaders.
/// Ruffle prepends this file onto every shader at runtime.

/// Global uniforms that are constant throughout a frame.
struct Globals {
    // The view matrix determined by the viewport and stage.
    view_matrix: mat4x4<f32>,
};

/// Transform uniforms that are changed per object.
struct Transforms {
    /// The world matrix that transforms this object into stage space.
    world_matrix: mat4x4<f32>,
};

/// Transform uniforms that are changed per object.
struct ColorTransforms {
    /// The multiplicative color transform of this object.
    mult_color: vec4<f32>,

    /// The additive color transform of this object.
    add_color: vec4<f32>,
};

/// Uniforms used by texture draws (bitmaps and gradients).
struct TextureTransforms {
    /// The transform matrix of the gradient or texture.
    /// Transforms from object space to UV space.
    texture_matrix: mat4x4<f32>,
};

struct PushConstants {
    transforms: Transforms,
    colorTransforms: ColorTransforms,
}

/// The vertex format shared among most shaders.
struct VertexInput {
    /// The position of the vertex in object space.
    @location(0) position: vec2<f32>,
};

/// Common uniform layout shared by all shaders.
@group(0) @binding(0) var<uniform> globals: Globals;
