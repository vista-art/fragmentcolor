# Kotlin / Android Healthcheck — Library Builds, Test Codegen Has Transpiler Bugs

**Last updated**: 2026-05-03

## Toolchain (now installed)

| Tool | Status |
|---|---|
| `sdkmanager` | `/opt/homebrew/share/android-commandlinetools` (cask: `android-commandlinetools`) |
| Android SDK | `platforms;android-34` + `build-tools;34.0.0` + `platform-tools` |
| Android NDK | `ndk;26.3.11579264` |
| `cargo-ndk` | `v4.1.2` (`cargo install cargo-ndk`) |
| `gradle` (system) | 9.5.0 — incompatible with AGP 8.2; **do not use directly** |
| `gradlew` (wrapper) | 8.7 (use this) |
| Java | 22 (system) — too new for Gradle 8.7. Use `openjdk@21` instead |

## Required env per build

```sh
export ANDROID_HOME=/opt/homebrew/share/android-commandlinetools
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/26.3.11579264
export JAVA_HOME=$(brew --prefix openjdk@21)/libexec/openjdk.jdk/Contents/Home
```

`platforms/kotlin/local.properties` (gitignored, per-machine):

```
sdk.dir=/opt/homebrew/share/android-commandlinetools
```

## Build status

| Step | Status |
|---|---|
| `./build_android` (cargo-ndk → .so + uniffi Kotlin bindings) | OK |
| `./gradlew :fragmentcolor:compileDebugKotlin` | OK |
| `./gradlew :fragmentcolor:assembleDebug` (full library AAR) | OK |
| `./gradlew :fragmentcolor:assembleAndroidTest` (test APK) | FAIL 248 compile errors in auto-generated `GeneratedExamples.kt` (transpiler bugs in `scripts/convert.rs`) |
| `./gradlew :fragmentcolor:connectedAndroidTest` | BLOCKED on `assembleAndroidTest` + needs connected device/emulator |

## Open work

### 1. `scripts/convert.rs` transpiler bugs

`GeneratedExamples.kt` emits invalid Kotlin in several patterns:

- **Tuple expressions** — JS `[[0, 0], [32, 32]]` becomes `arrayOf((0, 0), (32, 32))`. Kotlin has no tuple literals. Should emit a `ScreenRegion(x = 0, y = 0, width = 32, height = 32)` or proper list-of-lists.
- **Comma-expression args** — JS `createTexture((bytes, [w, h]))` becomes the literal Kotlin string instead of being unwrapped.
- **JS DOM API leakage** — `document.createElement("canvas")` appears in 5 generated functions. The transpiler should substitute the Android `Surface`-based `renderer.createTarget(surface)` path.
- **Array/List confusion** — `arrayOf(...)` where `listOf(...)` is required.
- **`render(arrayOf(pass, pass2), target)`** — generated examples pass `Array<Pass>`, but `RendererExtensions.kt` defines `render(passes: List<Pass>, target)`.

### 2. Genuine binding bugs surfaced (also flagged by other healthchecks)

- `Instance::new()` starts `next_location = 0` → collides with `@location(0)` vertex position (Web + Swift agents flagged the same)
- `readTexture` — now a proper `#[uniffi::method]` async binding (landed; `readTextureAsync` deleted)
- `TextureRegionMobile.from()` static method missing from uniffi binding

## To run instrumented tests end-to-end

1. Connect an Android device or start an AVD (`avdmanager` is now installed)
2. Fix the `scripts/convert.rs` transpiler so `assembleAndroidTest` succeeds
3. `./gradlew :fragmentcolor:connectedAndroidTest`
