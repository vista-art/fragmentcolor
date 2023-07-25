#version 330

uniform vec2 resolution;
uniform float radius;
uniform float border;
uniform vec3 color;

out vec4 fragColor;
void main()
{
    vec2 normalized = gl_FragCoord.xy / resolution;
    vec2 uv = normalized * 2. - 1.;
    
    float ratio = resolution.x / resolution.y;
    uv.x *= ratio;
    
    vec2 center = vec2(0.);
    
    float dist = distance(uv, center);
    
    float alpha = 1. - smoothstep(0., border, abs(dist-radius));
    
    fragColor = vec4(color, alpha);
}
