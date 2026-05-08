/// Enum with three kinds of variants: tuple / struct / multi-field.
pub fn enum_match_data() {
    enum Shape {
        Circle(f64),
        Rect(f64, f64),
        Square { side: f64 },
    }
    fn area(s: Shape) -> f64 {
        match s {
            Shape::Circle(r) => 3.14 * r * r,
            Shape::Rect(w, h) => w * h,
            Shape::Square { side } => side * side,
        }
    }
    let _ = area(Shape::Circle(2.0));
    let _ = area(Shape::Rect(3.0, 4.0));
    let _ = area(Shape::Square { side: 5.0 });
}
