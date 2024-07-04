//! Git Client
//!
//! Models the Git Client which is responsible for sending requests to the Git Server
use super::plumbing::commit::get_parent_commits;
use crate::{io_err, DEFAULT_GIT_PORT};
use std::{
    fs,
    io::{self, BufRead, BufReader, Read, Write},
    net::TcpStream,
};
use utils::object::object_db::get_object;
use utils::package::pack::Pack;

pub fn parse_url(url: &str) -> io::Result<(String, String)> {
    // git://127.0.0.1:8080/path/to/repo.git
    // 0: 127.0.0.1:8080
    // 1: /path/to/repo.git

    if let Some(stripped) = url.strip_prefix("git://") {
        let mut split = stripped.split('/');
        let mut link = split.next().ok_or(io_err!("Invalid url"))?.to_string();
        let mut repo = split.collect::<Vec<&str>>().join("/");
        repo.insert(0, '/');

        // If port isn't specified the use the default git port.
        if !link.contains(':') {
            link.push_str(&format!(":{}", DEFAULT_GIT_PORT));
        }

        Ok((link, repo))
    } else {
        Err(io_err!("Invalid url"))
    }
}

pub fn connect_to_server(protocol: &str, link: &str, repo: &str) -> io::Result<TcpStream> {
    let mut transmiter = TcpStream::connect(link)?;

    let mut send = protocol.to_string() + " " + repo + "\0";
    send.insert_str(0, format!("{:04x}", send.len() + 4).as_str());
    transmiter.write_all(send.as_bytes())?;

    Ok(transmiter)
}

// Returns a have line in the format pkt.
#[allow(dead_code)]
fn have_line(obj_id: &str) -> String {
    to_pkt_line_format("have", obj_id)
}

/// Sends the hashes of the objects the client has in it's object database.
#[allow(dead_code)]
pub fn send_have_lines(mut transmiter: &TcpStream) -> io::Result<()> {
    for entry in fs::read_dir(".git/objects")?.flatten() {
        let dir_name = entry.file_name().to_string_lossy().to_string();

        // If len == 2 => dir_name is a directory with objects.
        if dir_name.len() == 2 {
            let path = entry.path();
            for entry in fs::read_dir(path)?.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                let sha = format!("{dir_name}{file_name}");
                transmiter.write_all(have_line(&sha).as_bytes())?;
            }
        }
    }

    Ok(())
}

pub fn send_want_lines(refs: &[(String, String)], transmiter: &TcpStream) -> io::Result<usize> {
    let mut i = 0;
    for reference in refs {
        if get_object(&reference.0).is_err() {
            match i {
                0 => send_first_want_line(&reference.0, transmiter)?,
                _ => send_want_line(&reference.0, transmiter)?,
            }

            i += 1;
        }
    }

    Ok(i)
}

/// Sends the want lines to the server, if the client doesn't have the parents of
/// a commit then this commit is marked as a shallow line to be sent after.
pub fn send_want_lines_with_shallows(
    refs: &[(String, String)],
    transmiter: &TcpStream,
) -> io::Result<(Vec<String>, usize)> {
    let mut i = 0;
    let mut shallow_lines = vec![];
    while let Some(reference) = refs.get(i) {
        if has_parents_of_commit(&reference.0)? {
            match i {
                0 => send_first_want_line(&reference.0, transmiter)?,
                _ => send_want_line(&reference.0, transmiter)?,
            }

            i += 1;
        } else {
            // Mark it as shallow line.
            let shallow = shallow_line(&reference.0)?;
            shallow_lines.push(shallow);
        }
    }

    Ok((shallow_lines, i))
}

/// Check if has that obj_id and if it has its parents commits
pub fn has_parents_of_commit(obj_id: &str) -> io::Result<bool> {
    let (_, _, data) = get_object(obj_id)?;

    let parents = match get_parent_commits(&data) {
        Some(parents) => parents,
        None => return Ok(false),
    };

    for parent in parents {
        if get_object(&parent).is_err() {
            return Ok(false);
        }
    }

    Ok(true)
}

/// The client MUST write all obj-ids which it only has shallow copies of
/// (meaning that it does not have the parents of a commit)
/// as shallow lines so that the server is aware of the limitations of the client’s history.
fn shallow_line(obj_id: &str) -> io::Result<String> {
    let content = format!("{}\n", obj_id);
    let want_line = to_pkt_line_format("shallow", &content);
    Ok(want_line)
}

/// Sends the shallow lines to the server.
pub fn send_shallow_lines(lines: Vec<String>, mut transmiter: &TcpStream) -> io::Result<()> {
    for line in &lines {
        transmiter.write_all(line.to_owned().as_bytes())?;
    }

    Ok(())
}

/// Sends the max depth for shallow commit objects.
pub fn send_depth_line(n: usize, mut transmiter: &TcpStream) -> io::Result<()> {
    let content = format!("depth {n}");
    let want_line = to_pkt_line_format("deepen", &content);
    transmiter.write_all(want_line.as_bytes())?;
    Ok(())
}

