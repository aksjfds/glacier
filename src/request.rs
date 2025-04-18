use crate::handler::HandleReq;
use crate::prelude::HyperRequest;
use crate::response::Response;

pub struct Request {
    hyper_request: HyperRequest,
}

impl Request {
    pub fn new(req: HyperRequest) -> Self {
        Self { hyper_request: req }
    }

    pub fn filter<E>(self, handler: impl HandleReq<E>) -> Result<Request, E> {
        handler.filter(self)
    }

    pub fn async_filter<E>(
        self,
        handler: impl HandleReq<E>,
    ) -> impl Future<Output = Result<Request, E>> {
        handler.async_filter(self)
    }

    pub fn map<E>(self, handler: impl HandleReq<E>) -> Result<Response, E> {
        handler.handle(self)
    }

    pub fn async_map<E>(
        self,
        handler: impl HandleReq<E>,
    ) -> impl Future<Output = Result<Response, E>> {
        handler.async_handle(self)
    }
}

impl Request {
    pub fn param<'b, P>(&'b self) -> Option<P>
    where
        P: serde::Deserialize<'b>,
    {
        self.hyper_request
            .uri()
            .query()
            .map(serde_qs::from_str)?
            .ok()
    }
}

impl std::ops::Deref for Request {
    type Target = HyperRequest;

    fn deref(&self) -> &Self::Target {
        &self.hyper_request
    }
}

impl std::ops::DerefMut for Request {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.hyper_request
    }
}
