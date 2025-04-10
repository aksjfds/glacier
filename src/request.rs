use std::{
    net::IpAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};

use serde::Deserialize;

use crate::prelude::GlobalVal;

fn add_slash(route: &str) -> String {
    format!("/{}", route)
}

pub struct HttpRequest {
    pub req: http::Request<h2::RecvStream>,
    route_generator: Vec<String>,
    pub ip: String,
    pub global_val: Arc<GlobalVal>,
}

impl HttpRequest {
    pub fn new(req: http::Request<h2::RecvStream>, ip: IpAddr, global_val: Arc<GlobalVal>) -> Self {
        let mut route_generator: Vec<_> =
            req.uri().path().split("/").skip(1).map(add_slash).collect();
        route_generator.reverse();

        HttpRequest {
            req,
            route_generator,
            ip: ip.to_string(),
            global_val,
        }
    }

    pub fn next_route(&mut self) -> Option<String> {
        self.route_generator.pop()
    }

    pub fn param<'b, P>(&'b self) -> Result<P, ()>
    where
        P: Deserialize<'b>,
    {
        self.req
            .uri()
            .query()
            .map(serde_qs::from_str)
            .ok_or_else(|| tracing::debug!("Query Params is None!"))?
            .map_err(|e| tracing::debug!("Error when parsing params: {}", e))
    }

    pub fn counter(&self) -> usize {
        Arc::strong_count(&self.global_val)
    }
}

impl Deref for HttpRequest {
    type Target = http::Request<h2::RecvStream>;

    fn deref(&self) -> &Self::Target {
        &self.req
    }
}

impl DerefMut for HttpRequest {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.req
    }
}
