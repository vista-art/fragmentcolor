import org.fragmentcolor.*

val template = Shader("""
    struct Tint { color: vec4<f32> }
    @group(0) @binding(0) var<uniform> tint: Tint
    @vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.))
        return vec4<f32>(p[i], 0.0, 1.0)
    }
    @fragment fn fs() -> @location(0) vec4<f32> { return tint.color; }

""")
template.set("tint.color", floatArrayOf(1.0f, 0.0f, 0.0f, 1.0f))

// Independent copy — sets on """red""" do not bleed into """template""" or """blue""".
val red = template.duplicate()
val blue = template.duplicate()
blue.set("tint.color", floatArrayOf(0.0f, 0.4f, 1.0f, 1.0f))