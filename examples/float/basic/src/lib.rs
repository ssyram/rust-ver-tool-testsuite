/// f32/f64 basic arithmetic: +, -, *, / on finite values including negative and fractional;
/// tests whether verifiers model IEEE 754 arithmetic at all for concrete double/single constants.
pub fn float_basic() {
    let a: f64 = 1.0_f64;
    let b: f64 = 2.0_f64;
    let _add: f64 = a + b;   // 3.0
    let _sub: f64 = b - a;   // 1.0
    let _mul: f64 = a * b;   // 2.0
    let _div: f64 = a / b;   // 0.5

    let x: f32 = 0.1_f32;
    let y: f32 = 0.2_f32;
    let _sum: f32 = x + y;   // not exactly 0.3 in IEEE 754 — tooling must not fold naively

    let neg: f64 = -3.5_f64;
    let _neg_mul: f64 = neg * 2.0_f64; // -7.0
}
