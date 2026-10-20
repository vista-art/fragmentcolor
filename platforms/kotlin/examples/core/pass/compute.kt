import org.fragmentcolor.*

val cs = Shader("@compute @workgroup_size(8,8,1) fn cs_main() {}")
val pass = Pass.fromShader("compute", cs)