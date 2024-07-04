use super::{
    super::config_file::{config::Config, config_entry::ConfigEntry},
    fetch::add_to_refs,
    heads::*,
    read_tree::__read_tree,
    userconfig::*,
    write_tree::__write_tree,
};
use std::{
    collections::HashMap,
    fs::{self, File},
    io,
};
use utils::index_file::commands::{__read_index, __write_index};
use utils::index_file::index::Index;
use utils::plumbing::hash_object::__hash_object;

/// Hashes a string following the git object format. Returns a vector of bytes.
pub fn hash_object(data: &[u8], otype: &str, write: bool) -> io::Result<String> {
    Ok(__hash_object(data, otype, write, ".git")?.1)
}

/// Returns a vector of the entries in the .git/index file.
pub fn read_index() -> io::Result<Index> {
    let entries = __read_index(File::open(".git/index")?)?;
    let mut map = HashMap::with_capacity(entries.len());
    for entry in entries {
        map.insert(entry.get_path().to_string(), entry);
    }

    Ok(Index::with(map))
}

/// Writes the given entries to the .git/index file.
pub fn write_index(index: Index) -> io::Result<()> {
    let mut entries = index.get_entries();
    entries.sort_by_key(|e| e.get_path().to_string());
    __write_index(entries, File::create(".git/index")?)
}

/// Given a Tree object's hash, reads it's content and
/// generates an Index object from it. Returning it.
pub fn read_tree(root: &str, path: &str) -> io::Result<Index> {
    let mut entries = vec![];
    __read_tree(root, &mut entries, String::from(path))?;
    let map = HashMap::from_iter(entries.into_iter().map(|e| (e.get_path().to_string(), e)));
    Ok(Index::with(map))
}

/// Reads the content of the index file, creating a Tree
/// hierarchy of the objects in the object database. Returns
/// the hash of the root Tree object.
pub fn write_tree() -> io::Result<Vec<u8>> {
    let index = read_index().unwrap_or_default();
    __write_tree(&mut index.get_entries()[..], true)
}

/// Returns the hash of the commit object pointed to by HEAD.
pub fn get_head() -> Option<String> {
    let file = File::open(".git/HEAD").ok()?;
    __get_head_commit(file).ok()
}

/// Returns the hash of the commit object pointed to by the given branch.
fn get_branch_hash(branch: &str) -> Option<String> {
    let file = File::open(format!(".git/refs/{branch}")).ok()?;
    __get_branch(file).ok()
}

pub fn get_branch(branch: &str) -> Option<String> {
    if let Some(hash) = get_branch_hash(&format!("heads/{branch}")) {
        return Some(hash);
    }

    // Get remote.
    let config = Config::read().ok()?;
    let remote = match config.get(branch) {
        Some(ConfigEntry::Branch { remote, .. }) => remote,
        _ => return None,
    };

    if let Some(hash) = get_branch_hash(&format!("remotes/{remote}/{branch}")) {
        // Add the branch to the local refs.
        add_to_refs(&format!("refs/heads/{branch}"), &hash).ok()?;
        return Some(hash);
    }

    None
}

/// Updates the HEAD file to point to the given branch.
pub fn update_head(hash_commit: &str) -> io::Result<()> {
    __update_head_commit(hash_commit)?;
    Ok(())
}

/// Returns a Config struct containing the current user's
/// configuration for commit purposes.
pub fn get_userconfig() -> io::Result<UserConfig> {
    let file = match File::open(".git/.gitconfig") {
        Ok(file) => file,
        Err(_) => {
            set_userconfig("pepito", "default@fi.uba.ar", "cmdlog.txt", "all")?;
            File::open(".git/.gitconfig")?
        }
    };

    __get_userconfig(file)
}

/// Writes the given user and mail to the .git/.gitconfig file.
pub fn set_userconfig(user: &str, mail: &str, log_path: &str, log_mode: &str) -> io::Result<()> {
    let file = File::create(".git/.gitconfig")?;
    __set_userconfig(user, mail, log_path, log_mode, file)
}

/// Returns the name of the branch pointed to by HEAD.
/// If HEAD is detached, returns the hash of the commit.
pub fn get_cur_branch() -> io::Result<String> {
    let head = fs::read_to_string(".git/HEAD")?;

    if let Some(stripped) = head.strip_prefix("ref: ") {
        Ok(stripped.trim().to_string())
    } else {
        let hash = head.trim().to_string();
        Ok(hash)
    }
}
