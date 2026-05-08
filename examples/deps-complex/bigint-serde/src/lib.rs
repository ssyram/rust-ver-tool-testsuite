//! Tests: num-bigint::BigInt + serde derive + serde_json round-trip.
//! num-bigint's "serde" feature provides Serialize/Deserialize for BigInt.
//! A wrapper struct uses #[derive] so serde drives the outer shape while
//! num-bigint drives the inner BigInt serialization format.
//! Interaction: heterogeneous field serialization — primitive u64 field
//! goes through std path; BigInt field goes through num-bigint's custom impl.

use num_bigint::BigInt;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BigCounter {
    pub label: String,
    pub value: BigInt,
    pub generation: u64,
}

pub fn bigint_serde() {
    let bc = BigCounter {
        label: "factorial_100".to_string(),
        value: BigInt::from_str(
            "9332621544394415268169923885626670049071596826438162146859296389521759999322991560894146397615651828625369792082722375825118521091686400000000000000000000000000"
        ).unwrap(),
        generation: 7,
    };

    let json = serde_json::to_string(&bc).unwrap();
    let back: BigCounter = serde_json::from_str(&json).unwrap();
    let _ = back == bc;
}
