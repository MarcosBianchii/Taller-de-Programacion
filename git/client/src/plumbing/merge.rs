use super::{
    super::plumbing::{
        checkout::refactor_root_dir, commands::*, commit::get_parent_commits,
        work_dir::directify_tree,
    },
    commit::*,
    diff::{
        diff_tree::diff_tree,
        diff_type::*,
        lcs::{diff, FileDiff},
    },
    refs::get_ref,
};
use crate::ui::keep_or_remove_conflict::keep_or_remove_window::{
    KeepOrRemoveResult, KeepRemoveWindow,
};
use crate::{io_err, ui::conflicts_window::GtkConflict};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs::{self, File},
    io::{self, Write},
};
use utils::index_file::index::Index;
use utils::object::object_db::get_object;
use utils::plumbing::commit::get_commit_root;
use utils::plumbing::ls_tree::{hash_to_str, ls_tree, parse_ls_tree_entry};

#[macro_export]
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

// Returns true if a is ancestor of b.
// `a` and `b` are hashes to their respective commits.
//
// commits: [] <- ... <- [a] <- ... <- [b] <- ... <- []
//
fn __is_ancestor(a: &str, b: &str, steps: &mut HashSet<String>) -> io::Result<bool> {
    // Check if we have already visited b.
    if steps.contains(b) {
        return Ok(false);
    }

    // Added b to steps.
    steps.insert(b.to_string());

    // Check if a is b.
    if a == b {
        return Ok(true);
    }

    // Get b's data.
    let (_, _, data) = get_object(b)?;
    match get_parent_commits(&data) {
        Some(parents) => {
            for parent in parents {
                if __is_ancestor(a, &parent, steps)? {
                    return Ok(true);
                }
            }

            Ok(false)
        }
        None => Ok(false),
    }
}

// Returns true if a is ancestor of b.
pub fn is_ancestor(a: &str, b: &str) -> io::Result<bool> {
    __is_ancestor(a, b, &mut HashSet::new())
}

/// Gets the common ancestor of two commits.
fn __get_ancestor(
    mut a: Vec<u8>,
    mut b: Vec<u8>,
    steps1: &mut HashSet<String>,
    steps2: &mut HashSet<String>,
) -> io::Result<String> {
    loop {
        let parents1 = get_parent_commits(&a);
        let parents2 = get_parent_commits(&b);

        if let Some(parents) = parents1 {
            for parent in parents {
                if steps2.contains(&parent) {
                    return Ok(parent);
                }

                steps1.insert(parent.clone());
                let (_, _, data) = get_object(&parent)?;
                a = data;
            }
        }

        if let Some(parents) = parents2 {
            for parent in parents {
                if steps1.contains(&parent) {
                    return Ok(parent);
                }

                steps2.insert(parent.clone());
                let (_, _, data) = get_object(&parent)?;
                b = data;
            }
        }
    }
}

/// Returns the common ancestor of two commits.
pub fn get_ancestor(a: &[u8], b: &[u8]) -> io::Result<String> {
    __get_ancestor(
        a.to_vec(),
        b.to_vec(),
        &mut HashSet::new(),
        &mut HashSet::new(),
    )
}

// Creates a merge commit in the database and returns it's hash.
fn merge_commit(parent1: &str, parent2: &str, msg: &str) -> io::Result<String> {
    let root = hash_to_str(&write_tree()?);

    let mut commit = vec![];
    commit.write_all(b"tree ")?;
    commit.write_all(root.as_bytes())?;
    commit.write_all(b"\n")?;

    commit.write_all(format!("parent {parent1}\n").as_bytes())?;
    commit.write_all(format!("parent {parent2}\n").as_bytes())?;

    let author = get_userconfig()?.to_string();
    let time = get_time_fmt(chrono::Local::now());
    commit.write_all(format!("author {author} {time}\ncommitter {author} {time}\n").as_bytes())?;

    commit.write_all(format!("\n{msg}\n").as_bytes())?;

    hash_object(&commit, "commit", true)
}

