# Renderer::render_texture_py

Hidden WASM binding: non-consuming render variant for [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget).

This method exists only for the web (wasm) build to avoid consuming the [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget) JS object when calling render from JavaScript repeatedly. Prefer using renderer.render(shaderOrPassOrFrame, target) from JS; overload resolution will route to this variant when the second argument is a [TextureTarget](https://fragmentcolor.org/api/targets/texturetarget).
