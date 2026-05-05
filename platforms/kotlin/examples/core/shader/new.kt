
import org.fragmentcolor.*

val shader = Shader("""
    @vertex
    fn vs_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
        var pos = array<vec2<f32>, 3>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>( 3.0, -1.0),
            vec2<f32>(-1.0,  3.0)
        )
        return vec4<f32>(pos[index], 0.0, 1.0)
    }

    @group(0) @binding(0)
    var<uniform> resolution: vec2<f32>

    @fragment
    fn fs_main() -> @location(0) vec4<f32> {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red
    }

""")


val main = """
    @vertex fn vs(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
        let p = array<vec2<f32>,3>(vec2f(-1.,-1.), vec2f(3.,-1.), vec2f(-1.,3.))
        return vec4<f32>(p[i], 0.0, 1.0)
    }

    @fragment fn fs(@builtin(position) pos: vec4<f32>) -> @location(0) vec4<f32> {
        let d = circle(pos.xy - vec2<f32>(400.0, 300.0), 100.0)
        let n = simplex2(pos.xy * 0.01)
        return vec4<f32>(vec3<f32>(step(0.0, d) + n * 0.1), 1.0)
    }

"""

val shader2 = Shader.compose(listOf("sdf2d/circle", "noise/simplex2", main,))