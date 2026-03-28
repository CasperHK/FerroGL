//! Input device abstraction: touch, pointer, keyboard, and encoder.
//!
//! Equivalent to LVGL's `lv_indev` subsystem.  Drivers implement the
//! `InputDriver` trait; the FerroGL event loop calls `read_*` each frame.

// ─── Shared types ─────────────────────────────────────────────────────────────

/// Whether the input device is currently active (pressed / key-down).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    /// No interaction (finger up, key up, encoder still).
    Released,
    /// Active interaction (finger down, key down, encoder step).
    Pressed,
}

// ─── Pointer / touch ─────────────────────────────────────────────────────────

/// A single pointer (touch-finger, mouse, stylus) report.
#[derive(Debug, Clone, Copy)]
pub struct PointerData {
    /// X coordinate in display pixels.
    pub x: i16,
    /// Y coordinate in display pixels.
    pub y: i16,
    /// Whether the pointer is currently touching the surface.
    pub state: InputState,
}

impl PointerData {
    pub const fn new(x: i16, y: i16, state: InputState) -> Self {
        Self { x, y, state }
    }

    /// Returns `true` when this report is a press event.
    pub fn is_pressed(&self) -> bool {
        self.state == InputState::Pressed
    }
}

// ─── Keyboard ─────────────────────────────────────────────────────────────────

/// A keyboard key report.
#[derive(Debug, Clone, Copy)]
pub struct KeyData {
    /// The key code (e.g. ASCII value or platform-specific scan code).
    pub key: u32,
    /// Whether the key is currently held down.
    pub state: InputState,
}

/// Named key codes shared across platforms.
pub mod key {
    pub const UP: u32 = 0x0011;
    pub const DOWN: u32 = 0x0012;
    pub const LEFT: u32 = 0x0013;
    pub const RIGHT: u32 = 0x0014;
    pub const ENTER: u32 = 0x000A;
    pub const BACKSPACE: u32 = 0x0008;
    pub const ESCAPE: u32 = 0x001B;
    pub const HOME: u32 = 0x0002;
    pub const END: u32 = 0x0003;
    pub const DELETE: u32 = 0x007F;
}

// ─── Encoder / rotary ─────────────────────────────────────────────────────────

/// A rotary encoder report.
#[derive(Debug, Clone, Copy)]
pub struct EncoderData {
    /// Steps rotated since last read (positive = clockwise, negative = CCW).
    pub diff: i16,
    /// Whether the encoder push-button is pressed.
    pub state: InputState,
}

// ─── Gesture ─────────────────────────────────────────────────────────────────

/// Recognised swipe gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gesture {
    SwipeLeft,
    SwipeRight,
    SwipeUp,
    SwipeDown,
    DoubleTap,
}

/// A gesture report produced by the gesture recogniser.
#[derive(Debug, Clone, Copy)]
pub struct GestureData {
    pub gesture: Gesture,
    /// Pixel distance of the swipe.
    pub distance: u16,
}

// ─── InputDriver trait ────────────────────────────────────────────────────────

/// Implement this trait for your hardware to connect it to FerroGL's event loop.
///
/// Each method returns `None` when no new data is available.
pub trait InputDriver {
    /// Read the latest pointer/touch state.
    fn read_pointer(&mut self) -> Option<PointerData> {
        None
    }

    /// Read the latest key event.
    fn read_key(&mut self) -> Option<KeyData> {
        None
    }

    /// Read the latest encoder state.
    fn read_encoder(&mut self) -> Option<EncoderData> {
        None
    }

    /// Read a recognised gesture (optional; may always return `None`).
    fn read_gesture(&mut self) -> Option<GestureData> {
        None
    }
}

// ─── GestureDetector ──────────────────────────────────────────────────────────

/// Simple swipe-gesture detector built on top of pointer events.
///
/// Feed it `PointerData` reports and call `detect()` to check whether a swipe
/// has been completed.
pub struct GestureDetector {
    start_x: i16,
    start_y: i16,
    last_x: i16,
    last_y: i16,
    active: bool,
    /// Minimum distance (pixels) required to recognise a swipe.
    pub threshold: u16,
}

impl GestureDetector {
    pub fn new(threshold: u16) -> Self {
        Self { start_x: 0, start_y: 0, last_x: 0, last_y: 0, active: false, threshold }
    }

    /// Feed a pointer report into the detector.
    ///
    /// Returns a `GestureData` when a swipe is recognised, `None` otherwise.
    pub fn feed(&mut self, data: PointerData) -> Option<GestureData> {
        match data.state {
            InputState::Pressed if !self.active => {
                // Touch began.
                self.start_x = data.x;
                self.start_y = data.y;
                self.last_x = data.x;
                self.last_y = data.y;
                self.active = true;
                None
            }
            InputState::Pressed => {
                self.last_x = data.x;
                self.last_y = data.y;
                None
            }
            InputState::Released if self.active => {
                // Touch ended – check for swipe.
                self.active = false;
                self.detect()
            }
            _ => None,
        }
    }

    fn detect(&self) -> Option<GestureData> {
        let dx = self.last_x - self.start_x;
        let dy = self.last_y - self.start_y;
        let adx = dx.unsigned_abs();
        let ady = dy.unsigned_abs();

        if adx < self.threshold && ady < self.threshold {
            return None;
        }

        let (gesture, distance) = if adx > ady {
            if dx > 0 {
                (Gesture::SwipeRight, adx as u16)
            } else {
                (Gesture::SwipeLeft, adx as u16)
            }
        } else if dy > 0 {
            (Gesture::SwipeDown, ady as u16)
        } else {
            (Gesture::SwipeUp, ady as u16)
        };

        Some(GestureData { gesture, distance })
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_data_pressed() {
        let p = PointerData::new(50, 100, InputState::Pressed);
        assert!(p.is_pressed());
        assert_eq!(p.x, 50);
        assert_eq!(p.y, 100);
    }

    #[test]
    fn test_gesture_swipe_right() {
        let mut detector = GestureDetector::new(20);

        // Finger down at (0,0).
        assert!(detector.feed(PointerData::new(0, 0, InputState::Pressed)).is_none());
        // Finger moves right.
        assert!(detector.feed(PointerData::new(50, 0, InputState::Pressed)).is_none());
        // Finger up → swipe recognised.
        let g = detector.feed(PointerData::new(50, 0, InputState::Released));
        assert!(g.is_some());
        let gd = g.unwrap();
        assert_eq!(gd.gesture, Gesture::SwipeRight);
        assert_eq!(gd.distance, 50);
    }

    #[test]
    fn test_gesture_swipe_up() {
        let mut detector = GestureDetector::new(10);
        detector.feed(PointerData::new(0, 100, InputState::Pressed));
        detector.feed(PointerData::new(0, 30, InputState::Pressed));
        let g = detector.feed(PointerData::new(0, 30, InputState::Released)).unwrap();
        assert_eq!(g.gesture, Gesture::SwipeUp);
        assert_eq!(g.distance, 70);
    }

    #[test]
    fn test_gesture_below_threshold() {
        let mut detector = GestureDetector::new(50);
        detector.feed(PointerData::new(0, 0, InputState::Pressed));
        let g = detector.feed(PointerData::new(10, 0, InputState::Released));
        assert!(g.is_none());
    }

    #[test]
    fn test_key_constants_unique() {
        // Sanity check that key constants don't accidentally overlap.
        assert_ne!(key::UP, key::DOWN);
        assert_ne!(key::LEFT, key::RIGHT);
        assert_ne!(key::ENTER, key::BACKSPACE);
    }
}
