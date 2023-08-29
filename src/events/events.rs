use std::sync::{Arc, RwLock};

use winit::event_loop::{EventLoopClosed, EventLoopProxy};

use crate::enrichments::gaze::{Gaze, GazeEvent};
use crate::events::runner::EventHandler;
use crate::events::window;
use crate::renderer::Renderer;

#[derive(Debug, Clone, Copy)]
pub enum VipEvent {
    Gaze(GazeEvent),
}

pub struct EventManager {
    pub emitter: Arc<RwLock<EventLoopProxy<VipEvent>>>,
    pub handler: Option<Box<EventHandler<VipEvent>>>,
}

impl EventManager {
    pub fn new() -> Self {
        let handler = Box::new(EventHandler::<VipEvent>::new());
        let emitter = handler.get_event_loop().create_proxy();

        Self {
            emitter: Arc::new(RwLock::new(emitter)),
            handler: Some(handler),
        }
    }

    pub async fn config(&mut self, options: crate::Options) {
        let handler = self.handler.as_mut().unwrap();
        let event_loop = handler.get_event_loop();

        // @TODO  EventManager should not care about renderer or window at all.
        let window = window::init_window(event_loop, &options.window.unwrap_or_default());
        let mut renderer = Renderer::new(window);

        // add renderables form enrichment definitions
        for enrichment_options in options.enrichments.iter() {
            if enrichment_options.gaze.is_some() {
                let gaze_options = enrichment_options.gaze.as_ref().unwrap().clone();
                let gaze = Gaze::new(gaze_options);

                let renderable = gaze.renderable();

                renderer.add_renderable(renderable);
            }
        }

        renderer.initialize().await;

        handler.attach_renderer(renderer)
    }

    /// Trigger an event on the event loop.
    pub fn trigger(
        &self,
        event: &str,
        param1: f32,
        param2: f32,
    ) -> Result<(), EventLoopClosed<VipEvent>> {
        // @TODO each enrichment should be responsible for registering their own event.
        //       I have to find a way to statically map strings to Rust enums without exposing
        //       the enum to the public API side. Consider using the `strum` crate.
        let event = match event {
            "gaze::set_normalized_position" => {
                VipEvent::Gaze(GazeEvent::ChangeNormalizedPosition {
                    x: param1,
                    y: param2,
                })
            }
            "gaze::set_position" => {
                // let size = self.handler.as_mut().unwrap().get_renderer().window_size;
                // let x = (param1 * size.width as f32) as u32;
                // let y = (param2 * size.height as f32) as u32;
                VipEvent::Gaze(GazeEvent::ChangeNormalizedPosition {
                    x: param1,
                    y: param2,
                })
            }
            _ => panic!("Event not found"),
        };

        self.emitter.read().unwrap().send_event(event)
    }

    /// Get the main event handler.
    pub fn get_event_handler(&mut self) -> EventHandler<VipEvent> {
        let handler = *self.handler.take().unwrap();
        handler
    }
}
