# Kotlin / Android Healthcheck — BLOCKED

**Branch**: `claude/kotlin-healthcheck-v2`  
**Date**: 2026-05-03  

---

## Toolchain Status

| Tool | Status |
|------|--------|
| Java | OpenJDK 22.0.2 — present |
| `gradle` | **NOT FOUND** (not in PATH) |
| `ANDROID_HOME` | **EMPTY** — no Android SDK |
| `~/Library/Android/sdk` | **NOT FOUND** |
| `/opt/homebrew/opt/android-sdk` | **NOT FOUND** |
| `cargo-ndk` | NOT CHECKED (requires NDK first) |

**`./build_android` requires**: `cargo-ndk` + `ANDROID_NDK_HOME` pointing at a valid NDK.  
**`./gradlew` / `gradle`**: No gradlew wrapper exists in `platforms/kotlin/`. Build uses system `gradle` (missing) with AGP 8.2.2 and Kotlin 1.9.22.

---

## What to Install (in order)

1. **Android SDK** (via Android Studio or `sdkmanager`):
   - Minimum: `platforms;android-34` + `build-tools;34.0.0`
   - Set `ANDROID_HOME` (e.g. `~/Library/Android/sdk`)
   - Create `platforms/kotlin/local.properties` with `sdk.dir=/path/to/sdk`

2. **Android NDK** (r25c or later):
   - Via Android Studio → SDK Manager → NDK (Side by side), or `sdkmanager "ndk;25.2.9519653"`
   - Set `ANDROID_NDK_HOME`

3. **Gradle wrapper** (recommended — add `gradlew` to `platforms/kotlin/` via `gradle wrapper --gradle-version 8.2`):
   - Or install Gradle 8.2+ globally and ensure it is on PATH

4. **cargo-ndk**:
   ```
   cargo install cargo-ndk
   rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
   ```

5. **JNI artifacts** (required before kotlinc can compile):
   ```
   cd <repo-root>
   ./build_android
   ```
   This produces:
   - `platforms/kotlin/fragmentcolor/src/main/jniLibs/<abi>/libfragmentcolor.so` (4 ABIs)
   - `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/generated/*.kt` (uniffi output)

6. **Compile-only build** (no device):
   ```
   cd platforms/kotlin
   gradle :fragmentcolor:compileDebugKotlin
   ```

7. **Instrumented tests** (requires running emulator with Vulkan API 28+):
   ```
   cd platforms/kotlin
   gradle :fragmentcolor:connectedAndroidTest
   ```

---

## Source Code State (wave1-staging)

### Android project structure — OK
- `platforms/kotlin/build.gradle` — root Gradle file with AGP 8.2.2 + Kotlin 1.9.22
- `platforms/kotlin/fragmentcolor/build.gradle` — library module config, minSdk 28
- `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/RendererExtensions.kt` — hand-written
- `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/ShaderExtensions.kt` — hand-written
- `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/PassExtensions.kt` — hand-written
- `platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/Healthcheck.kt` — instrumented smoke test
- `platforms/kotlin/fragmentcolor/src/androidTest/java/org/fragmentcolor/GeneratedExamples.kt` — auto-generated compile-time checker
- `platforms/kotlin/fragmentcolor/src/main/java/org/fragmentcolor/generated/` — **empty** (populated by `./build_android`)
- `platforms/kotlin/fragmentcolor/src/main/jniLibs/*/` — **empty** (populated by `./build_android`)
- `platforms/kotlin/examples/` — 139 `.kt` code snippets for website

### Kotlin API drift findings (static read, no compile)

The `generated/` directory is empty (no uniffi output to type-check against), so a full compile-time audit is impossible without the JNI build. However, a manual read of `GeneratedExamples.kt` reveals several issues that will cause compile failures once the generated bindings are present:

#### 1. Web-only APIs copied into Kotlin examples
Several generated examples use `document.createElement("canvas")` — a JavaScript DOM API with no Kotlin equivalent. Affects:
- `_example_core_pass_Pass`
- `_example_core_renderer_Renderer`
- `_example_core_renderer_create_target`
- `_example_targets_target_Target`
- `_example_targets_window_target_WindowTarget`

