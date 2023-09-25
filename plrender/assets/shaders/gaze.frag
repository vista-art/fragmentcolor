#version 450

layout(set = 0, binding = 0) uniform Screen {
    vec2 resolution;
    float antialiaser;
};

layout(set = 1, binding = 0) uniform Circle {
    vec2 position;
    float radius;
    float border;
    vec4 color;
};

out vec4 fragColor;
void main()
{
    vec3 rgb = color.rgb;
    vec2 normalized = gl_FragCoord.xy / resolution;
    vec2 uv = normalized * 2. - 1.;
    float aa = antialiaser;
    
    float ratio = resolution.x / resolution.y;
    uv.x *= ratio;
    
    float dist = distance(uv, position);

    float alpha = 1. - smoothstep(border - aa, border + aa, abs(dist-radius));
    alpha *= color.a;
    
    fragColor = vec4(rgb, alpha);
}
