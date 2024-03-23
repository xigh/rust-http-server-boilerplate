use env_logger::{Builder, Target};
use getopts::Options;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use std::env;
use std::fs::OpenOptions;
use std::io::Write;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::TcpListener;

use std::fs::File;
use std::io::Read;
use toml::de::Error as TomlError;

use anyhow::Result;

mod app;
use app::{App, Config};

fn read_config<T: Into<String>>(filename: T) -> Result<Config, TomlError> {
    let mut file = File::open(filename.into()).expect("Unable to open config file");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .expect("Unable to read config file content");

    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optflag("?", "help", "Show this help")
        .optflag("h", "help", "Show this help")
        .optopt("v", "verbosity", "Set the log level", "LEVEL")
        .optopt("c", "config", "Set config file", "CONFIG_FILE")
        .optopt("a", "addr", "Set the address", "ADDR")
        .optopt("l", "logfile", "Set the logfile", "LOG_FILE");

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

    let config_file = matches
        .opt_str("l")
        .unwrap_or_else(|| String::from("config.toml"))
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

    let config = read_config(config_file).expect("could not read config_file");
    let app = Arc::new(App::new(config));
    let listener = TcpListener::bind(socket_addr).await?;

    // Run the server
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        let app = Arc::clone(&app);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(move |req| {
                    let app = Arc::clone(&app);
                    async move { app.handle_request(req).await }
                }))
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }

    // never return
    // Ok(())
}
