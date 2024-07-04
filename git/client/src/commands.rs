use super::config_file::config::Config;
use super::plumbing::{
    checkout::__checkout, clone::__clone, commands::*, commit::__commit, fetch::__fetch,
    ignore::set_to_be_ignored, log::__log, merge::__merge, push::__push, remote::__remote,
    tag::__tag,
};
use crate::config_file::config_entry::ConfigEntry;
use crate::plumbing::heads::get_head_name;
use crate::plumbing::refs::get_local_branches;
use crate::plumbing::{ignore::files_not_ignored, rebase::__rebase};

use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Write},
};
use utils::get_current_refs;
use utils::object::object_db::get_object;
use utils::plumbing::{
    hash_object::__hash_object,
    ls_tree::{__ls_tree, hash_to_str},
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

/// Downloads a repo from the server and it's state to
/// a directory with the same name as the repository.
#[allow(dead_code)]
pub fn clone(url: &str) -> io::Result<()> {
    __clone(url)
}

/// Downloads refs and objects from a remote repository.
#[allow(dead_code)]
pub fn fetch(remote: &str) -> io::Result<()> {
    __fetch(remote)?;
    Ok(())
}

/// Create an empty Git Repository at local directory
/// If you provide a directory, the command is run inside it.
/// If this directory does not exist, it will be created.
#[allow(dead_code)]
pub fn init(directory: &str) -> io::Result<()> {
    let path_local_repo = format!("{}/.git", directory);

    // If .git directory exists don't overwrite it.
    if fs::metadata(&path_local_repo).is_ok() {
        return Err(io_err!(".git directory already exists"));
    }

    fs::create_dir(&path_local_repo)?;

    // creates HEAD file
    let mut head = File::create(path_local_repo.clone() + "/HEAD")?;
    head.write_all(b"ref: refs/heads/master\n")?;

    //branch("master")?;

    // objects directory
    fs::create_dir(path_local_repo.clone() + "/objects")?;
    fs::create_dir(path_local_repo.clone() + "/info")?;
    fs::create_dir(path_local_repo.clone() + "/objects/pack")?;

    // refs directory
    fs::create_dir(path_local_repo.clone() + "/refs")?;
    fs::create_dir(path_local_repo.clone() + "/refs/heads")?;
    fs::create_dir(path_local_repo.clone() + "/refs/tags")?;

    fs::create_dir(path_local_repo.clone() + "/branches")?;
    File::create(path_local_repo.clone() + "/info/exclude")?;
    let mut desc = File::create(path_local_repo.clone() + "/description")?;
    desc.write_all(b"Unnamed repository; edit this file 'description' to name the repository.")?;

    let cmdlog_path = "cmdlog.txt";
    set_userconfig("pepito", "pepito@fi.uba.ar", "cmdlog.txt", "all")?;
    let mut ignore = File::create(".gitignore")?;
    ignore.write_all(format!("{}\n", cmdlog_path).as_bytes())?;

    // config file
    let mut config = File::create(path_local_repo.clone() + "/config")?;
    println!("{:?}", config);
    config.write_all("[core]\n    repositoryformatversion = 0\n    filemode = true\n    bare = false\n    logallrefupdates = true\n\n".as_bytes())?;

    Ok(())
}

/// Adds the given files to the index file.
#[allow(dead_code)]
pub fn add(files: Vec<String>) -> io::Result<()> {
    let ignored_files = set_to_be_ignored()?;
    let mut index = read_index().unwrap_or_default();
    for file in files {
        // add files who were already being tracked, or not ignored
        if index.contains_key(&file) || !ignored_files.contains(&file) {
            index.add(file, true, true)?;
        }
    }

    write_index(index)?;
    println!("add ok");
    Ok(())
}

/// Removes tracked files from the index.
#[allow(dead_code)]
pub fn rm(files: Vec<String>) -> io::Result<()> {
    let mut index = match read_index() {
        Ok(index) => index,
        _ => return Err(io_err!("index file does not exist")),
    };

    for file in files {
        index.remove(&file);
    }

    write_index(index)
}

#[derive(Debug)]
pub enum FileStatus {
    New(String),
    Staged(String),
    Modified(String),
    Deleted(String),
}

use std::fmt::{self, Display};
impl Display for FileStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (prefix, name) = match self {
            FileStatus::New(name) => ("New", name),
            FileStatus::Staged(name) => ("Staged", name),
            FileStatus::Modified(name) => ("Modified", name),
            FileStatus::Deleted(name) => ("Deleted", name),
        };

        write!(f, "{prefix}: {name}")
    }
}

