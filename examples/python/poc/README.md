# First POCs

This folder exists for historical reasons.

It contains two Proof-of-Concepts that may seem unrelated to our library, but they were what started it all!

- **Gaze Overlay** (`./gaze_overlay`)
- **AOI Extraction** (`/aoi_extraction`)
- **Python API Draft** (`/api_draft`)

## Gaze Overlay

We started this project to achieve **visual consistency** between Javascript and Python with a shared API.

### This example proves

- **Visual consistency**
- **Shaders as a shared language**
- **Shadertoy interoperability**

This example renders a gaze circle on top of a video using a signed distance function to draw the circle, which is still the same approach we use today.

We tested this same shader in all environments we were interested:

- **Python** (this example)
- **Web** (with [Shadertoy](https://www.shadertoy.com))
- **Desktop** (with [Kodelife](https://hexler.net/kodelife))

As we got consistent results in all of them, we decided to give this project a go.

## AOI Extraction

In the early stages of this project, we were exploring the potential of the `wgpu` library as our Hardware Abstraction Layer.

We created this example to test `wgpu` support for Python, and whether it supported **CPU fallback** for environments without a GPU.

### This example proves

- **Python support for WGPU**
- **CPU fallback for CI**
- **Compute shaders**

The **AOI Extraction** example uses a compute shader to extract AOIs from bitmasks encoded into a single PNG image.

It contains a **Dockerfile** targeted at CI pipelines which forces the library to use the CPU software rendering fallback adapter.

## PLRender API Draft

This file is **not a working example**. It was the result of a brainstorming session where we were exploring how the library API would look like.

While this example won't run, it was the **first usage** of the **"plrender"** keyword and is where the name of the library came from.
