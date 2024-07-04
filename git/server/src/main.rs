use server::config::ServerConfig;
use server::server::Server;
use server::serverhttp::ServerHttp;
use server::{handle_error, ServerError};
use std::str::FromStr;
use std::thread;
use std::{env, fs};
const DEFAULT_GIT_PORT: u32 = 9418; //from Daemon documentation
const DEFAULT_HTTP_PORT: u32 = 8080;

/// returns the http server port set in config file if there is any, or the default port
/// if there is none
fn get_http_port() -> u32 {
    if let Ok(str) = fs::read_to_string(".serverconfig") {
        if let Ok(server_config) = ServerConfig::from_str(str.as_str()) {
            return server_config.get_http_port();
        }
    }
    DEFAULT_HTTP_PORT
}

// Binding with a port number of 0 will request that
// the OS assigns a port to this listener. The port
// allocated can be queried via the TcpListener::local_addr method.
fn main() -> Result<(), ServerError> {
    let mut args = env::args();

    // Skip program name.
    let _ = match args.next() {
        Some(name) => name,
        None => {
            eprintln!("No program name");
            return Ok(());
        }
    };

    // Get port.
    let port = match args.next() {
        None => DEFAULT_GIT_PORT.to_string(),
        Some(port) => {
            if port.parse::<u32>().is_err() {
                eprintln!("Invalid port number");
                return Ok(());
            }

            port
        }
    };

    // Start http server.
    let http_port = get_http_port();
    thread::spawn(move || {
        let server = ServerHttp::new("127.0.0.1".to_string(), http_port);
        server
            .run()
            .ok()
            .map_or_else(|| println!("Error at start http server"), |_| {});
    });

    // Start git server.
    let full_address = format!("127.0.0.1:{port}");
    match Server::new(full_address) {
        Ok(server) => server.run()?,
        Err(e) => handle_error(e),
    }

    Ok(())
}
