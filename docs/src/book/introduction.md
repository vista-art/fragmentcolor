# Introduction

The `plrender` library aims to provide a unified API
to render **video enrichments** in the GPU (with a CPU fallback) across
multiple platforms and environments with native performance and minimal footprint.

The library is implemented in Rust,
leveraging the [wgpu](https://github.com/gfx-rs/wgpu) library for rendering,
which enables it to target a wide range of platforms:

| Backend   |   Web   |  Linux  |  MacOS  | Windows | Android |   iOS   | CI / CD |
| :-------- | :-----: | :-----: | :-----: | :-----: | :-----: | :-----: | :-----: |
| Metal     |   no    |   no    | **Yes** |   no    |   no    | **Yes** |   no    |
| Vulkan    |   no    | **Yes** | Yes[^2] | **Yes** | **Yes** | **Yes** | **Yes** |
| OpenGL    |   no    | **Yes** | Yes[^3] | **Yes** | **Yes** | Yes[^4] |   no    |
| WebGL     | Yes[^1] |   no    |   no    |   no    |   no    |   no    |   no    |
| WebGPU    | **Yes** |   no    |   no    |   no    |   no    |   no    |   no    |
| Dx11/Dx12 |   no    |   no    |   no    | **Yes** |   no    |   no    |   no    |

[^1]: Vertex and Fragment shaders only. No support for Compute shaders.
[^2]: Available through [MoltenVK](https://github.com/KhronosGroup/MoltenVK), a translation layer to Metal.
[^3]: OpenGL is deprecated in MacOS. It runs up to version 4.1 only: no support for Compute shaders.
[^4]: OpenGL is deprecated in iOS. It runs up to version 4.1 only: no support for Compute shaders.
