#![allow(async_fn_in_trait)]
use std::future::Future;

use crate::prelude::HttpRequest;

pub trait FilterExt<T, F, E> {
    fn filter(self, f: F) -> Result<T, E>;
}

impl<T, F, E> FilterExt<T, F, E> for Result<T, E>
where
    F: Fn(T) -> Result<T, E>,
{
    fn filter(self, f: F) -> Result<T, E> {
        match self {
            Ok(req) => f(req),
            Err(e) => Err(e),
        }
    }
}

pub trait AsyncFilterExt<T, F, Fut, E, OutErr> {
    async fn async_filter(self, f: F) -> Result<T, OutErr>;
}

impl<T, F, Fut, E, OutErr> AsyncFilterExt<T, F, Fut, E, OutErr> for Result<T, E>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = Result<T, OutErr>>,
    E: Into<OutErr>,
{
    async fn async_filter(self, f: F) -> Result<T, OutErr> {
        match self {
            Ok(req) => f(req).await,
            Err(e) => Err(e).map_err(Into::into),
        }
    }
}

pub trait AsyncMapExt<F, Fut, E, O> {
    async fn async_map(self, f: F) -> Result<O, E>;
}

impl<T, F, Fut, E, O> AsyncMapExt<F, Fut, E, O> for Result<T, E>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = O>,
{
    async fn async_map(self, f: F) -> Result<O, E> {
        match self {
            Ok(req) => Ok(f(req).await),
            Err(e) => Err(e),
        }
    }
}

pub trait ApplyExt<T, F, E, Output> {
    fn apply(self, f: F) -> Result<Output, E>;
}

impl<T, F, E, Output> ApplyExt<T, F, E, Output> for Result<T, E>
where
    F: Fn(T) -> Result<Output, E>,
{
    fn apply(self, f: F) -> Result<Output, E> {
        match self {
            Ok(req) => f(req),
            Err(e) => Err(e),
        }
    }
}

pub trait AsyncApplyExt<F, Fut, E, Output> {
    async fn async_apply(self, f: F) -> Result<Output, E>;
}

impl<F, Fut, E, Output> AsyncApplyExt<F, Fut, E, Output> for Result<HttpRequest, E>
where
    F: Fn(HttpRequest) -> Fut,
    Fut: Future<Output = Result<Output, E>>,
{
    async fn async_apply(self, f: F) -> Result<Output, E> {
        match self {
            Ok(req) => f(req).await,
            Err(e) => Err(e),
        }
    }
}
