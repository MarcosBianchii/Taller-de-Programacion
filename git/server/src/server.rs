use crate::handle_error;
use crate::pool::threadpool::ThreadPool;
use crate::server_err;
use crate::LogMsgStatus;
use crate::ServerError;
use std::sync::mpsc::Sender;
use std::{
    fs::{self, File, OpenOptions},
    io::{self, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{mpsc, Arc, Mutex},
    thread,
};
use utils::object::object_db::get_object_with_offset;
use utils::package::pack::Pack;
use utils::*;

pub const THREAD_POOL_SIZE: usize = 10;
const ZERO_ID: &str = "0000000000000000000000000000000000000000";

fn log_cmd(reader: mpsc::Receiver<String>) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("cmdlog.txt")?;

    while let Ok(line) = reader.recv() {
        file.write_all(line.as_bytes())?;
    }

    Ok(())
}

fn log_connection(execution_status: LogMsgStatus, sender: Arc<Mutex<Sender<String>>>) {
    if let Ok(guard) = sender.lock() {
        let time = chrono::Local::now().format("%d-%m-%Y %H:%M:%S");
        let log_msg = match execution_status {
            LogMsgStatus::CorrectExecution(protocol_and_repo) => {
                format!(
                    "{time}    {protocol_and_repo}    {}\n",
                    String::from("Command executed as expected")
                )
            }
            LogMsgStatus::ErrOnExecution(err_msg) => {
                format!("[Error] {time}  {err_msg}\n")
            }
        };

        let _ = guard.send(log_msg);
    }
}
// Initial Server struct implementation.
pub struct Server {
    address: String,
}

impl Server {
    pub fn new(address: String) -> Result<Self, ServerError> {
        Ok(Self { address })
    }

    fn parse_client_packtype<R: Read>(reader: &mut R) -> Result<Vec<u8>, ServerError> {
        let mut size = [0; 4];
        reader.read_exact(&mut size)?;

        // size has the length of the content in hexadecimal.
        let size = String::from_utf8_lossy(&size);

        // Convert hexadecimal to decimal.
        let size = match u32::from_str_radix(&size, 16) {
            Ok(size) => size as usize - 4,
            Err(_) => return Err(server_err!("Invalid hexadecimal size")),
        };

        // Read content from stream.
        let mut content = vec![0; size];
        reader.read_exact(&mut content)?;

        Ok(content)
    }

    // Should detect if a request is type upload-pack or receive-pack
    // And call upload_pack function or receive_pack function in function of that
    fn handle_connection(mut transmiter: TcpStream) -> Result<String, ServerError> {
        let response = Server::parse_client_packtype(&mut transmiter)?;
        let response = String::from_utf8_lossy(&response);
        let mut split = response.split(' ');

        if let (Some(protocol), Some(repo)) = (split.next(), split.next()) {
            let mut repo = repo[1..repo.find('\0').unwrap_or(repo.len())].trim();
            if repo.is_empty() {
                repo = ".git";
            }

            // Execute request.
            let res: Result<String, ServerError> = match protocol {
                "git-upload-pack" => match Self::upload_pack(transmiter, repo) {
                    Ok(()) => Ok(format!("{}   {}", "git-upload-pack", repo)),
                    Err(serv_err) => Err(serv_err),
                },
                "git-receive-pack" => match Self::receive_pack(transmiter, repo) {
                    Ok(()) => Ok(format!("{}   {}", "git-receive-pack", repo)),
                    Err(serv_err) => Err(serv_err),
                },
                _ => Err(server_err!("Invalid protocol type")),
            };

            res
        } else {
            Err(server_err!("Invalid request"))
        }
    }

