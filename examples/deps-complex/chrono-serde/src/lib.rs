//! Tests: chrono::DateTime<Utc> + serde derive + serde_json round-trip.
//! Three deps truly interact: chrono provides DateTime implementing
//! Serialize/Deserialize (via its "serde" feature), serde drives the
//! derive macros, serde_json serializes to JSON string and back.
//! Interaction: generic monomorphization of serde_json::to_string<T>
//! where T = EventRecord, which embeds chrono's DateTime.

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EventRecord {
    pub name: String,
    pub ts: DateTime<Utc>,
    pub count: u32,
}

pub fn chrono_serde() {
    let dt = Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0).unwrap();
    let rec = EventRecord {
        name: "deploy".to_string(),
        ts: dt,
        count: 42,
    };

    // serde_json::to_string monomorphises over EventRecord which contains
    // chrono::DateTime<Utc>; chrono's serde feature provides the impl.
    let json = serde_json::to_string(&rec).unwrap();
    let back: EventRecord = serde_json::from_str(&json).unwrap();
    let _ = back == rec;
}
