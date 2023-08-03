# RFC: PL Video Processor (vip)

## Request for Comments

Feedback on this proposal is welcome. Please share your thoughts on the design, the trade-offs considered, and any other aspects of the project you deem important.

## Summary

The `pl-video-processor` library aims to provide a unified API
to render **Pupil Labs' video enrichments** in the GPU (with a CPU fallback)
across multiple platforms and environments
with native performance and minimal footprint.

The library will be implemented in Rust,
leveraging the [wgpu](https://github.com/gfx-rs/wgpu) library for rendering,
which enables it to target a wide range of platforms,
including native (CLI), WebAssembly, Python, and Android/iOS.

## Motivation

Currently, we have to reimplement video enrichments processing and overlay rendering in a couple of our projects, using different technologies for different platforms (Web, Python & Android).

As we need to make sure they look similar across all platforms, this divergence leads to duplicated effort and poses challenges in maintaining and enhancing the rendering code.

By creating a unified video overlay renderer, the implementation of new visualizations can be centralized, making them readily available across all projects, improving maintainability and consistency.

## Distribution Targets

The first internal projects we want to target as consumers of this library are [pupil-cloud-ui](https://gitlab.com/pupil-labs/pupil-cloud/pupil-cloud-ui) and [pupil-invisible-toolkit](https://gitlab.com/pupil-labs/pupil-cloud/pupil-invisible-toolkit). The library will also ship with a standalone CLI application.

Support for our companion Andriod app is also planned for a later stage.

### Usage Examples

- **WebAssembly** ([wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) + [wasm-pack](https://rustwasm.github.io/wasm-pack/))

  ```bash
  npm install pl-video-processor
  ```

  ```javascript
  import "pl-video-processor";
  // Use it as a web component that wraps a video element
  ```
  
  ```html
  <pl-video-processor>
    <video src="scene.mp4" slot="scene" />
    <video src="eye.mp4" slot="eye" />
  </pl-video-processor>
  ```

- **Python library** ([pyo3](https://github.com/PyO3/pyo3) + [maturin](https://github.com/PyO3/maturin))

  ```bash
  pip install pl-video-processor
  ```

  ```python
  import pl_video_processor

  vip = pl_video_processor.VideoProcessor()
  vip.scene = "scene.mp4"
  vip.eye = "eye.mp4"

  for frame in vip.render():
    # Do something with the frame
    pass
  ```

- **CLI application**

  ```bash
  vip --scene scene.mp4 --gaze eye.mp4 --output output.mp4
  ```

  - **Implementation options:**
    - Directly in Rust with `clap`
    - In Python, with our `pl_video_processor` library + `click`

- **Android/iOS** (TBD)
  - The library can be used in our companion app as well. This will be implemented in a later stage.

## Features and API (WIP)

This is the design of the CLI implementation of the library. The Python and WebAssembly APIs will be similar, but idiomatic for their respective platforms.

### Configuration

The program will process the [Enrichment Options](#enrichment-options-wip) in this order of precedence:

1. From commandline arguments
2. From a specific `--config` argument
3. from a `vip.json` configuration file in the same directory
4. from environment variables

### Calculate gaze from eye video

The library receives two video streams as input: the **Scene Video** and the **Eye Video**. It calculates the gaze positions from the eye video, and renders the requested enrichments on top of the scene video.

It then outputs a single video stream with the enrichments applied:

```bash
vip --scene scene.mp4 --gaze eye.mp4 --output output.mp4
vip --scene scene.mp4 --gaze eye.mp4 --output pipe:1 | ffplay -fs -
```

### Calculate gaze from timeseries data

The library receives a scene video as input, and reads the **gaze positions from timeseries data**, either from a CSV file or from Clickhouse DB.

```bash
vip --scene scene.mp4 --gaze gaze.csv --output output.mp4
vip --scene scene.mp4 --gaze "<url to clickhouse db>" --output output.mp4
```

### Scene video undistortion

Optionally, the library can undistort the scene video and recalculate the gaze positions to match it:

```bash
vip --scene scene.mp4 --gaze eye.mp4 --output output.mp4 --undistort
```

### Enrichment Options (WIP)

#### Main options

```txt
  --scene, -s (Path; mandatory)
    Path to scene video file (default: None)

  --gaze, -g (Path or URL; mandatory)
    Path to gaze video/csv file or URL to Clickhouse DB (default: None)

  --output, -o (Path or pipe)
    Renders output video to file or stdout.
    Do not render if not provided. (default: None)

  --preview, -p (bool)
    Preview output video while rendering (default: True)

  --resolution, -r (ResolutionString)
    Video resolution of rendered output in the form of WIDTHxHEIGHT
    supports: original, 1920x1080, x1080, 1920x (default: original)

  --fps (int)
    Frames per second of the output video (default: 0)

  --start (float)
    Start time in seconds (default: 0.0)

  --stop (float)
    Stop time in seconds (default: 0.0)
```

#### Scene Video Options (config.scene)

```txt
  --scene.render (bool)
    Render scene video (default: True)

  --scene.color RGBA
    Color of scene during gaps / invalid frames (default: #808080FF)

  --scene.undistort (bool)
    Undistort scene video (default: False)

  --scene.privacy {no,blur,box,mosaic}
    Apply privacy shield, remove faces and license plates (default: no)

  --scene.audio (bool)
    Add audio to output (default: True)

  --scene.trim {no,start_and_end,all}
    Trim sections of video where scene video is not available (default: no)
```

#### Eye Video Options (config.eye)

```txt
  --eye.render (bool)
    Render eye video overlay (default: True)

  --eye.scale (float)
    Eye overlay video resize factor (default: 1.0)

  --eye.position {bottom,top,top_together,bottom_together}
    (default: top_together)
```

#### Gaze Options (config.gaze)

```txt
  --gaze.kind {cross,circle}
    Kind of target visualization (default: circle)

  --gaze.diameter Size
    Diameter of target, in pixels or percent of video frame (default: 5%)

  --gaze.fill Size
    Amount of target to fill from outer to center, for a target of diameter 100px default will make an outline of 4px (default: 4%)

  --gaze.color RGBA
    Color of target (default: #F4433699)

  --gaze.render (bool)
    Render gaze in video (default: True)

  --gaze.selection {average,all,first}
    Which gaze points to render per video frame (default: average)

  --gaze.not-worn-color RGBA
    Color of gaze target when device wasn't worn (default: #00000000)

  --gaze.blink-color RGBA
    Color of gaze target during a blink (default: #00000000)
```

#### Fixation Options (config.fixation)

```txt
FixationVis ['config.fixation']:
  --fixation.kind {cross,circle}
    Kind of target visualization (default: circle)

  --fixation.diameter Size
    Diameter of target, in pixels or percent of video frame (default: 5%)

  --fixation.fill Size
    Amount of target to fill from outer to center, for a target of diameter 100px default will make an outline of 4px (default: 4%)

  --fixation.color RGBA
    Color of target (default: #F4433699)

  --fixation.render (bool)
    Render fixations in video (default: True)

  --fixation.scaling {fixed,duration}
    Scale fixation size based on duration or not (default: duration)

  --fixation.labeled (bool)
    Add fixation id to fixations (default: True)
```

#### Scanpath Options (config.fixation.scanpath)

```txt
ScanPath ['config.fixation.scanpath']:
  --fixation.scanpath.seconds (float)
    History in seconds to show older points (scanpath) (default: 2)

  --fixation.scanpath.color RGBA
    Color of scanpath line (default: #99999999)

  --fixation.scanpath.thickness Size
    Thickness of scanpath line (default: 2px)
```

#### Heatmap Options (config.heatmap)

```txt
  --heatmap.render (bool)
    Render heatmap in video (default: False)

  --heatmap.colors RGBA[]
    Ordered array of colors for the heatmap's histogram cells, stronger color first (default: [#F4433699, #FF980099, #FFFF0099, #4CAF5099, #00CCFF80])

  --heatmap.opacity (float)
    Heatmap opacity (default: 0.5)

  --heatmap.bins (tuple: (x: int, y: int))
    Number of bins in heatmap (default: (x = width/50, y = fill the screen; same size as x))

  --heatmap.aoi (tuple[] (x: int, y: int, width: int, height: int)[])
    Array containing the areas of interest to render the heatmap.
    All areas outside the listed AOIs will not render.
    (default: [(x = 0, y = 0, w = width, h = height)])
```

#### IMU Options (config.imu)

```txt
  --imu.render (bool)
    Render imu overlay (default: True)
```

#### Watermark Options (config.watermark)

```txt
  --watermark.render (bool)
    Render watermark (default: True)

  --watermark.path [Path]
    Custom watermark path (default: None)

  --watermark.scale (float)
    Scale watermark size factor (default: 1.0)

  --watermark.opacity (float)
    Watermark opacity (default: 1.0)

  --watermark.position {top-left,top,top-right,bottom,bottom-left,bottom-right}
    (default: bottom-left)
```

## Drawbacks and Alternatives

The benefits of Rust's **performance** and **strong safety guarantees**, coupled with the ability to target multiple platforms with the same codebase with nearly zero footprint make it a compelling choice for this project.

However, one potential drawback of this approach is the introduction of a new language to our tech stack, which has a notoriously steep learning curve. This can introduce barriers to entry for new developers in our team.

Additionally, because we need to care about types and manage memory manually, the development cycle can be considerably slower than Python or Javascript.

### Alternatives

- **Embed a Python interpreter** in our Rust library using [RustPython](https://rustpython.github.io/), and simply use the same code we already have.

  - **Pros:**
    - This could be used as a way to accelerate development and have a working version of the library in less time.
    - We can gradually replace the Python code with Rust as needed.
    - It would allow us to use Python in the web browser.
  - **Cons:**
    - It would increase the package size.
    - It would greatly decrease performance.
    - It can introduce complex dependencies (ex. OpenCV) that could hurt portability.

- **Build it in Python** with the Python bindings for [wgpu-native](https://github.com/gfx-rs/wgpu-native). We can use the same GPU abstraction library as the original solution, and leverage the existing Python knowledge in our team.

  - **Pros:**
    - Probably the fastest development cycle from all options.
    - We would also have a working version of the library in less time.
  - **Cons:**
    - It would increase the package size.
    - It would greatly decrease performance.
    - Distribution to WebAssembly or native Android/iOS can be more complex.

- **Reduce scope: Make a tiny GPU rendering library** in Rust and expose simple primitives such as "circle", "cross", "line", "square", "text", "histogram", etc. and let each platform use it as needed.
  - **Pros:**
    - Simple to implement.
    - Extremely portable.
    - Can also have Python embedded like the options above.
    - Can be used as a building block for other projects.
  - **Cons:**
    - Does not solve our code duplication problem.
    - Wrong level of abstraction for our use case.
    - Many open-source alternatives exist.

## Roadmap

- [x] Build a simple POC to render gaze enrichment in the GPU
- [x] Figure out a way to target Python and Web Frontend (React) at the same time
- [x] Create the draft of the project and draft architecture
- [ ] **Collect feedback from the team** (this RFC)
- [ ] Adjust design as needed
- [ ] Basic project skeleton setup
- [ ] Create build pipeline for all targets
- [ ] Test hello-world with Python and WebAssembly
- [ ] Implement basic CLI setup (clap)
- [ ] Implement reading CSV from file or pipe
- [ ] Implement reading video from file or pipe
- [ ] Implement streaming video to file or stdout as-is
- [ ] Import and set up the wgpu library internally
- [ ] Hello triangle on top of the video
- [ ] Implement code for handling gaze data (200hz)
- [ ] Implement Overlay (gaze from CSV time series):
  - [ ] Implement gaze rendering
  - [ ] Implement fixation/scanpath rendering
  - [ ] Implement IMU overlay rendering
  - [ ] Implement watermark rendering
  - [ ] Implement heatmap rendering
- [ ] Implement reading from Clickhouse DB
- [ ] Implement Video processing (gaze from video):
  - [ ] Implement scene video undistortion
  - [ ] Implement eye video processing
- [ ] Refine CLI Implementation
- [ ] Refine documentation page and help texts
- [ ] Start using it in our projects
- [ ] Collect feedback, learn from it, and iterate

### Future development

- [ ] Pointcloud rendering
- [ ] Android / iOS integration
- [ ] Public Web Server Streaming API (?)
- [ ] Launch as open source to our clients (?)
- [ ] Publish as a public package in wasmer.io (?)

## Links

### Python / Pyo3 / Maturin

- [Pyo3 User Guide](https://pyo3.rs/v0.19.1)
- [Maturin User Guide](https://www.maturin.rs/)
- [Implementation Example](https://ohadravid.github.io/posts/2023-03-rusty-python)

### Rust Wasm Working Group

- [rustwasm-group](https://rustwasm.github.io/)
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/introduction.html)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/)
- [web-sys](https://rustwasm.github.io/wasm-bindgen/web-sys/index.html)

### GPU Abstraction Library

- Rust: [wgpu](https://github.com/gfx-rs/wgpu) (used by Firefox and Bevy game engine)
- C++: [dawn](https://dawn.googlesource.com/dawn) (used by Chrome and Webkit-based browsers)
- Bindings for many languages: [wgpu-native](https://github.com/gfx-rs/wgpu-native)
