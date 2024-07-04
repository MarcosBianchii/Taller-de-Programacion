use crate::get_handler::handle_get;
use crate::post_handler::handle_post;
use crate::put_handler::handle_put;
use crate::server::THREAD_POOL_SIZE;
use crate::server_err;
use crate::{pool::threadpool::ThreadPool, ServerError};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

pub struct ServerHttp {
    address: String,
    port: u32,
}

fn log_cmd(reader: mpsc::Receiver<String>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("httplog.txt")?;

    while let Ok(line) = reader.recv() {
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn log_connection(execution_status: String, sender: Arc<Mutex<Sender<String>>>) {
    if let Ok(guard) = sender.lock() {
        let time = chrono::Local::now().format("%d-%m-%Y %H:%M:%S");
        let log_msg = format!("{time} - {}\n\n\n", execution_status.trim());
        let _ = guard.send(log_msg);
    }
}

impl ServerHttp {
    pub fn new(address: String, port: u32) -> Self {
        ServerHttp { address, port }
    }

    pub fn run(&self) -> Result<(), ServerError> {
        let full_address = format!("{}:{}", self.address, self.port);
        let listener = TcpListener::bind(full_address)?;
        let pool = ThreadPool::build(THREAD_POOL_SIZE)?;

        // Set cmd logging.
        let (rv, rc) = mpsc::channel();
        thread::spawn(move || log_cmd(rc));
        let rv = Arc::new(Mutex::new(rv));

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(e) => {
                    println!("Error getting stream: {e}");
                    continue;
                }
            };

            let sender = rv.clone();
            let res = pool.execute(|| {
                let log = match Self::handle_connection(stream) {
                    Ok(msg) => {
                        println!("Connection handled");
                        msg
                    }
                    Err(e) => {
                        println!("Error handling connection: {e}");
                        e.to_string()
                    }
                };

                log_connection(log, sender);
            });

            if let Err(e) = res {
                println!("Error sending job to threadpool: {e}");
            }
        }

        Ok(())
    }

    fn handle_connection(mut stream: TcpStream) -> Result<String, ServerError> {
        let mut buf = [0; 5012];
        let size = stream.read(&mut buf)?;

        let request_str = String::from_utf8_lossy(&buf[..size]);
        let mut first_line = String::new();
        let mut body = String::new();
        let mut content_type = String::new();

        if let Some(body_start) = request_str.find("\r\n\r\n") {
            let (headers_str, body_str) = request_str.split_at(body_start + 4);

            if let Some(first_line_end) = headers_str.find('\n') {
                first_line.push_str(&headers_str[..first_line_end]);
            }

            content_type.push_str(extract_content_type(headers_str));
            body.push_str(body_str);
        }

        // In GET requests the Content-Type should be empty
        let (send, ret) = if !content_type.is_empty() && content_type != "application/json" {
            let send = "HTTP/1.1 415 Unsupported Media Type\r\n\r\n".to_string();
            let ret = format!("Unsupported media Type, found: {}", content_type);
            (send, ret)
        } else {
            let mut split = first_line.split_whitespace();
            let method = split.next().ok_or(server_err!("No method"))?;
            let path = split.next().ok_or(server_err!("No path"))?;
            let _ = split.next().ok_or(server_err!("No protocol"))?;

            let msg = match method.to_uppercase().as_ref() {
                "GET" => handle_get(path),
                "PUT" => handle_put(path),
                "POST" => handle_post(path, body),
                _ => "405 Method Not Allowed".to_string(),
            };

            let send = format!("HTTP/1.1 {msg}");
            let ret = format!("{method} {path}\n{}", send.trim());
            (send, ret)
        };

        println!("Sent: {send}");
        stream.write_all(send.as_bytes())?;
        stream.flush()?;
        Ok(ret)
    }
}

fn extract_content_type(headers: &str) -> &str {
    for line in headers.lines() {
        if line.starts_with("Content-Type:") {
            let (_, value) = line.split_at("Content-Type:".len());
            return value.trim();
        }
    }
    ""
}
