use crate::renderer::{state::State, RenderableTrait};
use crate::scene::Scene;
use cfg_if::cfg_if;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
#[cfg(feature = "camera")]
use winit::event::WindowEvent;

#[derive(Debug)]
pub struct Renderer {
    state: Option<State>,
}

impl Renderer {
    pub fn new() -> Self {
        Self { state: None }
    }

    // @TODO refactor. This was moved from PLRender main instance.
    pub fn run(&mut self) {
        let mut event_manager = self.event_manager_write_lock();
        let runner = event_manager.get_event_loop_runnner();
        drop(event_manager); // release to avoid deadlock

        // @TODO this should be responsibility of the wrappers
        #[cfg(wasm)]
        wasm_bindgen_futures::spawn_local(runner.run_event_loop());

        #[cfg(not(wasm))]
        pollster::block_on(runner.run_event_loop()); // this function never returns
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let state = self.state.as_mut().unwrap();

        // the surface is only required for on-screen rendering
        // they won't be present for file or callback targets.
        let frame = state.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut command_encoder =
            state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            cfg_if! { if #[cfg(feature = "depth")] {
                let depth_stencil_attachment = Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &state.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                });
            } else {
                let depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment> = None;
            }}

            // FOREACH frame
            // The render pass runs for each frame
            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),

                // FOREACH target
                // Notice this is an Array, we can render to multiple targets at once
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // clear color must be a property of the target
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],

                depth_stencil_attachment,
            });

            let instances = 0..state.instances.len() as u32;

            // now: THIS block should run for each object in the scene
            // One pipeline for each object that uses a different shader
            render_pass.set_pipeline(&state.render_pipeline);
            cfg_if! { if #[cfg(feature = "texture")] {
                render_pass.set_bind_group(0, &state.texture_bind_group, &[]);
                #[cfg(feature = "camera")]
                render_pass.set_bind_group(1, &state.camera_bind_group, &[]);
            } else {
                render_pass.set_bind_group(0, &state.bind_groups["Screen"], &[]);
                render_pass.set_bind_group(1, &state.bind_groups["Renderables"], &[]);
            }}

            render_pass.set_vertex_buffer(0, state.vertex_buffer.slice(..));
            #[cfg(feature = "instances")]
            render_pass.set_vertex_buffer(1, state.instance_buffer.slice(..));
            render_pass.set_index_buffer(state.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // this does not draw a frame, it draws a shader pair
            render_pass.draw_indexed(0..state.num_indices, 0, instances);
        }

        let command_buffer = command_encoder.finish();

        state.queue.submit(std::iter::once(command_buffer));

        frame.present();

        Ok(())
    }
}
