//! Tests: anyhow::Result + thiserror::Error derive — two-crate error-chain.
//! thiserror generates From impls and Display; anyhow provides the opaque
//! context wrapper. The interaction: anyhow::Context::context() wraps a
//! thiserror-typed Err, and anyhow's downcast_ref recovers the original type.
//! Verifiers must track the blanket From<thiserror_type> impl that anyhow
//! relies on, and the dynamic dispatch through anyhow::Error's vtable.

use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("empty input")]
    EmptyInput,
    #[error("invalid token at position {pos}: {token:?}")]
    InvalidToken { pos: usize, token: String },
    #[error("unexpected end of input after {consumed} tokens")]
    UnexpectedEof { consumed: usize },
}

fn tokenise(src: &str) -> Result<Vec<&str>, ParseError> {
    if src.is_empty() {
        return Err(ParseError::EmptyInput);
    }
    let tokens: Vec<&str> = src.split_whitespace().collect();
    for (i, t) in tokens.iter().enumerate() {
        if t.starts_with('!') {
            return Err(ParseError::InvalidToken {
                pos: i,
                token: t.to_string(),
            });
        }
    }
    Ok(tokens)
}

fn parse(src: &str) -> Result<usize> {
    let tokens = tokenise(src).context("tokenise step failed")?;
    if tokens.is_empty() {
        return Err(ParseError::UnexpectedEof { consumed: 0 }.into());
    }
    Ok(tokens.len())
}

pub fn error_chain() {
    let ok = parse("hello world foo");
    let _ = ok.unwrap();

    let bad = parse("hello !bang world");
    let err = bad.unwrap_err();
    // downcast to recover the original thiserror type through anyhow's wrapper
    let _ = err.downcast_ref::<ParseError>();

    let empty = parse("");
    let _ = empty.is_err();
}
