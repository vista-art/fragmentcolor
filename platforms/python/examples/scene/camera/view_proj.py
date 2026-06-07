from fragmentcolor import Camera

camera = Camera.perspective(1.047, 16.0 / 9.0, 0.1, 100.0).look_at([0.0, 0.0, 5.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])

view_proj = camera.view_proj()