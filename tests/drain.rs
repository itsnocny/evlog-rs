#![allow(unused_imports)]

use evlog::*;
use evlog::error::EvlogError;
use evlog::event::Event;
use evlog::redact::RedactConfig;
use evlog::sampling::SamplingConfig;
use serde_json::json;

#[test]
fn batched_drain_buffers_and_flushes() {
    use std::sync::{Arc, Mutex};
    
    struct MockDrain {
        events: Arc<Mutex<Vec<Event>>>,
    }
    impl evlog::drain::Drain for MockDrain {
        fn send(&self, event: &Event) {
            self.events.lock().unwrap().push(event.clone());
        }
    }

    let storage = Arc::new(Mutex::new(Vec::new()));
    let mock = MockDrain { events: storage.clone() };
    
    let drain = evlog::drain::BatchedDrain::new(mock, 3, 10);
    
    use evlog::drain::Drain;
    
    drain.send(&Event::new(Level::Info));
    assert_eq!(storage.lock().unwrap().len(), 0, "Should buffer the first event");
    
    drain.send(&Event::new(Level::Info));
    assert_eq!(storage.lock().unwrap().len(), 0, "Should buffer the second event");
    
    drain.send(&Event::new(Level::Info));
    assert_eq!(storage.lock().unwrap().len(), 3, "Should flush on the third event");
}
