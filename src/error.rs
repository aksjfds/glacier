#![allow(unused)]

use std::fmt::Display;

#[derive(thiserror::Error, Debug)]
pub enum GlacierError {
    #[error(transparent)]
    FromClient(#[from] std::io::Error),

    #[error("{0}")]
    FromRequest(&'static str),

    #[error("{0}")]
    Option(&'static str),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}
