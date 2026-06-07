from fragmentcolor import Camera

camera = Camera.perspective(1.047, 16.0 / 9.0, 0.1, 100.0).look_at([3.0, 2.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])

eye = camera.position()