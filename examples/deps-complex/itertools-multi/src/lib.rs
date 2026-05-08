//! Tests: itertools multi-combinator chain — chunks + tuple_windows + zip_eq.
//! All three combinators come from itertools::Itertools; this entry tests
//! deep combinator composition where each adapter wraps the previous one's
//! output type (lazy iterator nesting).
//! Verifiers must handle: associated-type chains through multiple levels of
//! iterator adapters, all defined inside the itertools crate.

use itertools::Itertools;

pub fn itertools_multi() {
    let data: Vec<i32> = (1..=12).collect();

    // chunks produces IntoChunks, whose iterator yields &Chunk (lazy).
    // Collect each chunk into a Vec, then use tuple_windows over pairs.
    let chunk_vecs: Vec<Vec<i32>> = data
        .iter()
        .copied()
        .chunks(3)
        .into_iter()
        .map(|c| c.collect::<Vec<_>>())
        .collect();

    // tuple_windows over adjacent pairs of chunk-Vecs
    let mut pair_sums = Vec::new();
    for (a, b) in chunk_vecs.iter().tuple_windows() {
        let sum_a: i32 = a.iter().sum();
        let sum_b: i32 = b.iter().sum();
        pair_sums.push(sum_a + sum_b);
    }

    // zip the chunk_vecs with their reversed counterpart, using zip_longest
    let rev: Vec<Vec<i32>> = chunk_vecs.iter().rev().cloned().collect();
    let zipped: Vec<(i32, i32)> = chunk_vecs
        .iter()
        .zip_longest(rev.iter())
        .map(|pair| {
            use itertools::EitherOrBoth::*;
            match pair {
                Both(a, b) => (a.iter().sum(), b.iter().sum()),
                Left(a) => (a.iter().sum(), 0),
                Right(b) => (0, b.iter().sum()),
            }
        })
        .collect();

    let _ = (pair_sums, zipped);
}
