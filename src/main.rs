use env_logger::{Builder, Target};
use getopts::Options;
use hyper::service::{make_service_fn, service_fn};
use hyper::{server::Server, Body, Method, Request, Response, StatusCode};
use log::{debug, error, info, warn};
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use tokio::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("?", "help", "Show this help")
        .optflag("h", "help", "Show this help")
        .optopt("v", "verbosity", "Set the log level", "LEVEL")
        .optopt("a", "addr", "Set the address", "ADDR")
        .optopt("l", "logfile", "Set the logfile", "LOGFILE");

    let show_usage = || {
        let prog_name = args.first().unwrap();
        let prog_name = Path::new(prog_name);
        let prog_name = prog_name.file_name().unwrap();
        let usage = opts.short_usage(prog_name.to_str().unwrap());
        eprintln!("{}", usage);
    };

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("{}", e);
            show_usage();
            return Ok(());
        }
    };

    let log_level = matches
        .opt_str("v")
        .unwrap_or_else(|| String::from("info"))
        .parse::<log::LevelFilter>()
        .unwrap();

    let addr = matches
        .opt_str("a")
        .unwrap_or_else(|| String::from("127.0.0.1:8080"));

    let log_file = matches
        .opt_str("l")
        .unwrap_or_else(|| String::from("debug.log"))
        .parse::<String>()
        .unwrap();

    if matches.opt_present("?") || matches.opt_present("h") {
        show_usage();
        return Ok(());
    }

    let log_file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(log_file.as_str())
        .unwrap();

    let mut builder = Builder::new();
    builder
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] [{}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter(None, log_level)
        .target(Target::Pipe(Box::new(log_file)))
        .init();

    // Create a new hyper server that listens on port 8000
    let socket_addr = match SocketAddr::from_str(addr.as_str()) {
        Ok(addr) => addr,
        Err(e) => panic!("Error parsing address: {}", e),
    };

    let make_service =
        make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle_request)) });
    let server = Server::bind(&socket_addr).serve(make_service);

    // Run the server
    info!("Listening on http://{}", socket_addr);
    if let Err(e) = server.await {
        error!("server error: {}", e);
    }

    Ok(())
}

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let method = req.method();
    let uri_path = req.uri().path();

    debug!("{method} {uri_path}");
    match (method, uri_path) {
        (&Method::GET, "/hello" | "/hello/") => handle_hello(),
        (&Method::GET, _) if uri_path.starts_with("/api/v1/") => {
            handle_api_v1(&uri_path[8..].trim_end_matches('/'))
        }
        _ => serve_file(uri_path).await,
    }
}

fn handle_hello() -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello, world!")))
}

fn handle_api_v1(sub_path: &str) -> Result<Response<Body>, hyper::Error> {
    warn!("api-v1");
    Ok(Response::new(Body::from(format!(
        "API v1 resource: {}",
        sub_path
    ))))
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
