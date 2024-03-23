use http_body_util::Full;
use serde::Deserialize;
use http::header::CONTENT_TYPE;
use hyper::{body::{Bytes, Incoming}, Method, Request, Response, StatusCode};
use log::debug;
use mysql_async::{Conn, Pool};
use serde_json::json;

use anyhow::Result;

mod data;
mod hello;
mod api;
mod files;

#[derive(Debug, Deserialize)]
pub struct Config {
    database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    db_user: String,
    db_pass: String,
    db_host: String,
    db_name: String,
}

pub struct App {
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
        }
    }

    pub async fn handle_request(&self, req: Request<Incoming>) -> Result<Response<Full<Bytes>>> {
        let method = req.method();
        let uri_path = req.uri().path();
        let headers = req.headers();
    
        debug!("{method} {uri_path}");
        match (method, uri_path) {
            (&Method::GET, "/hello" | "/hello/") => self.handle_hello(),
            (&Method::GET, _) if uri_path.starts_with("/api/v1/") => {
                self.handle_api_v1(&uri_path[8..].trim_end_matches('/'))
            }
            (&Method::GET, "/data") => self.handle_data().await,
            (&Method::POST, "/data") => {
                if headers.get(CONTENT_TYPE)
                    == Some(&http::HeaderValue::from_static("application/json"))
                {
                    self.add_data(req).await
                } else {
                    let mut response = Response::new(Full::new(Bytes::from("Unsupported Content-Type")));
                    *response.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                    Ok(response)
                }
            }
            _ => self.serve_file(uri_path).await,
        }
    }
    
    async fn get_conn(&self) -> Conn {
        let DatabaseConfig {db_user, db_pass, db_host, db_name} = &self.config.database;
        let database_url = format!("mysql://{db_user}:{db_pass}@{db_host}/{db_name}");
        let pool = Pool::new(database_url.as_str());
        pool.get_conn().await.unwrap()
    }
        
    fn create_error_response<T1: Into<String>, T2: Into<String>>(
        &self,
        status: StatusCode,
        error: T1,
        message: T2,
    ) -> Result<Response<Full<Bytes>>> {
        let response_body = json!({
            "error": error.into(),
            "message": message.into(),
        });
        let response_body = serde_json::to_string(&response_body).unwrap();
        let response = Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Full::new(Bytes::from(response_body)))
            .unwrap()
            ;
        Ok(response)
    }
}