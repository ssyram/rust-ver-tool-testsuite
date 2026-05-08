// Aeneas limitation: floating-point types (f32, f64) are not supported.
//
// When a function body contains a float literal or float arithmetic, Aeneas
// errors with:
//   "[Error] Improperly typed constant value"  (for float literals)
// or
//   "[Error] unsupported floats"               (for float arithmetic / binops)
//
// Floats appear in LLBC as a distinct scalar kind that Aeneas's interpreter
// and extractor do not handle; there is no mapping to any proof-assistant
// numeric type.
//
// Source: https://github.com/AeneasVerif/aeneas/issues/828

pub fn scale(x: f64, factor: f64) -> f64 {
    x * factor
}

pub fn clamp_f32(v: f32, lo: f32, hi: f32) -> f32 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

#[derive(Clone)]
pub enum Measurement {
    Missing,
    Value(f64),
}

pub fn make_measurement() -> Measurement {
    Measurement::Value(1.5)
}
