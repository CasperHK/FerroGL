//! Widget module: the full LVGL-equivalent widget set implemented in safe Rust.
//!
//! Widgets are composable value types that store their own state.  All
//! rendering goes through the `draw::Canvas` trait so widgets are
//! display-agnostic.

use crate::color::Color;
use crate::draw::{self, Canvas};
use crate::event::{Event, EventCode, EventHandler};
use crate::layout::Rect;
use crate::style::Style;

// ─── Widget trait ─────────────────────────────────────────────────────────────

/// The core trait implemented by every FerroGL widget.
pub trait Widget: EventHandler {
    /// Render the widget into `canvas`.
    fn draw<C: Canvas>(&self, canvas: &mut C);

    /// Return the widget's bounding rectangle.
    fn rect(&self) -> &Rect;

    /// Return `true` when the widget is currently visible.
    fn is_visible(&self) -> bool {
        true
    }

    /// Return `true` when the widget can receive input focus.
    fn is_focusable(&self) -> bool {
        false
    }
}

// ─── Button ───────────────────────────────────────────────────────────────────

/// A push-button widget.
pub struct Button {
    pub rect: Rect,
    pub label: &'static str,
    pub style: Style,
    pub pressed: bool,
    pub focused: bool,
}

impl Button {
    pub fn new(label: &'static str, rect: Rect) -> Self {
        Self { rect, label, style: Style::default(), pressed: false, focused: false }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Button {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        let bg = if self.pressed { self.style.bg_color.darken(80) } else { self.style.bg_color };
        draw::fill_rect(canvas, &self.rect, bg);
        if self.style.border_width > 0 {
            let border_color =
                if self.focused { Color::BLUE } else { self.style.border_color };
            draw::draw_rect(
                canvas,
                &self.rect,
                border_color,
                self.style.border_width as u16,
            );
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn is_focusable(&self) -> bool {
        true
    }
}

impl EventHandler for Button {
    fn handle(&mut self, event: &Event) {
        match event.code {
            EventCode::Pressed => self.pressed = true,
            EventCode::Released | EventCode::Clicked => self.pressed = false,
            EventCode::Focused => self.focused = true,
            EventCode::Defocused => self.focused = false,
            _ => {}
        }
    }
}

// ─── Label ────────────────────────────────────────────────────────────────────

/// A text-display widget.
pub struct Label {
    pub rect: Rect,
    pub text: &'static str,
    pub style: Style,
}

impl Label {
    pub fn new(text: &'static str, rect: Rect) -> Self {
        Self { rect, text, style: Style::default() }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Widget for Label {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        draw::fill_rect(canvas, &self.rect, self.style.bg_color);
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl EventHandler for Label {
    fn handle(&mut self, _event: &Event) {}
}

// ─── Slider ───────────────────────────────────────────────────────────────────

/// A horizontal slider widget.
pub struct Slider {
    pub rect: Rect,
    pub style: Style,
    /// Minimum value.
    pub min: i32,
    /// Maximum value.
    pub max: i32,
    /// Current value (clamped to `[min, max]`).
    pub value: i32,
    pub focused: bool,
}

impl Slider {
    pub fn new(rect: Rect, min: i32, max: i32) -> Self {
        Self {
            rect,
            style: Style::default(),
            min,
            max,
            value: min,
            focused: false,
        }
    }

    /// Set the value, clamping to `[min, max]`.
    pub fn set_value(&mut self, v: i32) {
        self.value = v.clamp(self.min, self.max);
    }

    /// Return the normalised position of the knob (0–255).
    pub fn normalised(&self) -> u8 {
        if self.max == self.min {
            return 0;
        }
        ((self.value - self.min) as u32 * 255 / (self.max - self.min) as u32) as u8
    }
}

impl Widget for Slider {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        // Track.
        draw::fill_rect(canvas, &self.rect, self.style.bg_color);
        draw::draw_rect(canvas, &self.rect, self.style.border_color, 1);

        // Fill bar.
        let fill_w = (self.rect.width as u32 * self.normalised() as u32 / 255) as u16;
        let fill = Rect { width: fill_w, ..self.rect };
        draw::fill_rect(canvas, &fill, Color::BLUE);

        // Knob.
        let knob_x = self.rect.x + fill_w.saturating_sub(4);
        let knob = Rect { x: knob_x, y: self.rect.y, width: 8, height: self.rect.height };
        draw::fill_rect(canvas, &knob, if self.focused { Color::CYAN } else { Color::WHITE });
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn is_focusable(&self) -> bool {
        true
    }
}

impl EventHandler for Slider {
    fn handle(&mut self, event: &Event) {
        match event.code {
            EventCode::ValueChanged => {
                if let Some(v) = event.param {
                    self.set_value(v as i32);
                }
            }
            EventCode::Focused => self.focused = true,
            EventCode::Defocused => self.focused = false,
            _ => {}
        }
    }
}

// ─── Checkbox ─────────────────────────────────────────────────────────────────

/// A checkbox / toggle widget.
pub struct Checkbox {
    pub rect: Rect,
    pub label: &'static str,
    pub style: Style,
    pub checked: bool,
    pub focused: bool,
}

impl Checkbox {
    pub fn new(label: &'static str, rect: Rect) -> Self {
        Self { rect, label, style: Style::default(), checked: false, focused: false }
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
}

impl Widget for Checkbox {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        let box_size = self.rect.height.min(self.rect.width);
        let box_rect = Rect { x: self.rect.x, y: self.rect.y, width: box_size, height: box_size };

        draw::fill_rect(canvas, &box_rect, self.style.bg_color);
        let border = if self.focused { Color::BLUE } else { self.style.border_color };
        draw::draw_rect(canvas, &box_rect, border, 1);

        if self.checked {
            // Draw an X (two diagonals).
            let x0 = box_rect.x + 2;
            let y0 = box_rect.y + 2;
            let x1 = box_rect.x + box_rect.width.saturating_sub(2);
            let y1 = box_rect.y + box_rect.height.saturating_sub(2);
            draw::draw_line(canvas, x0, y0, x1, y1, self.style.text_color);
            draw::draw_line(canvas, x1, y0, x0, y1, self.style.text_color);
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn is_focusable(&self) -> bool {
        true
    }
}

impl EventHandler for Checkbox {
    fn handle(&mut self, event: &Event) {
        match event.code {
            EventCode::Clicked => self.checked = !self.checked,
            EventCode::Focused => self.focused = true,
            EventCode::Defocused => self.focused = false,
            _ => {}
        }
    }
}

// ─── Switch ───────────────────────────────────────────────────────────────────

/// An iOS-style toggle switch widget.
pub struct Switch {
    pub rect: Rect,
    pub style: Style,
    pub on: bool,
    pub focused: bool,
}

impl Switch {
    pub fn new(rect: Rect) -> Self {
        Self { rect, style: Style::default(), on: false, focused: false }
    }
}

impl Widget for Switch {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        let track_color = if self.on { Color::GREEN } else { Color::GRAY };
        draw::fill_rect(canvas, &self.rect, track_color);
        draw::draw_rect(canvas, &self.rect, self.style.border_color, 1);

        // Knob (circles drawn as small filled squares for simplicity).
        let knob_w = self.rect.height.saturating_sub(4);
        let knob_x = if self.on {
            self.rect.x + self.rect.width - knob_w - 2
        } else {
            self.rect.x + 2
        };
        let knob = Rect { x: knob_x, y: self.rect.y + 2, width: knob_w, height: knob_w };
        draw::fill_rect(canvas, &knob, Color::WHITE);
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }

    fn is_focusable(&self) -> bool {
        true
    }
}

impl EventHandler for Switch {
    fn handle(&mut self, event: &Event) {
        match event.code {
            EventCode::Clicked => self.on = !self.on,
            EventCode::Focused => self.focused = true,
            EventCode::Defocused => self.focused = false,
            _ => {}
        }
    }
}

// ─── Arc ──────────────────────────────────────────────────────────────────────

/// A circular arc progress indicator.
pub struct Arc {
    pub rect: Rect,
    pub style: Style,
    /// Start angle of the background arc (degrees, clockwise from right).
    pub bg_start_angle: u16,
    /// End angle of the background arc.
    pub bg_end_angle: u16,
    /// Current value (0–100).
    pub value: u8,
}

impl Arc {
    pub fn new(rect: Rect) -> Self {
        Self {
            rect,
            style: Style::default(),
            bg_start_angle: 135,
            bg_end_angle: 45,
            value: 0,
        }
    }

    pub fn set_value(&mut self, v: u8) {
        self.value = v.min(100);
    }

    fn center(&self) -> (u16, u16) {
        (self.rect.x + self.rect.width / 2, self.rect.y + self.rect.height / 2)
    }

    fn radius(&self) -> u16 {
        self.rect.width.min(self.rect.height) / 2
    }
}

impl Widget for Arc {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        let (cx, cy) = self.center();
        let r = self.radius();

        // Background arc.
        draw::draw_arc(canvas, cx, cy, r, self.bg_start_angle, self.bg_end_angle, Color::GRAY);

        // Foreground arc proportional to value.
        let sweep = (270u32 * self.value as u32 / 100) as u16;
        let end_angle = (self.bg_start_angle + sweep) % 360;
        draw::draw_arc(canvas, cx, cy, r, self.bg_start_angle, end_angle, Color::BLUE);
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl EventHandler for Arc {
    fn handle(&mut self, event: &Event) {
        if event.code == EventCode::ValueChanged {
            if let Some(v) = event.param {
                self.set_value(v.min(100) as u8);
            }
        }
    }
}

// ─── Bar (progress bar) ───────────────────────────────────────────────────────

/// A horizontal progress / level bar.
pub struct Bar {
    pub rect: Rect,
    pub style: Style,
    pub min: i32,
    pub max: i32,
    pub value: i32,
    /// Optional secondary (range start) value for range-mode bars.
    pub start_value: Option<i32>,
}

impl Bar {
    pub fn new(rect: Rect, min: i32, max: i32) -> Self {
        Self { rect, style: Style::default(), min, max, value: min, start_value: None }
    }

    pub fn set_value(&mut self, v: i32) {
        self.value = v.clamp(self.min, self.max);
    }

    pub fn set_start_value(&mut self, v: i32) {
        self.start_value = Some(v.clamp(self.min, self.max));
    }

    fn value_to_x(&self, v: i32) -> u16 {
        if self.max == self.min {
            return self.rect.x;
        }
        let frac = (v - self.min) as u32 * self.rect.width as u32 / (self.max - self.min) as u32;
        self.rect.x + frac as u16
    }
}

impl Widget for Bar {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        // Background.
        draw::fill_rect(canvas, &self.rect, self.style.bg_color);
        draw::draw_rect(canvas, &self.rect, self.style.border_color, 1);

        // Fill.
        let start_x = match self.start_value {
            Some(sv) => self.value_to_x(sv),
            None => self.rect.x,
        };
        let end_x = self.value_to_x(self.value);
        if end_x > start_x {
            let fill = Rect {
                x: start_x,
                y: self.rect.y,
                width: end_x - start_x,
                height: self.rect.height,
            };
            draw::fill_rect(canvas, &fill, Color::BLUE);
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl EventHandler for Bar {
    fn handle(&mut self, event: &Event) {
        if event.code == EventCode::ValueChanged {
            if let Some(v) = event.param {
                self.set_value(v as i32);
            }
        }
    }
}

// ─── Image ────────────────────────────────────────────────────────────────────

/// A static image widget backed by a raw pixel buffer.
///
/// The buffer is expected to be in RGB-565 format (2 bytes per pixel).
pub struct Image {
    pub rect: Rect,
    pub style: Style,
    pub data: &'static [u8],
    pub img_width: u16,
    pub img_height: u16,
}

impl Image {
    pub fn new(
        rect: Rect,
        data: &'static [u8],
        img_width: u16,
        img_height: u16,
    ) -> Self {
        Self { rect, style: Style::default(), data, img_width, img_height }
    }
}

impl Widget for Image {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        let cols = self.rect.width.min(self.img_width);
        let rows = self.rect.height.min(self.img_height);
        for row in 0..rows {
            for col in 0..cols {
                let idx = (row as usize * self.img_width as usize + col as usize) * 2;
                if idx + 1 < self.data.len() {
                    let word = (self.data[idx] as u16) << 8 | self.data[idx + 1] as u16;
                    let color = Color::from_rgb565(word);
                    canvas.set_pixel(self.rect.x + col, self.rect.y + row, color);
                }
            }
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl EventHandler for Image {
    fn handle(&mut self, _event: &Event) {}
}

// ─── Canvas widget ────────────────────────────────────────────────────────────

/// A free-drawing canvas widget backed by a fixed-size pixel buffer.
///
/// The const generic `BUF` is the total number of pixels (width × height).
pub struct CanvasWidget<const BUF: usize> {
    pub rect: Rect,
    pub style: Style,
    pixels: heapless::Vec<Color, BUF>,
    pub img_width: u16,
    pub img_height: u16,
}

impl<const BUF: usize> CanvasWidget<BUF> {
    pub fn new(rect: Rect, img_width: u16, img_height: u16) -> Self {
        let mut pixels = heapless::Vec::new();
        for _ in 0..BUF {
            pixels.push(Color::BLACK).ok();
        }
        Self { rect, style: Style::default(), pixels, img_width, img_height }
    }

    /// Clear the internal canvas to `color`.
    pub fn clear(&mut self, color: Color) {
        for p in self.pixels.iter_mut() {
            *p = color;
        }
    }

    /// Set a pixel inside the internal canvas (coordinates relative to the canvas).
    pub fn set_pixel(&mut self, x: u16, y: u16, color: Color) {
        if x < self.img_width && y < self.img_height {
            let idx = y as usize * self.img_width as usize + x as usize;
            if idx < self.pixels.len() {
                self.pixels[idx] = color;
            }
        }
    }
}

impl<const BUF: usize> Widget for CanvasWidget<BUF> {
    fn draw<C: Canvas>(&self, canvas: &mut C) {
        for row in 0..self.img_height {
            for col in 0..self.img_width {
                let idx = row as usize * self.img_width as usize + col as usize;
                if idx < self.pixels.len() {
                    canvas.set_pixel(self.rect.x + col, self.rect.y + row, self.pixels[idx]);
                }
            }
        }
    }

    fn rect(&self) -> &Rect {
        &self.rect
    }
}

impl<const BUF: usize> EventHandler for CanvasWidget<BUF> {
    fn handle(&mut self, _event: &Event) {}
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    struct NullCanvas;
    impl Canvas for NullCanvas {
        fn set_pixel(&mut self, _x: u16, _y: u16, _c: Color) {}
        fn get_pixel(&self, _x: u16, _y: u16) -> Color { Color::BLACK }
        fn width(&self) -> u16 { 320 }
        fn height(&self) -> u16 { 240 }
    }

    fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
        Rect { x, y, width: w, height: h }
    }

    #[test]
    fn test_button_press_release() {
        let mut btn = Button::new("OK", rect(0, 0, 80, 30));
        btn.handle(&Event::new(EventCode::Pressed));
        assert!(btn.pressed);
        btn.handle(&Event::new(EventCode::Released));
        assert!(!btn.pressed);
    }

    #[test]
    fn test_button_draw_no_panic() {
        let btn = Button::new("OK", rect(0, 0, 80, 30));
        btn.draw(&mut NullCanvas);
    }

    #[test]
    fn test_checkbox_toggle() {
        let mut cb = Checkbox::new("Accept", rect(0, 0, 20, 20));
        assert!(!cb.checked);
        cb.handle(&Event::new(EventCode::Clicked));
        assert!(cb.checked);
        cb.handle(&Event::new(EventCode::Clicked));
        assert!(!cb.checked);
    }

    #[test]
    fn test_switch_toggle() {
        let mut sw = Switch::new(rect(0, 0, 60, 24));
        assert!(!sw.on);
        sw.handle(&Event::new(EventCode::Clicked));
        assert!(sw.on);
    }

    #[test]
    fn test_slider_clamp() {
        let mut sl = Slider::new(rect(0, 0, 200, 20), 0, 100);
        sl.set_value(150);
        assert_eq!(sl.value, 100);
        sl.set_value(-10);
        assert_eq!(sl.value, 0);
    }

    #[test]
    fn test_slider_normalised() {
        let mut sl = Slider::new(rect(0, 0, 200, 20), 0, 100);
        sl.set_value(50);
        // normalised ≈ 127
        let n = sl.normalised();
        assert!((n as i16 - 127).abs() <= 2);
    }

    #[test]
    fn test_bar_value() {
        let mut bar = Bar::new(rect(0, 0, 200, 20), 0, 100);
        bar.set_value(75);
        assert_eq!(bar.value, 75);
        bar.draw(&mut NullCanvas);
    }

    #[test]
    fn test_arc_value_clamp() {
        let mut arc = Arc::new(rect(50, 50, 80, 80));
        arc.set_value(200);
        assert_eq!(arc.value, 100);
        arc.draw(&mut NullCanvas);
    }

    #[test]
    fn test_canvas_widget_pixels() {
        let mut cw: CanvasWidget<400> = CanvasWidget::new(rect(0, 0, 20, 20), 20, 20);
        cw.clear(Color::WHITE);
        cw.set_pixel(5, 5, Color::RED);
        // Can't read back through Widget::draw without a real canvas, but at
        // least verify the internal pixel is stored correctly.
        assert_eq!(cw.pixels[5 * 20 + 5], Color::RED);
    }
}