// Call the interface to solve the conflict.
// It opens a window and shows the user the file with the conflicts to solve.
// The user can edit the file and save it with a button
// The function returns the content of the file after the user has saved it
#[allow(dead_code, unused_variables)]
fn solve_conflict(file: &str, file_path: &str) -> io::Result<()> {
    let app = GtkConflict::new(file.to_string(), file_path.to_string())?;
    app.run();
    Ok(())
}

// Call the interface to solve the conflict.
// It opens a window and shows the user two buttons. Keep or mantein the file.
#[allow(dead_code, unused_variables)]
fn solve_keep_or_remove_conflict(file_path: &str) -> io::Result<KeepOrRemoveResult> {
    match KeepRemoveWindow::new(file_path.to_string()) {
        Ok(window) => {
            window.run();
            Ok(window.get_custom_state())
        }
        Err(err) => Ok(KeepOrRemoveResult::Error),
    }
}

// Creates a conflict block.
fn conflict_block(head: &str, other: &str, other_branch_name: &str) -> String {
    format!("\n<<<<<<< HEAD\n{head}\n=======\n{other}\n>>>>>>> {other_branch_name}\n")
}

/// Merges the changes from both branches and returns the result.
/// There are two possible outcomes:
/// 1. The merge is successful and the result is the new file merged.
/// 2. The merge is not successful and the result is the file with the conflicts.
///   The conflicts are marked with the following format:
///
///
/// <<<<<<< HEAD
/// <content of the file in HEAD>
/// =======
/// <content of the file in the other branch>
/// >>>>>>> <other branch name>
///
/// If there are conflicts, the function returns the number of conflicts.
/// If there are no conflicts, the function returns 0.
/// The function also returns the final file.
/// The function receives the ancestor file, the file in HEAD and the file in the other branch.
/// The function also receives the name of the other branch.
///
/////    S  A  R
///// S [x][x][x]
///// A [x][x][x]
///// R [x][x][x]
///
fn merge_changes_into_file(
    mut diff_head: VecDeque<FileDiff>,
    mut diff_other: VecDeque<FileDiff>,
    other_branch_name: &str,
) -> (String, usize) {
    let mut final_file = vec![];
    let mut building_block = false;
    let mut conflicts = 0;

    let mut head_sublock = vec![];
    let mut other_sublock = vec![];

    loop {
        use FileDiff::*;
        match (diff_head.front(), diff_other.front()) {
            // If both diffs are empty, break.
            (None, None) => {
                if building_block {
                    let block = conflict_block(
                        &head_sublock.join("\n"),
                        &other_sublock.join("\n"),
                        other_branch_name,
                    );

                    final_file.push(block);
                }

                break;
            }

            // Add the non-emtpy diff.
            (Some(Same(line)), None) | (None, Some(Same(line))) => {
                if building_block {
                    let block = conflict_block(
                        &head_sublock.join("\n"),
                        &other_sublock.join("\n"),
                        other_branch_name,
                    );

                    final_file.push(block);
                    building_block = false;
                    conflicts += 1;
                    head_sublock.clear();
                    other_sublock.clear();
                }

                final_file.push(line.clone());
                diff_head.pop_front();
                diff_other.pop_front();
            }

            // Add the non-emtpy diff.
            (Some(Added(line)), None) => {
                if building_block {
                    head_sublock.push(line.clone());
                } else {
                    final_file.push(line.clone());
                }

                diff_head.pop_front();
            }

            // Add the non-emtpy diff.
            (None, Some(Added(line))) => {
                if building_block {
                    other_sublock.push(line.clone());
                } else {
                    final_file.push(line.clone());
                }

                diff_other.pop_front();
            }

            // Ignore removed lines.
            (Some(Removed(_)) | None, Some(Removed(_)) | None) => {
                diff_head.pop_front();
                diff_other.pop_front();
            }

            // If both lines are the same, add one of them.
            (Some(Same(line)), Some(Same(_))) => {
                if building_block {
                    let block = conflict_block(
                        &head_sublock.join("\n"),
                        &other_sublock.join("\n"),
                        other_branch_name,
                    );

                    final_file.push(block);
                    building_block = false;
                    conflicts += 1;
                    head_sublock.clear();
                    other_sublock.clear();
                }

                final_file.push(line.clone());
                diff_head.pop_front();
                diff_other.pop_front();
            }

            // Add in HEAD.
            (Some(Same(line)), Some(Removed(_))) => {
                // -- CONFLICT --
                building_block = true;

                // Push the line to the sublock.
                head_sublock.push(line.clone());
                diff_head.pop_front();
                diff_other.pop_front();
            }

            // Add in other.
            (Some(Removed(_)), Some(Same(line))) => {
                // -- CONFLICT --
                building_block = true;

                // Push the line to the sublock.
                other_sublock.push(line.clone());
                diff_head.pop_front();
                diff_other.pop_front();
            }

            // Added in HEAD.
            (Some(Added(line)), Some(Same(_))) => {
                if building_block {
                    head_sublock.push(line.clone());
                } else {
                    final_file.push(line.clone());
                }

                diff_head.pop_front();
            }

            // Added in other.
            (Some(Same(_)), Some(Added(line))) => {
                if building_block {
                    other_sublock.push(line.clone());
                } else {
                    final_file.push(line.clone());
                }

                diff_other.pop_front();
            }

            // Add in HEAD.
            (Some(Added(line)), Some(Removed(_))) => {
                // -- CONFLICT --
                building_block = true;

                // Push the line to the sublock.
                head_sublock.push(line.clone());
                diff_head.pop_front();
            }

            // Add in other.
            (Some(Removed(_)), Some(Added(line2))) => {
                // -- CONFLICT --
                building_block = true;

                // Push the line to the sublock.
                other_sublock.push(line2.clone());
                diff_other.pop_front();
            }

            // Both added.
            (Some(Added(line1)), Some(Added(line2))) => {
                if line1 == line2 {
                    if building_block {
                        let block = conflict_block(
                            &head_sublock.join("\n"),
                            &other_sublock.join("\n"),
                            other_branch_name,
                        );

                        final_file.push(block);
                        building_block = false;
                        conflicts += 1;
                        head_sublock.clear();
                        other_sublock.clear();
                    }

                    final_file.push(line1.clone());
                    diff_head.pop_front();
                    diff_other.pop_front();
                } else {
                    // -- CONFLICT --
                    building_block = true;

                    // Fill head_sublock.
                    while let Some(Added(line)) = diff_head.front() {
                        head_sublock.push(line.clone());
                        diff_head.pop_front();
                    }

                    // Fill other_sublock.
                    while let Some(Added(line)) = diff_other.front() {
                        other_sublock.push(line.clone());
                        diff_other.pop_front();
                    }
                }
            }
        }
    }

    (final_file.join("\n"), conflicts)
}

