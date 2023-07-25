import av
import pyglet
from pyglet.gl import *
import numpy as np
from pathlib import Path
from dataclasses import dataclass
from pyglet.graphics.shader import Shader, ShaderProgram

container = av.open('videos/charge_teaser.mp4')
stream = next(s for s in container.streams if s.type == 'video')
window = pyglet.window.Window(width=stream.width, height=stream.height)
pixel_density = window.get_pixel_ratio()
batch = pyglet.graphics.Batch()

vertex_source = Path('shaders/gaze.vert').read_text()
fragment_source = Path('shaders/gaze.frag').read_text()


@dataclass
class Circle:
    radius: float = .05
    border: float = .003
    color: tuple = (1., 0., 0.)
    position: tuple = (stream.width // 2, stream.height // 2)


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
    glEnable(GL_BLEND)
    glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)
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
