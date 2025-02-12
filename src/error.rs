#![allow(unused)]

use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum GlacierError {
    #[error(transparent)]
    FromClient(#[from] std::io::Error),

    #[error("error when parsing Request: {0}")]
    FromRequest(String),
}
