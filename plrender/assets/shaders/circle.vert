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
layout(location = 0) in vec3 _p2vs_location0;

void main() {
    VertexInput vertex = VertexInput(_p2vs_location0);
    VertexOutput out_ = VertexOutput(vec4(0.0));
    out_.position = vec4(vertex.position, 1.0);
    VertexOutput _e6 = out_;
    gl_Position = _e6.position;
    gl_Position.yz = vec2(-gl_Position.y, gl_Position.z * 2.0 - gl_Position.w);
    return;
}

