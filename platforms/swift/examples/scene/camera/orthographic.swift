import FragmentColor

// A 16:9 viewport, 10 world units tall, depth range 0.1..100.
let camera = try Camera.orthographic(-8.0, 8.0, -4.5, 4.5, 0.1, 100.0)