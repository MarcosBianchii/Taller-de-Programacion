use super::commit::{
    get_author_and_time, get_commit_msg, get_committer_and_time, get_parent_commits,
};
use chrono::{DateTime, FixedOffset};
use std::{collections::HashSet, io};
use utils::object::object_db::get_object;

fn get_log(
    hash: &str,
    visited: &mut HashSet<String>,
) -> io::Result<Vec<(String, Option<DateTime<FixedOffset>>)>> {
    visited.insert(hash.to_string());
    let mut commits = vec![];

    let mut new_msg = String::new();
    let (_, _, data) = get_object(hash)?;

    // Append commit's hash.
    new_msg.push_str(&format!("commit {}\n", hash));

    // If this is a merge commit, append the parent commits.
    if let Some(parents) = get_parent_commits(&data) {
        if parents.len() > 1 {
            new_msg.push_str(&format!(
                "Merge: {}\n",
                parents
                    .iter()
                    .map(|p| &p[..7])
                    .collect::<Vec<&str>>()
                    .join(" ")
            ));
        }
    }

    let mut commit_time = None;

    // Append author and time.
    if let Some((author, _)) = get_author_and_time(&data) {
        new_msg.push_str(&format!("Author: {author}\n"));

        // Get committer's time.
        if let Some((_, time)) = get_committer_and_time(&data) {
            // Format time.
            if let Ok(time) = DateTime::parse_from_str(&time, "%s %z") {
                new_msg.push_str(&format!("Date:   {time}\n"));
                commit_time = Some(time);
            }
        }
    }

    // Append commit message.
    if let Some(msg) = get_commit_msg(&data) {
        new_msg.push_str(&format!("\n\t{}\n", msg));
    }

    // Add it to the commit history.
    commits.push((new_msg, commit_time));

    // Iter through parents.
    let parents = get_parent_commits(&data)
        .unwrap_or_default()
        .into_iter()
        .rev();

    for parent in parents {
        if visited.contains(&parent) {
            continue;
        }

        commits.append(&mut get_log(&parent, visited).unwrap_or_default());
    }

    Ok(commits)
}

/// Travels through the commit history of the given commit recompiling a readable
/// version of the commit to show a history log to the user.
pub fn __log(hash: &str, visited: &mut HashSet<String>) -> io::Result<Vec<String>> {
    let mut log = get_log(hash, visited)?;
    log.sort_by_key(|(_, time)| time.unwrap_or_default());
    log.reverse();
    Ok(log.into_iter().map(|(msg, _)| msg).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test() {
        let commit = "378edd0df9db2008a0a7ec90e14770b22ca81b2b";
        let log = __log(commit, &mut HashSet::new()).unwrap();
        println!("{log:#?}");
    }

    #[test]
    #[ignore]
    fn time_fmt() {
        let time = "1699028021 -0300";
        let time = DateTime::parse_from_str(time, "%s %z").unwrap();

        println!("{time}");
    }
}
