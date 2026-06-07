import org.fragmentcolor.*

val renderer = Renderer()
val mesh = Mesh()
mesh.addVertex( Vertex.new(listOf(0.0f, 0.5f, 0.0f)).set(Vertex.NORMAL, floatArrayOf(0.0f, 0.0f, 1.0f)).set(Vertex.UV0, listOf(0.5f, 1.0f)), )
mesh.addVertex( Vertex.new(listOf(-0.5f, -0.5f, 0.0f)).set(Vertex.NORMAL, floatArrayOf(0.0f, 0.0f, 1.0f)).set(Vertex.UV0, listOf(0.0f, 0.0f)), )
mesh.addVertex( Vertex.new(listOf(0.5f, -0.5f, 0.0f)).set(Vertex.NORMAL, floatArrayOf(0.0f, 0.0f, 1.0f)).set(Vertex.UV0, listOf(1.0f, 0.0f)), )

val template = Material.pbr(renderer).baseColor(listOf(0.85f, 0.4f, 0.2f, 1.0f))
val pass = Pass("scene")

val m1 = Model(mesh.clone(), template.clone())
m1.translate(listOf(-1.0f, 0.0f, 0.0f))
pass.addModel(m1)

val m2 = Model(mesh, template)
m2.translate(listOf(1.0f, 0.0f, 0.0f))
pass.addModel(m2)