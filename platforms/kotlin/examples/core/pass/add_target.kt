import org.fragmentcolor.*

val r = Renderer()
val tex_target = r.createTextureTarget([512, 512])

val p = Pass("shadow")
p.addTarget(tex_target)