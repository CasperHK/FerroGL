//! Drawing module: hardware-agnostic 2-D primitives.
//!
//! Equivalent to LVGL's `lv_draw` subsystem. All rendering is done through
//! the `Canvas` trait so the same code works on any pixel buffer or display.

use crate::color::Color;
use crate::layout::Rect;

// ─── Canvas trait ─────────────────────────────────────────────────────────────

/// A 2-D pixel surface that drawing primitives render into.
///
/// Implement this trait for your framebuffer or display driver to gain access
/// to all drawing functions in this module.
pub trait Canvas {
    /// Write a single pixel. Out-of-bounds writes should be silently ignored.
    fn set_pixel(&mut self, x: u16, y: u16, color: Color);

    /// Read a single pixel (used for alpha-compositing). Returns `Color::BLACK`
    /// when out of bounds.
    fn get_pixel(&self, x: u16, y: u16) -> Color;

    /// Display width in pixels.
    fn width(&self) -> u16;

    /// Display height in pixels.
    fn height(&self) -> u16;
}

// ─── Primitives ───────────────────────────────────────────────────────────────

/// Fill the entire canvas with `color`.
pub fn clear<C: Canvas>(canvas: &mut C, color: Color) {
    let (w, h) = (canvas.width(), canvas.height());
    for y in 0..h {
        for x in 0..w {
            canvas.set_pixel(x, y, color);
        }
    }
}

/// Fill a rectangle with a solid color.
pub fn fill_rect<C: Canvas>(canvas: &mut C, rect: &Rect, color: Color) {
    let x_end = rect.x.saturating_add(rect.width);
    let y_end = rect.y.saturating_add(rect.height);
    for y in rect.y..y_end {
        for x in rect.x..x_end {
            canvas.set_pixel(x, y, color);
        }
    }
}

/// Draw a hollow rectangle outline with the given `line_width`.
pub fn draw_rect<C: Canvas>(canvas: &mut C, rect: &Rect, color: Color, line_width: u16) {
    for i in 0..line_width {
        // Top
        for x in rect.x..rect.x + rect.width {
            canvas.set_pixel(x, rect.y + i, color);
        }
        // Bottom
        for x in rect.x..rect.x + rect.width {
            canvas.set_pixel(x, rect.y + rect.height.saturating_sub(1 + i), color);
        }
        // Left
        for y in rect.y..rect.y + rect.height {
            canvas.set_pixel(rect.x + i, y, color);
        }
        // Right
        for y in rect.y..rect.y + rect.height {
            canvas.set_pixel(rect.x + rect.width.saturating_sub(1 + i), y, color);
        }
    }
}

/// Draw a horizontal line from `(x, y)` with length `len`.
pub fn draw_hline<C: Canvas>(canvas: &mut C, x: u16, y: u16, len: u16, color: Color) {
    for i in 0..len {
        canvas.set_pixel(x + i, y, color);
    }
}

/// Draw a vertical line from `(x, y)` with length `len`.
pub fn draw_vline<C: Canvas>(canvas: &mut C, x: u16, y: u16, len: u16, color: Color) {
    for i in 0..len {
        canvas.set_pixel(x, y + i, color);
    }
}

/// Draw a line between `(x0, y0)` and `(x1, y1)` using Bresenham's algorithm.
pub fn draw_line<C: Canvas>(canvas: &mut C, x0: u16, y0: u16, x1: u16, y1: u16, color: Color) {
    let (mut x, mut y) = (x0 as i32, y0 as i32);
    let (x1, y1) = (x1 as i32, y1 as i32);

    let dx = (x1 - x).abs();
    let dy = -((y1 - y).abs());
    let sx: i32 = if x < x1 { 1 } else { -1 };
    let sy: i32 = if y < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x >= 0 && y >= 0 {
            canvas.set_pixel(x as u16, y as u16, color);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            if x == x1 {
                break;
            }
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            if y == y1 {
                break;
            }
            err += dx;
            y += sy;
        }
    }
}

/// Draw a circle outline using the Midpoint Circle Algorithm.
pub fn draw_circle<C: Canvas>(canvas: &mut C, cx: u16, cy: u16, radius: u16, color: Color) {
    let (cx, cy) = (cx as i32, cy as i32);
    let mut x = radius as i32;
    let mut y = 0i32;
    let mut err = 0i32;

    while x >= y {
        plot_circle_points(canvas, cx, cy, x, y, color);
        y += 1;
        err += 1 + 2 * y;
        if 2 * (err - x) + 1 > 0 {
            x -= 1;
            err += 1 - 2 * x;
        }
    }
}

