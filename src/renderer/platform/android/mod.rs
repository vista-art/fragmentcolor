pub mod target;
pub use target::*;

#[cfg_attr(mobile, derive(uniffi::Object))]
pub struct FragmentColor {
    surface: Option<wgpu::Surface<'static>>,
    wrapped: Arc<crate::FragmentColor>,
}

/// NOTE: FragmentColor needs 2 raw pointers to connect with:
///  - the JNI environment
///  - the Android Surface object
///
/// Unfortunately uniffi currently does not support interfacing with raw
/// pointers. In addition, in order to get the JNIEnv pointer, we need to
/// use a proper JNI function, which is not possible to do with uniffi.
///
/// So in the end we do not expose this function and wrap it into a ugly raw ffi
impl super::FragmentColor {
    pub async fn init(env: *mut JNIEnv<'_>, surface: jobject) -> (Renderer, Target) {
        let window = AndroidNativeWindow::new(env, surface);
        let window_width = window.width();
        let window_height = window.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: BACKENDS,
            ..Default::default()
        });

        let handle: Box<dyn wgpu::WindowHandle> = Box::new(window);
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Window(handle))
            .expect("Failed to create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let (device, queue) = crate::platform::all::request_device(&adapter).await;

        let capabilitiess = surface.get_capabilities(&adapter);
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilitiess.formats[0].remove_srgb_suffix(),
            width: u32::max(window_width, 1),
            height: u32::max(window_height, 1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: capabilitiess.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![],
        };

        surface.configure(stage.device(), &surface_configuration);

        let renderer = crate::Renderer::new(device, queue);

        (renderer, surface)
    }
}

#[cfg_attr(mobile, uniffi::export)]
impl FragmentColor {
    #[cfg_attr(mobile, uniffi::constructor)]
    pub async fn headless() -> Self {
        let context = headless().await;

        Self {
            surface: None,
            wrapped: crate::FragmentColor::new(context).into(),
        }
    }

    pub fn draw(&self, composition: &ffi::Composition) {
        let Some(surface) = self.surface.as_ref() else {
            panic!("Cannot draw on a headless stage, use `render_bitmap` instead");
        };

        let composition = composition.wrapped.read().unwrap();

        let surface_texture = surface
            .get_current_texture()
            .expect("Failed to get texture");

        self.wrapped
            .render(&composition, Destination::Texture(&surface_texture.texture))
            .expect("Failed rendering");

        surface_texture.present();
    }
}

#[no_mangle]
#[jni_fn("org.fragmentcolor.FragmentColor")]
pub fn fragmentcolor_init(env: *mut JNIEnv, _: JClass, surface: jobject) -> *const FragmentColor {
    let stage = pollster::block_on(FragmentColor::init(env, surface));
    Arc::into_raw(Arc::new(stage))
}
