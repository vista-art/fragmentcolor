use crate::events::{Event, EventLoop};

pub struct App {
    _event_loop: EventLoop<Event>,
}

pub struct AppOptions {
    pub log_level: Option<String>,
}

impl Default for AppOptions {
    fn default() -> Self {
        Self {
            log_level: Some("info".to_string()),
        }
    }
}

impl App {
    pub fn new(_options: AppOptions) -> Self {
        // @TODO start logger like in the old lib

        let _event_loop = EventLoop::new();
        Self { _event_loop }
    }

    pub fn run(&mut self) {
        //self.event_loop.run();
        todo!()
    }
}