/// Process the first line of references.
pub fn process_first_line(reference: &str) -> (String, String) {
    let mut ref_name = "";
    // Check for symref.
    if let Some(i) = reference.find("symref=") {
        let ofs = "symref=".len();

        // Find the end of the reference name.
        let end = reference[i + ofs..]
            .find(' ')
            .unwrap_or(reference[i + ofs..].len());

        ref_name = &reference[i + ofs..i + ofs + end];
    }

    // XXXX<hash> refs/heads/*
    let whsp_split = reference.split_whitespace().collect::<Vec<&str>>();
    let obj_id = whsp_split[0][4..].to_string();
    if ref_name.is_empty() {
        ref_name = &whsp_split[1][..whsp_split[1].find('\0').unwrap_or(whsp_split[1].len())];
    }

    (obj_id, ref_name.to_string())
}

pub fn parse_references(refs: Vec<u8>) -> io::Result<Vec<(String, String)>> {
    let refs = String::from_utf8_lossy(&refs);
    let refs = refs.lines().collect::<Vec<&str>>();
    let mut want_refs = vec![];

    // Parse first line separately.
    let head_ref = process_first_line(refs[0]);
    want_refs.push(head_ref);

    // Parse the rest.
    for reference in refs[1..].iter() {
        if *reference == "0000" {
            // end of references.
            break;
        }

        // XXXX<hash> refs/heads/*
        let whsp_split = reference.split_whitespace().collect::<Vec<&str>>();

        // Save in want_refs.
        let obj_id = whsp_split[0][4..].to_string();
        let ref_name = whsp_split[1].to_string();
        want_refs.push((obj_id, ref_name));
    }

    Ok(want_refs)
}

pub fn get_current_refs(path: &str) -> io::Result<Vec<(String, String)>> {
    let entries = fs::read_dir(path)?;
    let mut refs = vec![];

    for entry in entries.flatten() {
        let file_name_os_str = entry.file_name();
        let file_name = match file_name_os_str.to_str() {
            None => return Err(io_err!("Invalid file name")),
            Some(name) => name,
        };

        let file_path = format!("{path}/{file_name}");
        if entry.file_type()?.is_dir() {
            refs.append(&mut get_current_refs(&file_path)?);
        } else {
            let content = fs::read_to_string(entry.path())?.trim().to_string();
            if let Some(stripped) = file_path.strip_prefix(".git/") {
                refs.push((content, stripped.to_string()));
            }
        }
    }

    Ok(refs)
}

/// Returns a Vec of tuples (HASH, REFERENCE) of the current
/// references in the repository.
pub fn get_refs_vec() -> io::Result<Vec<(String, String)>> {
    let mut refs = get_current_refs(".git/refs")?;
    refs.sort_by(|a, b| a.1.cmp(&b.1));
    Ok(refs)
}

/// Returns the current references of the repository in
/// pkt-line format.
pub fn get_refs_fmt() -> io::Result<String> {
    let refs = get_refs_vec()?;
    let mut ret = String::new();

    for (hash, path) in &refs {
        ret.push_str(&to_pkt_line_format(hash, path));
    }

    Ok(ret)
}

/// Processes the ACK lines in the server response.
pub fn process_ack_line(transmiter: &mut TcpStream) -> io::Result<String> {
    let mut reader = BufReader::new(transmiter);

    // Read first ACK.
    let mut line = String::new();
    reader.read_line(&mut line)?;

    Ok(line)
}

/// Process pack file sent by server
pub fn process_pack_file(response: Vec<u8>) -> io::Result<()> {
    let reader = BufReader::with_capacity(response.len(), response.as_slice());
    Pack::unpack(reader)
}

/// Convert to pkt line format
/// Must indicate the size in bytes of the content and the content
/// can be binary data
pub fn to_pkt_line_format(request_command: &str, content: &str) -> String {
    let res = if content.is_empty() {
        request_command.to_string()
    } else {
        format!("{request_command} {content}\n")
    };
    // Format "{length in hexadecimal}{content: command + repository name}"
    format!("{:04x}{}", res.len() + 4, res)
}

/// Send the first line of references HEAD, write the capabilities
fn send_first_want_line(first_reference: &str, mut transmiter: &TcpStream) -> io::Result<()> {
    let content = format!("{first_reference}\0\n"); // Capabilities.
    let first_want = to_pkt_line_format("want", &content);
    transmiter.write_all(first_want.as_bytes())?;
    Ok(())
}

// Send a want line
fn send_want_line(obj_id: &str, mut transmiter: &TcpStream) -> io::Result<()> {
    let content = format!("{}\n", obj_id);
    let want_line = to_pkt_line_format("want", &content);
    transmiter.write_all(want_line.as_bytes())?;
    Ok(())
}

/// Get response from sever
pub fn get_response<R: Read>(transmiter: &mut R) -> Vec<u8> {
    // Buffer for the Daemon response
    let mut buffer = [0; 1024];
    // Read the response from Git Daemon
    let mut references = Vec::new();

    while let Ok(n) = transmiter.read(&mut buffer) {
        if n == 0 {
            break;
        }

        references.extend_from_slice(&buffer[..n]);

        // End of response
        if references.ends_with(b"0000") {
            break;
        }
    }

    references
}
