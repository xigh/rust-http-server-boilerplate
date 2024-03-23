use http_body_util::Full;
use hyper::{body::Bytes, Response};
use anyhow::Result;
use log::warn;

use super::App;

impl App {
    pub fn handle_api_v1(&self, sub_path: &str) -> Result<Response<Full<Bytes>>> {
        warn!("api-v1");
        Ok(Response::new(Full::new(Bytes::from(format!(
            "API v1 resource: {}",
            sub_path
        )))))
    }
}
