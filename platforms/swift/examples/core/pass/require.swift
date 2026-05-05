import FragmentColor
let renderer = Renderer()
let target = try await renderer.createTextureTarget([100,100])
let color = Pass("color")
let blurx = Pass("blur_x")
try blurx.require(color); // color before blur_x
let blury = Pass("blur_y")
try blury.require(blurx); // blur_x before blur_y
let compose = Pass("compose")
try compose.require(color)
try compose.require(blury); // fan-in; color and blur_y before compose
try renderer.render(compose, target); // compose renders last