use super::diff_type::Diff;
use super::lcs::FileDiff;
use crate::commands::{cat_file, ls_tree};
use crate::io_err;
use crate::plumbing::diff::diff_tree::diff_tree;
use crate::plumbing::diff::diff_type::DiffType;
use crate::plumbing::diff::lcs::diff;
use std::collections::VecDeque;
use std::io;
use utils::object::object_db::get_object;
use utils::plumbing::commit::get_commit_root;
use utils::plumbing::ls_tree::parse_ls_tree_entry;

/// Represents a patch between two files.
/// The patch is represented as a vector of
/// FileDiff (difference between the lines of the file).
/// The patch also contains the path of the file,
/// the old and new hash of the file.

#[derive(Debug)]
pub struct Patch {
    pub path: String,
    pub old: Option<String>,
    pub new: Option<String>,
    pub difftype: DiffType,
    pub diff: VecDeque<FileDiff>,
}

pub fn differences_beetween_files(patch: &Patch) -> String {
    let mut s = String::new();
    for line in &patch.diff {
        s.push_str(&format!("{}\n", line));
    }
    s
}

pub fn get_patch_of_tree_diffs(diffs: &[Diff], path: &str) -> io::Result<Vec<Patch>> {
    let mut patches = vec![];
    for difference in diffs {
        let (_, otype, hash, name) = parse_ls_tree_entry(&difference.line);
        let path = if path.is_empty() {
            name
        } else {
            format!("{path}/{name}")
        };

        use DiffType::*;
        match (&difference.tag, otype) {
            (Unchanged, "blob") => {
                let (_, _, content) = cat_file(hash)?;
                let diff = diff(&content, &content);

                patches.push(Patch {
                    path,
                    old: Some(hash.to_string()),
                    new: Some(hash.to_string()),
                    difftype: Unchanged,
                    diff,
                });
            }

            (Added, "blob") => {
                let (_, _, content) = cat_file(hash)?;
                let diff = diff("", &content);

                patches.push(Patch {
                    path,
                    old: None,
                    new: Some(hash.to_string()),
                    difftype: Added,
                    diff,
                })
            }

            (Removed, "blob") => {
                let (_, _, content) = cat_file(hash)?;
                let diff = diff(&content, "");

                patches.push(Patch {
                    path,
                    old: Some(hash.to_string()),
                    new: None,
                    difftype: Removed,
                    diff,
                })
            }

            (Modified(line), "blob") => {
                let (_, _, new_hash, _) = parse_ls_tree_entry(line);
                let (_, _, old_content) = cat_file(hash)?;
                let (_, _, new_content) = cat_file(new_hash)?;

                // Calculate their difference.
                let diff = diff(&old_content, &new_content);

                patches.push(Patch {
                    path,
                    old: Some(hash.to_string()),
                    new: Some(new_hash.to_string()),
                    difftype: Modified(line.to_string()),
                    diff,
                })
            }

            (Unchanged, "tree") => {
                let tree = ls_tree(hash)?;
                let diffs: Vec<_> = diff_tree(&tree, &tree).collect();
                let tree_diffs = get_patch_of_tree_diffs(&diffs, &path)?;
                patches.extend(tree_diffs);
            }

            (Added, "tree") => {
                let tree = ls_tree(hash)?;
                let diffs: Vec<_> = diff_tree("", &tree).collect();
                let tree_diffs = get_patch_of_tree_diffs(&diffs, &path)?;
                patches.extend(tree_diffs);
            }

            (Removed, "tree") => {
                let tree = ls_tree(hash)?;
                let diffs: Vec<_> = diff_tree(&tree, "").collect();
                let tree_diffs = get_patch_of_tree_diffs(&diffs, &path)?;
                patches.extend(tree_diffs);
            }

            (Modified(line), "tree") => {
                let (_, _, new_hash, _) = parse_ls_tree_entry(line);
                let old_tree = ls_tree(hash)?;
                let new_tree = ls_tree(new_hash)?;

                let diffs: Vec<_> = diff_tree(&old_tree, &new_tree).collect();
                let tree_diffs = get_patch_of_tree_diffs(&diffs, &path)?;
                patches.extend(tree_diffs);
            }

            _ => return Err(io_err!("Invalid diff")),
        }
    }

    // Sort patches by path.
    patches.sort_by_key(|p| p.path.clone());
    Ok(patches)
}

/// Receives two commits and calculates the differences between them.
pub fn diff_commit(hash1: &str, hash2: &str) -> io::Result<Vec<Patch>> {
    let (_, _, commit1) = get_object(hash1)?;
    let (_, _, commit2) = get_object(hash2)?;

    // Get trees.
    let tree1 = ls_tree(&get_commit_root(&commit1)?)?;
    let tree2 = ls_tree(&get_commit_root(&commit2)?)?;

    // Get the diff between the two trees.
    let diff: Vec<_> = diff_tree(&tree1, &tree2).collect();
    let patch = get_patch_of_tree_diffs(&diff, "")?;
    Ok(patch)
}

impl Patch {
    pub fn formatted_path(&self) -> String {
        match self.difftype {
            DiffType::Added => format!("+ {}", &self.path),
            DiffType::Modified(_) => format!("m {}", &self.path),
            DiffType::Removed => format!("- {}", &self.path),
            _ => self.path.clone(),
        }
    }
}
