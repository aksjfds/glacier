#![allow(unused)]
use std::{
    error::Error as StdError,
    fmt::{Debug, Display},
    io,
};

pub(crate) type BoxError = Box<dyn StdError>;
pub struct GlacierError {
    kind: ErrorKind,
    source: Option<BoxError>,
}
impl GlacierError {
    fn new(kind: ErrorKind, source: Option<BoxError>) -> Self {
        GlacierError { kind, source }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    FromClient,
}

pub(crate) fn from_client<E: Into<BoxError>>(e: E) -> GlacierError {
    GlacierError::new(ErrorKind::FromClient, Some(e.into()))
}

//
//
//
//
//
//
//
//
//
//
//
//
//
impl Debug for GlacierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("source", &self.source)
            .finish()
    }
}

impl Display for GlacierError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .field("source", &self.source)
            .finish()
    }
}
