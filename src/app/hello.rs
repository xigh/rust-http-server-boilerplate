use hyper::{Body, Response};

use super::App;

impl App {
    pub fn handle_hello(&self) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::new(Body::from("Hello, world!")))
    }
}