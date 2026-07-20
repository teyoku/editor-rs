pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}

pub struct Viewport {
    pub row_offset: usize,
    pub col_offset: usize,
}

impl Viewport {
    pub fn new() -> Self {
        Self { row_offset: 0, col_offset: 0 }
    }
}