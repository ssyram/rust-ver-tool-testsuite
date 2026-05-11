/// Custom struct + impl method. Entry fn takes (i32, i32) (v1 type matrix),
/// internally constructs Point and calls method → i32 return (manhattan norm).
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn manhattan(&self) -> i32 {
        let ax = if self.x < 0 { -self.x } else { self.x };
        let ay = if self.y < 0 { -self.y } else { self.y };
        ax + ay
    }
}

pub fn manhattan_of(x: i32, y: i32) -> i32 {
    let p = Point { x, y };
    p.manhattan()
}
