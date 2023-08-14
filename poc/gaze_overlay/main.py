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
resolution = np.array([window.width, window.height], dtype=np.float32)
resolution *= pixel_density
aa_thresold = 2. / resolution.min()


glClearColor(1, 1, 1, 0)
glEnable(GL_BLEND)
glBlendFunc(GL_SRC_ALPHA, GL_ONE_MINUS_SRC_ALPHA)
batch = pyglet.graphics.Batch()
vertex_source = Path('shaders/gaze.vert').read_text()
fragment_source = Path('shaders/gaze.frag').read_text()
vertex_shader = Shader(vertex_source, 'vertex')
fragment_shader = Shader(fragment_source, 'fragment')
program = ShaderProgram(vertex_shader, fragment_shader)


@dataclass
class Circle:
    radius: float = .05
    border: float = .004
    color: tuple = (1., 0., 0., .5)
    position: tuple = (0., 0.)


circle = Circle()


@window.event
def on_draw():
    frame = next(container.decode(video=0))
    img = frame.to_image()
    image = pyglet.image.ImageData(
        img.width, img.height, 'rgb', img.tobytes(), pitch=-img.width * 3)

    program.uniforms['position'].set(circle.position)

    image.blit(0, 0)
    batch.draw()


@window.event
def on_mouse_motion(x, y, dx, dy):
    ratio = resolution[0] / resolution[1]
    normalized = (x / window.width, y / window.height)
    circle.position = (ratio * (normalized[0] * 2 - 1), normalized[1] * 2 - 1)


def init():
    program.uniforms['position'].set(circle.position)
    program.uniforms['resolution'].set(resolution)
    program.uniforms['antialiaser'].set(aa_thresold)
    program.uniforms['radius'].set(circle.radius)
    program.uniforms['border'].set(circle.border)
    program.uniforms['color'].set(circle.color)

    size = 1.  # circle.radius + circle.border + aa_thresold
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
        4, GL_TRIANGLES, indices=indices, batch=batch, vertex=('f', vertices))


if __name__ == '__main__':
    init()
    pyglet.app.run()