/// Draw a filled circle.
pub fn fill_circle<C: Canvas>(canvas: &mut C, cx: u16, cy: u16, radius: u16, color: Color) {
    let (cx, cy) = (cx as i32, cy as i32);
    let mut x = radius as i32;
    let mut y = 0i32;
    let mut err = 0i32;

    while x >= y {
        // Fill horizontal spans between the symmetric points.
        draw_hline_i32(canvas, cx - x, cy + y, 2 * x + 1, color);
        draw_hline_i32(canvas, cx - x, cy - y, 2 * x + 1, color);
        draw_hline_i32(canvas, cx - y, cy + x, 2 * y + 1, color);
        draw_hline_i32(canvas, cx - y, cy - x, 2 * y + 1, color);

        y += 1;
        err += 1 + 2 * y;
        if 2 * (err - x) + 1 > 0 {
            x -= 1;
            err += 1 - 2 * x;
        }
    }
}

/// Draw an arc from `start_angle` to `end_angle` (degrees, 0° = right, clockwise).
///
/// Uses a lookup-free integer approximation. For a high-quality implementation
/// swap out the trig calls with a sine table.
pub fn draw_arc<C: Canvas>(
    canvas: &mut C,
    cx: u16,
    cy: u16,
    radius: u16,
    start_angle: u16,
    end_angle: u16,
    color: Color,
) {
    // Walk the full circle and only plot points within [start, end].
    let (cx, cy) = (cx as i32, cy as i32);
    let r = radius as i32;
    let mut x = r;
    let mut y = 0i32;
    let mut err = 0i32;

    while x >= y {
        for &(px, py) in &circle_octant_points(cx, cy, x, y) {
            if point_in_arc_range(px - cx, py - cy, r, start_angle, end_angle) {
                if px >= 0 && py >= 0 {
                    canvas.set_pixel(px as u16, py as u16, color);
                }
            }
        }
        y += 1;
        err += 1 + 2 * y;
        if 2 * (err - x) + 1 > 0 {
            x -= 1;
            err += 1 - 2 * x;
        }
    }
}

/// Blit a 1-bit-per-pixel glyph bitmap onto the canvas.
///
/// `bitmap` is stored row-major; each row is `ceil(width/8)` bytes.
/// Set bits are drawn with `fg`; clear bits are skipped (transparent).
pub fn draw_glyph<C: Canvas>(
    canvas: &mut C,
    dst_x: u16,
    dst_y: u16,
    width: u8,
    height: u8,
    bitmap: &[u8],
    fg: Color,
) {
    let stride = (width as usize + 7) / 8;
    for row in 0..height as usize {
        for col in 0..width as usize {
            let byte = bitmap[row * stride + col / 8];
            let bit = 7 - (col % 8);
            if (byte >> bit) & 1 != 0 {
                canvas.set_pixel(dst_x + col as u16, dst_y + row as u16, fg);
            }
        }
    }
}

// ─── Helpers (not pub) ────────────────────────────────────────────────────────

fn plot_circle_points<C: Canvas>(canvas: &mut C, cx: i32, cy: i32, x: i32, y: i32, color: Color) {
    for &(px, py) in &circle_octant_points(cx, cy, x, y) {
        if px >= 0 && py >= 0 {
            canvas.set_pixel(px as u16, py as u16, color);
        }
    }
}

fn circle_octant_points(cx: i32, cy: i32, x: i32, y: i32) -> [(i32, i32); 8] {
    [
        (cx + x, cy + y),
        (cx + y, cy + x),
        (cx - y, cy + x),
        (cx - x, cy + y),
        (cx - x, cy - y),
        (cx - y, cy - x),
        (cx + y, cy - x),
        (cx + x, cy - y),
    ]
}

fn draw_hline_i32<C: Canvas>(canvas: &mut C, x: i32, y: i32, len: i32, color: Color) {
    if y < 0 || len <= 0 {
        return;
    }
    let x_start = x.max(0) as u16;
    let x_end = (x + len).max(0) as u16;
    let y = y as u16;
    for px in x_start..x_end {
        canvas.set_pixel(px, y, color);
    }
}

