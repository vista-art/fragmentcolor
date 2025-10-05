# AndroidTarget

[AndroidTarget](https://fragmentcolor.org/api/hidden/platforms/android/androidtarget) is an Android-specific wrapper around [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget).

It forwards all rendering to an internal [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) created from an Android Surface. See [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget) for the full [Target](https://fragmentcolor.org/api/core/target) behavior and semantics.

- Canonical object: [WindowTarget](https://fragmentcolor.org/api/targets/windowtarget)
- [Target](https://fragmentcolor.org/api/core/target) trait docs: [Target](https://fragmentcolor.org/api/core/target)

## Example

```kotlin
// Kotlin/UniFFI (illustrative)
val renderer = Renderer.newAndroid()
val target = renderer.createTargetAndroid(jniEnvPtr, surfaceObj)

// Render normal objects
val shader = Shader.default()
renderer.render(shader, target)
```
