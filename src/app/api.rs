use hyper::{Body, Response};
use log::warn;

use super::App;

impl App {
    pub fn handle_api_v1(&self, sub_path: &str) -> Result<Response<Body>, hyper::Error> {
        warn!("api-v1");
        Ok(Response::new(Body::from(format!(
            "API v1 resource: {}",
            sub_path
        ))))
    }
}
