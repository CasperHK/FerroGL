//! Layout engine: Flexbox-inspired and Grid layouts, optimized for embedded CPUs.

/// Layout direction for flex containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Row,
    Column,
}

/// Child alignment within the cross axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

/// Axis-aligned bounding rectangle (coordinates in pixels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    /// Returns `true` when the point `(px, py)` is inside this rectangle.
    pub fn contains(&self, px: u16, py: u16) -> bool {
        px >= self.x
            && px < self.x + self.width
            && py >= self.y
            && py < self.y + self.height
    }

    /// Returns the intersection of `self` and `other`, or `None` if they do not overlap.
    pub fn intersect(&self, other: &Rect) -> Option<Rect> {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let x2 = (self.x + self.width).min(other.x + other.width);
        let y2 = (self.y + self.height).min(other.y + other.height);
        if x2 > x && y2 > y {
            Some(Rect { x, y, width: x2 - x, height: y2 - y })
        } else {
            None
        }
    }
}

/// A child item in a flex layout.
#[derive(Debug, Clone)]
pub struct LayoutChild {
    /// `0` = fixed size, `>0` = flex factor (proportional share of remaining space).
    pub flex: u8,
    pub min_size: u16,
    pub max_size: u16,
    pub rect: Rect,
}

/// A flex-based layout container holding up to 8 children.
pub struct Layout {
    pub direction: Direction,
    pub align: Align,
    pub children: heapless::Vec<LayoutChild, 8>,
    pub rect: Rect,
}

impl Layout {
    /// Create an empty layout with the given direction, alignment and bounding rectangle.
    pub fn new(direction: Direction, align: Align, rect: Rect) -> Self {
        Self {
            direction,
            align,
            children: heapless::Vec::new(),
            rect,
        }
    }

    /// Add a child. Returns `Err(())` when the 8-child limit is reached.
    pub fn add_child(&mut self, child: LayoutChild) -> Result<(), ()> {
        self.children.push(child).map_err(|_| ())
    }

    /// Compute (update) the bounding rectangles of every child.
    pub fn compute(&mut self) {
        let total_flex: u16 = self.children.iter().map(|c| c.flex as u16).sum();
        let mut offset = 0u16;
        let main_size = match self.direction {
            Direction::Row => self.rect.width,
            Direction::Column => self.rect.height,
        };

        // Total fixed space consumed by non-flex children.
        let fixed_total: u16 = self
            .children
            .iter()
            .filter(|c| c.flex == 0)
            .map(|c| match self.direction {
                Direction::Row => c.rect.width,
                Direction::Column => c.rect.height,
            })
            .sum();

        let flex_space = main_size.saturating_sub(fixed_total);

        for child in self.children.iter_mut() {
            let size = if child.flex == 0 || total_flex == 0 {
                match self.direction {
                    Direction::Row => child.rect.width,
                    Direction::Column => child.rect.height,
                }
            } else {
                ((flex_space as u32 * child.flex as u32) / total_flex as u32) as u16
            };

            // Clamp to [min_size, max_size].
            let size = size.max(child.min_size).min(child.max_size);

            match self.direction {
                Direction::Row => {
                    child.rect.x = self.rect.x + offset;
                    child.rect.y = self.rect.y;
                    child.rect.width = size;
                    let cross = self.rect.height;
                    child.rect.height = match self.align {
                        Align::Stretch => cross,
                        _ => child.rect.height.min(cross),
                    };
                    // Center / end alignment on the cross axis.
                    if self.align == Align::Center && child.rect.height < cross {
                        child.rect.y = self.rect.y + (cross - child.rect.height) / 2;
                    } else if self.align == Align::End && child.rect.height < cross {
                        child.rect.y = self.rect.y + cross - child.rect.height;
                    }
                    offset += size;
                }
                Direction::Column => {
                    child.rect.x = self.rect.x;
                    child.rect.y = self.rect.y + offset;
                    child.rect.height = size;
                    let cross = self.rect.width;
                    child.rect.width = match self.align {
                        Align::Stretch => cross,
                        _ => child.rect.width.min(cross),
                    };
                    if self.align == Align::Center && child.rect.width < cross {
                        child.rect.x = self.rect.x + (cross - child.rect.width) / 2;
                    } else if self.align == Align::End && child.rect.width < cross {
                        child.rect.x = self.rect.x + cross - child.rect.width;
                    }
                    offset += size;
                }
            }
        }
    }
}

// ─── Grid layout ─────────────────────────────────────────────────────────────

