//! One frame: plan and record each pass, then submit and present.

pub(super) mod bind_groups;
pub(super) mod draws;
mod record;
mod samplers;

use crate::core::renderer::checks::pass::check_target;
#[cfg(not(wasm))]
use crate::core::renderer::device_errors::scopes::Origin;
use crate::core::renderer::{RenderContext, Renderable, Renderer, RendererError};
use crate::geometry::{MeshError, MeshObject, Upload};
use crate::{ShaderObject, SurfaceError, Target, TargetFrame};
use draws::{PREVIEW_TRIES, PassDraws};
use std::sync::atomic::Ordering;

impl Renderer {
    /// Draws `renderable` into `target` once the checks that need no frame pass.
    pub(super) fn draw(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), RendererError> {
        let context = self.context.read().clone();
        // On the web a GPU error an earlier frame left comes first.
        #[cfg(wasm)]
        if let Some(context) = context.as_ref() {
            context.late.take()?;
        }
        // What another Renderer made is refused before load uploads into this one.
        check_target(context.as_ref(), renderable, target)?;
        // Native render uploads the Material maps this Renderer lacks. Wasm can't block, so they wait for `load`.
        #[cfg(not(wasm))]
        futures::executor::block_on(self.load(renderable))?;
        context
            .ok_or(RendererError::NoContext)?
            .render(renderable, target)
    }
}

impl RenderContext {
    /// Renders any `Renderable` (Shader, Pass, or iterable of Pass) to a Target.
    ///
    /// A frame that acquired a surface texture presents it on every path, failed
    /// frames included, since Vulkan takes an acquired image back only through a present.
    /// wgpu clears a texture that no valid submit drew into, so a failed frame shows
    /// a cleared image, unless the Apple compute split already submitted the passes before it failed.
    pub(super) fn render(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
    ) -> Result<(), RendererError> {
        #[cfg(wasm)]
        self.late.take()?;
        let mut acquired = None;
        // A frame that fails drops its binning, so the next one bins again.
        let failed = self.lights_failed.clone();
        let result = self.validate_with(
            "frame",
            || self.submit(renderable, target, &mut acquired),
            move || failed.store(true, Ordering::Relaxed),
        );
        if result.is_err() {
            self.lights_failed.store(true, Ordering::Relaxed);
        }
        if let Some(frame) = acquired
            && frame.auto_present()
        {
            frame.present(&self.queue);
        }
        result
    }

    /// Records every pass of `renderable` and submits the commands.
    /// The frame they draw into, when a pass draws, goes in `acquired`,
    /// so the caller presents it whether recording succeeds or not.
    fn submit(
        &self,
        renderable: &impl Renderable,
        target: &impl Target,
        acquired: &mut Option<Box<dyn TargetFrame>>,
    ) -> Result<(), RendererError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        let pass_list = renderable.passes();
        // Compute passes never draw into the target, so a render made only of
        // compute passes neither checks the target nor acquires a frame.
        let draws_frame = pass_list.iter().any(|pass| !pass.is_compute());
        // Planning uploads mesh buffers to this device, so a foreign target is refused first.
        if draws_frame {
            self.check_owner(target)?;
        }
        let draws = pass_list
            .iter()
            .map(|pass| match pass.is_compute() {
                true => self.plan_compute(pass).map(|()| PassDraws::default()),
                false => self.plan_pass(pass),
            })
            .collect::<Result<Vec<_>, _>>()?;

        if draws_frame {
            *acquired = Some(self.acquire_frame(target)?);
        }
        let frame = acquired.as_deref();

        // Uploads land at the next submit, so every pass recorded
        // before it needs its own pool offsets. Rewind once per render:
        // a rewind per pass makes every pass read the last pass's uniforms.
        self.buffer_pool.write().reset();
        self.storage_pool.write().reset();
        // Only a render that draws ages the texture pool,
        // so compute steps between frames don't free a frame's textures.
        if frame.is_some() {
            self.texture_pool.write().start_render();
            self.start_lights();
        }

        // Apple's Metal driver does not reliably flush tile-based storage-texture writes between
        // two compute passes recorded in the same command buffer, so a subsequent sampled
        // (`texture_2d<…>`) read returns zeros on Apple Silicon. Submitting the encoder between
        // sequential compute passes introduces a submission boundary that reliably flushes the
        // tile memory. Render passes are unaffected (encoder-end already syncs there).
        #[cfg(apple)]
        let mut prev_was_compute = false;
        for (pass, draws) in pass_list.iter().zip(&draws) {
            let pass = pass.as_ref();
            #[cfg(apple)]
            {
                if prev_was_compute && pass.is_compute() {
                    self.queue.submit(Some(encoder.finish()));
                    encoder = self
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Command Encoder (Metal compute sync split)"),
                        });
                }
                prev_was_compute = pass.is_compute();
            }
            if pass.is_compute() {
                self.process_compute_pass(&mut encoder, pass)?
            } else if let Some(frame) = frame {
                self.process_render_pass(&mut encoder, pass, frame, draws)?
            }
        }

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }

    /// Uploads the mesh of a quick view whose check passed at `passed`.
    ///
    /// The upload counts only when the mesh's generation stayed at `passed` to its end.
    /// Every mutator moves the generation before it lets the data go, so the pack then
    /// saw the state the check saw. A mesh that changed meanwhile is checked and uploaded
    /// again, and one that keeps changing returns `MeshError::Busy`, so no frame draws
    /// a pack the check didn't see, such as an instance that `clear_instances` removed.
    fn upload_checked(
        &self,
        shader: &ShaderObject,
        mesh: &MeshObject,
        passed: u64,
    ) -> Result<Upload, RendererError> {
        let mut passed = passed;
        for _ in 0..PREVIEW_TRIES {
            let upload = mesh.upload(self)?;
            if mesh.generation() == passed {
                return Ok(upload);
            }
            match shader.check_preview()? {
                Some(again) => passed = again,
                None => return Ok(upload),
            }
        }
        Err(MeshError::Busy(PREVIEW_TRIES).into())
    }

    /// Try to get a frame once, and on Lost/Outdated, retry exactly once.
    /// This is a centralized, generic helper; specific targets may still
    /// perform their own recovery internally (e.g., WindowTarget).
    /// A surface that fails validation sends the details to the device, so on
    /// native the error is marked as the GPU's, and the frame scope reports those details.
    pub(super) fn acquire_frame(
        &self,
        target: &impl Target,
    ) -> Result<Box<dyn TargetFrame>, SurfaceError> {
        let frame = match target.get_current_frame() {
            Err(SurfaceError::Lost) | Err(SurfaceError::Outdated) => {
                // Retry exactly once.
                target.get_current_frame()
            }
            frame => frame,
        };
        #[cfg(not(wasm))]
        if matches!(frame, Err(SurfaceError::Validation)) {
            Origin::Gpu.record();
        }
        frame
    }
}
