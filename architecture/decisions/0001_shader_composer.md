# Shader Composer

## Decision

The [shader parser idea](#initial-idea-the-shader-parser) has been abandoned in favor of a **Shader Composer**, more specifically [naga-oil](https://github.com/bevyengine/naga_oil).

This allows us to mix custom shaders with our built-in shaders at runtime, while greatly improving maintainability and performance.

### Shader Inputs

The old shader input in our POC was a hardcoded Circle struct, which inspired the parser idea:

```glsl
struct Circle {
    vec2 position;
    float radius;
    float border;
    vec4 color;
};
uniform Circle circle;
```

The new input is a generic **Globals** struct with a 4x4 view_projection matrix, and a **Locals** struct with a 4x4 transform matrix, which is a common pattern in the CG industry:

```glsl
struct Globals {
    mat4 view_projection;
    // ... other shared global data
};

struct Locals {
    mat4 transform;
    vec4 color;
    vec2 bounds;
    // ... other object metadata
};
```

This setup allows us to create arbitrary user-defined scenes, while improving performance by packing multiples objects in a single `draw()` call.

## Context

### Initial Idea: The Shader Parser

Our initial idea was to build a **Shader Parser** to generate the **Javascript** + **Python** scene objects with getters and setters for each property in the shader's struct.

This would enable us to use existing tooling like [Shadertoy](https://www.shadertoy.com/) or [KodeLife](https://hexler.net/kodelife) to prototype the shaders, and then run the parser to generate the glue code for the library. The structs defined in the shader would be the blueprint for the public API object.

#### Shader Parser Example

For the **Circle** struct in the shader above, the parser would generate a `Circle` class with the following API in JS and Python:

```python
circle.set_radius(float);
circle.set_border(float);

circle.set_position_x(float);
circle.set_position_y(float);

circle.set_color_r(float);
circle.set_color_g(float);
circle.set_color_b(float);
circle.set_color_a(float);

# The Vec4 components xyzw, rgba, stpq, and 0123 are equivalent
# So this would be valid code as well:
circle.set_color_x(float);
circle.set_position_r(float);
circle.set_position(r: float, y: float); # named
circle.set_position(float, float); # positional
```

## The Problem

As I started implementing the API, I realized that naively building a 1:1 map from a Shader struct to a Scene object in the Public API cannot work without severely sacrificing performance.

The problem is that **for each different shader, the library needs to rebuild the rendering pipeline**, which is a very expensive operation. While we can fit multiple pipelines in one frame, doing it once per object cannot scale beyond basic use cases.

If we have one hardcoded shader for each user-created scene object, we would need to rebuild the pipeline **multiple times per frame**, which would introduce a huge performance bottleneck.

## The Solution

We want to have the same **prototyping capabilities** as Shadertoy, while supporting a **user-defined scene** without killing the library's performance.

How can we achieve that?

### How Other Libraries do It

Because the shader is the topmost upstram code in a rendering library, everything else needs to be built around it. Libraries have to define their own constraints, coordinate system, and data structures to fit a fairly static set of shaders.

Most libraries have at least one common shader that can be shared between all scene objects, so they can pack as many objects as possible in a single `draw()` call.

They normally do that with two structs: a **Globals** struct with a 4x4 view_projection matrix, and a **Locals** struct with a 4x4 transform matrix. Both structs can contain metadata, but the dual transformation matrices is a widely adopted common pattern.

So, instead of a hardcoded Circle struct like we had before, the shader inputs look like this instead:

```glsl
struct Globals {
    mat4 view_projection;
    // ... other shared global data
};

struct Locals {
    mat4 transform;
    vec4 color;
    vec2 bounds;
    // ... other object metadata
};
```

### How about Shadertoy?

Shadertoy has a static scene containing a single fullscreen quad. There is no concept of a scene graph or objects.

It compiles a single static shader with global variables like resolution, time, delta, mouse position, etc., and this shader calls a custom `mainImage()` entry point, where the user can write their fragment code.

When the user changes the code, it recompiles the pipeline, but this is infrequent enough to not be a problem. By comparison, we'd need to recompile the pipeline multiple times per frame if we had a different shader for each object in the scene.
