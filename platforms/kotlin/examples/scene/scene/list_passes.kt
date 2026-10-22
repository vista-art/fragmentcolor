import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), )
val scene = Scene()
scene.add(Model(mesh, Material.pbr()))

// Compose, don't clear: keep whatever the previous pass drew.
for (p in scene.listPasses()) {
    p.loadPrevious()
}