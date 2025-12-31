//! Widgets: Buttons, charts, etc.

pub trait Widget {
    fn draw(&self /*, target: &mut DrawTarget */);
}

pub struct Button {
    pub label: &'static str,
    // ...more fields to be added...
}

impl Widget for Button {
    fn draw(&self /*, target: &mut DrawTarget */) {
        // Drawing logic to be implemented
    }
}
