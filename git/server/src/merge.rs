use std::collections::{HashMap, VecDeque};
use std::io::{self, Write};
use utils::index_file::index::Index;
use utils::plumbing::commit::get_time_fmt;
use utils::plumbing::diff::diff_tree::diff_tree;
use utils::plumbing::diff::diff_type::{Diff, DiffType};
use utils::plumbing::diff::lcs::{diff, FileDiff};
use utils::plumbing::hash_object::__hash_object;
use utils::plumbing::ls_tree::{hash_to_str, parse_ls_tree_entry};
use utils::{
    get_ancestor_from_repo, get_branch_from_repo, io_err, read_tree_from_repo,
    update_branch_from_repo, write_tree_from_repo,
};
use utils::{
    is_ancestor_from_repo,
    object::object_db::get_object_from_repo,
    plumbing::{commit::get_commit_root, ls_tree::ls_tree_from_repo},
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

fn merge_changes_into_file(
    mut diff_base: VecDeque<FileDiff>,
    mut diff_head: VecDeque<FileDiff>,
) -> String {
    let mut final_string = vec![];

    loop {
        use FileDiff::*;
        match (diff_base.front(), diff_head.front()) {
            (None, None) => break,

            (Some(Same(line)), None) | (None, Some(Same(line))) => {
                final_string.push(line.clone());
                diff_base.pop_front();
                diff_head.pop_front();
            }

            (Some(Added(line)), None) => {
                final_string.push(line.clone());
                diff_base.pop_front();
            }

            (None, Some(Added(line))) => {
                final_string.push(line.clone());
                diff_head.pop_front();
            }

            (Some(Removed(_)), Some(Same(_))) | (Some(Same(_)), Some(Removed(_))) => {
                diff_base.pop_front();
                diff_head.pop_front();
            }

            (Some(Removed(_)) | None, Some(Removed(_)) | None) => {
                diff_base.pop_front();
                diff_head.pop_front();
            }

            (Some(Same(line)), Some(Same(_))) => {
                final_string.push(line.clone());
                diff_base.pop_front();
                diff_head.pop_front();
            }

            (Some(Added(line)), Some(Same(_))) => {
                final_string.push(line.clone());
                diff_base.pop_front();
            }

            (Some(Same(_)), Some(Added(line))) => {
                final_string.push(line.clone());
                diff_head.pop_front();
            }

            (Some(Added(line)), Some(Added(_))) => {
                // Both lines should be equal.
                final_string.push(line.clone());
                diff_base.pop_front();
                diff_head.pop_front();
            }

            _ => {}
        }
    }

    final_string.join("\n")
}

fn resolve_merge(
    ancestor_data: Vec<u8>,
    base_data: Vec<u8>,
    head_data: Vec<u8>,
) -> io::Result<Vec<u8>> {
    let file_ancestor = String::from_utf8_lossy(&ancestor_data);
    let file_base = String::from_utf8_lossy(&base_data);
    let file_head = String::from_utf8_lossy(&head_data);

    let diff1 = diff(&file_ancestor, &file_base);
    let diff2 = diff(&file_ancestor, &file_head);

    let file_string = merge_changes_into_file(diff1, diff2);
    Ok(file_string.into_bytes())
}

/////
/////      U   R   M   A
/////   U [x] [ ] [x] [ ]
/////   R [ ] [ ] [ ] [ ]
/////   M [x] [ ] [x] [ ]
/////   A [ ] [ ] [ ] [x]
/////
fn refactor_dir(
    diffs1: HashMap<String, Diff>,
    mut diffs2: HashMap<String, Diff>,
    dir: String,
    index: &mut Index,
    repo: &str,
) -> io::Result<()> {
    for (name, diff1) in diffs1 {
        // Build the current
        // path for this object.
        let path = if dir.is_empty() {
            name.clone()
        } else {
            dir.clone() + "/" + &name
        };

        println!(" Diff1 tag: {:?}", diff1.tag);

        // Get the diff from the same file
        // from the other branch.
        if let Some(diff2) = diffs2.remove(&name) {
            let (mode, _, hash1, _) = parse_ls_tree_entry(&diff1.line);
            let (_, otype, hash2, _) = parse_ls_tree_entry(&diff2.line);
            println!("Diff2 tag: {:?}", diff2.tag);

            use DiffType::*;
            match (diff1.tag, diff2.tag, otype) {
                (Unchanged, Unchanged, "blob") => {
                    let (_, _, data) = get_object_from_repo(hash1, repo)?;
                    index.add_from_repo(path, repo, mode, data, false)?;
                }

                (Unchanged, Modified(line), "blob") | (Modified(line), Unchanged, "blob") => {
                    let (_, _, hash, _) = parse_ls_tree_entry(&line);
                    let (_, _, data) = get_object_from_repo(hash, repo)?;
                    index.add_from_repo(path, repo, mode, data, false)?;
                }

                (Added, Added, "blob") => {
                    let (_, _, data1) = get_object_from_repo(hash1, repo)?;
                    let (_, _, data2) = get_object_from_repo(hash2, repo)?;

                    // If the data is the same then just add it.
                    if data1 == data2 {
                        index.add_from_repo(path, repo, mode, data2, false)?;
                    } else {
                        // There shouldn't be any conflicts.
                        let data = resolve_merge(vec![], data1, data2)?;
                        index.add_from_repo(path, repo, mode, data, true)?;
                    }
                }

                (Modified(line1), Modified(line2), "blob") => {
                    let (_, _, hash_base_blob, _) = parse_ls_tree_entry(&line1);
                    let (_, _, hash_head_blob, _) = parse_ls_tree_entry(&line2);

                    // Get data.
                    let (_, _, data_base_blob) = get_object_from_repo(hash_base_blob, repo)?;
                    let (_, _, data_head_blob) = get_object_from_repo(hash_head_blob, repo)?;

                    // If the data is the same then just add it.
                    if data_base_blob == data_head_blob {
                        index.add_from_repo(path, repo, mode, data_head_blob, false)?;
                    } else {
                        // There shouldn't be any conflicts.
                        let ancestor_file_hash = hash1;
                        let (_, _, ancestor_data) = get_object_from_repo(ancestor_file_hash, repo)?;
                        let data = resolve_merge(ancestor_data, data_base_blob, data_head_blob)?;
                        index.add_from_repo(path, repo, mode, data, true)?;
                    }
                }

                (Unchanged, Unchanged, "tree") => {
                    let index2 = read_tree_from_repo(hash1, &path, repo)?;
                    index2.merge(index);
                }

                (Unchanged, Modified(line), "tree") | (Modified(line), Unchanged, "tree") => {
                    let (_, _, hash, _) = parse_ls_tree_entry(&line);
                    let index2 = read_tree_from_repo(hash, &path, repo)?;
                    index2.merge(index);
                }

                (Modified(line1), Modified(line2), "tree") => {
                    let ancestor_tree = ls_tree_from_repo(hash1, repo)?;

                    let (_, _, hash1, _) = parse_ls_tree_entry(&line1);
                    let (_, _, hash2, _) = parse_ls_tree_entry(&line2);
                    let base_tree = ls_tree_from_repo(hash1, repo)?;
                    let head_tree = ls_tree_from_repo(hash2, repo)?;

                    let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &base_tree));
                    let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &head_tree));

                    refactor_dir(diffs1, diffs2, path, index, repo)?;
                }

                _ => {}
            }
        } else {
            // The file was Added in the base branch.
            let (mode, otype, hash, _) = parse_ls_tree_entry(&diff1.line);
            println!("Otype: {otype}");
            match otype {
                "blob" => {
                    let (_, _, data) = get_object_from_repo(hash, repo)?;
                    index.add_from_repo(path, repo, mode, data, false)?;
                }

                "tree" => {
                    let index2 = read_tree_from_repo(hash, &path, repo)?;
                    index2.merge(index);
                }

                _ => return Err(io_err!("Invalid object type.")),
            }
        }
        println!("\n");
    }

    for (_, diff2) in diffs2 {
        println!("Diff2 tag: {:?}", diff2.tag);

        let (mode, otype, hash, name) = parse_ls_tree_entry(&diff2.line);
        println!("Diff2: otype: {otype}\n\n");
        let path = if dir.is_empty() {
            name.clone()
        } else {
            dir.clone() + "/" + &name
        };

        match otype {
            "blob" => {
                let (_, _, data) = get_object_from_repo(hash, repo)?;
                index.add_from_repo(path, repo, mode, data, false)?;
            }

            "tree" => {
                let index2 = read_tree_from_repo(hash, &path, repo)?;
                index2.merge(index);
            }

            _ => return Err(io_err!("Invalid object type.")),
        }
    }

    Ok(())
}