/// Returns a vector of FileStatus indicating
/// the changes in the working directory.
#[allow(dead_code)]
pub fn status() -> io::Result<Vec<FileStatus>> {
    let mut changes = vec![];
    let index = read_index().unwrap_or_default();
    let paths = files_not_ignored()?;

    // Iter over paths and check if
    // they are in the index.
    for path in &paths {
        if let Some(entry) = index.get(path) {
            let data = fs::read_to_string(path)?;
            let hash = __hash_object(data.as_bytes(), "blob", false, ".git")?.0;
            // If the hash of the file is different
            // then the file was modified.
            if entry.get_hash() != hash {
                // If paths are the same but hash is different
                // then this file was modified.
                changes.push(FileStatus::Modified(path.to_string()));
            } else if entry.is_staged() {
                changes.push(FileStatus::Staged(path.to_string()));
            }
        } else {
            // If it was not found in the index
            // then it is an untracked file.
            changes.push(FileStatus::New(path.to_string()));
        }
    }

    // Iter over index entries to check if
    // some of them are missing, indicating
    // that they were deleted.
    for (path, _) in index.iter() {
        if !paths.contains(path) {
            changes.push(FileStatus::Deleted(path.to_string()));
        }
    }

    Ok(changes)
}

/// Creates a commit object with the given message.
/// Message should not be '\n' terminated.
#[allow(dead_code)]
pub fn commit(msg: &str) -> io::Result<String> {
    __commit(msg)
}

/// Returns a String with the data of the given hash's object.
#[allow(dead_code)]
pub fn cat_file(hash: &str) -> io::Result<(String, String, String)> {
    let (otype, osize, data) = get_object(hash)?;
    Ok((otype, osize, String::from_utf8_lossy(&data).to_string()))
}

/// Creates a new branch with the given name.
#[allow(dead_code)]
pub fn branch(name: Option<String>) -> io::Result<Option<Vec<String>>> {
    if let Some(name) = name {
        match get_head() {
            None => Err(io_err!("HEAD is not pointing to any commit")),
            Some(commit) => {
                let mut branch = File::create(format!(".git/refs/heads/{name}"))?;
                branch.write_all(commit.as_bytes())?;
                branch.write_all(b"\n")?;
                Ok(None)
            }
        }
    } else {
        Ok(get_local_branches().ok())
    }
}

/// Refactors the current directory to match the given branch's state.
#[allow(dead_code)]
pub fn checkout(branch: &str) -> io::Result<()> {
    __checkout(branch)
}

/// Makes a fusion of the given branch and the current one.
/// If neither branch share a common ancestor, then a merge commit
/// will be created holding both branches' commits as parents.
#[allow(dead_code)]
pub fn merge(branch: &str) -> io::Result<()> {
    __merge(branch, "heads")
}

/// Returns a vector with the history of the current branch.
/// The vector is ordered from the oldest to the newest commit.
#[allow(dead_code)]
pub fn log() -> io::Result<Vec<String>> {
    let head = get_head().ok_or(io_err!("HEAD is not pointing to any commit"))?;
    __log(&head, &mut HashSet::new())
}

#[allow(dead_code)]
pub enum RemoteCommand {
    Add { name: String, url: String },
    Rem { name: String },
    List,
}

/// Manage set of tracked repositories. Adds, removes or lists
/// remote repositories tracked by git.
pub fn remote(cmd: RemoteCommand) -> io::Result<Option<Vec<String>>> {
    let mut config = Config::read()?;
    match __remote(cmd, &mut config) {
        ret @ Ok(None) => {
            config.write()?;
            ret
        }

        any => any,
    }
}

