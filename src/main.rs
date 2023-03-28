use hyper::{Body, Method, Request, Response, StatusCode, server::Server};
use hyper::service::{make_service_fn, service_fn};
use std::path::Path;
use tokio::fs;

#[tokio::main]
async fn main() {
    // Create a new hyper server that listens on port 8000
    let addr = ([127, 0, 0, 1], 8000).into();
    let make_service = make_service_fn(|_conn| async {
        Ok::<_, hyper::Error>(service_fn(handle_request))
    });
    let server = Server::bind(&addr).serve(make_service);

    // Run the server
    println!("Listening on http://{}", addr);
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let method = req.method();
    let uri_path = req.uri().path();

    println!("{method} {uri_path}");
    match (method, uri_path) {
        (&Method::GET, "/hello" | "/hello/") => handle_hello(),
        (&Method::GET, _) if uri_path.starts_with("/api/v1/") => handle_api_v1(&uri_path[8..].trim_end_matches('/')),
        _ => serve_file(uri_path).await,
    }
}

fn handle_hello() -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello, world!")))
}

fn handle_api_v1(sub_path: &str) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from(format!("API v1 resource: {}", sub_path))))
}

async fn serve_file(path: &str) -> Result<Response<Body>, hyper::Error> {
    let mut file_path = Path::new("www").join(&path[1..]);
    if file_path.is_dir() {
        file_path.push("index.htm");
    }
    match fs::read(file_path).await {
        Ok(content) => Ok(Response::new(Body::from(content))),
        Err(_) => handle_not_found(),
    }
}

fn handle_not_found() -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::from("Not Found"));
    *response.status_mut() = StatusCode::NOT_FOUND;
    Ok(response)
}
