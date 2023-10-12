use crate::{Event, EventLoop};

pub struct EventDispatcher {
    pub emitter: EventLoopProxy<Event>,
}

impl EventDispatcher {
    pub fn new(event_loop: EventLoop) -> Self {
        let emitter = event_loop.proxy();

        Self { emitter }
    }

    /// Trigger an event on the event loop.
    pub fn dispatch(&self, event: Event) -> Result<(), EventLoopClosed<Event>> {
        self.emitter.send_event(Event)
    }
}
