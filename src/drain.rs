use crate::event::Event;
use crate::pretty;

/// Trait for sending log events to external services.
pub trait Drain: Send + Sync {
    /// Process a completed event.
    fn send(&self, event: &Event);

    /// Flush any buffered events. Called on shutdown.
    fn flush(&self) {}
}

/// Console drain that writes events to stdout.
///
/// In pretty mode, formats events as a colored tree.
/// In JSON mode, writes one JSON line per event.
pub struct ConsoleDrain {
    pretty: bool,
    silent: bool,
}

impl ConsoleDrain {
    pub fn new(pretty: bool, silent: bool) -> Self {
        Self { pretty, silent }
    }
}

impl Drain for ConsoleDrain {
    fn send(&self, event: &Event) {
        if self.silent {
            return;
        }

        if self.pretty {
            println!("{}", pretty::format_event(event));
        } else {
            println!("{}", event.to_json());
        }
    }
}

/// A drain that buffers events and flushes them to an inner drain in batches.
///
/// Useful for network drains (Axiom, Datadog, etc.) to reduce per-event overhead.
/// Events are flushed when the buffer reaches `batch_size`, and any remaining
/// events are flushed on drop.
pub struct BatchedDrain<D: Drain> {
    inner: D,
    buffer: std::sync::Mutex<Vec<Event>>,
    batch_size: usize,
    max_buffer: usize,
}

impl<D: Drain> BatchedDrain<D> {
    /// Create a new batched drain.
    ///
    /// - `inner`: the drain to forward batches to
    /// - `batch_size`: flush when the buffer reaches this many events
    /// - `max_buffer`: drop oldest events if buffer exceeds this size
    pub fn new(inner: D, batch_size: usize, max_buffer: usize) -> Self {
        Self {
            inner,
            buffer: std::sync::Mutex::new(Vec::with_capacity(batch_size)),
            batch_size,
            max_buffer,
        }
    }
}

impl<D: Drain> Drain for BatchedDrain<D> {
    fn send(&self, event: &Event) {
        let should_flush;
        {
            // We use a block here to strictly limit the duration of the mutex lock.
            // Pushing to the vector is fast. We do NOT want to hold the lock while
            // flushing, as flushing involves IO (network, disk) which is slow and
            // could cause thread contention or deadlocks.
            let mut buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
            
            // Drop oldest if over max buffer to prevent unbounded memory growth (OOM)
            if buf.len() >= self.max_buffer {
                buf.remove(0);
            }
            buf.push(event.clone());
            should_flush = buf.len() >= self.batch_size;
        } // Lock is dropped here
        
        // Now it's safe to perform the slow IO operation without blocking other threads
        // trying to log events.
        if should_flush {
            self.flush();
        }
    }

    fn flush(&self) {
        let events: Vec<Event>;
        {
            let mut buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
            events = std::mem::take(&mut *buf);
        }
        for event in &events {
            self.inner.send(event);
        }
        self.inner.flush();
    }
}

impl<D: Drain> Drop for BatchedDrain<D> {
    fn drop(&mut self) {
        self.flush();
    }
}

/// A drain that forwards events to multiple drains simultaneously.
///
/// ```rust,no_run
/// use evlog::drain::{FanoutDrain, ConsoleDrain};
///
/// let drain = FanoutDrain::new(vec![
///     Box::new(ConsoleDrain::new(true, false)),
///     // Box::new(my_axiom_drain),
/// ]);
/// ```
pub struct FanoutDrain {
    drains: Vec<Box<dyn Drain>>,
}

impl FanoutDrain {
    /// Create a fanout drain from a list of drains.
    pub fn new(drains: Vec<Box<dyn Drain>>) -> Self {
        Self { drains }
    }
}

impl Drain for FanoutDrain {
    fn send(&self, event: &Event) {
        for drain in &self.drains {
            drain.send(event);
        }
    }

    fn flush(&self) {
        for drain in &self.drains {
            drain.flush();
        }
    }
}
