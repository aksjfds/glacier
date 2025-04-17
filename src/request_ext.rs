#![allow(async_fn_in_trait)]
use std::future::Future;

use crate::prelude::{HttpRequest, HttpResponse};

pub trait FilterExt<E> {
    fn filter_with(self, f: impl Fn(HttpRequest) -> Result<HttpRequest, E>) -> Result<HttpRequest, E>;
}
impl<E> FilterExt<E> for HttpRequest {
    fn filter_with(self, f: impl Fn(HttpRequest) -> Result<HttpRequest, E>) -> Result<HttpRequest, E> {
        f(self)
    }
}

pub trait AsyncFilterExt<F, E> {
    fn async_filter(self, f: F) -> impl Future<Output = Result<HttpRequest, E>>;
}
impl<F, Fut, E> AsyncFilterExt<F, E> for HttpRequest
where
    F: Fn(HttpRequest) -> Fut,
    Fut: Future<Output = Result<HttpRequest, E>>,
{
    fn async_filter(self, f: F) -> impl Future<Output = Result<HttpRequest, E>> {
        f(self)
    }
}

pub trait ApplyExt<F, E> {
    fn apply(self, f: impl Fn(HttpRequest) -> Result<HttpResponse, E>) -> Result<HttpResponse, E>;
}
impl<F, E> ApplyExt<F, E> for HttpRequest {
    fn apply(self, f: impl Fn(HttpRequest) -> Result<HttpResponse, E>) -> Result<HttpResponse, E> {
        f(self)
    }
}

pub trait AsyncApplyExt<F, E> {
    fn async_apply(self, f: F) -> impl Future<Output = Result<HttpResponse, E>>;
}
impl<F, Fut, E> AsyncApplyExt<F, E> for HttpRequest
where
    F: Fn(HttpRequest) -> Fut,
    Fut: Future<Output = Result<HttpResponse, E>>,
{
    fn async_apply(self, f: F) -> impl Future<Output = Result<HttpResponse, E>> {
        f(self)
    }
}
