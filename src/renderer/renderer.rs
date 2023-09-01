use std::collections::HashMap;

#[cfg(not(feature = "texture"))]
use crate::renderer::{
    renderable::{AnyRenderable, Renderables},
    state::State,
};
use crate::{controllers::VipController, renderer::Renderable};
use cfg_if::cfg_if;
use winit::{event::WindowEvent, window::Window};

#[derive(Debug)]
pub struct Renderer {
    controllers: HashMap<String, VipController>,
    renderables: Renderables, // maybe not needed, because the controllers own them
    state: Option<State>,
    pub window: Window,
    pub window_size: winit::dpi::PhysicalSize<u32>,
}

impl Renderer {
    pub fn new(window: Window) -> Self {
        let window_size = window.inner_size();
        Self {
            window,
            window_size,
            controllers: HashMap::new(),
            renderables: Vec::new(),
            state: None,
        }
    }

    #[allow(dead_code)]
    pub fn add_controller(&mut self, key: String, controller: VipController) {
        self.controllers.insert(key, controller);
    }

    pub fn get_controller_for(&mut self, key: String) -> Option<&mut VipController> {
        self.controllers.get_mut(&key)
    }

    #[allow(dead_code)]
    pub fn add_renderables(&mut self, renderables: Renderables) {
        self.renderables.extend(renderables);
    }

    #[allow(dead_code)]
    pub fn add_renderable(&mut self, renderable: AnyRenderable) {
        self.renderables.push(renderable);
    }

    /// Initialize the renderer state. This function should
    /// be called after all renderables have been added.
    pub async fn initialize(&mut self) {
        let state = State::new(self.window(), &self.renderables).await;
        self.state = Some(state);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn window_input(&mut self, event: &WindowEvent) {
        match event {
            #[cfg(feature = "camera")]
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == &self.window.id() => {
                let state = self.state.as_mut().unwrap();
                state.camera_controller.handle_event(event)
            }
            _ => {}
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let state = self.state.as_mut().unwrap();

        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            state.config.width = self.window_size.width;
            state.config.height = self.window_size.height;
            state.surface.configure(&state.device, &state.config);

            cfg_if! { if #[cfg(feature = "camera")] {
                state.camera.aspect = state.config.width as f32 / state.config.height as f32;
            } else {
            }}

            cfg_if! { if #[cfg(feature = "depth")] {
                use crate::renderer::debug::texture::Texture;
                state.depth_texture =
                    Texture::create_depth_texture(&state.device, &state.config, "depth_texture");
            } else {
            }}
        }
    }

    pub fn recover(&mut self) {
        self.resize(self.window_size);
    }

    pub fn update(&mut self) {
        let state = self.state.as_mut().unwrap();

        cfg_if! { if #[cfg(feature = "camera")] {
            state.camera_controller.update_camera(&mut state.camera);
            state.camera_uniform.update(&state.camera);

            let buffer = &state.camera_buffer;
            state.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[state.camera_uniform]));
        } else {
            for (_, controller) in &self.controllers {
                if controller.should_update() {
                    for renderable in controller.renderables() {
                        renderable.update();
                        let label = renderable.label();
                        let buffer = &state.buffers[&label];

                        state.queue.write_buffer(buffer, 0, &renderable.uniform_bytes().as_slice());
                    }
                }
            }
        }}
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let state = self.state.as_mut().unwrap();

        let output = state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut commend_encoder =
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

            let mut render_pass = commend_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment,
            });

            cfg_if! { if #[cfg(feature = "instances")] {
                let instances = 0..state.instances.len() as u32;
            } else {
                let instances = 0..1;
            }}

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

            render_pass.draw_indexed(0..state.num_indices, 0, instances);
        }

        let command_buffer = commend_encoder.finish();

        state.queue.submit(std::iter::once(command_buffer));
        output.present();

        Ok(())
    }
}
