from fragmentcolor import Shader

template = Shader("""
    struct Tint { color: vec4<f32> }
    @group(0) @binding(0) var<uniform> tint: Tint;
    @vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.));
        return vec4<f32>(p[i], 0.0, 1.0);
    }
    @fragment fn fs() -> @location(0) vec4<f32> { return tint.color; }

""")
template.set("tint.color", [1.0, 0.0, 0.0, 1.0])

# Independent copy — sets on `red` do not bleed into `template` or `blue`.
red = template.duplicate()
blue = template.duplicate()
blue.set("tint.color", [0.0, 0.4, 1.0, 1.0])
