pub const DEFAULT_VERTEX_SHADER: &str = r#"
    #version 450

    layout(location = 0) in vec2 position;

    void main() {
        gl_Position = vec4(position, 0.0, 1.0);
    }
"#;

pub const DEFAULT_FRAGMENT_SHADER: &str = r#"
    #version 450

    layout(location = 0) out vec4 fragColor;

    void main() {
        fragColor = vec4(1.0, 0.0, 0.0, 1.0);
    }
"#;

pub const SHADERTOY_WRAPPER: &str = r#"
    uniform vec3      iResolution;           // viewport resolution (in pixels)
    uniform float     iTime;                 // shader playback time (in seconds)
    uniform float     iTimeDelta;            // render time (in seconds)
    uniform float     iFrameRate;            // shader frame rate
    uniform int       iFrame;                // shader playback frame
    uniform float     iChannelTime[4];       // channel playback time (in seconds)
    uniform vec3      iChannelResolution[4]; // channel resolution (in pixels)
    uniform vec4      iMouse;                // mouse pixel coords. xy: current (if MLB down)

    void main() {
        vec4 fragColor;
        mainImage(fragColor, gl_FragCoord.xy);
        gl_FragColor = fragColor;
    }

    {{shader}}
"#;

pub const DEFAULT_SHADER: &str = r#"
    struct VertexOutput {
        @builtin(position) coords: vec4<f32>,
    }

    @group(0) @binding(0)
    var<uniform> resolution: vec2<f32>;

    @vertex
    fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
        let x = f32(i32(in_vertex_index) - 1);
        let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
        return VertexOutput(vec4<f32>(x, y, 0.0, 1.0));
    }

    @fragment
    fn fs_main(pixel: VertexOutput) -> @location(0) vec4<f32> {
        // Touch resolution to ensure it's preserved in the module
        let _dummy = resolution.x + resolution.y * 0.0;
        return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    }
"#;
