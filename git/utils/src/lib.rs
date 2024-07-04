pub mod index_file;
pub mod object;
pub mod package;
pub mod plumbing;
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, Read, Write},
    net::TcpStream,
};

use index_file::{index::Index, index_entry::IndexEntry};
use object::object_db::get_object_from_repo;
use plumbing::{
    commit::get_parent_commits,
    hash_object::__hash_object,
    ls_tree::{ls_tree_from_repo, parse_ls_tree_entry},
};

#[macro_export]
macro_rules! io_err {
    ($msg:literal) => {
        io::Error::new(
            io::ErrorKind::Other,
            format!("{} - {}:{}", $msg, file!(), line!()),
        )
    };
}

#[macro_export]
macro_rules! log_ok {
    ($msg:literal) => {
        if let Err(err) = log_command($msg.to_string(), LogMsgStatus::CorrectExecution) {
            println!("Logging failed with error : {}", err);
        }
    };
}

#[macro_export]
macro_rules! log_err {
    ($cmd:literal, $err:expr) => {
        if let Err(err) = log_command(
            $cmd.to_string(),
            LogMsgStatus::ErrOnExecution($err.to_string()),
        ) {
            println!("Logging failed with error : {}", err);
        }
    };
}

pub fn send_a_flush_pkt(mut transmiter: &TcpStream) -> io::Result<()> {
    transmiter.write_all(b"0000")
}

/// Returns a hashmap of the current branches and its respectives commits (k: branch_path, v: obj_id)
pub fn get_current_refs() -> io::Result<HashMap<String, String>> {
    get_refs_from_with_prefix(".git/refs", ".git/")
}

pub fn get_local_refs() -> io::Result<HashMap<String, String>> {
    get_refs_from_with_prefix(".git/refs/heads", ".git/")
}

pub fn get_remote_refs() -> io::Result<HashMap<String, String>> {
    get_refs_from_with_prefix(".git/refs/remotes", ".git/")
}

// Returns a hashmap of the current branches and its respective commits
// with a certain offset of where the .git folder is.
pub fn get_refs_from_with_prefix(path: &str, prefix: &str) -> io::Result<HashMap<String, String>> {
    let entries = fs::read_dir(path)?;
    let mut refs = HashMap::new();

    for entry in entries.flatten() {
        let file_name_os_str = entry.file_name();
        let file_name = match file_name_os_str.to_str() {
            None => return Err(io_err!("Invalid file name")),
            Some(name) => name,
        };

        let file_path = format!("{path}/{file_name}");

        if entry.file_type()?.is_dir() {
            refs.extend(get_refs_from_with_prefix(&file_path, prefix)?.into_iter());
        } else {
            let content = fs::read_to_string(entry.path())?.trim().to_string();
            if let Some(stripped) = file_path.strip_prefix(prefix) {
                refs.insert(stripped.to_string(), content);
            }
        }
    }

    Ok(refs)
}

/// Returns tags in the form of (ref path, hash).
pub fn get_tags() -> io::Result<HashMap<String, String>> {
    get_refs_from_with_prefix(".git/refs/tags", ".git/")
}

/// Returns the hash of the commit object pointed to by HEAD.
pub fn get_head() -> Option<String> {
    get_head_with_offset(".git")
}

/// Returns the hash of the commit object pointet to by HEAD
/// with a certain offset of where the .git folder is.
pub fn get_head_with_offset(offset: &str) -> Option<String> {
    let file = File::open(format!("{offset}/HEAD")).ok()?;
    __get_head_commit_with_offset(file, offset).ok()
}

enum Head<E> {
    Detached(String),
    Refered(String),
    Err(E),
}

fn cur_branch_file_path_with_offset<R: Read>(mut head: R, offset: &str) -> Head<io::Error> {
    let mut branch = String::new();

    // If there is an error reading HEAD, return it.
    if let Err(e) = head.read_to_string(&mut branch).map_err(Head::Err) {
        return e;
    }

    let branch = branch.replace('\n', "");
    if let Some(stripped) = branch.strip_prefix("ref: ") {
        // HEAD is checked out.
        Head::Refered(format!("{offset}/") + stripped)
    } else {
        // HEAD is detached.
        let hash = branch;
        Head::Detached(hash)
    }
}

// Opens the current file containing the last commit pointed by HEAD.
fn cur_branch_file_path<R: Read>(head: R) -> Head<io::Error> {
    cur_branch_file_path_with_offset(head, ".git")
}

pub fn __get_head_commit_with_offset<R: Read>(head: R, offset: &str) -> io::Result<String> {
    match cur_branch_file_path_with_offset(head, offset) {
        Head::Refered(branch) => {
            let s = fs::read_to_string(branch)?;
            Ok(s.replace('\n', ""))
        }
        Head::Detached(hash) => Ok(hash),
        Head::Err(e) => Err(e),
    }
}