// Checks for conflicts and calls solve_conflict if one is detected.
#[allow(unused_variables)]
fn resolve_merge(
    ancestor: Vec<u8>,
    data_head: Vec<u8>,
    data_other: Vec<u8>,
    other_branch_commit: &str,
    file_path: &str,
) -> io::Result<()> {
    let file_ancestor = String::from_utf8_lossy(&ancestor);
    let file_head = String::from_utf8_lossy(&data_head);
    let file_other = String::from_utf8_lossy(&data_other);

    let diff1 = diff(&file_ancestor, &file_head);
    let diff2 = diff(&file_ancestor, &file_other);

    // Merge the changes and check for conflicts.
    let (file_string, conflicts) = merge_changes_into_file(diff1, diff2, other_branch_commit);

    if conflicts > 0 {
        solve_conflict(&file_string, file_path)?;
    } else {
        let mut file = File::create(file_path)?;
        file.write_all(file_string.as_bytes())?;
    }

    Ok(())
}

/// Applies the given diffs to the working directory.
///
/// Unchanged  ->  (Unchanged | Removed | Modified) & !Added
/// Removed    ->  (Unchanged | Removed | Modified) & !Added
/// Modified   ->  (Unchanged | Removed | Modified) & !Added
/// Addeded    -> !(Unchanged | Removed | Modified) & (Added | Non-existant)
///
/////
/////      U   R   M   A
/////   U [ ] [x] [x] [ ]
/////   R [x] [ ] [x] [ ]
/////   M [ ] [x] [x] [ ]
/////   A [ ] [ ] [ ] [x]
/////
pub fn refactor_dir(
    diffs1: HashMap<String, Diff>,
    mut diffs2: HashMap<String, Diff>,
    dir: String,
    other_branch_commit: &str,
    index: &mut Index,
) -> io::Result<()> {
    for (name, diff1) in diffs1 {
        // Build the current
        // path for this object.
        let path = if dir.is_empty() {
            name.clone()
        } else {
            dir.clone() + "/" + &name
        };

        // Get the diff from the same file
        // from the other branch.
        if let Some(diff2) = diffs2.remove(&name) {
            // This implementation assumes the type of
            // the object didn't change in bewteen branches.
            let (_, _, hash1, _) = parse_ls_tree_entry(&diff1.line);
            let (_, otype, hash2, _) = parse_ls_tree_entry(&diff2.line);

            use DiffType::*;
            match (diff1.tag, diff2.tag, otype) {
                (Unchanged, Removed, "blob") => {
                    fs::remove_file(&path)?;
                    index.remove(&path);
                }

                // If the file was modified in the other branch but
                // not in this one then bring the file from the other branch.
                (Unchanged, Modified(line), "blob") => {
                    let (_, _, hash, _) = parse_ls_tree_entry(&line);
                    let (_, _, data) = get_object(hash)?;
                    let mut file = File::create(&path)?;
                    file.write_all(&data)?;
                    index.add(path, false, true)?;
                }

                (Removed, Modified(line), "blob") => match solve_keep_or_remove_conflict(&path)? {
                    KeepOrRemoveResult::Delete => { /* File doesn't exist in workspace */ }
                    KeepOrRemoveResult::Keep => {
                        let (_, _, hash, _) = parse_ls_tree_entry(&line);
                        let (_, _, data) = get_object(hash)?;

                        // Write the file.
                        let mut file = File::create(&path)?;
                        file.write_all(&data)?;
                        index.add(path, false, true)?;
                    }

                    KeepOrRemoveResult::Error => {
                        return Err(io_err!("Error with merging KeepOrRemove"))
                    }
                },

                (Modified(_), Removed, "blob") => match solve_keep_or_remove_conflict(&path)? {
                    KeepOrRemoveResult::Keep => { /* File already exists in workspace */ }
                    KeepOrRemoveResult::Delete => {
                        fs::remove_file(&path)?;
                        index.remove(&path);
                    }

                    KeepOrRemoveResult::Error => {
                        return Err(io_err!("Error with merging KeepOrRemove"))
                    }
                },

                // If the file was added in both branches
                // then resolve possible conflict.
                (Added, Added, "blob") => {
                    let (_, _, data1) = get_object(hash1)?;
                    let (_, _, data2) = get_object(hash2)?;

                    // If the data is the same then
                    // just write it to the file.
                    if data1 == data2 {
                        let mut file = File::create(&path)?;
                        file.write_all(&data1)?;
                    } else {
                        // Check for conflict and
                        // resolve it (if any).

                        // Ancestor data is an empty vec due to the
                        // fact both branches added the file.
                        resolve_merge(vec![], data1, data2, other_branch_commit, &path)?;
                    }

                    // Add it to index.
                    index.add(path, false, true)?;
                }

                // If the file was modified in both branches
                // then resolve possible conflict.
                (Modified(line1), Modified(line2), "blob") => {
                    let (_, _, hash_head_blob, _) = parse_ls_tree_entry(&line1);
                    let (_, _, hash_other_blob, _) = parse_ls_tree_entry(&line2);

                    // Get data.
                    let (_, _, data_head_blob) = get_object(hash_head_blob)?;
                    let (_, _, data_other_blob) = get_object(hash_other_blob)?;

                    // If the data is the same then
                    // just write it to the file.
                    if data_head_blob == data_other_blob {
                        let mut file = File::create(&path)?;
                        file.write_all(&data_head_blob)?;
                    } else {
                        // Check for conflict and
                        // resolve it (if any).
                        let ancestor_file_hash = hash1;
                        let (_, _, ancestor_data) = get_object(ancestor_file_hash)?;
                        resolve_merge(
                            ancestor_data,
                            data_head_blob,
                            data_other_blob,
                            other_branch_commit,
                            &path,
                        )?;
                    }

                    // Add it to index.
                    index.add(path, false, true)?;
                }

                (Unchanged, Removed, "tree") => {
                    let _ = fs::remove_dir_all(&path);
                    index.remove(&path);
                }

                // If the directory was modified in the other branch
                // but not in this one then bring the directory from the other branch.
                (Unchanged, Modified(line), "tree") => {
                    let (_, _, hash, _) = parse_ls_tree_entry(&line);
                    directify_tree(hash, &path)?;

                    // Generate an index from the tree.
                    let index2 = read_tree(hash, &path)?;
                    index2.merge(index);
                }

                (Removed, Modified(line), "tree") => match solve_keep_or_remove_conflict(&path)? {
                    KeepOrRemoveResult::Delete => { /* Dir doesn't exist in workspace */ }
                    KeepOrRemoveResult::Keep => {
                        let (_, _, hash, _) = parse_ls_tree_entry(&line);
                        directify_tree(hash, &path)?;

                        // Generate an index from the tree.
                        let index2 = read_tree(hash, &path)?;
                        index2.merge(index);
                    }

                    KeepOrRemoveResult::Error => {
                        return Err(io_err!("Error with merging KeepOrRemove"))
                    }
                },

                (Modified(_), Removed, "tree") => match solve_keep_or_remove_conflict(&path)? {
                    KeepOrRemoveResult::Keep => { /* Dir already exists in workspace */ }
                    KeepOrRemoveResult::Delete => {
                        fs::remove_dir_all(&path)?;
                        index.remove(&path);
                    }

                    KeepOrRemoveResult::Error => {
                        return Err(io_err!("Error with merging KeepOrRemove"))
                    }
                },

                // Same for added.
                (Added, Added, "tree") => {
                    fs::create_dir_all(&path)?;

                    // Get the hash's trees.
                    let head_tree = ls_tree(hash1)?;
                    let other_tree = ls_tree(hash2)?;

                    // Calculate the diffs (they are all adds).
                    let diffs1 = diff_2_map!(diff_tree("", &head_tree));
                    let diffs2 = diff_2_map!(diff_tree("", &other_tree));

                    // Recurse.
                    refactor_dir(diffs1, diffs2, path, other_branch_commit, index)?;
                }

                // If both were modified then recurse.
                (Modified(line1), Modified(line2), "tree") => {
                    // Get the ancestor's state of the tree.
                    let ancestor_tree = ls_tree(hash1)?;

                    // Get the current states in both HEAD and the other branch.
                    let (_, _, hash1, _) = parse_ls_tree_entry(&line1);
                    let (_, _, hash2, _) = parse_ls_tree_entry(&line2);
                    let head_tree = ls_tree(hash1)?;
                    let other_tree = ls_tree(hash2)?;

                    // Calculate their differences.
                    let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &head_tree));
                    let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &other_tree));

                    // Recurse.
                    refactor_dir(diffs1, diffs2, path, other_branch_commit, index)?;
                }

                _ => {}
            }
        }
    }

    // Add the files that were added in the other branch.
    for (_, diff2) in diffs2 {
        let (_, otype, hash, name) = parse_ls_tree_entry(&diff2.line);
        let path = if dir.is_empty() {
            name.clone()
        } else {
            dir.clone() + "/" + &name
        };

        match otype {
            "blob" => {
                let (_, _, data) = get_object(hash)?;
                let mut file = File::create(&path)?;
                file.write_all(&data)?;
                index.add(name, false, true)?;
            }

            "tree" => {
                directify_tree(hash, &path)?;

                // Generate an index from the tree.
                let index2 = read_tree(hash, &path)?;
                index2.merge(index);
            }

            _ => return Err(io_err!("invalid object type")),
        }
    }

    Ok(())
}

