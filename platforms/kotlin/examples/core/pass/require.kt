import org.fragmentcolor.*
val renderer = Renderer()
val target = renderer.createTextureTarget([100,100])
val color = Pass("color")
val blurx = Pass("blur_x")
blurx.require(color); // color before blur_x
val blury = Pass("blur_y")
blury.require(blurx); // blur_x before blur_y
val compose = Pass("compose")
compose.require(color)
compose.require(blury); // fan-in; color and blur_y before compose
renderer.render(compose, target); // compose renders last