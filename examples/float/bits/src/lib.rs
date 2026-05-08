/// f32/f64 bit-level reinterpretation via to_bits / from_bits (safe transmute semantics);
/// probes whether verifiers track the integer ↔ float bitvector identity, including
/// the concrete bit pattern of -0.0 (sign bit set, all else zero) and NaN payloads.
pub fn float_bits() {
    // round-trip: from_bits(to_bits(x)) == x for any finite
    let x: f64 = 1.5_f64;
    let bits: u64 = x.to_bits();
    let _back: f64 = f64::from_bits(bits);

    // well-known bit patterns
    let zero_bits: u64 = (0.0_f64).to_bits();          // 0x0000_0000_0000_0000
    let neg_zero_bits: u64 = (-0.0_f64).to_bits();     // 0x8000_0000_0000_0000
    let _sign_differs: bool = zero_bits != neg_zero_bits; // true

    // NaN: bit pattern has non-zero mantissa; sign bit irrelevant
    let nan_bits: u64 = f64::NAN.to_bits();
    let _nan_exp_mask: u64 = nan_bits & 0x7FF0_0000_0000_0000; // all 1s in exponent

    // f32 analogue
    let f: f32 = -1.0_f32;
    let fb: u32 = f.to_bits();          // 0xBF80_0000
    let _fr: f32 = f32::from_bits(fb);
}
