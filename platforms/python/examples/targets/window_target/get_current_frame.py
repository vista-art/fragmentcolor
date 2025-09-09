from fragmentcolor import Renderer


renderer = Renderer()
target = renderer.create_texture_target([16, 16])
frame = target.get_current_frame(); // Acquire a frame
_format = frame.format()
