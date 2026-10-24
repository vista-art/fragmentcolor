import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex(Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)))
val detail = Model(mesh, Material.pbr())

// Far from the camera, hide the detail Model.
detail.setVisible(false)
// Closer again, show it.
detail.setVisible(true)