use fragmentcolor:{Pass, Renderer}
color = Pass("color")
blurx = Pass("blur_x")
blurx.require(color); # color before blur_x
blury = Pass("blur_y")
blury.require(blurx); # blur_x before blur_y
compose = Pass("compose")
compose.require([color, blury]); # fan-in; color and blur_y before compose
renderer.render(compose, target); # compose renders last