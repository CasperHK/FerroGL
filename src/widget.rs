//! Widgets: Buttons, charts, etc.

use crate::layout::Rect;

/// Basic event/message type for widgets.
pub enum WidgetEvent {
    Pressed,
    Released,
    Clicked,
    ValueChanged(u32),
    // ...extend as needed...
}

/// Widget trait for all UI elements.
pub trait Widget {
    /// Draw the widget into a framebuffer or via a driver.
    fn draw(&self, area: &Rect, buffer: &mut [u8]);

    /// Handle an event (e.g., touch, state change).
    fn handle_event(&mut self, event: WidgetEvent);
    
    /// Get the widget's bounding rectangle.
    fn rect(&self) -> &Rect;
}

/// A simple Button widget.
pub struct Button {
    pub label: &'static str,
    pub rect: Rect,
    pub pressed: bool,
}

impl Button {
    pub fn new(label: &'static str, rect: Rect) -> Self {
        Self {
            label,
            rect,
            pressed: false,
        }
    }
}

impl Widget for Button {
    fn draw(&self, area: &Rect, buffer: &mut [u8]) {
        // Stub: In a real implementation, draw the button into the buffer.
        // For now, just mark the area as "dirty" or set a flag.
        let _ = (area, buffer);
        // Drawing logic would go here.
    }

    fn handle_event(&mut self, event: WidgetEvent) {
        match event {
            WidgetEvent::Pressed => self.pressed = true,
            WidgetEvent::Released | WidgetEvent::Clicked => self.pressed = false,
            _ => {}
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

// Additional widgets (Label, Slider, etc.) can be added here.