    /// Binds to self's address and listens for requests.
    ///
    /// Upon success listens to the port waiting for a client to stablish a connection.
    /// Every message received from clients is assigned to separate threads.
    ///
    pub fn run(&self) -> Result<(), ServerError> {
        let listener = TcpListener::bind(&self.address)?;
        let pool = ThreadPool::build(THREAD_POOL_SIZE)?;

        // Set cmd logging.
        let (rv, rc) = mpsc::channel();
        thread::spawn(move || log_cmd(rc));
        let rv = Arc::new(Mutex::new(rv));

        println!("Ready to rumble!");

        for stream in listener.incoming() {
            let stream = match stream {
                Ok(stream) => stream,
                Err(_) => {
                    println!("ERROR: Stream connection was unsuccessful");
                    continue;
                }
            };

            let sender = rv.clone();
            let res = pool.execute(|| {
                let log_msg_status = match Self::handle_connection(stream) {
                    Ok(protocol) => LogMsgStatus::CorrectExecution(protocol),
                    Err(err) => LogMsgStatus::ErrOnExecution(err.to_string()),
                };

                log_connection(log_msg_status, sender);
            });

            if let Err(error) = res {
                handle_error(error);
            }
        }

        Ok(())
    }

    // Formats the references into pkt-line format.
    fn fmt_refs(refs: Vec<(String, String)>) -> String {
        refs.into_iter()
            .map(|(name, hash)| Self::to_pkt_line_format(&hash, &name))
            .collect::<Vec<String>>()
            .join("")
    }

    fn get_tag_peel_hash(data: &[u8]) -> String {
        let mut sub_hash = "".to_string();

        for line in data.split(|&c| c == b'\n') {
            let line = String::from_utf8_lossy(line);
            if let Some(stripped) = line.strip_prefix("object ") {
                sub_hash = stripped.trim().to_string();
                break;
            }
        }

        sub_hash
    }

    // Returns the references of the repository.
    fn get_all_refs(repo: &str) -> Result<Vec<(String, String)>, ServerError> {
        let refs = get_refs_from_with_prefix(
            &(repo.to_string() + "/refs/heads"),
            &(repo.to_string() + "/"),
        )?;

        let mut sv_refs = Vec::from_iter(refs);
        let tags = get_refs_from_with_prefix(
            &(repo.to_string() + "/refs/tags"),
            &(repo.to_string() + "/"),
        )?;

        // Peel tags.
        let mut complete_tags = vec![];
        for (ref_path, hash) in tags.iter() {
            let (mut otype, _, mut data) = get_object_with_offset(hash, repo)?;
            complete_tags.push((ref_path.clone(), hash.clone()));

            while otype == "tag" {
                let peel_hash = Self::get_tag_peel_hash(&data);
                (otype, _, data) = get_object_with_offset(&peel_hash, repo)?;
                complete_tags.push((ref_path.clone() + "^{}", peel_hash));
            }
        }

        sv_refs.extend(complete_tags);
        sv_refs.sort_by_key(|(k, _)| k.to_owned());
        Ok(sv_refs)
    }

    /// The process invoked for the Git Client for fetching data from Git Server
    /// Using git transport protocol
    /// Should respond with list of all references the repository has
    fn upload_pack<T: Write + Read>(mut transmiter: T, repo: &str) -> Result<(), ServerError> {
        let mut sent_head = false;

        if let Some(head) = get_head_with_offset(repo) {
            let headfile = fs::read_to_string(format!("{repo}/HEAD"))?;
            if let Some(stripped) = headfile.strip_prefix("ref: ") {
                let headref = stripped.trim();
                let head_line =
                    Self::to_pkt_line_format(&head, &format!("HEAD\0symref=HEAD:{headref}"));

                transmiter.write_all(head_line.as_bytes())?;
                sent_head = true;
            }
        }

        Self::send_all_references(&mut transmiter, repo, sent_head)?;

        // Get want lines.
        let want_lines = get_want_lines(&mut transmiter)?;
        if want_lines.is_empty() {
            return Err(server_err!("Client is up to date"));
        }

        let have_lines = get_have_lines(&mut transmiter)?;
        println!("want_lines: {want_lines:?}");
        println!("have_lines: {have_lines:?}");

        // Send NAK and packfile.
        transmiter.write_all(b"0008NAK\n")?;
        let pack = Pack::from_with_offset(want_lines, repo)?.as_bytes()?;
        transmiter.write_all(&pack)?;
        Ok(())
    }

