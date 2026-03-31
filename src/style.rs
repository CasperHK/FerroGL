//! Style module: colors, borders, padding, corner radius, and opacity.
//!
//! Equivalent to LVGL's `lv_style` subsystem. A `Style` is a plain value
//! type (no heap allocation) that can be composed and overridden.

use crate::color::Color;

/// A complete visual style description for a widget.
///
/// All fields have sensible defaults via `Style::default()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    /// Background fill color (alpha-aware).
    pub bg_color: Color,
    /// Foreground / text color.
    pub text_color: Color,
    /// Border line color.
    pub border_color: Color,
    /// Border line width in pixels (`0` = no border).
    pub border_width: u8,
    /// Corner radius in pixels (`0` = square corners).
    pub radius: u8,
    /// Global opacity: `255` = fully opaque, `0` = invisible.
    pub opacity: u8,
    /// Internal padding (top edge).
    pub pad_top: u8,
    /// Internal padding (bottom edge).
    pub pad_bottom: u8,
    /// Internal padding (left edge).
    pub pad_left: u8,
    /// Internal padding (right edge).
    pub pad_right: u8,
    /// Shadow blur radius in pixels (`0` = no shadow).
    pub shadow_radius: u8,
    /// Shadow color.
    pub shadow_color: Color,
    /// Shadow X offset (signed, encoded as i8).
    pub shadow_ofs_x: i8,
    /// Shadow Y offset (signed, encoded as i8).
    pub shadow_ofs_y: i8,
}

impl Style {
    /// A sensible default style: white background, black text, 1-px border.
    pub fn new() -> Self {
        Self::default()
    }

    // ── Builder-style setters ──────────────────────────────────────────────

    pub fn with_bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    pub fn with_text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    pub fn with_border(mut self, color: Color, width: u8) -> Self {
        self.border_color = color;
        self.border_width = width;
        self
    }

    pub fn with_radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }

    pub fn with_opacity(mut self, opacity: u8) -> Self {
        self.opacity = opacity;
        self
    }

    /// Set uniform padding on all four sides.
    pub fn with_padding(mut self, pad: u8) -> Self {
        self.pad_top = pad;
        self.pad_bottom = pad;
        self.pad_left = pad;
        self.pad_right = pad;
        self
    }

    /// Set padding per side.
    pub fn with_padding_trbl(mut self, top: u8, right: u8, bottom: u8, left: u8) -> Self {
        self.pad_top = top;
        self.pad_right = right;
        self.pad_bottom = bottom;
        self.pad_left = left;
        self
    }

    pub fn with_shadow(mut self, color: Color, radius: u8, ofs_x: i8, ofs_y: i8) -> Self {
        self.shadow_color = color;
        self.shadow_radius = radius;
        self.shadow_ofs_x = ofs_x;
        self.shadow_ofs_y = ofs_y;
        self
    }

    // ── Preset themes ──────────────────────────────────────────────────────

    /// A dark-mode style preset.
    pub fn dark() -> Self {
        Self {
            bg_color: Color::from_hex(0x1E1E2E),
            text_color: Color::WHITE,
            border_color: Color::from_hex(0x444444),
            border_width: 1,
            radius: 4,
            opacity: 255,
            pad_top: 4,
            pad_bottom: 4,
            pad_left: 8,
            pad_right: 8,
            shadow_radius: 0,
            shadow_color: Color::BLACK,
            shadow_ofs_x: 0,
            shadow_ofs_y: 0,
        }
    }

    /// A minimal style with no border and no padding.
    pub fn minimal() -> Self {
        Self {
            bg_color: Color::TRANSPARENT,
            border_width: 0,
            pad_top: 0,
            pad_bottom: 0,
            pad_left: 0,
            pad_right: 0,
            ..Self::default()
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            bg_color: Color::WHITE,
            text_color: Color::BLACK,
            border_color: Color::BLACK,
            border_width: 1,
            radius: 0,
            opacity: 255,
            pad_top: 4,
            pad_bottom: 4,
            pad_left: 4,
            pad_right: 4,
            shadow_radius: 0,
            shadow_color: Color::BLACK,
            shadow_ofs_x: 0,
            shadow_ofs_y: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_chain() {
        let s = Style::new()
            .with_bg_color(Color::BLUE)
            .with_text_color(Color::WHITE)
            .with_border(Color::RED, 2)
            .with_radius(8)
            .with_opacity(200)
            .with_padding(10);

        assert_eq!(s.bg_color, Color::BLUE);
        assert_eq!(s.text_color, Color::WHITE);
        assert_eq!(s.border_color, Color::RED);
        assert_eq!(s.border_width, 2);
        assert_eq!(s.radius, 8);
        assert_eq!(s.opacity, 200);
        assert_eq!(s.pad_top, 10);
        assert_eq!(s.pad_left, 10);
    }

    #[test]
    fn test_dark_preset() {
        let s = Style::dark();
        assert_eq!(s.text_color, Color::WHITE);
        assert!(s.radius > 0);
    }

    #[test]
    fn test_minimal_preset() {
        let s = Style::minimal();
        assert_eq!(s.border_width, 0);
        assert_eq!(s.pad_top, 0);
    }

    #[test]
    fn test_padding_trbl() {
        let s = Style::new().with_padding_trbl(1, 2, 3, 4);
        assert_eq!((s.pad_top, s.pad_right, s.pad_bottom, s.pad_left), (1, 2, 3, 4));
    }
}
