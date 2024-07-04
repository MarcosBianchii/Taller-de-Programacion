use std::str::FromStr;

use crate::{server_err, ServerError};

pub struct ServerConfig {
    http_port: u32,
    git_transport_port: u32,
}

// Parses a given string to a given type returning an io::Result.
fn parse<T: FromStr>(val: &str) -> Result<T, ServerError> {
    val.parse()
        .map_err(|_| ServerError::from("Invalid config file".to_string()))
}

impl FromStr for ServerConfig {
    type Err = ServerError;
    fn from_str(s: &str) -> Result<ServerConfig, ServerError> {
        if !s.starts_with("[ports]") {
            return Err(server_err!("Incorrect server configuration format"));
        }
        // set ports to default in case they are not specified
        let mut http_port = 8080;
        let mut git_transport_port = 9418;

        let iterator: Vec<_> = s.split('\n').collect();
        for line in iterator {
            if let Some((key, value)) = line.split_once(" = ") {
                match key {
                    "http" => http_port = parse(value)?,
                    "git_transport" => git_transport_port = parse(value)?,
                    _ => return Err(server_err!("Invalid config file")),
                }
            }
        }

        Ok(ServerConfig {
            http_port,
            git_transport_port,
        })
    }
}

impl ServerConfig {
    pub fn get_http_port(&self) -> u32 {
        self.http_port
    }
    pub fn get_git_transport_port(&self) -> u32 {
        self.git_transport_port
    }
}
