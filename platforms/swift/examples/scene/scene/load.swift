import FragmentColor
import Foundation
let glb_bytes = try Data(contentsOf: URL(fileURLWithPath: "path/to/model.glb"))

// A path to a """.gltf""" file with its buffers and images, or to a """.glb""" file.
let scene = try Scene.load("path/to/model.gltf")

// The bytes of a """.glb""" file, read from disk or the network.
let scene2 = try Scene.load(glb_bytes)