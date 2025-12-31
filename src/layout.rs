//! Layout engine: Flexbox-inspired, optimized for embedded CPUs

pub enum Direction {
    Row,
    Column,
}

pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct LayoutChild {
    pub flex: u8, // 0 = fixed, >0 = flexible
    pub min_size: u16,
    pub max_size: u16,
    pub rect: Rect,
}

pub struct Layout {
    pub direction: Direction,
    pub align: Align,
    pub children: heapless::Vec<LayoutChild, 8>,
    pub rect: Rect,
}

impl Layout {
    pub fn new(direction: Direction, align: Align, rect: Rect) -> Self {
        Self {
            direction,
            align,
            children: heapless::Vec::new(),
            rect,
        }
    }

    pub fn add_child(&mut self, child: LayoutChild) -> Result<(), ()> {
        self.children.push(child).map_err(|_| ())
    }

    /// Compute layout for all children, updating their rects.
    pub fn compute(&mut self) {
        let total_flex: u16 = self.children.iter().map(|c| c.flex as u16).sum();
        let mut offset = 0u16;
        let main_size = match self.direction {
            Direction::Row => self.rect.width,
            Direction::Column => self.rect.height,
        };
        let mut fixed_total = 0u16;
        for c in self.children.iter() {
            if c.flex == 0 {
                fixed_total += match self.direction {
                    Direction::Row => c.rect.width,
                    Direction::Column => c.rect.height,
                };
            }
        }
        let flex_space = if main_size > fixed_total {
            main_size - fixed_total
        } else {
            0
        };

        for child in self.children.iter_mut() {
            let size = if child.flex == 0 || total_flex == 0 {
                match self.direction {
                    Direction::Row => child.rect.width,
                    Direction::Column => child.rect.height,
                }
            } else {
                (flex_space * child.flex as u16) / total_flex
            };
            match self.direction {
                Direction::Row => {
                    child.rect.x = self.rect.x + offset;
                    child.rect.y = self.rect.y;
                    child.rect.width = size;
                    child.rect.height = self.rect.height;
                    offset += size;
                }
                Direction::Column => {
                    child.rect.x = self.rect.x;
                    child.rect.y = self.rect.y + offset;
                    child.rect.width = self.rect.width;
                    child.rect.height = size;
                    offset += size;
                }
            }
        }
        // Alignment logic can be added here (center, end, stretch)
    }
}
