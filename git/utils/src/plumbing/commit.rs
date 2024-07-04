use std::io::{self, BufRead};

use chrono::{DateTime, Local};

use crate::{io_err, object::object_db::get_object_from_repo};

// Returns the 40 char hash of the commit's root tree.
pub fn get_commit_root(mut data: &[u8]) -> io::Result<String> {
    data.consume("tree ".len());
    let mut root = String::new();
    data.read_line(&mut root)?;
    Ok(root.trim().to_string())
}

/// Commits can have multiple parents. Because of Merge Commits.
pub fn get_parent_commits(data: &[u8]) -> Option<Vec<String>> {
    let mut parents = vec![];
    for line in String::from_utf8_lossy(data).lines() {
        if line.starts_with("parent") {
            let parent = line[7..].trim().to_string();
            parents.push(parent);
        }
    }

    if parents.is_empty() {
        return None;
    }

    Some(parents)
}

/// Returns the message of the commit.
pub fn get_commit_msg(data: &[u8]) -> Option<String> {
    data.lines().flatten().last()
}

/// Returns the author line of the commit.
pub fn get_author_and_time(data: &[u8]) -> Option<(String, String)> {
    let mut author = String::new();
    let mut time = String::new();

    for line in data.lines().flatten() {
        if let Some(stripped) = line.strip_prefix("author ") {
            let components: Vec<_> = stripped.split_whitespace().collect();
            let items = components.len();
            author = components[..items - 2].join(" ");
            time = components[items - 2..].join(" ");
            break;
        }
    }

    if author.is_empty() || time.is_empty() {
        return None;
    }

    Some((author, time))
}

/// Returns the committer line of the commit.
pub fn get_committer_and_time(data: &[u8]) -> Option<(String, String)> {
    let mut committer = String::new();
    let mut time = String::new();

    for line in data.lines().flatten() {
        if let Some(stripped) = line.strip_prefix("committer ") {
            let components: Vec<_> = stripped.split_whitespace().collect();
            let items = components.len();
            committer = components[..items - 2].join(" ");
            time = components[items - 2..].join(" ");
            break;
        }
    }

    if committer.is_empty() || time.is_empty() {
        return None;
    }

    Some((committer, time))
}

// Returns date formated for commit purposes.
// This is "<unix timestamp> <UTC offset>".
pub fn get_time_fmt(date: DateTime<Local>) -> String {
    let stamp = date.timestamp();
    let offset = date.offset().to_string().replace(':', "");
    format!("{stamp} {offset}")
}

// Returns a Vec of commit objects from the given hash until the given hash.
pub fn get_commits_until_from_repo(hash: &str, until: &str, repo: &str) -> io::Result<Vec<String>> {
    let mut commits = vec![];
    if hash == until {
        return Ok(commits);
    }

    commits.push(hash.to_string());
    let (_, _, commit) = get_object_from_repo(hash, repo)?;
    match get_parent_commits(&commit) {
        None => return Err(io_err!("No parent commit found")),
        Some(parents) => {
            let hashes = get_commits_until_from_repo(&parents[0].to_string(), until, repo)?;
            commits.extend(hashes);
        }
    };

    // Reverse the vec.
    commits.reverse();
    Ok(commits)
}