fn merge_commit(
    parent1: &str,
    parent2: &str,
    msg: &str,
    index: Index,
    repo: &str,
) -> io::Result<String> {
    let root = hash_to_str(&write_tree_from_repo(index, repo)?);

    let mut commit = vec![];
    commit.write_all(format!("tree {root}\n").as_bytes())?;

    commit.write_all(format!("parent {parent1}\n").as_bytes())?;
    commit.write_all(format!("parent {parent2}\n").as_bytes())?;

    let author = "server <server@fi.uba.ar>";
    let time = get_time_fmt(chrono::Local::now());
    commit.write_all(format!("author {author} {time}\n").as_bytes())?;
    commit.write_all(format!("committer {author} {time}\n").as_bytes())?;

    commit.write_all(format!("\n{msg}\n").as_bytes())?;

    Ok(__hash_object(&commit, "commit", true, repo)?.1)
}

pub fn merge_pr(repo: &str, base: &str, head: &str) -> io::Result<String> {
    let base_hash = get_branch_from_repo(base, repo)?;
    let head_hash = get_branch_from_repo(head, repo)?;

    if is_ancestor_from_repo(&base_hash, &head_hash, repo)? {
        println!("Fast-Forward");
        // Move base branch pointer to head commit.
        update_branch_from_repo(base, &head_hash, repo)?;
        Ok(head_hash)
    } else {
        println!("3-Way Merge");
        // Get base and head commits.
        let (_, _, base_data) = get_object_from_repo(&base_hash, repo)?;
        let (_, _, head_data) = get_object_from_repo(&head_hash, repo)?;
        let ancestor = get_ancestor_from_repo(&base_data, &head_data, repo)?;
        let (_, _, ancestor_data) = get_object_from_repo(&ancestor, repo)?;

        // Get trees' commit hashes.
        let base_tree_root = get_commit_root(&base_data)?;
        let head_tree_root = get_commit_root(&head_data)?;
        let ancestor_tree_root = get_commit_root(&ancestor_data)?;

        // Get trees' data.
        let base_tree = ls_tree_from_repo(&base_tree_root, repo)?;
        let head_tree = ls_tree_from_repo(&head_tree_root, repo)?;
        let ancestor_tree = ls_tree_from_repo(&ancestor_tree_root, repo)?;

        // Merge trees.
        let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &base_tree));
        let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &head_tree));

        let mut index = Index::new();
        refactor_dir(diffs1, diffs2, "".to_string(), &mut index, repo)?;

        println!("SALIMOS DE REFACTOR_DIR");

        let msg = format!("Merge {head} into {base}");
        let commit = merge_commit(&base_hash, &head_hash, &msg, index, repo)?;

        // Move base branch pointer to head commit.
        update_branch_from_repo(base, &commit, repo)?;
        Ok(commit)
    }
}
