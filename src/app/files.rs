use http::StatusCode;
use http_body_util::Full;
use tokio::fs;
use std::path::Path;
use hyper::{body::Bytes, Response};
use anyhow::Result;

use super::App;

impl App {
    pub async fn serve_file(&self, path: &str) -> Result<Response<Full<Bytes>>> {
        let mut file_path = Path::new("www").join(&path[1..]);
        if file_path.is_dir() {
            file_path.push("index.htm");
        }
        match fs::read(file_path).await {
            Ok(content) => Ok(Response::new(Full::new(Bytes::from(content)))),
            Err(_) => self.handle_not_found(),
        }
    }
    
    fn handle_not_found(&self) -> Result<Response<Full<Bytes>>> {
        let mut response = Response::new(Full::new(Bytes::from("Not Found")));
        *response.status_mut() = StatusCode::NOT_FOUND;
        Ok(response)
    }
}