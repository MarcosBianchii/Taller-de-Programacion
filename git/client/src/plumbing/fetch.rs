use super::super::config_file::config::Config;
use crate::config_file::config_entry::ConfigEntry;
use crate::io_err;
use crate::protocol::*;
use std::{
    fs::{self, File},
    io::{self, Write},
};

const ZERO_ID: &str = "0000000000000000000000000000000000000000";

// Creates the files for the tags that the client doesn't have.
fn create_unexisting_tags<'a>(
    tags: impl Iterator<Item = &'a (String, String)>,
) -> io::Result<usize> {
    let mut created = 0;
    for (hash, ref_path) in tags {
        print!("tag: {ref_path}");

        if ref_path.contains("^{}") {
            println!(" (skipped)");
            continue;
        }

        println!();
        let path = ".git/".to_string() + ref_path;
        if fs::metadata(&path).is_err() {
            let mut file = File::create(&path)?;
            file.write_all(hash.as_bytes())?;
            created += 1;
        }
    }

    Ok(created)
}

pub fn add_to_refs(path: &str, hash: &str) -> io::Result<()> {
    // Create path till file if it doesn't exist yet.
    let path_split = path.split('/').collect::<Vec<&str>>();
    let path_till_file = path_split[..path_split.len() - 1].join("/");
    fs::create_dir_all(format!(".git/{path_till_file}"))?;

    // Write to file.
    let mut file = File::create(format!(".git/{path}"))?;
    let hash = hash.to_string() + "\n";
    file.write_all(hash.as_bytes())
}

#[allow(unreachable_code)]
pub fn __fetch(remote: &str) -> io::Result<(Option<String>, Option<String>)> {
    // Get the remote's url.
    let config = Config::read()?;
    let url = match config.get(remote) {
        Some(ConfigEntry::Remote { url, .. }) => url,
        _ => return Err(io_err!("Remote not found")),
    };

    // Generate TCP with own IP Address.
    let (link, repo) = parse_url(url)?;
    let mut transmiter = connect_to_server("git-upload-pack", &link, &repo)?;

    // Read the references from Git Daemon
    let refs = get_response(&mut transmiter);

    println!("refs: {}", String::from_utf8_lossy(&refs));

    // Process references received.
    let mut refs = parse_references(refs)?;

    println!("{:?}", refs);

    if refs[0].0 == ZERO_ID {
        transmiter.write_all(b"0000")?;
        return Err(io_err!("No refs received"));
    }

    // Save the first which has the HEAD.
    let mut ret = (None, None);
    let head = refs.remove(0);
    println!("head de fetch: {:?}", head);
    if head.1.contains("HEAD") {
        ret.0 = Some(head.0);

        if let Some(stripped) = head.1.strip_prefix("HEAD:") {
            // ref is a symref of HEAD.
            ret.1 = Some(stripped.to_string());
        } else {
            // ref is HEAD.
            ret.1 = Some("HEAD".to_string());
        }
    } else {
        // ref isn't HEAD.
        refs.insert(0, head);
    }

    // Create the tags the client doesn't have.
    let tags_created =
        create_unexisting_tags(refs.iter().filter(|(_, path)| path.contains("refs/tags")))?;

    // Send want lines.
    let sent = send_want_lines(&refs, &transmiter)?;
    transmiter.write_all(b"0000")?;

    // If no want lines were sent, then the client is up to date.
    if sent == 0 && tags_created == 0 {
        return Err(io_err!("Already up to date"));
    } else if sent == 0 {
        return Err(io_err!("No new commits"));
    }

    // Send have lines and done.
    send_have_lines(&transmiter)?;
    let done_line = to_pkt_line_format("done", "");
    transmiter.write_all(done_line.as_bytes())?;

    // Process ack lines.
    process_ack_line(&mut transmiter)?;

    // Receive pack-file from server and
    // write the objects to the objects db.
    let pack_file = get_response(&mut transmiter);
    process_pack_file(pack_file)?;

    // Add refs to .git directory.
    let mut config = Config::read()?;
    for (hash, path) in refs {
        // refs/heads/branch_name

        if path.contains("refs/tags") {
            continue;
        }

        // Add to remotes.
        let path = path.replace("heads", &format!("remotes/{remote}"));
        add_to_refs(&path, &hash)?;

        // Add it to config.
        let path_split = path.split('/').collect::<Vec<&str>>();
        if let Some(branch_name) = path_split.last() {
            config.add(ConfigEntry::new_branch(branch_name, remote));
        }
    }

    config.write()?;
    Ok(ret)
}
