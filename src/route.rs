// #![allow(unused)]

use std::collections::HashMap;

use crate::{request::Request, response::Response};

type F = fn(Request, Response);

pub struct Routes {
    inner: HashMap<&'static str, F>,
}

unsafe impl Sync for Routes {}
unsafe impl Send for Routes {}

impl Routes {
    pub fn new() -> Self {
        Routes {
            inner: HashMap::new(),
        }
    }

    pub fn add(&mut self, path: &'static str, func: F) {
        self.inner.insert(path, func);
    }

    pub fn query<'a>(&self, path: &'a str) -> Option<&F> {
        if let Some(func) = self.inner.get(path) {
            Some(func)
        } else {
            None
        }
    }
}
