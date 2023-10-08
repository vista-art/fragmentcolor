# Shadertoy Interoperability

## Decision

We can have custom entry points, including a `mainImage()` with exactly the same signature as Shadertoy, so users can prototype the shader there and copy & paste it into our library.

## Handling user-defined shaders

We don't need to build a parser and generate glue code, like we intended to do before.

The library can receive the custom shader code as a string, validate it, and merge it to the global shader before building the pipeline.

Custom-shaded objects would still share the same pipeline as the other objects. The only difference is that its Local uniform would have a flag indicating it uses a custom shader, and a pointer to its entry point.
