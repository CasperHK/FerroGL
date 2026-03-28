//! Event system: typed events, object-style dispatch.
//!
//! Equivalent to LVGL's `lv_event` subsystem.

// ─── Event codes ─────────────────────────────────────────────────────────────

/// The set of events that can be emitted by a widget or system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventCode {
    // ── Pointer / touch ──────────────────────────────────────────────────
    /// The object has been pressed (finger/stylus down).
    Pressed,
    /// The pointer is pressing the object and moving.
    Pressing,
    /// The pointer was released before a long-press threshold.
    Released,
    /// A short press-and-release (tap).
    Clicked,
    /// The press was held beyond the long-press threshold.
    LongPressed,
    /// The pointer left the object's bounding box while pressed.
    PressLost,

    // ── Value changes ─────────────────────────────────────────────────────
    /// The widget's value has changed (slider, toggle, etc.).
    ValueChanged,

    // ── Focus / keyboard ─────────────────────────────────────────────────
    /// The object received keyboard/encoder focus.
    Focused,
    /// The object lost focus.
    Defocused,
    /// A key was pressed while the object is focused.
    Key,

    // ── Lifecycle ─────────────────────────────────────────────────────────
    /// The object is about to be deleted.
    Delete,
    /// The object is about to be drawn (allows custom rendering).
    Draw,
    /// The object has been created.
    Created,

    // ── Layout ───────────────────────────────────────────────────────────
    /// The object's size or position changed.
    SizeChanged,
    /// The object's style was updated.
    StyleChanged,

    // ── Scroll ───────────────────────────────────────────────────────────
    /// The scroll position changed.
    Scroll,
    /// Scroll animation has ended.
    ScrollEnd,

    // ── Custom ───────────────────────────────────────────────────────────
    /// Application-defined event; the inner byte distinguishes sub-types.
    Custom(u8),
}

// ─── Event ───────────────────────────────────────────────────────────────────

/// An event object passed to every registered handler.
#[derive(Debug, Clone, Copy)]
pub struct Event {
    /// The type of event.
    pub code: EventCode,
    /// Optional additional data (e.g. the key code for `Key` events).
    pub param: Option<u32>,
}

impl Event {
    /// Create a simple event with no extra data.
    pub const fn new(code: EventCode) -> Self {
        Self { code, param: None }
    }

    /// Create an event with a numeric parameter.
    pub const fn with_param(code: EventCode, param: u32) -> Self {
        Self { code, param: Some(param) }
    }
}

// ─── EventHandler trait ───────────────────────────────────────────────────────

/// Any object that can receive events implements this trait.
pub trait EventHandler {
    fn handle(&mut self, event: &Event);
}

// ─── EventDispatcher ─────────────────────────────────────────────────────────

/// A lightweight event dispatcher that holds up to `N` static callbacks.
///
/// Callbacks are plain function pointers (`fn(&Event)`) so no closure
/// allocation is required.
pub struct EventDispatcher<const N: usize = 8> {
    handlers: heapless::Vec<fn(&Event), N>,
}

impl<const N: usize> EventDispatcher<N> {
    pub fn new() -> Self {
        Self { handlers: heapless::Vec::new() }
    }

    /// Register a callback. Returns `Err(())` when the dispatcher is full.
    pub fn add_handler(&mut self, handler: fn(&Event)) -> Result<(), ()> {
        self.handlers.push(handler).map_err(|_| ())
    }

    /// Dispatch an event to every registered callback.
    pub fn dispatch(&self, event: &Event) {
        for h in self.handlers.iter() {
            h(event);
        }
    }
}

impl<const N: usize> Default for EventDispatcher<N> {
    fn default() -> Self {
        Self::new()
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_new() {
        let e = Event::new(EventCode::Clicked);
        assert_eq!(e.code, EventCode::Clicked);
        assert!(e.param.is_none());
    }

    #[test]
    fn test_event_with_param() {
        let e = Event::with_param(EventCode::Key, 65);
        assert_eq!(e.param, Some(65));
    }

    #[test]
    fn test_dispatcher_calls_handlers() {
        use core::sync::atomic::{AtomicU32, Ordering};
        static CALL_COUNT: AtomicU32 = AtomicU32::new(0);

        fn handler_a(_e: &Event) {
            CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        }
        fn handler_b(_e: &Event) {
            CALL_COUNT.fetch_add(1, Ordering::Relaxed);
        }

        // Reset counter (tests may run in any order).
        CALL_COUNT.store(0, Ordering::Relaxed);

        let mut dispatcher: EventDispatcher<4> = EventDispatcher::new();
        dispatcher.add_handler(handler_a).unwrap();
        dispatcher.add_handler(handler_b).unwrap();

        let event = Event::new(EventCode::Pressed);
        dispatcher.dispatch(&event);

        assert_eq!(CALL_COUNT.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_dispatcher_capacity_exceeded() {
        fn noop(_e: &Event) {}
        let mut d: EventDispatcher<2> = EventDispatcher::new();
        assert!(d.add_handler(noop).is_ok());
        assert!(d.add_handler(noop).is_ok());
        assert!(d.add_handler(noop).is_err()); // overflow
    }

    #[test]
    fn test_custom_event() {
        let e = Event::new(EventCode::Custom(42));
        if let EventCode::Custom(id) = e.code {
            assert_eq!(id, 42);
        } else {
            panic!("expected Custom variant");
        }
    }
}
