//! Tests: std::collections::HashMap<String, Vec<i32>> + serde derive + serde_json.
//! serde must handle: nested generic types (HashMap<K, V> where V = Vec<i32>),
//! a struct with a #[derive(Serialize, Deserialize)] wrapping it, and an Option
//! field to exercise serde's skip_serializing_if interaction.
//! Interaction: serde's derive macro generates code for the whole nested type
//! tree; serde_json traverses it at runtime — verifiers must track the generic
//! monomorphisation chain through two levels of nesting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Dataset {
    pub name: String,
    pub rows: HashMap<String, Vec<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

pub fn collections_serde() {
    let mut rows = HashMap::new();
    rows.insert("alpha".to_string(), vec![1, 2, 3]);
    rows.insert("beta".to_string(), vec![10, 20]);

    let ds = Dataset {
        name: "experiment_1".to_string(),
        rows,
        description: Some("pilot run".to_string()),
    };

    let json = serde_json::to_string(&ds).unwrap();
    // round-trip: back may differ in key order but values must match
    let back: Dataset = serde_json::from_str(&json).unwrap();
    let _ = back.name == ds.name;
    let _ = back.rows.get("alpha") == Some(&vec![1, 2, 3]);
}
