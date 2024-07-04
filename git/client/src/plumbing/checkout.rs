use super::{
    super::commands::ls_tree,
    super::plumbing::{
        commands::*, diff::diff_tree::diff_tree, diff::diff_type::*, work_dir::directify_tree,
    },
    heads::update_head_reference,
};
use crate::io_err;
use std::{
    fs::{self, File},
    io::{self, Write},
};
use utils::object::object_db::get_object;
use utils::plumbing::{commit::get_commit_root, ls_tree::parse_ls_tree_entry};

/// Applies the given diffs to the working directory.
pub fn refactor_root_dir(diffs: Vec<Diff>, path: &str) -> io::Result<()> {
    for diff in diffs {
        let (_, otype, hash, name) = parse_ls_tree_entry(&diff.line);
        let path = path.to_string() + "/" + &name;

        use DiffType::*;
        match (diff.tag, otype) {
            (Unchanged, "blob" | "tree") => {}
            (Removed, "blob") => fs::remove_file(&path)?,
            (Removed, "tree") => fs::remove_dir_all(&path)?,

            (Modified(line), "blob") => {
                let (_, _, new_hash, _) = parse_ls_tree_entry(&line);
                let mut file = File::create(path)?;
                let (_, _, data) = get_object(new_hash)?;
                file.write_all(&data)?;
            }

            (Added, "blob") => {
                let mut file = File::create(path)?;
                let (_, _, data) = get_object(hash)?;
                file.write_all(&data)?;
            }

            (Modified(line), "tree") => {
                let (_, _, other_hash, _) = parse_ls_tree_entry(&line);

                // Get trees.
                let cur_tree = ls_tree(hash)?;
                let other_tree = ls_tree(other_hash)?;

                // Calculate differences between
                // trees and apply them to work dir.
                let diffs = diff_tree(&cur_tree, &other_tree).collect();
                refactor_root_dir(diffs, &path)?;
            }

            (Added, "tree") => {
                fs::create_dir(&path)?;
                directify_tree(hash, &path)?;
            }

            _ => return Err(io_err!("invalid object type")),
        }
    }

    Ok(())
}

pub fn __checkout(branch: &str) -> io::Result<()> {
    // Get HEAD commit object.
    let head = get_head().ok_or(io_err!("HEAD is not pointing to any commit"))?;
    let (_, _, head_commit) = get_object(&head)?;

    // Get commit object for
    // the branch to checkout.
    let hash = get_branch(branch).ok_or(io_err!("branch does not exist"))?;
    let (otype, _, refs_commit) = get_object(&hash)?;

    // Validate object is a commit.
    if otype != "commit" {
        return Err(io_err!("{hash} is not a commit"));
    }

    // Get tree hash from commit.
    let cur_tree_root = get_commit_root(&head_commit)?;
    let ref_tree_root = get_commit_root(&refs_commit)?;

    // Get String representation of trees.
    let cur_tree = ls_tree(&cur_tree_root)?;
    let ref_tree = ls_tree(&ref_tree_root)?;

    // Calculate differences between
    // trees and apply them to work dir.
    let diffs: Vec<_> = diff_tree(&cur_tree, &ref_tree).collect();
    refactor_root_dir(diffs, ".")?;

    // Update index.
    let index = read_tree(&ref_tree_root, "")?;
    write_index(index)?;

    // update HEAD to point to branch
    update_head_reference(branch)?;
    Ok(())
}