/// Underlying imlementation of get_head.
pub fn __get_head_commit<R: Read>(head: R) -> io::Result<String> {
    __get_head_commit_with_offset(head, ".git")
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

/// Underlying implementation of get_branch.
pub fn __get_branch<R: Read>(mut branch: R) -> io::Result<String> {
    let mut hash = String::new();
    branch.read_to_string(&mut hash)?;
    Ok(hash)
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
        if references.ends_with(b"00000009done\n") || references.ends_with(b"0000") {
            break;
        }
    }

    references
}

fn read_line<R: Read>(reader: &mut R) -> io::Result<String> {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf)?;

    // Check if the line is a flush-pkt.
    if &buf == b"0000" {
        println!("EXIT FLUSH");
        return Ok(String::new());
    }

    // size has the length of the content in hexadecimal.
    let size = String::from_utf8_lossy(&buf);

    // Convert hexadecimal to decimal.
    let size = match u32::from_str_radix(&size, 16) {
        Ok(size) => size as usize - 4,
        Err(_) => return Err(io_err!("Invalid hexadecimal size")),
    };

    // Read content from stream.
    let mut content = vec![0; size];
    reader.read_exact(&mut content)?;

    if &content == b"done\n" {
        return Ok(String::new());
    }

    Ok(String::from_utf8_lossy(&content).to_string())
}

/// Get response from sever
pub fn get_want_lines<R: Read>(transmiter: &mut R) -> io::Result<Vec<String>> {
    let first_want = read_line(transmiter)?;
    let mut want_lines = vec![];

    if let Some(stripped) = first_want.strip_prefix("want ") {
        if let Some(i) = stripped.find('\n') {
            let (want, _) = stripped.split_at(i);
            want_lines.push(want.trim().to_string());
        } else {
            return Err(io_err!("Invalid want line"));
        }
    } else {
        return Ok(vec![]);
    }

    loop {
        let line = read_line(transmiter)?;
        println!("line: {:?}, len: {}", line, line.len());
        if let Some(stripped) = line.strip_prefix("want ") {
            want_lines.push(stripped.trim().to_string());
        } else if line.is_empty() {
            // Encountered flush-pkt.
            break;
        }
    }

    Ok(want_lines)
}

pub fn get_have_lines<R: Read>(transmiter: &mut R) -> io::Result<Vec<String>> {
    let mut have_lines = vec![];

    loop {
        let line = read_line(transmiter)?;
        if let Some(stripped) = line.strip_prefix("have ") {
            have_lines.push(stripped.trim().to_string());
        } else if line.is_empty() {
            // Encountered done line.
            break;
        }
    }

    Ok(have_lines)
}

/*
object <hash>
type <type>
tag <tag_name>
tagger <tagger> <time>
<msg>
*/

/// Parses a tag object returning (type, hash)
pub fn parse_tag(data: &[u8]) -> Result<String, &'static str> {
    let mut object = String::new();

    for line in data.split(|&c| c == b'\n') {
        if let Some(hash) = line.strip_prefix(b"object ") {
            object = String::from_utf8_lossy(hash).to_string();
            break;
        }
    }

    if object.is_empty() {
        Err("Invalid tag object")
    } else {
        Ok(object)
    }
}

/// Returns the hash of the commit object pointed to by a branch.
pub fn get_branch_from_repo(name: &str, repo: &str) -> io::Result<String> {
    Ok(fs::read_to_string(format!("{repo}/refs/heads/{name}"))?
        .trim()
        .to_string())
}

fn __is_ancestor_from_repo(
    a: &str,
    b: &str,
    repo: &str,
    steps: &mut HashSet<String>,
) -> io::Result<bool> {
    // Check if we have already visited b.
    if steps.contains(b) {
        return Ok(false);
    }

    // Added b to steps.
    steps.insert(b.to_string());

    // Check if a is b.
    if a == b {
        return Ok(true);
    }

    // Get b's data.
    let (_, _, data) = get_object_from_repo(b, repo)?;
    match get_parent_commits(&data) {
        Some(parents) => {
            for parent in parents {
                if __is_ancestor_from_repo(a, &parent, repo, steps)? {
                    return Ok(true);
                }
            }

            Ok(false)
        }
        None => Ok(false),
    }
}

// Returns true if a is ancestor of b within a given repo.
pub fn is_ancestor_from_repo(a: &str, b: &str, repo: &str) -> io::Result<bool> {
    __is_ancestor_from_repo(a, b, repo, &mut HashSet::new())
}

