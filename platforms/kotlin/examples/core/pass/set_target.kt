import org.fragmentcolor.*

val r = Renderer()
val tex_target = r.createTextureTarget(512u, 512u)

val p = Pass("shadow")
p.setTarget(tex_target)