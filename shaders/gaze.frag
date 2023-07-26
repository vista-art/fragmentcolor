#version 330

uniform float antialiaser;
uniform vec2 resolution;
uniform vec2 position;
uniform float radius;
uniform float border;
uniform vec4 color;

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
