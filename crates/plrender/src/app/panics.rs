// @TODO catalog the rest. Glob search "TECH DEBT"
//       and move them all here.
//       This will catch the most important ones.
//       Then glob search ".expect", ".unwrap", and ".panic"
pub(crate) struct Panic;
impl Panic {
    pub fn trying_to_create_app_directly() -> ! {
        panic!(
            "App already initialized.

            Please use PLRender::app() to start the main App instance."
        )
    }
}

/// # (expect) To
///
/// Internal Syntax Sugar for Rust's `expect` calls.
///
/// To be used as: `some_result.expect(To::find_default_image())`
pub(crate) struct To;
impl To {
    pub fn find_default_image() -> &'static str {
        "Default image does not exist!"
    }
}
