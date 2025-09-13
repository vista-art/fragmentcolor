Hidden WASM binding: non-consuming render variant for CanvasTarget.

This method exists only for the web (wasm) build to avoid consuming the CanvasTarget JS object when calling render from JavaScript repeatedly. Prefer using renderer.render(shaderOrPassOrFrame, target) from JS; overload resolution will route to this variant when the second argument is a CanvasTarget.
