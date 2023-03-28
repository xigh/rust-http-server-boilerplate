use std::path::Path;
use mysql_async::prelude::Queryable;
use serde::Deserialize;
use http::header::CONTENT_TYPE;
use hyper::{Body, Method, Request, Response, StatusCode};
use log::{debug, warn};
use mysql_async::Error as MySQLError;
use mysql_async::{Conn, Pool};
use serde_json::{json, Value};
use tokio::fs;

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

#[derive(Debug, Deserialize)]
struct UserData {
    name: String,
    email: String,
    _foobar: Option<String>, // optional type
}

impl App {
    pub fn new(config: Config) -> Self {
        Self {
            config,
        }
    }

    pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
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
                    let mut response = Response::new(Body::from("Unsupported Content-Type"));
                    *response.status_mut() = StatusCode::UNSUPPORTED_MEDIA_TYPE;
                    Ok(response)
                }
            }
            _ => self.serve_file(uri_path).await,
        }
    }
    
    fn handle_hello(&self) -> Result<Response<Body>, hyper::Error> {
        Ok(Response::new(Body::from("Hello, world!")))
    }
    
    fn handle_api_v1(&self, sub_path: &str) -> Result<Response<Body>, hyper::Error> {
        warn!("api-v1");
        Ok(Response::new(Body::from(format!(
            "API v1 resource: {}",
            sub_path
        ))))
    }
    
    async fn serve_file(&self, path: &str) -> Result<Response<Body>, hyper::Error> {
        let mut file_path = Path::new("www").join(&path[1..]);
        if file_path.is_dir() {
            file_path.push("index.htm");
        }
        match fs::read(file_path).await {
            Ok(content) => Ok(Response::new(Body::from(content))),
            Err(_) => self.handle_not_found(),
        }
    }
    
    fn handle_not_found(&self) -> Result<Response<Body>, hyper::Error> {
        let mut response = Response::new(Body::from("Not Found"));
        *response.status_mut() = StatusCode::NOT_FOUND;
        Ok(response)
    }
    
    async fn get_conn(&self) -> Conn {
        let DatabaseConfig {db_user, db_pass, db_host, db_name} = &self.config.database;
        let database_url = format!("mysql://{db_user}:{db_pass}@{db_host}/{db_name}");
        let pool = Pool::new(database_url.as_str());
        pool.get_conn().await.unwrap()
    }
    
    async fn handle_data(&self) -> Result<Response<Body>, hyper::Error> {
        let mut conn = self.get_conn().await;
    
        let query_result = conn
            .query("SELECT id, name, email FROM users")
            .await
            .unwrap();
    
        let users: Vec<Value> = query_result
            .into_iter()
            .map(|row| {
                let (id, name, email): (u64, String, String) = mysql_async::from_row(row);
                json!({
                    "id": id,
                    "name": name,
                    "email": email,
                })
            })
            .collect();
    
        let json_response = serde_json::to_string(&users).unwrap();
        let response = Response::builder()
            .header("Content-Type", "application/json")
            .body(Body::from(json_response))
            .unwrap();
    
        Ok(response)
    }
        
    async fn add_data(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
        let bytes = match hyper::body::to_bytes(req.into_body()).await {
            Ok(bytes) => bytes,
            Err(error) => return self.create_error_response(
                StatusCode::BAD_REQUEST,
                "Could not read body",
                format!("{}", error),
            ),
        };
    
        let user_data: UserData = match serde_json::from_slice(&bytes) {
            Ok(user_data) => user_data,
            Err(error) => return self.create_error_response(
                StatusCode::BAD_REQUEST,
                "Could not decode body",
                format!("{}", error),
            ),
        };
    
        let mut conn = self.get_conn().await;
    
        let insert_query = format!(
            "INSERT INTO users (name, email) VALUES ('{}', '{}')",
            user_data.name, user_data.email
        );
    
        let result = conn.query_drop(insert_query).await;
    
        match result {
            Ok(_) => {
                let last_insert_id = conn.last_insert_id();
                let json_response = json!({
                    "last_insert_id": last_insert_id,
                });
                let response_body = serde_json::to_string(&json_response).unwrap();
                let response = Response::builder()
                    .header("Content-Type", "application/json")
                    .body(Body::from(response_body))
                    .unwrap();
                Ok(response)
            }
            Err(error) => match error {
                MySQLError::Server(server_error) if server_error.code == 1062 => self.create_error_response(
                    StatusCode::CONFLICT,
                    "Duplicate entry",
                    server_error.message,
                ),
                _ => self.create_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error",
                    format!("{}", error),
                ),
            },
        }
    }
    
    fn create_error_response<T1: Into<String>, T2: Into<String>>(
        &self,
        status: StatusCode,
        error: T1,
        message: T2,
    ) -> Result<Response<Body>, hyper::Error> {
        let response_body = json!({
            "error": error.into(),
            "message": message.into(),
        });
        let response_body = serde_json::to_string(&response_body).unwrap();
        let response = Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(response_body))
            .unwrap();
        Ok(response)
    }
}