/// Gets the common ancestor of two commits.
fn __get_ancestor_from_repo(
    mut a: Vec<u8>,
    mut b: Vec<u8>,
    repo: &str,
    steps1: &mut HashSet<String>,
    steps2: &mut HashSet<String>,
) -> io::Result<String> {
    loop {
        let parents1 = get_parent_commits(&a);
        let parents2 = get_parent_commits(&b);

        if let Some(parents) = parents1 {
            for parent in parents {
                if steps2.contains(&parent) {
                    return Ok(parent);
                }

                steps1.insert(parent.clone());
                let (_, _, data) = get_object_from_repo(&parent, repo)?;
                a = data;
            }
        }

        if let Some(parents) = parents2 {
            for parent in parents {
                if steps1.contains(&parent) {
                    return Ok(parent);
                }

                steps2.insert(parent.clone());
                let (_, _, data) = get_object_from_repo(&parent, repo)?;
                b = data;
            }
        }
    }
}

/// Returns the common ancestor of two commits.
pub fn get_ancestor_from_repo(a: &[u8], b: &[u8], repo: &str) -> io::Result<String> {
    __get_ancestor_from_repo(
        a.to_vec(),
        b.to_vec(),
        repo,
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

/// Moves a branch to a new commit within a repository.
pub fn update_branch_from_repo(name: &str, hash: &str, repo: &str) -> io::Result<()> {
    let mut branch = File::create(format!("{repo}/refs/heads/{name}"))?;
    branch.write_all(hash.as_bytes())?;
    branch.write_all(b"\n")?;
    Ok(())
}

fn __read_tree_from_repo(
    hash: &str,
    entries: &mut Vec<IndexEntry>,
    path: String,
    repo: &str,
) -> io::Result<()> {
    // Get a String representation of tree.
    let tree = ls_tree_from_repo(hash, repo)?;

    // Iterate over the objects in the tree.
    for line in tree.lines() {
        // Get data about the object.
        let (_, otype, hash, name) = parse_ls_tree_entry(line);
        let path = match path.as_str() {
            "" => name.to_string(),
            _ => path.clone() + "/" + &name,
        };

        match otype {
            "blob" => {
                let entry = IndexEntry::new_from_repo_with_hash(&path, hash)?;
                entries.push(entry);
            }

            "tree" => __read_tree_from_repo(hash, entries, path, repo)?,
            _ => return Err(io_err!("invalid object type")),
        }
    }

    Ok(())
}

/// Given a Tree object's hash, reads it's content and
/// generates an Index object from it. Returning it.
pub fn read_tree_from_repo(root: &str, path: &str, repo: &str) -> io::Result<Index> {
    let mut entries = vec![];
    __read_tree_from_repo(root, &mut entries, String::from(path), repo)?;
    let map = HashMap::from_iter(entries.into_iter().map(|e| (e.get_path().to_string(), e)));
    Ok(Index::with(map))
}

fn __write_tree_from_repo(entries: &mut [IndexEntry], repo: &str) -> io::Result<Vec<u8>> {
    let mut tree_entries = vec![];

    let mut i = 0;
    while i < entries.len() {
        let path = entries[i].get_path().to_string();
        // Check if path contains '/'. In that case, there should
        // be a Tree object for this exact directory.
        if let Some(index) = path.find('/') {
            let name = &path[..index];

            let from = i;
            // Iterate over entries until we find one that is
            // not in the same sub-directory.
            while i < entries.len() {
                let path = entries[i].get_path().to_string();
                if entries[i].get_path().starts_with(name) {
                    // Remove this directory's name from the entry's path.
                    entries[i].set_path(&path[index + 1..]);
                    i += 1;
                } else {
                    break;
                }
            }

            // Recursively call __write_tree_from_repo to get this sub-tree's hash.
            let hash = __write_tree_from_repo(&mut entries[from..i], repo)?;
            tree_entries.write_all(format!("40000 {name}\0").as_bytes())?;
            tree_entries.write_all(&hash)?;
        } else {
            // If there is no '/', then the Tree object
            // containing this blob has already been created.
            let mode = entries[i].get_mode();
            let name = entries[i].get_path();
            let hash = entries[i].get_hash();
            tree_entries.write_all(format!("{mode} {name}\0").as_bytes())?;
            tree_entries.write_all(hash)?;
            i += 1;
        }
    }

    Ok(__hash_object(&tree_entries, "tree", true, repo)?.0)
}

pub fn write_tree_from_repo(index: Index, repo: &str) -> io::Result<Vec<u8>> {
    __write_tree_from_repo(&mut index.get_entries()[..], repo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn tags() {
        let tags = get_tags().unwrap();
        println!("tags: {tags:?}");

        let refs = get_local_refs().unwrap();
        println!("refs: {refs:?}");
    }

    #[test]
    #[ignore]
    fn remotes() {
        let remotes = get_remote_refs().unwrap();
        println!("remotes: {remotes:?}");
    }
}
