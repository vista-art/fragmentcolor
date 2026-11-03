import org.fragmentcolor.*

val renderer = Renderer()
val target = renderer.createTextureTarget(listOf(64u, 64u))

val wall = Mesh()
wall.addVertices(listOf(Vertex.pbr(listOf(-2.0f, -2.0f, 0.0f)), Vertex.pbr(listOf(2.0f, -2.0f, 0.0f)), Vertex.pbr(listOf(2.0f, 2.0f, 0.0f)), Vertex.pbr(listOf(-2.0f, 2.0f, 0.0f))))
wall.setIndices(listOf(0u, 1u, 2u, 0u, 2u, 3u))

val camera = Camera.perspective(1.047f, 1.0f, 0.1f, 100.0f).lookAt(listOf(0.0f, 0.0f, 4.0f), listOf(0.0f, 0.0f, 0.0f), listOf(0.0f, 1.0f, 0.0f))
val bulb = Light.point(listOf(0.0f, 0.5f, 1.0f), listOf(1.0f, 0.95f, 0.8f)).setIntensity(15.0f)
bulb.setRange(3.0f)

val pass = Pass("lamp")
pass.add(Model(wall, Material.pbr()))
pass.add(camera)
pass.add(bulb)
renderer.render(pass, target)