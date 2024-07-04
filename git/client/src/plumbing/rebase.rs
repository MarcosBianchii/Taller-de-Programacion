use super::commands::{
    get_head, get_userconfig, hash_object, read_index, update_head, write_index, write_tree,
};
use super::commit::{get_author_and_time, get_commit_msg, get_parent_commits, get_time_fmt};
use super::diff::diff_tree::diff_tree;
use super::merge::{get_ancestor, refactor_dir};
use crate::commands::ls_tree;
use crate::{diff_2_map, io_err};
use chrono::Local;
use std::collections::HashMap;
use std::io::{self, Write};
use utils::object::object_db::get_object;
use utils::plumbing::commit::get_commit_root;
use utils::plumbing::ls_tree::{hash_to_str, parse_ls_tree_entry};

// Returns a Vec of commit objects from the given hash until the given hash.
fn get_commits_until(hash: &str, until: &str) -> io::Result<Vec<Vec<u8>>> {
    let mut commits = vec![];
    let mut current = hash.to_string();

    while current != until {
        let (_, _, commit) = get_object(&current)?;
        current = match get_parent_commits(&commit) {
            None => return Err(io_err!("No parent commit found")),
            Some(parents) => {
                if parents.len() > 1 {
                    return Err(io_err!("Merge commits are not supported"));
                }

                parents[0].to_string()
            }
        };

        commits.push(commit);
    }

    // Reverse the vec.
    commits.reverse();

    Ok(commits)
}

fn rebase_commit(author: &str, time: &str, msg: &str, parent: &str) -> io::Result<String> {
    let root = hash_to_str(&write_tree()?);

    // Write root tree.
    let mut commit = vec![];
    commit.write_all(format!("tree {root}\n").as_bytes())?;

    // Append parent commit.
    commit.write_all(format!("parent {parent}\n").as_bytes())?;

    // Append author and committer.
    commit.write_all(format!("author {author} {time}\n").as_bytes())?;

    let committer = get_userconfig()?.to_string();
    let time = get_time_fmt(Local::now());
    commit.write_all(format!("committer {committer} {time}\n").as_bytes())?;

    // Append commit message.
    commit.write_all(format!("\n{msg}\n").as_bytes())?;

    // Hash commit object and update HEAD.
    hash_object(&commit, "commit", true)
}

pub fn __rebase(head: &str, other: &str, other_branch_name: &str) -> io::Result<()> {
    // Get commit objects.
    let (otype1, _, head_commit) = get_object(head)?;
    let (otype2, _, other_commit) = get_object(other)?;

    if otype1 != "commit" || otype2 != "commit" {
        return Err(io_err!("Not a commit object"));
    }

    // Get the common ancestor of the two commits.
    let ancestor = get_ancestor(&head_commit, &other_commit)?;
    let (_, _, ancestor_data) = get_object(&ancestor)?;
    let ancestor_tree_root = get_commit_root(&ancestor_data)?;

    // Get string representation.
    let ancestor_tree = ls_tree(&ancestor_tree_root)?;

    // Get the commits from the head commit until the ancestor.
    let commits = get_commits_until(head, &ancestor)?;

    // Update HEAD.
    update_head(other)?;

    for commit in commits {
        // Get both branches' trees.
        let head_commit = get_head().ok_or(io_err!("HEAD not found"))?;
        let (_, _, head_data) = get_object(&head_commit)?;
        let head_tree_root = get_commit_root(&head_data)?;
        let commit_tree_root = get_commit_root(&commit)?;

        // Get string representation.
        let head_tree = ls_tree(&head_tree_root)?;
        let commit_tree = ls_tree(&commit_tree_root)?;

        // Calculate the differences between the ancestor and both branches
        let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &head_tree));
        let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &commit_tree));

        // Apply the cnahges to the working directory and update index.
        let mut index = read_index().unwrap_or_default();
        refactor_dir(
            diffs1,
            diffs2,
            "".to_string(),
            other_branch_name,
            &mut index,
        )?;

        write_index(index)?;

        // Create rebase commit.
        let (author, time) = get_author_and_time(&commit).ok_or(io_err!("Invalid commit"))?;
        let msg = get_commit_msg(&commit).unwrap_or_default();
        let commit = rebase_commit(&author, &time, &msg, &head_commit)?;

        // Update HEAD.
        update_head(&commit)?;
    }

    Ok(())
}
