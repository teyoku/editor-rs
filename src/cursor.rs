pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new() -> Self {
        Self { x: 0, y: 0 }
    }
}
