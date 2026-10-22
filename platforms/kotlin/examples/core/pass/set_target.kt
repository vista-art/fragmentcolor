import org.fragmentcolor.*

val r = Renderer()
val tex_target = r.createTextureTarget(listOf(512, 512))

val p = Pass("shadow")
p.setTarget(tex_target)