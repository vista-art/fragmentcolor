import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)))
val blob = Model(mesh, Material.pbr())

// At a wide zoom level, skip the detail blobs.
blob.setVisible(false)
// Zoomed back in, turn them on again.
blob.setVisible(true)