/// Pulls changes from a remote repository and
/// merges them with the current branch.
pub fn pull(remote: &str) -> io::Result<()> {
    get_head().ok_or(io_err!("HEAD is not pointing to any commit"))?;
    __fetch(remote)?;
    let head_name = get_head_name()?;
    __merge(&head_name, &format!("remotes/{remote}"))
}

pub enum PushCommand {
    SetUpstream { branch: String, remote: String },
    Push,
}

/// Sends changes to the remote repository. If the cmd is
/// SetUpstream, it then sets the given branch to point
/// to the given remote and then pushes the changes.
pub fn push(cmd: PushCommand) -> io::Result<()> {
    let mut config = Config::read()?;

    use PushCommand::*;
    match cmd {
        SetUpstream { branch, remote } => {
            match config.get(&branch) {
                Some(ConfigEntry::Branch { .. }) => {}
                Some(ConfigEntry::Remote { .. }) => {
                    return Err(io_err!("Remote name already exists"));
                }
                _ => {}
            }

            let entry = ConfigEntry::new_branch(&branch, &remote);
            config.insert(branch, entry);
            config.write()?;

            // Push the changes.
            __push(&remote)
        }

        Push => {
            let cur_branch = get_head_name()?;
            match config.get(&cur_branch) {
                Some(ConfigEntry::Branch { remote, .. }) => __push(remote),
                _ => Err(io_err!("Current branch has no remote")),
            }
        }
    }
}

/// Returns a readable representation of given
/// tree object's data, represented by it's hash.
pub fn ls_tree(hash: &str) -> io::Result<String> {
    let (otype, _, data) = get_object(hash)?;
    if otype != "tree" {
        return Err(io_err!("Object is not a tree"));
    }

    __ls_tree(&data)
}

/// Receives an array of file paths and returns an array with all those who are
/// set to be ignored in a .gitignore file and not being tracked
pub fn check_ignore(files: Vec<String>) -> io::Result<Vec<String>> {
    let ignored_files = set_to_be_ignored()?;
    let index = read_index().unwrap_or_default();
    let mut res = vec![];

    for file in files {
        // add files who are set to be ignored and not tracked
        if !index.contains_key(&file) && ignored_files.contains(&file) {
            res.push(file);
        }
    }

    Ok(res)
}
/// Returns a readable representation of the current
/// index file.
pub fn ls_files(stage: bool) -> io::Result<String> {
    Ok(read_index()?
        .get_entries()
        .into_iter()
        .map(|entry| {
            let hash = hash_to_str(entry.get_hash());
            if stage {
                format!(
                    "{} {} {}\t{}",
                    entry.get_mode(),
                    hash,
                    entry.get_stage(),
                    entry.get_path(),
                )
            } else {
                entry.get_path().to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n"))
}

/// returns a tuple with the name of the reference as the key and its hash as the value
pub fn show_ref() -> io::Result<Vec<(String, String)>> {
    let mut refs: Vec<(String, String)> = get_current_refs()?.into_iter().collect();
    refs.sort_by_key(|(ref_name, _)| ref_name.clone());

    Ok(refs)
}

pub enum TagCommand {
    Add {
        name: String,
        hash: Option<String>,
        msg: Option<String>,
    },
    AddForce {
        name: String,
        hash: Option<String>,
        msg: Option<String>,
    },
    Del {
        name: String,
    },
    List,
}

/// Creates a new tag with the given name. If the user specified
/// a hash, then the tag will point to that hash. Otherwise, it
/// will point to the current HEAD. If the user specified a message,
/// then the tag will be an annotated tag.
pub fn tag(cmd: TagCommand) -> io::Result<Option<Vec<String>>> {
    __tag(cmd)
}

/// Implementation of `git rebase`. Rebase the given branch
/// into the current one.
pub fn rebase(branch: &str) -> io::Result<()> {
    let head_hash = get_head().ok_or(io_err!("HEAD is not pointing to any commit"))?;
    let branch_hash = get_branch(branch).ok_or(io_err!("Branch does not exist"))?;

    __rebase(&head_hash, &branch_hash, branch)
}
