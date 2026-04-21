import FragmentColor
let renderer = Renderer()
let target = try await renderer.createTextureTarget([100,100])
let color = Pass("color")
let blurx = Pass("blur_x")
blurx.require(color); // color before blur_x
let blury = Pass("blur_y")
blury.require(blurx); // blur_x before blur_y
let compose = Pass("compose")
compose.require(color)
compose.require(blury); // fan-in; color and blur_y before compose
renderer.render(compose, target); // compose renders last