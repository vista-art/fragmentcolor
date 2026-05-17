import org.fragmentcolor.*

val mesh = Mesh()
mesh.addVertex( Vertex.pbr(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.UV0, floatArrayOf(0.5f, 1.0f)), )
val model = Model(mesh, Material.pbr()?)

val scene = Scene()
scene.add(model)

// LOD switch: hide every model the user just loaded, based on a
// camera-distance heuristic the caller computes elsewhere.
for m in scene.models() {
    m.setVisible(false)
}