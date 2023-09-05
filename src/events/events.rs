use crate::events::handler::EventHandler;
use std::sync::{Arc, RwLock};
use winit::event_loop::{EventLoop, EventLoopClosed, EventLoopProxy};

#[derive(Debug, Clone)]
pub struct VipEvent {
    pub controller: String,
    pub event: String,
    pub args: VipArguments,
}

pub type VipArguments = Vec<String>;

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

    pub fn event_loop(&self) -> &EventLoop<VipEvent> {
        self.handler.as_ref().unwrap().get_event_loop()
    }

    pub fn attach_renderer(&mut self, renderer: Arc<RwLock<crate::renderer::Renderer>>) {
        self.handler
            .as_mut()
            .unwrap()
            .attach_renderer(renderer.clone());
    }

    /// Trigger an event on the event loop.
    pub fn trigger(
        &self,
        controller: &str,
        event: &str,
        args: VipArguments,
    ) -> Result<(), EventLoopClosed<VipEvent>> {
        self.emitter.read().unwrap().send_event(VipEvent {
            controller: controller.to_string(),
            event: event.to_string(),
            args,
        })
    }

    /// Get the main event handler.
    pub fn get_event_loop_runnner(&mut self) -> EventHandler<VipEvent> {
        let handler = *self.handler.take().unwrap();
        handler
    }
}
