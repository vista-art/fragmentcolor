#version 330
in vec2 vertex;

//uniform vec2 position;
//uniform vec2 resolution;

void main()
{
    //vec2 translation = vertex + position;
    //float ratio = resolution.x / resolution.y;
    //gl_Position = vec4(translation.x / ratio, translation.y, 0., 1.);
    gl_Position = vec4(vertex, 0., 1.);
}
