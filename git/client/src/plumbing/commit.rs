use super::commands::*;
use chrono::{DateTime, Local};
use std::io::{self, BufRead, Write};

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

/// Returns the message of the commit.
pub fn get_commit_msg(data: &[u8]) -> Option<String> {
    data.lines().flatten().last()
}

// Returns date formated for commit purposes.
// This is "<unix timestamp> <UTC offset>".
pub fn get_time_fmt(date: DateTime<Local>) -> String {
    let stamp = date.timestamp();
    let offset = date.offset().to_string().replace(':', "");
    format!("{stamp} {offset}")
}

pub fn __commit(msg: &str) -> io::Result<String> {
    let root = write_tree()?;

    let root = root.iter().fold(String::new(), |mut acc, byte| {
        acc.push_str(&format!("{:02x}", byte));
        acc
    });

    // Write root tree.
    let mut commit = vec![];
    commit.write_all(b"tree ")?;
    commit.write_all(root.as_bytes())?;
    commit.write_all(b"\n")?;

    // Append parent commit's hash if it exists.
    if let Some(parent) = get_head() {
        commit.write_all(format!("parent {parent}\n").as_bytes())?;
    }

    // Append author and committer.
    let author = get_userconfig()?.to_string();
    let time = get_time_fmt(Local::now());
    commit.write_all(format!("author {author} {time}\ncommitter {author} {time}\n").as_bytes())?;

    // Append commit message.
    commit.write_all(format!("\n{msg}\n").as_bytes())?;

    // Hash commit object and update HEAD.
    let hash = hash_object(&commit, "commit", true)?;
    update_head(&hash)?;

    // Update index.
    let mut index = read_index().unwrap_or_default();
    index.unstage_all();
    write_index(index)?;

    println!("commit ok");
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn author_time() {
        let commit = r#"
tree 56b63f1f2f71a303c4f4ce290f19bfead2eb63f6
parent 043ff0989fe9f8a1acb29df759efda45e9d94e0e
author Pepito <pepito@fi.uba.ar> 1699028021 -0300
committer Pepito <pepito@fi.uba.ar> 1699028021 -0300

clippy
"#;
        let (author, time) = get_author_and_time(commit.as_bytes()).unwrap();
        assert_eq!(author, "Pepito <pepito@fi.uba.ar>");
        assert_eq!(time, "1699028021 -0300");
    }

    #[test]
    fn msg() {
        let commit = r#"
tree 56b63f1f2f71a303c4f4ce290f19bfead2eb63f6
parent 043ff0989fe9f8a1acb29df759efda45e9d94e0e
author Pepito <pepito@fi.uba.ar> 1699028021 -0300
committer Pepito <pepito@fi.uba.ar> 1699028021 -0300

clippy, siempre es clippy ;(
"#;
        let msg = get_commit_msg(commit.as_bytes()).unwrap();
        println!("{msg}");
    }
}