    // Updates a single ref.
    fn update_ref(path: &str, _: &str, new: &str, repo: &str) -> Result<(), ServerError> {
        let path = path.replace('\0', "");
        if new == ZERO_ID {
            // Delete reference.
            let _ = fs::remove_file(format!("{repo}/{path}"));
        } else {
            // Create path till file if it doesn't exist yet.
            let path_split = path.split('/').collect::<Vec<&str>>();
            let path_till_file = path_split[..path_split.len() - 1].join("/");

            fs::create_dir_all(format!("{repo}/{path_till_file}"))?;

            // Write to file.
            println!("path: {path:?}");
            let mut file = File::create(format!("{repo}/{path}"))?;
            file.write_all(new.as_bytes())?;
        }

        Ok(())
    }

    // Updates the references with the ones the client sends.
    fn update_refs<T: Write + Read>(transmiter: &mut T, repo: &str) -> Result<(), ServerError> {
        let mut buf = [0; 4];

        loop {
            transmiter.read_exact(&mut buf)?;
            if &buf == b"0000" {
                // End of references.
                break;
            }

            // Size has the length of the content in hexadecimal.
            let size = String::from_utf8_lossy(&buf);
            let size = match u32::from_str_radix(&size, 16) {
                Ok(size) => size as usize - 4,
                Err(_) => return Err(server_err!("Invalid hexadecimal size")),
            };

            // Read content from stream.
            let mut content = vec![0; size];
            transmiter.read_exact(&mut content)?;

            let content = String::from_utf8_lossy(&content);
            let content = content.trim();
            let mut split = content.split(' ');

            // Update reference.
            match (split.next(), split.next(), split.next()) {
                (Some(old), Some(new), Some(name)) => {
                    println!("-------------");
                    Self::update_ref(name, old, new, repo)?;
                }

                _ => return Err(server_err!("Invalid reference")),
            }
        }

        Ok(())
    }

    /// The process invoked for the Git Client to send data to Git Server
    /// Should respond with list of all references the repository has
    fn receive_pack<T: Write + Read>(mut transmiter: T, repo: &str) -> Result<(), ServerError> {
        println!("0");

        Self::send_all_references(&mut transmiter, repo, false)?;

        println!("1");

        // Read the references from client and parse them.
        Self::update_refs(&mut transmiter, repo)?;

        println!("2");

        // Read Packfile.
        let reader = BufReader::new(&mut transmiter);
        Pack::unpack_with_offset(reader, repo)?;

        println!("3");

        Ok(())
    }

    /// Convert to pkt line format
    /// Must indicate the size in bytes of the content and the content
    /// can be binary data
    fn to_pkt_line_format(request_command: &str, content: &str) -> String {
        let res: String = if content.is_empty() {
            request_command.to_string()
        } else {
            format!("{request_command} {content}\n")
        };
        // Format "{length in hexadecimal}{content: command + repository name}"
        format!("{:04x}{}", res.len() + 4, res)
    }

    /// Return a pkt-line stream with all references and current value
    /// Is used to initialize the Reference Discovery process between Client and Server
    fn send_all_references<W: Write>(
        writer: &mut W,
        repo: &str,
        sent_head: bool,
    ) -> Result<(), ServerError> {
        let mut refs = Self::get_all_refs(repo)?;

        // List capabilities in first line if HEAD wasn't sent.
        if let Some(first) = refs.first_mut() {
            if !sent_head {
                first.1 = first.1.replace('\n', "\0");
            }
        }

        // If list is empty then send the ZERO_ID.
        if refs.is_empty() {
            refs.push(("capabilities^{}\0".to_string(), ZERO_ID.to_string()));
        }

        // Send with flush pkt.
        let references = Self::fmt_refs(refs);
        writer.write_all(references.as_bytes())?;
        writer.write_all(b"0000")?;
        Ok(())
    }
}
