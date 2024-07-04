use crate::{
    config_file::{config::Config, config_entry::ConfigEntry},
    io_err,
    protocol::{connect_to_server, parse_url},
};
use std::io::{self, Write};
use utils::package::pack::Pack;
use utils::*;

use super::merge::is_ancestor;

const ZERO_ID: &str = "0000000000000000000000000000000000000000";

#[derive(Debug)]
pub struct SendEntry {
    old_id: String,
    new_id: String,
    reference_path: String,
}

impl SendEntry {
    pub fn new(old_id: String, new_id: String, reference_path: String) -> Self {
        Self {
            old_id,
            new_id,
            reference_path,
        }
    }

    pub fn to_pkt_format(&self) -> String {
        let res = format!("{} {} {}\n", self.old_id, self.new_id, self.reference_path);
        format!("{:04x}{}", res.len() + 4, res)
    }
}

/// returns a vector of tuples (local_obj_id, remote_branch_path) of the outdated references
fn get_references_to_send(
    remote_references: &mut Vec<(String, String)>,
) -> io::Result<Vec<SendEntry>> {
    let mut outdated_references: Vec<SendEntry> = Vec::new();

    // get a hashmap of current branches and its respectives commits (k: branch_path, v: obj_id)
    let mut current_refs = get_local_refs()?;
    current_refs.extend(get_tags()?);

    for (remote_obj_id, remote_branch_path) in remote_references {
        // If the hashes of the local branch and the remote branch are not equal, we have to update/remove the remote branch
        if let Some(local_obj_id) = current_refs.get(&remote_branch_path.replace('\0', "")) {
            // Primera iteracion: ya se que la rama local es padre de la rama remota, asi que creamos un update
            if local_obj_id != remote_obj_id {
                // we push the obj_id of the commit that the remote branch should be updated to (aka, the local commit )
                if remote_obj_id == ZERO_ID {
                    println!("Entro remote_obj_id == ZERO_ID {remote_branch_path}");
                    outdated_references.push(SendEntry::new(
                        ZERO_ID.to_string(),
                        local_obj_id.to_string(),
                        remote_branch_path.to_string(),
                    ));
                } else if is_ancestor(remote_obj_id, local_obj_id)? {
                    println!("Entro otro {remote_branch_path}");
                    outdated_references.push(SendEntry::new(
                        remote_obj_id.to_string(),
                        local_obj_id.to_string(),
                        remote_branch_path.to_string(),
                    ));
                }
            }

            // we take the branch out of the hashmap, so we can know which branches are not in the remote repo
            current_refs.remove(&remote_branch_path.replace('\0', ""));
        }
    }

    // the remaining branches in the hashmap are the ones that are not in the remote repo, so we have to create them
    for (local_branch_path, local_obj_id) in current_refs {
        println!("Entro tercero {local_branch_path}");
        outdated_references.push(SendEntry::new(
            ZERO_ID.to_string(),
            local_obj_id,
            local_branch_path,
        ));
    }

    Ok(outdated_references)
}

pub fn parse_references(refs: Vec<u8>) -> io::Result<Vec<(String, String)>> {
    let refs = String::from_utf8_lossy(&refs);
    let mut refs = refs.lines().collect::<Vec<&str>>();
    let mut want_refs = vec![];

    // Parse first line separately.
    let first_line = refs.remove(0);
    if first_line.contains(ZERO_ID) {
        want_refs.push((ZERO_ID.to_string(), "refs/heads/master".to_string()));
    } else {
        let whsp_split = first_line.split_whitespace().collect::<Vec<&str>>();
        let obj_id = whsp_split[0][4..].to_string();
        let ref_name =
            whsp_split[1][..whsp_split[1].find('\0').unwrap_or(whsp_split[1].len())].to_string();
        want_refs.push((obj_id, ref_name));
    }

    // Parse the rest.
    for reference in refs.iter() {
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

pub fn __push(remote: &str) -> io::Result<()> {
    // Get the remote's url.
    let config = Config::read()?;
    let url = match config.get(remote) {
        Some(ConfigEntry::Remote { url, .. }) => url,
        _ => return Err(io_err!("Remote not found")),
    };

    let (link, repo) = parse_url(url)?;
    let mut transmiter = connect_to_server("git-receive-pack", &link, &repo)?;

    // Read the references from server and parse them.
    let references = get_response(&mut transmiter);
    let mut refs = parse_references(references)?;

    println!("refs: {:?}", refs);

    // Get local refs and compare.
    let outdated_refs = get_references_to_send(&mut refs)?;

    // Prepare the references to send.
    let send_refs = outdated_refs
        .iter()
        .fold(String::new(), |acc, x| acc + &x.to_pkt_format())
        + "0000";

    println!("send_refs: {send_refs}");

    let hashes = outdated_refs.into_iter().map(|p| p.new_id).collect();
    let pack = Pack::from(hashes)?.as_bytes()?;

    // Send the references and the pack.
    transmiter.write_all(send_refs.as_bytes())?;
    transmiter.write_all(&pack)?;

    Ok(())
}
