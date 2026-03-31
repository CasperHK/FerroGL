//! Color module: 32-bit ARGB color type, RGB-565 conversion, and blending.
//!
//! This is the Rust equivalent of LVGL's `lv_color` module.

/// A 32-bit ARGB color value.
///
/// `a = 255` is fully opaque; `a = 0` is fully transparent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    // ── Named constants ────────────────────────────────────────────────────

    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0, a: 255 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0, a: 255 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255, a: 255 };
    pub const YELLOW: Color = Color { r: 255, g: 255, b: 0, a: 255 };
    pub const CYAN: Color = Color { r: 0, g: 255, b: 255, a: 255 };
    pub const MAGENTA: Color = Color { r: 255, g: 0, b: 255, a: 255 };
    pub const GRAY: Color = Color { r: 128, g: 128, b: 128, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    // ── Constructors ───────────────────────────────────────────────────────

    /// Create a fully opaque color from RGB components.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a color with an explicit alpha channel.
    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Create a color from a packed `0xRRGGBB` value (fully opaque).
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    // ── Format conversion ──────────────────────────────────────────────────

    /// Convert to the 16-bit RGB-565 format used by many embedded displays.
    ///
    /// Bit layout: `[R4..R0][G5..G0][B4..B0]`
    pub fn to_rgb565(self) -> u16 {
        let r = (self.r as u16 >> 3) & 0x1F;
        let g = (self.g as u16 >> 2) & 0x3F;
        let b = (self.b as u16 >> 3) & 0x1F;
        (r << 11) | (g << 5) | b
    }

    /// Reconstruct a `Color` from a 16-bit RGB-565 value (alpha set to 255).
    pub fn from_rgb565(val: u16) -> Self {
        let r5 = ((val >> 11) & 0x1F) as u8;
        let g6 = ((val >> 5) & 0x3F) as u8;
        let b5 = (val & 0x1F) as u8;
        Self {
            // Expand 5-bit → 8-bit and 6-bit → 8-bit by replicating the MSBs.
            r: (r5 << 3) | (r5 >> 2),
            g: (g6 << 2) | (g6 >> 4),
            b: (b5 << 3) | (b5 >> 2),
            a: 255,
        }
    }

    /// Pack the color into a `[r, g, b, a]` array.
    pub fn to_rgba_array(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    // ── Color operations ───────────────────────────────────────────────────

    /// Linearly interpolate between `self` and `other`.
    ///
    /// `factor = 0` returns `self`; `factor = 255` returns `other`.
    pub fn mix(self, other: Color, factor: u8) -> Color {
        let f = factor as u32;
        let inv = 255u32 - f;
        Color {
            r: ((self.r as u32 * inv + other.r as u32 * f) / 255) as u8,
            g: ((self.g as u32 * inv + other.g as u32 * f) / 255) as u8,
            b: ((self.b as u32 * inv + other.b as u32 * f) / 255) as u8,
            a: ((self.a as u32 * inv + other.a as u32 * f) / 255) as u8,
        }
    }

    /// Alpha-composite `src` over `dst` (Porter-Duff "source over").
    pub fn blend(dst: Color, src: Color) -> Color {
        match src.a {
            255 => src,
            0 => dst,
            alpha => {
                let a = alpha as u32;
                let inv = 255u32 - a;
                Color {
                    r: ((dst.r as u32 * inv + src.r as u32 * a) / 255) as u8,
                    g: ((dst.g as u32 * inv + src.g as u32 * a) / 255) as u8,
                    b: ((dst.b as u32 * inv + src.b as u32 * a) / 255) as u8,
                    a: dst.a.max(alpha),
                }
            }
        }
    }

    /// Return a darkened copy of this color.
    ///
    /// `amount = 0` → no change; `amount = 255` → black.
    pub fn darken(self, amount: u8) -> Color {
        self.mix(Color::BLACK, amount)
    }

    /// Return a lightened copy of this color.
    ///
    /// `amount = 0` → no change; `amount = 255` → white.
    pub fn lighten(self, amount: u8) -> Color {
        self.mix(Color::WHITE, amount)
    }

    /// Compute a rough luminance value (0 = black, 255 = white).
    pub fn luminance(self) -> u8 {
        // ITU-R BT.601 coefficients, scaled to integer arithmetic.
        ((self.r as u32 * 299 + self.g as u32 * 587 + self.b as u32 * 114) / 1000) as u8
    }
}

impl Default for Color {
    fn default() -> Self {
        Color::BLACK
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb565_roundtrip() {
        let original = Color::new(0xF8, 0xFC, 0xF8); // multiples of 8/4
        let encoded = original.to_rgb565();
        let decoded = Color::from_rgb565(encoded);
        // Small rounding differences are expected after the expand step.
        assert!((original.r as i16 - decoded.r as i16).abs() <= 8);
        assert!((original.g as i16 - decoded.g as i16).abs() <= 4);
        assert!((original.b as i16 - decoded.b as i16).abs() <= 8);
    }

    #[test]
    fn test_blend_opaque_src() {
        let dst = Color::new(0, 0, 0);
        let src = Color::new(255, 0, 0);
        assert_eq!(Color::blend(dst, src), src);
    }

    #[test]
    fn test_blend_transparent_src() {
        let dst = Color::new(100, 150, 200);
        let src = Color::with_alpha(255, 0, 0, 0);
        assert_eq!(Color::blend(dst, src), dst);
    }

    #[test]
    fn test_mix_half() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(200, 100, 50);
        let m = a.mix(b, 128);
        // Should be approximately half of b.
        assert!((m.r as i16 - 100).abs() <= 2);
    }

    #[test]
    fn test_from_hex() {
        let c = Color::from_hex(0xFF8040);
        assert_eq!(c.r, 0xFF);
        assert_eq!(c.g, 0x80);
        assert_eq!(c.b, 0x40);
    }

    #[test]
    fn test_darken_lighten() {
        let c = Color::new(128, 128, 128);
        let dark = c.darken(255);
        assert_eq!(dark, Color::BLACK);
        let light = c.lighten(255);
        assert_eq!(light, Color::WHITE);
    }
}
