use crate::{HasDisplaySize, Size};

impl HasDisplaySize for winit::window::Window {
    fn size(&self) -> Size {
        let size = self.inner_size();
        Size {
            width: size.width,
            height: size.height,
            depth: None,
        }
    }
}

impl HasDisplaySize for &winit::window::Window {
    fn size(&self) -> Size {
        let size = self.inner_size();
        Size {
            width: size.width,
            height: size.height,
            depth: None,
        }
    }
}

impl HasDisplaySize for std::sync::Arc<winit::window::Window> {
    fn size(&self) -> Size {
        let size = self.inner_size();
        Size {
            width: size.width,
            height: size.height,
            depth: None,
        }
    }
}
