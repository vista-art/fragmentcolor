import FragmentColor

let camera = Camera.perspective(60.0.toRadians(), 16.0 / 9.0, 0.1, 100.0).lookAt([3.0, 2.0, 8.0], [0.0, 0.0, 0.0], [0.0, 1.0, 0.0])

let eye = camera.position()