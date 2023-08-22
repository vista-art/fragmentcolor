use std::sync::{Arc, RwLock};

use winit::{
    event_loop::{EventLoop, EventLoopBuilder, EventLoopClosed, EventLoopProxy},
    window::{Window, WindowBuilder},
};

#[cfg(target_arch = "wasm32")]
use {
    gloo_utils::{document, window as web_window},
    wasm_bindgen::JsCast,
    winit::platform::web::WindowBuilderExtWebSys,
};

use crate::enrichments::gaze::GazeEvent;
use crate::events::runner;
use crate::state::State;

#[derive(Debug, Clone, Copy)]
pub enum VipEvent {
    Gaze(GazeEvent),
}

pub struct EventLoopContainer {
    state: Option<State>,
    event_loop: EventLoop<VipEvent>,
    event_loop_runner: Box<dyn FnOnce(EventLoop<VipEvent>, State) + Send>,
}

impl EventLoopContainer {
    fn new(event_loop_runner: Box<dyn FnOnce(EventLoop<VipEvent>, State) + Send>) -> Self {
        Self {
            state: None,
            event_loop: EventLoopBuilder::<VipEvent>::with_user_event().build(),
            event_loop_runner,
        }
    }

    fn set_state(&mut self, state: State) {
        self.state = Some(state);
    }

    fn get_event_loop(&self) -> &EventLoop<VipEvent> {
        &self.event_loop
    }

    pub fn run(self) {
        (self.event_loop_runner)(self.event_loop, self.state.expect("State not set"));
    }
}

pub struct EventManager {
    pub event_loop_proxy: Arc<RwLock<EventLoopProxy<VipEvent>>>,
    pub event_loop_container: Option<Box<EventLoopContainer>>,
}

impl EventManager {
    pub fn new() -> Self {
        let event_loop_runner = Box::new(runner::run_event_loop);
        let event_loop_container = Box::new(EventLoopContainer::new(event_loop_runner));
        let event_loop = event_loop_container.get_event_loop();

        Self {
            event_loop_proxy: Arc::new(RwLock::new(event_loop.create_proxy())),
            event_loop_container: Some(event_loop_container),
        }
    }

    pub fn trigger(&self, event: VipEvent) -> Result<(), EventLoopClosed<VipEvent>> {
        self.event_loop_proxy.read().unwrap().send_event(event)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn config(&mut self, options: crate::Options) {
        let window = self.init_window();
        let state = State::new(window, options.enrichments).await;

        self.event_loop_container.as_mut().unwrap().set_state(state)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn init_window(&self) -> Window {
        let event_loop = self.event_loop_container.as_ref().unwrap().get_event_loop();

        let window = WindowBuilder::new()
            .build(event_loop)
            .expect("Couldn't build window");

        window
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub async fn run(&mut self) {
        let event_loop_container = *self.event_loop_container.take().unwrap();
        event_loop_container.run()
    }
}
