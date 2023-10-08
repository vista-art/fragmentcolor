# Rendering pre-defined shapes

There are two main ways to render 2D shapes in a GPU:

- **tesselation**: we build the shape with many triangles, and the GPU renders it as a regular mesh.

- **distance functions**: the shape mesh is just a simple quad, and the actual shape is rendered by a Signed Distance Function (SDF) in the fragment shader.

## Decision

Support both, but add tesselation behind a feature flag.

We use distance functions for most of our pre-defined simple shapes. I plan to use tesselation to render complex shapes and SVG files in the future.

## Combining multiple shapes in one draw() call

Using the **Shader Composer**, we can create a Shapes shader file with all possible shapes defined as signed distance functions.

In the CPU side, we'll define an enum with all possible distance functions, and send a byte flag to the GPU to choose the correct function on-the-fly.

This way, all SDF shapes can share the same shader and the same draw call. We can render millions of them if we want.
