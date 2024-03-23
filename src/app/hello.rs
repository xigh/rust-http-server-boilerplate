use http_body_util::Full;
use hyper::{body::Bytes, Response};
use anyhow::Result;

use super::App;

impl App {
    pub fn handle_hello(&self) -> Result<Response<Full<Bytes>>> {
        Ok(Response::new(Full::new(Bytes::from("Hello, world!"))))
    }
}