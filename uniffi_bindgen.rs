// Entry point for the `uniffi-bindgen` CLI helper. Mobile build scripts run
// `cargo run --bin uniffi-bindgen generate --library ...` against the
// compiled fragmentcolor library to emit Swift/Kotlin bindings.
//
// See: https://mozilla.github.io/uniffi-rs/latest/tutorial/foreign_language_bindings.html

fn main() {
    uniffi::uniffi_bindgen_main()
}