/// Underlying implementation of `git merge`.
pub fn __merge(branch: &str, subfolder: &str) -> io::Result<()> {
    // Get the merge's commit hash.
    let refs = get_ref(branch, subfolder)?;

    // Get the current HEAD commit hash.
    let head = match get_head() {
        Some(hash) => hash,
        None => return Err(io_err!("HEAD is not pointing to any commit")),
    };

    // Get HEAD's and ref's data.
    let (_, _, head_data) = get_object(&head)?;
    let (_, _, refs_data) = get_object(&refs)?;

    // Get their root trees.
    let head_tree_root = get_commit_root(&head_data)?;
    let refs_tree_root = get_commit_root(&refs_data)?;

    // Get their String representation.
    let head_tree = ls_tree(&head_tree_root)?;
    let refs_tree = ls_tree(&refs_tree_root)?;

    if is_ancestor(&head, &refs)? {
        println!("Fast-Forward");
        // Calculate diffs and refactor dir.
        let diffs: Vec<_> = diff_tree(&head_tree, &refs_tree).collect();
        refactor_root_dir(diffs, ".")?;

        // Update index.
        let index = read_tree(&refs_tree_root, "")?;
        write_index(index)?;

        // Update head to point to branch's commit.
        update_head(&refs)?;
    } else {
        println!("3-Way-Merge");
        // Get the common ancestor tree.
        let ancestor = get_ancestor(&head_data, &refs_data)?;
        let (_, _, ancestor_data) = get_object(&ancestor)?;
        let ancestor_tree_root = get_commit_root(&ancestor_data)?;

        // Get string representation.
        let ancestor_tree = ls_tree(&ancestor_tree_root)?;

        // Calculate the differences between the ancestor and both branches.
        let diffs1 = diff_2_map!(diff_tree(&ancestor_tree, &head_tree));
        let diffs2 = diff_2_map!(diff_tree(&ancestor_tree, &refs_tree));

        // Apply the changes to the working directory and update index.
        let mut index = read_index().unwrap_or_default();
        refactor_dir(diffs1, diffs2, "".to_string(), &refs, &mut index)?;
        write_index(index)?;

        // Create merge commit.
        let cur_branch = get_cur_branch()?;
        let msg = format!("Merge {branch} into {cur_branch}");
        let commit = merge_commit(&head, &refs, &msg)?;

        // Update HEAD.
        update_head(&commit)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn merge_and_show_patch(o: &str, a: &str, b: &str) {
        let diff1 = diff(o, a);
        let diff2 = diff(o, b);

        println!("diff1: {:#?}\n\n", diff1);
        println!("diff2: {:#?}\n\n", diff2);

        let (file, _) = merge_changes_into_file(diff1, diff2, "origin");

        println!("{file}");
    }

    #[test]
    fn merge1() {
        let a = r#"Lorem ipsum dolor sit amet
consectetur adipiscing elit. Donec
jaimito
pablito
pedrito"#;

        let b = r#"Lorem ipsum dolor sit amet
consectetur adipiscing elit. Donec
lobortis pellentesque metus eu accumsan.
Pellentesque sed varius dolor, sed pharetra felis.
pepito
Nunc nunc lorem, finibus id molestie sed"#;

        let o = r#"Lorem ipsum dolor sit amet
consectetur adipiscing elit. Donec
lobortis pellentesque metus eu accumsan.
Donec sed massa vitae urna elementum sagittis.
Pellentesque sed varius dolor, sed pharetra felis.
Nunc nunc lorem, finibus id molestie sed"#;

        merge_and_show_patch(o, a, b);
    }

    #[test]
    fn merge2() {
        let a = r#"Lorem ipsum dolor sit amet
pablito
pedrito
juancito
Donec sed massa vitae urna elementum sagittis.
jaimito
"#;
        let b = r#"consectetur adipiscing elit. Donec
consectetur adipiscing elit. Donec
pedrito
Donec sed massa vitae urna elementum sagittis.
Pellentesque sed varius dolor, sed pharetra felis.
"#;

        let o = r#"Lorem ipsum dolor sit amet
consectetur adipiscing elit. Donec
lobortis pellentesque metus eu accumsan.
Donec sed massa vitae urna elementum sagittis.
Pellentesque sed varius dolor, sed pharetra felis.
Nunc nunc lorem, finibus id molestie sed"#;

        merge_and_show_patch(o, a, b);
    }

    #[test]
    fn merge3() {}
}
