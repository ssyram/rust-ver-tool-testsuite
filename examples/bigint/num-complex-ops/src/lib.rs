//! Tests: num_complex::Complex<i64> arithmetic and Complex<f64> transcendentals.
//! Deps: num-complex + num-traits (pure Rust, no C deps).
//! Note: num-complex works over any Num type; uses i64 for integer domain
//!       and f64 for norm/arg/exp (relevant for verifiers targeting float ops).
//! Operations: new, add, sub, mul, div (Complex<i64>),
//!             conj, norm_sqr, re/im accessors,
//!             Complex<f64>::norm, arg, exp, powi.

use num_complex::Complex;

pub fn num_complex_ops() {
    // --- integer complex arithmetic ---
    let a: Complex<i64> = Complex::new(3, 4);
    let b: Complex<i64> = Complex::new(1, -2);

    let _sum  = a + b;
    let _diff = a - b;
    let _prod = a * b;   // (3+4i)(1-2i) = 11 - 2i

    // conjugate and norm squared (stays in integer domain)
    let _conj     = a.conj();
    let _norm_sqr = a.norm_sqr();  // 3^2 + 4^2 = 25

    // re / im field access
    let _re = a.re;
    let _im = a.im;

    // --- float complex: norm, arg, exp ---
    let z: Complex<f64> = Complex::new(1.0_f64, 1.0_f64);
    let _norm = z.norm();       // sqrt(2)
    let _arg  = z.arg();        // pi/4
    let _exp  = z.exp();        // e^(1+i)
    let _powi = z.powi(3);      // (1+i)^3

    // division (Complex<i64>)
    // 3+4i divided by 1-2i: multiply by conjugate numerator/denominator
    let num = Complex::<i64>::new(3, 4);
    let den = Complex::<i64>::new(1, -2);
    // use f64 for exact division result
    let nf = Complex::<f64>::new(3.0, 4.0);
    let df = Complex::<f64>::new(1.0, -2.0);
    let _div = nf / df;

    let _ = (num, den);
}