/// A cell inside a grid layout.
#[derive(Debug, Clone, Copy)]
pub struct GridCell {
    /// Column index (0-based).
    pub col: u8,
    /// Row index (0-based).
    pub row: u8,
    /// Column span (≥1).
    pub col_span: u8,
    /// Row span (≥1).
    pub row_span: u8,
    /// Computed bounding rectangle (filled in by `GridLayout::compute`).
    pub rect: Rect,
}

/// A fixed-column, fixed-row grid layout holding up to 16 cells.
///
/// Column and row counts are fixed at construction time; widths and heights are
/// distributed evenly (uniform tracks).
pub struct GridLayout {
    pub cols: u8,
    pub rows: u8,
    pub cells: heapless::Vec<GridCell, 16>,
    pub rect: Rect,
    /// Gap between columns (pixels).
    pub col_gap: u16,
    /// Gap between rows (pixels).
    pub row_gap: u16,
}

impl GridLayout {
    /// Create a new grid layout.
    pub fn new(cols: u8, rows: u8, rect: Rect) -> Self {
        Self {
            cols,
            rows,
            cells: heapless::Vec::new(),
            rect,
            col_gap: 0,
            row_gap: 0,
        }
    }

    /// Add a cell to the grid. Returns `Err(())` when the 16-cell limit is reached.
    pub fn add_cell(&mut self, cell: GridCell) -> Result<(), ()> {
        self.cells.push(cell).map_err(|_| ())
    }

    /// Compute bounding rectangles for every cell using uniform track sizing.
    pub fn compute(&mut self) {
        if self.cols == 0 || self.rows == 0 {
            return;
        }

        let total_col_gap = self.col_gap * (self.cols.saturating_sub(1) as u16);
        let total_row_gap = self.row_gap * (self.rows.saturating_sub(1) as u16);
        let avail_w = self.rect.width.saturating_sub(total_col_gap);
        let avail_h = self.rect.height.saturating_sub(total_row_gap);

        let track_w = avail_w / self.cols as u16;
        let track_h = avail_h / self.rows as u16;

        for cell in self.cells.iter_mut() {
            let col = cell.col as u16;
            let row = cell.row as u16;
            let cs = cell.col_span.max(1) as u16;
            let rs = cell.row_span.max(1) as u16;

            cell.rect.x = self.rect.x + col * (track_w + self.col_gap);
            cell.rect.y = self.rect.y + row * (track_h + self.row_gap);
            cell.rect.width = track_w * cs + self.col_gap * cs.saturating_sub(1);
            cell.rect.height = track_h * rs + self.row_gap * rs.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rect_contains() {
        let r = Rect { x: 10, y: 10, width: 100, height: 50 };
        assert!(r.contains(10, 10));
        assert!(r.contains(109, 59));
        assert!(!r.contains(9, 10));
        assert!(!r.contains(110, 10));
    }

    #[test]
    fn test_rect_intersect() {
        let a = Rect { x: 0, y: 0, width: 100, height: 100 };
        let b = Rect { x: 50, y: 50, width: 100, height: 100 };
        let i = a.intersect(&b).unwrap();
        assert_eq!(i, Rect { x: 50, y: 50, width: 50, height: 50 });

        let c = Rect { x: 200, y: 200, width: 10, height: 10 };
        assert!(a.intersect(&c).is_none());
    }

    #[test]
    fn test_flex_row_layout() {
        let bounds = Rect { x: 0, y: 0, width: 300, height: 100 };
        let mut layout = Layout::new(Direction::Row, Align::Start, bounds);
        // Two equal-flex children.
        layout
            .add_child(LayoutChild { flex: 1, min_size: 0, max_size: 300, rect: Rect::default() })
            .unwrap();
        layout
            .add_child(LayoutChild { flex: 1, min_size: 0, max_size: 300, rect: Rect::default() })
            .unwrap();
        layout.compute();
        assert_eq!(layout.children[0].rect.width, 150);
        assert_eq!(layout.children[1].rect.x, 150);
    }

    #[test]
    fn test_grid_layout() {
        let bounds = Rect { x: 0, y: 0, width: 300, height: 200 };
        let mut grid = GridLayout::new(3, 2, bounds);
        grid.add_cell(GridCell { col: 0, row: 0, col_span: 1, row_span: 1, rect: Rect::default() })
            .unwrap();
        grid.add_cell(GridCell { col: 1, row: 0, col_span: 2, row_span: 1, rect: Rect::default() })
            .unwrap();
        grid.compute();
        // Each track is 100px wide, 100px tall.
        assert_eq!(grid.cells[0].rect.width, 100);
        assert_eq!(grid.cells[0].rect.height, 100);
        // Col span 2 → 200px wide.
        assert_eq!(grid.cells[1].rect.width, 200);
        assert_eq!(grid.cells[1].rect.x, 100);
    }
}
