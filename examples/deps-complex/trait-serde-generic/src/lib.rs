//! Tests: user-defined trait bounded by serde::Serialize + generic fn dispatch.
//! Defines a local Summarise trait with a summary() method. A generic function
//! emit<T: Summarise + serde::Serialize>(v: T) calls both the local trait method
//! and serde_json::to_string. Two concrete types implement Summarise; the
//! generic function is monomorphised over both.
//! Interaction: serde derive + local trait bound interact in the same where
//! clause; verifiers must handle compound trait bounds across local and extern
//! traits, plus the associated monomorphisation of serde_json's generic fn.

use serde::Serialize;

pub trait Summarise {
    fn summary(&self) -> String;
}

#[derive(Debug, Serialize)]
pub struct Article {
    pub title: String,
    pub word_count: u32,
}

#[derive(Debug, Serialize)]
pub struct Comment {
    pub author: String,
    pub upvotes: i32,
}

impl Summarise for Article {
    fn summary(&self) -> String {
        format!("Article '{}' ({} words)", self.title, self.word_count)
    }
}

impl Summarise for Comment {
    fn summary(&self) -> String {
        format!("Comment by {} (+{})", self.author, self.upvotes)
    }
}

fn emit<T: Summarise + Serialize>(v: &T) -> (String, String) {
    let desc = v.summary();
    let json = serde_json::to_string(v).unwrap();
    (desc, json)
}

pub fn trait_serde_generic() {
    let art = Article {
        title: "Formal Methods in Rust".to_string(),
        word_count: 3200,
    };
    let cmt = Comment {
        author: "fermat".to_string(),
        upvotes: 42,
    };

    // two distinct monomorphisations of emit<T>
    let (d1, j1) = emit(&art);
    let (d2, j2) = emit(&cmt);

    let _ = (d1, j1, d2, j2);
}
