use core::fmt;
use std::{collections::HashMap, fs, io::Error};

use pull_request::PullRequest;
pub mod body_parameters;
pub mod commit;
pub mod config;
mod get_handler;
pub mod merge;
pub mod merge_test;
mod pool;
mod post_handler;
pub mod pull_request;
mod pullrequest_controller;
mod put_handler;
pub mod server;
pub mod serverhttp;

/// Instanciates ServerError from a string literal
/// formating with execution file and line.
#[macro_export]
macro_rules! server_err {
    ($msg:literal) => {
        ServerError::from(format!("ERROR: {} - {}:{}", $msg, file!(), line!()))
    };
}

// Custom error for error propagation.
#[derive(Debug)]
pub enum ServerError {
    IoError(Error),
    StringError(String),
}

impl From<Error> for ServerError {
    fn from(error: Error) -> Self {
        ServerError::IoError(error)
    }
}

impl From<String> for ServerError {
    fn from(error: String) -> Self {
        ServerError::StringError(error)
    }
}

pub fn handle_error(error: ServerError) {
    match error {
        ServerError::IoError(error) => println!("{error}"),
        ServerError::StringError(error) => println!("{error}"),
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg = match self {
            ServerError::IoError(error) => error.to_string(),
            ServerError::StringError(error) => error.to_string(),
        };
        write!(f, "{msg}")
    }
}
pub enum LogMsgStatus {
    ErrOnExecution(String),
    CorrectExecution(String),
}
/// receives the path {repo}.git/pulls and returns all the pull request in a vector
pub fn __get_pull_requests(path: &str) -> Result<HashMap<usize, PullRequest>, &'static str> {
    // validate that path exists
    let _path = std::path::Path::new(path);
    if !_path.exists() || !_path.is_dir() {
        return Err("400 The path to the pull requests does not exist\r\n\r\n");
    }

    let pull_request_dirs = fs::read_dir(path).map_err(|_| "500 Error reading pull\r\n\r\n")?;
    let mut pull_requests = HashMap::new();
    let mut pull_id;
    for pull_request_dir in pull_request_dirs.flatten() {
        let path = match pull_request_dir.file_name().to_str() {
            Some(pull_id_str) => {
                match pull_id_str.parse::<usize>() {
                    Ok(_pull_id) => pull_id = _pull_id,
                    Err(_) => {
                        return Err("500 failure while trying to build pull request path\r\n\r\n")
                    }
                }
                format!("{}/{}", path, pull_id_str)
            }
            None => return Err("500 error in directory reading\r\n\r\n"),
        };
        let pull_content = fs::read_to_string(path)
            .map_err(|_| "500 Error while trying to open pull request file\r\n\r\n")?;
        let pull_request: PullRequest = serde_json::from_str(&pull_content)
            .map_err(|_| "500 Error at deserialize pull request\r\n\r\n")?;
        pull_requests.insert(pull_id, pull_request);
    }

    Ok(pull_requests)
}
