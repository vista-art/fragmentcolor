//! The renderer's tests, one file for each thing they cover,
//! and the fixtures they share: `SpyTarget`, `setup` and `Uncaptured`.

use crate::core::renderer::*;
use parking_lot::Mutex;
use std::sync::atomic::AtomicUsize;

mod checks;
mod error_scopes;
mod render;

/// Collects every GPU error the device reports outside an error scope.
fn record_uncaptured(context: &RenderContext) -> Uncaptured {
    let uncaptured = Arc::new(Mutex::new(Vec::new()));
    let sink = uncaptured.clone();
    context
        .device
        .on_uncaptured_error(Arc::new(move |error: wgpu::Error| {
            sink.lock().push(error.to_string())
        }));
    uncaptured
}

/// Whether `result` is a validation error filed under `expected`.
fn is_validation(result: &Result<(), RendererError>, expected: &str) -> bool {
    matches!(result, Err(RendererError::DeviceError { kind: ErrorKind::Validation, label, .. }) if label == expected)
}

/// How often a `SpyTarget` handed out a frame, and how often one was presented.
#[derive(Default)]
struct Counts {
    acquired: AtomicUsize,
    presented: AtomicUsize,
}

impl Counts {
    fn acquired(&self) -> usize {
        self.acquired.load(Ordering::SeqCst)
    }

    fn presented(&self) -> usize {
        self.presented.load(Ordering::SeqCst)
    }
}

/// A frame that presents like a window's and counts each present.
struct SpyFrame {
    view: wgpu::TextureView,
    counts: Arc<Counts>,
}

impl TargetFrame for SpyFrame {
    fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    fn format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }

    fn size(&self) -> wgpu::Extent3d {
        SPY_SIZE
    }

    fn present(self: Box<Self>, _queue: &wgpu::Queue) {
        self.counts.presented.fetch_add(1, Ordering::SeqCst);
    }
}

/// A target whose frames count how often they are acquired and presented.
struct SpyTarget {
    view: wgpu::TextureView,
    counts: Arc<Counts>,
}

const SPY_SIZE: wgpu::Extent3d = wgpu::Extent3d {
    width: 8,
    height: 8,
    depth_or_array_layers: 1,
};

impl SpyTarget {
    /// A target that draws into an `Rgba8Unorm` texture with `usage`.
    fn new(context: &RenderContext, usage: wgpu::TextureUsages) -> Self {
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Spy frame"),
            size: SPY_SIZE,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
            view_formats: &[],
        });
        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            counts: Arc::default(),
        }
    }
}

impl crate::targets::FrameSource for SpyTarget {
    fn get_current_frame(&self) -> Result<Box<dyn TargetFrame>, crate::SurfaceError> {
        self.counts.acquired.fetch_add(1, Ordering::SeqCst);
        Ok(Box::new(SpyFrame {
            view: self.view.clone(),
            counts: self.counts.clone(),
        }))
    }

    fn owner(&self) -> Option<Arc<RenderContext>> {
        None
    }
}

impl Target for SpyTarget {
    fn size(&self) -> Size {
        Size::from((SPY_SIZE.width, SPY_SIZE.height))
    }

    fn resize(&mut self, _size: impl Into<Size>) -> Result<(), crate::texture::TextureError> {
        Ok(())
    }

    async fn get_image(&self) -> Result<Vec<u8>, crate::texture::TextureError> {
        Ok(Vec::new())
    }
}

/// The GPU errors the device reported outside an error scope.
type Uncaptured = Arc<Mutex<Vec<String>>>;

/// A renderer with a context and a sink for the GPU errors no scope catches.
async fn setup() -> Result<(Renderer, Arc<RenderContext>, Uncaptured), InitializationError> {
    let renderer = Renderer::new();
    let context = renderer.context(None).await?;
    let uncaptured = record_uncaptured(&context);
    Ok((renderer, context, uncaptured))
}
