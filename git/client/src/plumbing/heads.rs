use std::{
    fs::{self, File},
    io::{self, Read, Write},
};

use crate::io_err;

use super::commands::get_branch;

enum Head<E> {
    Detached(String),
    Refered(String),
    Err(E),
}

// Opens the current file containing the last commit pointed by HEAD.
fn cur_branch_file_path<R: Read>(mut head: R) -> Head<io::Error> {
    let mut branch = String::new();

    // If there is an error reading HEAD, return it.
    if let Err(e) = head.read_to_string(&mut branch).map_err(Head::Err) {
        return e;
    }

    let branch = branch.replace('\n', "");
    if let Some(stripped) = branch.strip_prefix("ref: ") {
        // HEAD is checked out.
        Head::Refered(".git/".to_string() + stripped)
    } else {
        // HEAD is detached.
        let hash = branch;
        Head::Detached(hash)
    }
}

/// Underlying imlementation of get_head.
pub fn __get_head_commit<R: Read>(head: R) -> io::Result<String> {
    match cur_branch_file_path(head) {
        Head::Refered(branch) => {
            let s = fs::read_to_string(branch)?;
            Ok(s.replace('\n', ""))
        }
        Head::Detached(hash) => Ok(hash),
        Head::Err(e) => Err(e),
    }
}

/// Underlying implementation of update_head.
/// Updates the commit that HEAD file points to
/// with new commit.
/// It doesn't change the current branch, it just changes the commit.
pub fn __update_head_commit(hash_commit: &str) -> io::Result<()> {
    // update HEAD file to point to branch
    let head = File::open(".git/HEAD")?;

    match cur_branch_file_path(head) {
        Head::Refered(branch) => {
            let mut branch = File::create(branch)?;
            branch.write_all(hash_commit.as_bytes())?;
            branch.write_all(b"\n")?;
            Ok(())
        }

        Head::Detached(hash) => {
            let mut head = File::create(".git/HEAD")?;
            head.write_all(hash.as_bytes())?;
            head.write_all(b"\n")?;
            Ok(())
        }

        Head::Err(e) => Err(e),
    }
}

/// Takes as parameter a reference like: heads/master, remotes/origin/master, etc.
/// It updates the HEAD file to point to that branch or reference.
pub fn update_head_reference(reference: &str) -> io::Result<()> {
    get_branch(reference).ok_or(io_err!("Invalid reference"))?;

    // Point to that reference
    let mut head = File::create(".git/HEAD")?;
    head.write_all(format!("ref: refs/heads/{reference}\n").as_bytes())?;
    Ok(())
}

/// Underlying implementation of get_branch.
pub fn __get_branch<R: Read>(mut branch: R) -> io::Result<String> {
    let mut hash = String::new();
    branch.read_to_string(&mut hash)?;
    Ok(hash.trim().to_string())
}

/// Returns the name of the current branch.
/// If HEAD is detached, returns the hash of the commit.
pub fn get_head_name() -> io::Result<String> {
    match cur_branch_file_path(File::open(".git/HEAD")?) {
        Head::Refered(_) => {
            let branch = fs::read_to_string(".git/HEAD")?;
            // get the name of the branch
            let branch = branch.replace('\n', "");
            let branch = branch
                .strip_prefix("ref: refs/heads/")
                .ok_or(io_err!("Invalid state for HEAD"))?;
            Ok(branch.to_string())
        }

        Head::Detached(_) => Err(io_err!("HEAD is detached")),
        Head::Err(e) => Err(e),
    }
}

/// Writes the content in reference to HEAD file
/// in the format: `ref: <reference>\n`.
pub fn move_head(reference: &str) -> io::Result<()> {
    let mut file = File::create(".git/HEAD")?;
    file.write_all(b"ref: ")?;
    file.write_all(reference.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(())
}
