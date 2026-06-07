import FragmentColor

let camera = try Camera.perspective(1.047, 16.0 / 9.0, 0.1, 100.0).lookAt([3.0, 2.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])

let eye = camera.position()