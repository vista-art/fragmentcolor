# IosTarget

[iOS wrapper around WindowTarget](https://fragmentcolor.org/api/targets/windowtarget). Implements the [Target](https://fragmentcolor.org/api/core/target) interface via an internal [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) created from a CAMetalLayer.

- Canonical object: [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget)
- [Target](https://fragmentcolor.org/api/core/target) trait docs: [Target](https://fragmentcolor.org/api/core/target)

## Example

```swift
// Swift/UniFFI (illustrative)
import FragmentColor

func makeTarget(layerPtr: UInt64) async throws {
    let renderer = Renderer.newIos()
    let target = try await renderer.createTargetIos(layerPtr)
    // Render as usual
}
```

