use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use winit::event_loop::{EventLoopClosed, EventLoopProxy};

use crate::controllers::gaze::{Gaze, GazeEvent};
use crate::events::handler::EventHandler;
use crate::events::window;
use crate::renderer::Renderer;

// @TODO investigate how Bevy uses the () event type for their event loop.
//
//       CONTEXT:
//       --------------------------------------------------------------
//       We follow the official Winit example for user events, which uses
//       a custom enum, but it does not ellaborate on how to manage it
//       in a large codebase. We follow their example implementation.
//
//       This enum is causing some issues, because Rust enums cannot be
//       serialized as C enums, and support for bindings is not yet
//       implemented in wasm-bindgen.
//
//       The End Goal is to have a customizable event type that users of
//       our API can use to register callbacks, and we have to handle them
//       dynamically. This struct works for now, but it's hard to extend.
#[derive(Debug, Clone, Copy)]
pub enum VipEvent {
    Gaze(GazeEvent),
    // Fixation(FixationEvent),
    // ... etc
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

    pub fn config(&mut self, options: crate::Options) -> Arc<RefCell<Renderer>> {
        let handler = self.handler.as_mut().unwrap();
        let event_loop = handler.get_event_loop();

        // @TODO  EventManager should not care about renderer or window at all.
        let window = window::init_window(event_loop, &options.window.unwrap_or_default());
        let renderer = Arc::new(RefCell::new(Renderer::new(window)));

        // @TODO make this dynamic
        for controller_option in options.controllers.iter() {
            if controller_option.gaze.is_some() {
                let gaze_options = controller_option.gaze.as_ref().unwrap().clone();
                let gaze = Box::new(Gaze::new(gaze_options));
                let mut renderer = renderer.borrow_mut();
                renderer.add_controller("Gaze".to_string(), gaze);
            }
        }

        handler.attach_renderer(renderer.clone());
        renderer
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
            "gaze::set_normalized_position" => VipEvent::Gaze(GazeEvent::ChangePosition {
                x: param1,
                y: param2,
            }),
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
