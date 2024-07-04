use super::super::commands::{init, remote, RemoteCommand};
use super::super::plumbing::commands::update_head;
use super::commands::read_tree;
use super::commands::write_index;
use super::fetch::__fetch;
use super::heads::move_head;
use super::work_dir::directify_tree;
use crate::commands::branch;
use crate::io_err;
use crate::protocol::parse_url;
use std::{env, fs, io};
use utils::object::object_db::get_object;
use utils::plumbing::commit::get_commit_root;

// /path/to/repo.git -> repo
fn get_repo_name(repo: &str) -> io::Result<String> {
    if let Some(stripped) = repo.strip_suffix(".git") {
        if let Some(repo) = stripped.split('/').last() {
            // Because repo's path always starts with '/'
            // this should be the only successful case.
            if repo.is_empty() {
                Ok("cloned-repo".to_string())
            } else {
                Ok(repo.to_string())
            }
        } else {
            Err(io_err!("Invalid repo name"))
        }
    } else {
        Err(io_err!("Invalid repo name"))
    }
}

pub fn __clone(url: &str) -> io::Result<()> {
    let (_, repo) = parse_url(url)?;
    let repo = get_repo_name(&repo)?;

    // Check if the directory exists
    if fs::metadata(&repo).is_ok() {
        return Err(io_err!("Repo already exists"));
    }

    // Create the directory if it doesn't exist.
    fs::create_dir(&repo)?;

    // Move cwd.
    let mut cwd = env::current_dir()?;
    cwd.push(&repo);
    env::set_current_dir(cwd)?;

    // Initialize a git repo.
    init(".")?;

    // Add the new remote.
    let name = "origin".to_string();
    let url = url.to_string();
    remote(RemoteCommand::Add { name, url })?;

    // Bring objects and references.
    let head = __fetch("origin")?;

    println!("HEAD: {:?}", head);

    // Update head and create the main branch.
    let reference = if let Some(head) = &head.1 {
        move_head(head)?;
        head.split('/').last().ok_or(io_err!("Invalid head"))?
    } else {
        "master"
    };

    println!("reference: {reference}");

    // We need the head to have a hash
    // if not present return an error.
    let head = head.0.ok_or(io_err!("No head found"))?;
    update_head(&head)?;
    branch(Some(reference.to_string()))?;
    println!("head: {head}");

    // Get tree hash from commit.
    let (_, _, head_commit) = get_object(&head)?;
    let cur_tree_root = get_commit_root(&head_commit)?;
    directify_tree(&cur_tree_root, ".")?;

    // Write index.
    let index = read_tree(&cur_tree_root, "")?;
    write_index(index)?;
    Ok(())
}
