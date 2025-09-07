use crate::Size;

#[derive(Clone, Copy, Debug, Default)]
pub struct TestWindow {
    pub(crate) size: Size,
}

pub fn test_window(size: impl Into<Size>) -> TestWindow {
    TestWindow { size: size.into() }
}

