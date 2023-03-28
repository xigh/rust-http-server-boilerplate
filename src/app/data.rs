use serde::Deserialize;
use hyper::{Body, Request, Response, StatusCode};
use mysql_async::prelude::Queryable;
use mysql_async::Error as MySQLError;
use serde_json::{json, Value};
use super::App;

#[derive(Debug, Deserialize)]
struct UserData {
    name: String,
    email: String,
    _foobar: Option<String>, // optional type
}

impl App {
    pub async fn handle_data(&self) -> Result<Response<Body>, hyper::Error> {
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
        
    pub async fn add_data(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
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
}