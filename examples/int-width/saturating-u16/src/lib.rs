/// u16 saturating_* arithmetic: clamps to [0, 65535] instead of wrapping.
/// Verifiers must distinguish saturating from wrapping semantics; tools that
/// model both as unconstrained or conflate them will produce wrong results
/// on boundary queries.
pub fn int_width_saturating_u16() {
    let max: u16 = u16::MAX; // 65535

    // Saturate at MAX
    let _add: u16 = max.saturating_add(1_u16);    // 65535 (clamped)
    let _add2: u16 = max.saturating_add(1000_u16); // 65535

    // Saturate at 0 (MIN for unsigned)
    let _sub: u16 = 0_u16.saturating_sub(1_u16);  // 0
    let _sub2: u16 = 5_u16.saturating_sub(10_u16); // 0

    // Non-saturating (stays mid-range)
    let _mid: u16 = 1000_u16.saturating_add(2000_u16); // 3000

    // Multiplication: saturate when product exceeds MAX
    let _mul: u16 = 1000_u16.saturating_mul(1000_u16); // 65535 (1e6 > 65535)
    let _mul2: u16 = 100_u16.saturating_mul(100_u16);  // 10000 (fits)
}
