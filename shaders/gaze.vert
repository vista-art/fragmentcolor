#version 330
in vec2 position;
uniform vec2 resolution;

void main()
{
    float ratio = resolution.x / resolution.y;
    gl_Position = vec4(position.x / ratio, position.y, 0., 1.);
}
