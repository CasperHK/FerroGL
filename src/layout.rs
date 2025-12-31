//! Layout engine: Flexbox-inspired, optimized for embedded CPUs

pub enum Direction {
    Row,
    Column,
}

pub struct Layout {
    pub direction: Direction,
    // ...more fields to be added...
}

impl Layout {
    pub fn new(direction: Direction) -> Self {
        Self { direction }
    }
    // ...layout methods to be added...
}