/// Integer-only angle check using octant signs instead of `atan2`.
///
/// Returns `true` when the vector `(dx, dy)` falls within the arc sweep
/// from `start_angle` to `end_angle` (degrees, clockwise from right).
fn point_in_arc_range(dx: i32, dy: i32, _r: i32, start: u16, end: u16) -> bool {
    // Map (dx, dy) → approximate angle in [0, 360).
    let angle = approx_angle(dx, dy);
    if start <= end {
        angle >= start && angle <= end
    } else {
        // Wrap-around arc (e.g. 300° → 60°).
        angle >= start || angle <= end
    }
}

/// Coarse 16-direction angle approximation (no division, no lookup table).
fn approx_angle(dx: i32, dy: i32) -> u16 {
    // Use the sign + magnitude ratio to determine one of 16 sectors × 22.5°.
    let adx = dx.unsigned_abs();
    let ady = dy.unsigned_abs();
    let sector: u16 = if adx >= ady {
        if dx >= 0 && dy >= 0 {
            0
        } else if dx >= 0 {
            315
        } else if dy >= 0 {
            180
        } else {
            135
        }
    } else if dx >= 0 && dy >= 0 {
        90
    } else if dx >= 0 {
        270
    } else if dy >= 0 {
        180
    } else {
        225
    };
    sector
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal in-memory canvas for unit tests.
    struct TestCanvas {
        width: u16,
        height: u16,
        pixels: heapless::Vec<Color, 1024>,
    }

    impl TestCanvas {
        fn new(w: u16, h: u16) -> Self {
            let mut pixels = heapless::Vec::new();
            for _ in 0..(w as usize * h as usize) {
                pixels.push(Color::BLACK).ok();
            }
            Self { width: w, height: h, pixels }
        }

        fn pixel(&self, x: u16, y: u16) -> Color {
            self.pixels[y as usize * self.width as usize + x as usize]
        }
    }

    impl Canvas for TestCanvas {
        fn set_pixel(&mut self, x: u16, y: u16, color: Color) {
            if x < self.width && y < self.height {
                self.pixels[y as usize * self.width as usize + x as usize] = color;
            }
        }
        fn get_pixel(&self, x: u16, y: u16) -> Color {
            if x < self.width && y < self.height {
                self.pixels[y as usize * self.width as usize + x as usize]
            } else {
                Color::BLACK
            }
        }
        fn width(&self) -> u16 {
            self.width
        }
        fn height(&self) -> u16 {
            self.height
        }
    }

    #[test]
    fn test_fill_rect() {
        let mut c = TestCanvas::new(20, 20);
        fill_rect(&mut c, &Rect { x: 2, y: 2, width: 4, height: 4 }, Color::WHITE);
        assert_eq!(c.pixel(2, 2), Color::WHITE);
        assert_eq!(c.pixel(5, 5), Color::WHITE);
        assert_eq!(c.pixel(6, 2), Color::BLACK); // outside
    }

    #[test]
    fn test_draw_hline() {
        let mut c = TestCanvas::new(20, 20);
        draw_hline(&mut c, 0, 5, 10, Color::RED);
        for x in 0..10 {
            assert_eq!(c.pixel(x, 5), Color::RED);
        }
        assert_eq!(c.pixel(10, 5), Color::BLACK);
    }

    #[test]
    fn test_draw_line_diagonal() {
        let mut c = TestCanvas::new(20, 20);
        draw_line(&mut c, 0, 0, 9, 9, Color::GREEN);
        // All diagonal pixels should be set.
        for i in 0..10u16 {
            assert_eq!(c.pixel(i, i), Color::GREEN, "pixel ({i},{i}) not set");
        }
    }

    #[test]
    fn test_draw_circle_center_pixel_unset() {
        let mut c = TestCanvas::new(32, 32);
        draw_circle(&mut c, 16, 16, 8, Color::WHITE);
        // Center should not be filled.
        assert_eq!(c.pixel(16, 16), Color::BLACK);
    }

    #[test]
    fn test_fill_circle_center_set() {
        let mut c = TestCanvas::new(32, 32);
        fill_circle(&mut c, 15, 15, 5, Color::WHITE);
        assert_eq!(c.pixel(15, 15), Color::WHITE);
    }

    #[test]
    fn test_clear() {
        let mut c = TestCanvas::new(8, 8);
        clear(&mut c, Color::BLUE);
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(c.pixel(x, y), Color::BLUE);
            }
        }
    }
}
