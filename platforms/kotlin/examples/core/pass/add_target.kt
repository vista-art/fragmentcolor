import org.fragmentcolor.*

val r = Renderer()
val tex_target = r.createTextureTarget(512u, 512u)

val p = Pass("shadow")
p.addTarget(tex_target)