// Stub symbols for iOS-only uniffi exports that the generated
// `FragmentColor.swift` references unconditionally.
//
// The macOS slice of `build/ios-macos/FragmentColorFFI.xcframework` is built
// for local `swift build` on a developer's Mac. The Rust `cdylib` is compiled
// for `aarch64-apple-darwin` (not iOS), so it does not export the iOS-only
// `Renderer.create_target_ios` (Metal `CAMetalLayer` integration). This file
// supplies the missing symbols so the linker resolves them; calling them at
// runtime traps because there is no `CAMetalLayer` on macOS.
//
// The checksum below must match the value emitted by `uniffi-bindgen` for
// `Renderer.createTarget` — search `FragmentColor.swift` for
// `uniffi_fragmentcolor_checksum_method_renderer_createtarget`. If uniffi
// changes the FFI signature, regenerate the bindings and update the literal
// here. The bake-in is acceptable because this file only ships in
// developer-local artifacts (`build/ios-macos/macos-arm64`), never in the
// published iOS xcframework.

#include <stdint.h>

typedef struct RustCallStatus {
    int8_t code;
    // Simplified — actual struct has more fields, but only the symbol layout
    // matters for linking.
    uint64_t error_buf[3];
} RustCallStatus;

uint16_t uniffi_fragmentcolor_checksum_method_renderer_createtarget(void) {
    return 58343;
}

uint64_t uniffi_fragmentcolor_fn_method_renderer_createtarget(
    uint64_t ptr,
    uint64_t metal_layer_ptr,
    RustCallStatus *out_status
) {
    (void)ptr;
    (void)metal_layer_ptr;
    (void)out_status;
    // Unreachable on macOS — CAMetalLayer is not available.
    __builtin_trap();
    return 0;
}
