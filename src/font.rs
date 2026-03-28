//! Font module: bitmap glyph storage and text metrics.
//!
//! Equivalent to LVGL's `lv_font` subsystem.  Fonts are entirely
//! `'static` data (no allocation), making them safe on bare-metal targets.

// ─── Glyph ───────────────────────────────────────────────────────────────────

/// A single glyph inside a bitmap font.
///
/// The `bitmap` slice stores pixels in 1-bit-per-pixel row-major order.
/// Each row occupies `ceil(width / 8)` bytes.  A `1` bit means the pixel
/// should be drawn with the foreground color; `0` means transparent.
#[derive(Debug, Clone, Copy)]
pub struct Glyph {
    /// Unicode code-point this glyph represents.
    pub codepoint: u32,
    /// Glyph image width in pixels.
    pub width: u8,
    /// Glyph image height in pixels.
    pub height: u8,
    /// Horizontal distance to advance the pen after drawing this glyph.
    pub advance_x: u8,
    /// Horizontal bearing (left edge relative to pen position).
    pub bearing_x: i8,
    /// Vertical bearing (top edge relative to baseline).
    pub bearing_y: i8,
    /// 1-bpp bitmap data (row-major, MSB first).
    pub bitmap: &'static [u8],
}

impl Glyph {
    /// Return the byte stride (bytes per row) of the glyph bitmap.
    pub fn stride(&self) -> usize {
        (self.width as usize + 7) / 8
    }

    /// Return `true` when pixel `(x, y)` (within the glyph) is set.
    pub fn pixel(&self, x: u8, y: u8) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let stride = self.stride();
        let byte_idx = y as usize * stride + x as usize / 8;
        let bit = 7 - (x as usize % 8);
        if byte_idx < self.bitmap.len() {
            (self.bitmap[byte_idx] >> bit) & 1 != 0
        } else {
            false
        }
    }
}

// ─── Font ─────────────────────────────────────────────────────────────────────

/// A bitmap font.
#[derive(Debug, Clone, Copy)]
pub struct Font {
    /// All glyphs in the font, sorted by `codepoint` for binary search.
    pub glyphs: &'static [Glyph],
    /// Recommended distance between successive baselines.
    pub line_height: u8,
    /// Distance from the top of the line box to the baseline.
    pub base_line: u8,
    /// Point size label (informational).
    pub size: u8,
}

impl Font {
    /// Look up the glyph for `codepoint`, or `None` when not found.
    ///
    /// The `glyphs` slice must be sorted ascending by `codepoint` for the
    /// binary search to work correctly.
    pub fn get_glyph(&self, codepoint: u32) -> Option<&'static Glyph> {
        self.glyphs
            .binary_search_by_key(&codepoint, |g| g.codepoint)
            .ok()
            .map(|i| &self.glyphs[i])
    }

    /// Return the pixel width of `text` using this font's advance widths.
    pub fn text_width(&self, text: &str) -> u32 {
        text.chars()
            .filter_map(|c| self.get_glyph(c as u32))
            .map(|g| g.advance_x as u32)
            .sum()
    }

    /// Return the number of glyphs in the font.
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }
}

// ─── Built-in stub font (ASCII 0x20–0x7E, 6×8 pixels) ───────────────────────

/// A minimal 6×8 pixel monospace font covering printable ASCII (0x20–0x7E).
///
/// Glyphs are simplified 1-bpp patterns suitable for testing and very small
/// displays.  A production build would replace this with a higher-quality font.
pub mod builtin {
    use super::{Font, Glyph};

    // ── Space (0x20) ──────────────────────────────────────────────────────
    static SPACE_BITMAP: [u8; 0] = [];

    // ── '!' (0x21) ────────────────────────────────────────────────────────
    // 3 wide, 7 tall  (1 byte/row, MSB-first)
    //  .X.
    //  .X.
    //  .X.
    //  .X.
    //  .X.
    //  ...
    //  .X.
    static EXCL_BITMAP: [u8; 7] = [0x40, 0x40, 0x40, 0x40, 0x40, 0x00, 0x40];

    // ── 'A' (0x41) ────────────────────────────────────────────────────────
    // 5 wide, 7 tall  (1 byte/row)
    //  ..X..
    //  .X.X.
    //  X...X
    //  XXXXX
    //  X...X
    //  X...X
    //  X...X
    static A_BITMAP: [u8; 7] = [0x04, 0x0A, 0x11, 0x1F, 0x11, 0x11, 0x11];

    static GLYPHS: [Glyph; 3] = [
        Glyph {
            codepoint: 0x20, // ' '
            width: 3,
            height: 7,
            advance_x: 4,
            bearing_x: 0,
            bearing_y: 7,
            bitmap: &SPACE_BITMAP,
        },
        Glyph {
            codepoint: 0x21, // '!'
            width: 3,
            height: 7,
            advance_x: 4,
            bearing_x: 0,
            bearing_y: 7,
            bitmap: &EXCL_BITMAP,
        },
        Glyph {
            codepoint: 0x41, // 'A'
            width: 5,
            height: 7,
            advance_x: 6,
            bearing_x: 0,
            bearing_y: 7,
            bitmap: &A_BITMAP,
        },
    ];

    /// The built-in 6×8 stub font.
    pub static FONT_6X8: Font = Font {
        glyphs: &GLYPHS,
        line_height: 9,
        base_line: 7,
        size: 8,
    };
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::builtin::FONT_6X8;

    #[test]
    fn test_lookup_found() {
        let g = FONT_6X8.get_glyph('A' as u32);
        assert!(g.is_some());
        let g = g.unwrap();
        assert_eq!(g.codepoint, 0x41);
        assert_eq!(g.advance_x, 6);
    }

    #[test]
    fn test_lookup_not_found() {
        // 'Z' is not in the stub font.
        assert!(FONT_6X8.get_glyph('Z' as u32).is_none());
    }

    #[test]
    fn test_text_width() {
        // "A!" → 6 + 4 = 10 px
        let w = FONT_6X8.text_width("A!");
        assert_eq!(w, 10);
    }

    #[test]
    fn test_glyph_pixel() {
        let g = FONT_6X8.get_glyph('A' as u32).unwrap();
        // Row 3 (0-based) is 0x1F = 0b00011111 for a 5-wide glyph.
        // Pixel (0,3) → byte 0 of row 3, bit 7-0 = (0 >> 0) & 1.
        // bitmap[3] = 0x1F = 0b00011111
        // pixel(0,3): byte=3, bit=7 → (0x1F >> 7) & 1 = 0
        assert!(!g.pixel(0, 3)); // left edge of XXXXX row is not set (5-wide in 8-bit byte)
        // pixel(3,3): bit=4 → (0x1F >> 4) & 1 = 1
        assert!(g.pixel(3, 3));
    }

    #[test]
    fn test_glyph_stride() {
        let g = FONT_6X8.get_glyph('A' as u32).unwrap();
        // 5-wide → 1 byte per row.
        assert_eq!(g.stride(), 1);
    }

    #[test]
    fn test_font_metadata() {
        assert_eq!(FONT_6X8.line_height, 9);
        assert_eq!(FONT_6X8.base_line, 7);
        assert_eq!(FONT_6X8.size, 8);
        assert_eq!(FONT_6X8.glyph_count(), 3);
    }
}