These are translation artifacts from `scripts/convert.rs` — the converter did not substitute the Android `Surface`-based target creation path.

#### 2. `texture.id()` returns `TextureId` record (not `ULong`)
`ShaderExtensions.kt` line 92 calls `texture.id()` expecting to pass the result directly to `TextureMeta(id = texture.id(), ...)`. The uniffi binding for `id()` returns a `TextureId` record `{ id: ULong }`. The `TextureMeta` constructor may expect a `TextureId` object — this needs type-checking once uniffi output is available.

`GeneratedExamples.kt` lines 320, 328, 344 call `renderer.readTexture(texture.id())` and `renderer.readTextureAsync(texture.id())`. The mobile renderer's `unregisterTexture` takes a raw `u64` (bypasses uniffi TextureId), but `readTexture`/`readTextureAsync` methods are **not present** in `src/renderer/platform/mobile/mod.rs` (only `waitIdle`, `createTextureTarget`, `createTexture`, `createStorageTexture`, `render`, `createDepthTexture`, `unregisterTexture`, `createExternalTexture`).

#### 3. `renderer.readTexture()` / `renderer.readTextureAsync()` — missing mobile binding
`GeneratedExamples.kt` references `renderer.readTexture(...)` and `renderer.readTextureAsync(...)`. These methods do not exist in `src/renderer/platform/mobile/mod.rs`. The mobile renderer only exposes `Texture.getImage()` for readback (async, on the `Texture` object). This is an **API gap**.

#### 4. Array/list notation inconsistencies in generated examples
Many generated examples use Kotlin-adjacent but non-idiomatic constructs:
- `arrayOf(...)` used where `listOf(...)` or `uintArrayOf(...)` may be required
- `[...]` array literals (Rust/JS syntax) appear in some examples, e.g. `Shader.new([...])`, `m.addVertices([...])`
- Tuple syntax `(a, b)` used for pair arguments, e.g. `createStorageTexture((arrayOf(64, 64), TextureFormat.Rgba))`

These will fail to compile in Kotlin as written. They are converter artifacts from `scripts/convert.rs`.

#### 5. `renderer.render(arrayOf(pass, pass2), target)` — should be `listOf`
`_example_core_pass_Pass` calls `renderer.render(arrayOf(pass, pass2), target)`. The `RendererExtensions.kt` extension expects `List<Pass>` not `Array<Pass>`.

#### 6. `Instance::new()` location collision (same bug as Web agent found)
`_example_geometry_mesh_add_instance` uses `Instance.new()`. The Web agent found that `Instance::new()` has a naming collision with the uniffi `new` constructor. This should be audited once the generated bindings are available.

#### 7. `Shader.Companion.setRegistry` self-call infinite loop
`ShaderExtensions.kt` line 14: `fun Shader.Companion.setRegistry(baseUrl: String) { Shader.setRegistry(baseUrl) }` — this extension calls itself recursively. The internal call resolves to the same extension function rather than the uniffi-generated method. This is a **genuine binding bug** (infinite recursion at runtime).

---

## Compile Outcome
**SKIPPED** — Android SDK and NDK not present; `./build_android` cannot run; no JNI `.so` and no generated `fragmentcolor.kt` to compile against.

## Test Outcome
**SKIPPED** — no device/emulator available and build is blocked.

---

## Summary: What Blocks the Healthcheck

1. No Android SDK (`ANDROID_HOME` empty, no SDK at standard paths)
2. No Android NDK (`ANDROID_NDK_HOME` unset)
3. No `gradlew` wrapper (system `gradle` also missing)
4. Therefore `./build_android` cannot run → no JNI `.so` artifacts → no uniffi-generated `fragmentcolor.kt` → `compileDebugKotlin` cannot proceed

Once the above are resolved, the healthcheck should run `./build_android && cd platforms/kotlin && gradle :fragmentcolor:compileDebugKotlin` and fix the compile errors documented above.
