import av
import pyglet
from pyglet.gl import *
import numpy as np
from dataclasses import dataclass
from pyglet.graphics.shader import Shader, ShaderProgram

container = av.open('videos/charge_teaser.mp4')
stream = next(s for s in container.streams if s.type == 'video')
window = pyglet.window.Window(width=stream.width, height=stream.height)
pixel_density = window.get_pixel_ratio()
batch = pyglet.graphics.Batch()


@dataclass
class Circle:
    radius: float = .05
    border: float = .003
    color: tuple = (1., 0., 0.)
    position: tuple = (stream.width // 2, stream.height // 2)


vertex_source = """
    #version 330
    in vec2 position;
    uniform vec2 resolution;
    
    void main()
    {
        float ratio = resolution.x / resolution.y;
        gl_Position = vec4(position.x / ratio, position.y, 0., 1.);
    }
"""

fragment_source = """
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
        
        if (alpha > 0.) {
            fragColor = vec4(color, 1.);
        } else {
            discard;
        }
    }
"""


@window.event
def on_draw():
    frame = next(container.decode(video=0))
    img = frame.to_image()
    image = pyglet.image.ImageData(
        img.width, img.height, 'rgb', img.tobytes(), pitch=-img.width * 3)

    image.blit(0, 0)
    batch.draw()


def init():
    circle = Circle()

    glClearColor(1, 1, 1, 0)
    vertex_shader = Shader(vertex_source, 'vertex')
    fragment_shader = Shader(fragment_source, 'fragment')
    program = ShaderProgram(vertex_shader, fragment_shader)

    resolution = np.array([window.width, window.height], dtype=np.float32)
    resolution *= pixel_density

    program.uniforms['resolution'].set(resolution)
    program.uniforms['radius'].set(circle.radius)
    program.uniforms['border'].set(circle.border)
    program.uniforms['color'].set(circle.color)

    size = circle.radius + circle.border
    vertices = np.array([
        -size, -size,
        -size, size,
        size, size,
        size, -size,
    ], dtype=np.float32)

    indices = np.array([
        0, 1, 2,
        0, 2, 3,
    ], dtype=np.uint8)

    program.vertex_list_indexed(
        4, GL_TRIANGLES, indices=indices, batch=batch, position=('f', vertices))


if __name__ == '__main__':
    init()
    pyglet.app.run()
