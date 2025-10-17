use crossterm::event::{self, Event};
use std::sync::mpsc;
use std::time::Duration;

pub struct InputWatcher {
    receiver: mpsc::Receiver<Event>,
}

impl Default for InputWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl InputWatcher {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();

        std::thread::spawn(move || loop {
            if let Ok(true) = event::poll(Duration::from_millis(10)) {
                if let Ok(event) = event::read() {
                    if sender.send(event).is_err() {
                        break;
                    }
                }
            }
        });

        Self { receiver }
    }

    pub fn try_recv(&self) -> Result<Event, mpsc::TryRecvError> {
        self.receiver.try_recv()
    }
}
