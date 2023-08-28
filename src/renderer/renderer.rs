use crate::{
    renderer::{
        renderable::{AnyRenderable, RenderableOperations, Renderables},
        state::State,
    },
    shapes::{Circle, CircleUniform},
};
use cfg_if::cfg_if;
use winit::{event::WindowEvent, window::Window};

pub struct Renderer {
    renderables: Renderables,
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
            renderables: Vec::new(),
            state: None,
        }
    }

    #[allow(dead_code)]
    pub fn add_renderables<T: RenderableOperations + 'static>(&mut self, renderables: Vec<T>) {
        let renderables: Renderables = renderables
            .into_iter()
            .map(|renderable| Box::new(renderable) as AnyRenderable)
            .collect();

        self.renderables.extend(renderables);
    }

    pub fn add_renderable<T: RenderableOperations + 'static>(&mut self, renderable: T) {
        let renderable: AnyRenderable = Box::new(renderable);

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
                state.depth_texture =
                    Texture::create_depth_texture(&state.device, &state.config, "depth_texture");
            } else {
            }}
        }
    }

    pub fn recover(&mut self) {
        self.resize(self.window_size);
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        cfg_if! { if #[cfg(feature = "camera")] {
            self.camera_controller.handle_event(event)
        } else {
            match event {
                _ => false,
            }
        }}
    }

    pub fn update(&mut self) {
        let state = self.state.as_mut().unwrap();

        cfg_if! { if #[cfg(feature = "camera")] {
            state.camera_controller.update_camera(&mut state.camera);
            state.camera_uniform.update(&state.camera);

            state.queue.write_buffer(
                &state.camera_buffer,
                0,
                bytemuck::cast_slice(&[state.camera_uniform]),
            );
        } else {
            for renderable in &self.renderables {
                let label = renderable.label();

                renderable.update();

                // Look ma, now I can access the concrete types!
                if let Some(_) = renderable.as_any().downcast_ref::<Circle>() {
                    let circle_uniform = *renderable.uniform().borrow_mut().downcast_mut::<CircleUniform>().unwrap();

                    state.queue.write_buffer(
                        &state.buffers[&label],
                        0,
                        bytemuck::cast_slice(&[circle_uniform]),
                    );
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
                for (i, renderable) in self.renderables.iter().enumerate() {
                    let label = renderable.label();
                    render_pass.set_bind_group(i as u32, &state.bind_groups[&label], &[]);
                }
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
