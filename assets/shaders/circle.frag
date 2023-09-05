#version 310 es

precision highp float;
precision highp int;

struct VertexInput {
    vec3 position;
};
struct VertexOutput {
    vec4 position;
};
struct FragmentOutput {
    vec4 color;
};
struct Screen {
    float antialiaser;
    vec2 resolution;
};
struct Circle {
    vec2 position;
    float radius;
    float border;
    vec4 color;
};
uniform Screen_block_0Fragment { Screen _group_0_binding_0_fs; };

uniform Circle_block_1Fragment { Circle _group_1_binding_0_fs; };

layout(location = 0) out vec4 _fs2p_location0;

void main() {
    VertexOutput in_ = VertexOutput(gl_FragCoord);
    FragmentOutput out_1 = FragmentOutput(vec4(0.0));
    vec2 uv = vec2(0.0);
    float aa = _group_0_binding_0_fs.antialiaser;
    float border = _group_1_binding_0_fs.border;
    float radius = _group_1_binding_0_fs.radius;
    vec4 color = _group_1_binding_0_fs.color;
    vec4 _e16 = _group_1_binding_0_fs.color;
    vec3 rgb = _e16.xyz;
    vec2 _e22 = _group_0_binding_0_fs.resolution;
    vec2 normalized = (in_.position.xy / _e22);
    uv = ((normalized * 2.0) - vec2(1.0));
    float _e34 = _group_0_binding_0_fs.resolution.x;
    float _e38 = _group_0_binding_0_fs.resolution.y;
    float _e40 = uv.x;
    uv.x = (_e40 * (_e34 / _e38));
    vec2 _e42 = uv;
    float dist = distance(_e42, in_.position.xy);
    float alpha = ((1.0 - smoothstep((border - aa), (border + aa), abs((dist - radius)))) * color.w);
    out_1.color = vec4(rgb.x, rgb.y, 1.0, 1.0);
    FragmentOutput _e61 = out_1;
    _fs2p_location0 = _e61.color;
    return;
}

