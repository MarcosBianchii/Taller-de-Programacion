use std::collections::{HashMap, VecDeque};
use std::io;
use utils::plumbing::diff::lcs::FileDiff;
use utils::{
    get_ancestor_from_repo, io_err, is_ancestor_from_repo,
    object::object_db::get_object_from_repo,
    plumbing::{
        commit::get_commit_root,
        diff::{
            diff_tree::diff_tree,
            diff_type::{Diff, DiffType},
            lcs::diff,
        },
        ls_tree::{ls_tree_from_repo, parse_ls_tree_entry},
    },
};

macro_rules! diff_2_map {
    ($diffs:expr) => {{
        let mut map = HashMap::new();
        for diff in $diffs {
            let (_, _, _, name) = parse_ls_tree_entry(&diff.line);
            map.insert(name, diff);
        }

        map
    }};
}

///
/////    S  A  R
///// S [o][b][o]
///// A [a][x][x]
///// R [o][x][o]
///
fn check_merge_changes(mut diff_a: VecDeque<FileDiff>, mut diff_b: VecDeque<FileDiff>) -> bool {
    loop {
        use FileDiff::*;
        match (diff_a.front(), diff_b.front()) {
            (None, None) => return true,

            (Some(Added(_)), Some(Same(_)) | None) => {
                diff_a.pop_front();
            }

            (Some(Same(_)) | None, Some(Added(_))) => {
                diff_b.pop_front();
            }

            (Some(Added(_)), Some(Removed(_))) | (Some(Removed(_)), Some(Added(_))) => {
                return false
            }

            (Some(Added(line1)), Some(Added(line2))) => {
                if line1 != line2 {
                    return false;
                }

                diff_a.pop_front();
                diff_b.pop_front();
            }

            _ => {
                diff_a.pop_front();
                diff_b.pop_front();
            }
        }
    }
}

fn check_file_merge_conflict(ancestor: Vec<u8>, data_a: Vec<u8>, data_b: Vec<u8>) -> bool {
    let file_ancestor = String::from_utf8_lossy(&ancestor);
    let file_a = String::from_utf8_lossy(&data_a);
    let file_b = String::from_utf8_lossy(&data_b);

    let diff1 = diff(&file_ancestor, &file_a);
    let diff2 = diff(&file_ancestor, &file_b);

    // Check for conflicts.
    check_merge_changes(diff1, diff2)
}
/////
/////      U   R   M   A
/////   U [ ] [ ] [ ] [ ]
/////   R [ ] [ ] [x] [ ]
/////   M [ ] [x] [x] [ ]
/////   A [ ] [ ] [ ] [x]
/////
fn check_conflicts(
    diffs1: HashMap<String, Diff>,
    mut diffs2: HashMap<String, Diff>,
    repo: &str,
) -> io::Result<()> {
    for (name, diff1) in diffs1 {
        // Get the diff from the same file
        // from the other branch.
        if let Some(diff2) = diffs2.remove(&name) {
            // This implementation assumes the type of
            // the object didn't change in bewteen branches.
            let (_, _, hash1, _) = parse_ls_tree_entry(&diff1.line);
            let (_, otype, hash2, _) = parse_ls_tree_entry(&diff2.line);

            use DiffType::*;
            match (diff1.tag, diff2.tag, otype) {
                (Modified(_), Removed, _) | (Removed, Modified(_), _) => {
                    return Err(io_err!("Conflict"));
                }

                (Modified(line1), Modified(line2), "blob") => {
                    let (_, _, hash_a_blob, _) = parse_ls_tree_entry(&line1);
                    let (_, _, hash_b_blob, _) = parse_ls_tree_entry(&line2);

                    // Get data.
                    let (_, _, data_a_blob) = get_object_from_repo(hash_a_blob, repo)?;
                    let (_, _, data_b_blob) = get_object_from_repo(hash_b_blob, repo)?;

                    if data_a_blob != data_b_blob {
                        let ancestor_file_hash = hash1;
                        let (_, _, ancestor_data) = get_object_from_repo(ancestor_file_hash, repo)?;
                        if !check_file_merge_conflict(ancestor_data, data_a_blob, data_b_blob) {
                            return Err(io_err!("Conflict"));
                        }
                    }
                }

                (Added, Added, "blob") => {
                    let (_, _, data_a_blob) = get_object_from_repo(hash1, repo)?;
                    let (_, _, data_b_blob) = get_object_from_repo(hash2, repo)?;

                    if data_a_blob != data_b_blob
                        && !check_file_merge_conflict(vec![], data_a_blob, data_b_blob)
                    {
                        return Err(io_err!("Conflict"));
                    }
                }

                (Modified(line1), Modified(line2), "tree") => {
                    // Get the ancestor's state of the tree.
                    let ancestor_tree = ls_tree_from_repo(hash1, repo)?;

                    // Get the current states in both HEAD and the other branch.
                    let (_, _, hash1, _) = parse_ls_tree_entry(&line1);
                    let (_, _, hash2, _) = parse_ls_tree_entry(&line2);
                    let a_tree = ls_tree_from_repo(hash1, repo)?;
                    let b_tree = ls_tree_from_repo(hash2, repo)?;

                    // Calculate their differences.
                    let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &a_tree));
                    let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &b_tree));

                    // Recurse.
                    check_conflicts(diffs1, diffs2, repo)?;
                }

                (Added, Added, "tree") => {
                    // Get the hash's trees.
                    let a_tree = ls_tree_from_repo(hash1, repo)?;
                    let b_tree = ls_tree_from_repo(hash2, repo)?;

                    // Calculate the diffs (they are all adds).
                    let diffs1 = diff_2_map!(diff_tree("", &a_tree));
                    let diffs2 = diff_2_map!(diff_tree("", &b_tree));

                    // Recurse.
                    check_conflicts(diffs1, diffs2, repo)?;
                }

                _ => { /* Every other arm is valid */ }
            }
        }
    }

    Ok(())
}

pub fn validate_merge(repo: &str, base_hash: &str, head_hash: &str) -> io::Result<()> {
    if is_ancestor_from_repo(base_hash, head_hash, repo)? {
        return Ok(());
    }

    // Get the commits' data.
    let (_, _, base_data) = get_object_from_repo(base_hash, repo)?;
    let (_, _, head_data) = get_object_from_repo(head_hash, repo)?;
    let ancestor = get_ancestor_from_repo(&base_data, &head_data, repo)?;
    let (_, _, ancestor_data) = get_object_from_repo(&ancestor, repo)?;

    // Get their tree roots.
    let base_tree_root = get_commit_root(&base_data)?;
    let head_tree_root = get_commit_root(&head_data)?;
    let ancestor_tree_root = get_commit_root(&ancestor_data)?;

    // Get the trees' data.
    let base_tree = ls_tree_from_repo(&base_tree_root, repo)?;
    let head_tree = ls_tree_from_repo(&head_tree_root, repo)?;
    let ancestor_tree = ls_tree_from_repo(&ancestor_tree_root, repo)?;

    // Check if there are any conflicts.
    let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &base_tree));
    let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &head_tree));

    check_conflicts(diffs1, diffs2, repo)
}
