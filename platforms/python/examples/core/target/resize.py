
from fragmentcolor import Renderer, Target

renderer = Renderer()
target = renderer.create_texture_target([64, 32])

target.resize([128, 64])
