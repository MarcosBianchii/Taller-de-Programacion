use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::io;
use utils::plumbing::commit::{
    get_author_and_time, get_commit_msg, get_commit_root, get_committer_and_time,
    get_parent_commits,
};
use utils::{io_err, object::object_db::get_object_from_repo};

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentCommit {
    pub sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    name: String,
    email: String,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Committer {
    name: String,
    email: String,
    date: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tree {
    pub sha: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub sha: String,
    pub author: Author,
    pub committer: Committer,
    pub message: String,
    pub parents: Vec<ParentCommit>,
    pub tree: Tree,
}

// Splits a name and mail from a string like "name <mail>".
fn split_name_mail(name_mail: &str) -> (String, String) {
    let mut split = name_mail.split(' ');

    let mut name = String::new();
    let mut mail = String::new();

    if let Some(commit_name) = split.next() {
        name = commit_name.to_string();
    }

    if let Some(commit_mail) = split.next() {
        mail = commit_mail.replace(['<', '>'], "");
    }

    (name, mail)
}

impl Commit {
    pub fn new(hash: &str, repo: &str) -> io::Result<Self> {
        let (otype, _, data) = get_object_from_repo(hash, repo)?;
        if otype != "commit" {
            return Err(io_err!("Object is not a commit"));
        }

        // Get hash.
        let sha = hash.to_string();

        // Get author and committer.
        let (author, time) = get_author_and_time(&data).ok_or(io_err!("Corrupted commit"))?;
        let (name, email) = split_name_mail(&author);
        let date = DateTime::parse_from_str(&time, "%s %z")
            .map_err(|_| io_err!("Corrupted commit"))?
            .to_rfc2822();

        let author = Author { name, email, date };

        let (committer, time) = get_committer_and_time(&data).ok_or(io_err!("Corrupted commit"))?;
        let (name, email) = split_name_mail(&committer);
        let date = DateTime::parse_from_str(&time, "%s %z")
            .map_err(|_| io_err!("Corrupted commit"))?
            .to_rfc2822();

        let committer = Committer { name, email, date };

        // Get commit message.
        let message = get_commit_msg(&data).ok_or(io_err!("Corrupted commit"))?;

        // Get parent commits.
        let mut parents = vec![];
        if let Some(commit_parents) = get_parent_commits(&data) {
            for parent in commit_parents {
                parents.push(ParentCommit { sha: parent });
            }
        }

        let tree = Tree {
            sha: get_commit_root(&data)?,
        };

        Ok(Self {
            sha,
            author,
            committer,
            message,
            parents,
            tree,
        })
    }
}
