use std::{fs, io, path::Path};

use super::{commands::read_index, work_dir::parse_work_dir};

// Patterns
// begins with a /, has a / in the middle, or both ---> the pattern is relavite to the .gitignore file location
// if it ends with a /, then the pattern only matches directories. If it doesnt end with / it may match any type of direntry
// /removes entries set to be ignored by patterns set in the .gitignore file

/// Example: if the pattern "example_name" is set in the .gitignore file, in a working directory containign the following
/// entrances: /example_name/file.txt  /example_name/folder/file2.txt /file3.txt /example_name.txt
/// only file3.txt and example_name.txt would be tracked
// pub fn remove_ignored(paths: Vec<String>) -> io::Result<Vec<String>> {
//     let mut unignored_files = paths.clone();
//     let patterns = get_ignore_patterns()?;

//     //for each pattern, remove all the files that match said pattern
//     for pattern in patterns {
//         unignored_files = filter_with_pattern(unignored_files, pattern.as_str())?;
//     }

//     Ok(unignored_files)
// }

fn get_ignore_patterns(path: String) -> io::Result<Vec<String>> {
    let mut patterns: Vec<String> = Vec::new();

    if !Path::new(&path).exists() {
        return Ok(patterns);
    }

    let gitignore_content = fs::read_to_string(path)?;

    //filter empty lines, comments (which start with '#'), and trims empty spaces
    patterns = gitignore_content
        .split('\n')
        .filter(|entry| !entry.is_empty() && !entry.starts_with('#'))
        .map(|entry| entry.trim().to_string())
        .collect();

    Ok(patterns)
}

/// receives the paths and the filtering pattens. Returns all the paths that match any of the patterns
fn filter_with_patterns(
    paths: Vec<String>,
    patterns: Vec<String>,
) -> io::Result<(Vec<String>, Vec<String>)> {
    let mut ignored_files: Vec<String> = Vec::new();
    let mut not_ingored_files: Vec<String> = Vec::new();

    if patterns.is_empty() {
        return Ok((ignored_files, paths));
    }

    'path_loop: for path in &paths {
        for pattern in &patterns {
            for path_entry in path.split('/') {
                if path_entry == pattern.as_str() && !ignored_files.contains(path) {
                    ignored_files.push(path.clone());
                    continue 'path_loop;
                }
            }
        }
        not_ingored_files.push(path.clone());
    }

    Ok((ignored_files, not_ingored_files))
}

/// underlying implementation of files_ignored
/// the prefix parameter represents the part of the path removed in a previous call to the paths in curr_dir
/// so that all curr_dir paths are relative to the folder we are searching in the current call
fn __set_to_be_ignored(curr_dir: Vec<String>, prefix: String) -> io::Result<Vec<String>> {
    let mut ignore_patterns = Vec::new();

    if curr_dir.contains(&".gitignore".to_string()) {
        let path_format = format!("{}.gitignore", prefix);

        ignore_patterns = get_ignore_patterns(path_format)?;
    }
    let (mut ignored, not_ignored) = filter_with_patterns(curr_dir, ignore_patterns)?;
    for relative_path in &mut ignored {
        relative_path.insert_str(0, prefix.as_str());
    }

    let mut i = 0;
    while i < not_ignored.len() {
        // Check if path contains '/'. In that case, we create a
        // new array of paths for the current sub-directory
        if let Some(index) = not_ignored[i].find('/') {
            let folder_name = &not_ignored[i][..index];
            // Iterate over entries until we find one that is
            // not in the same sub-directory.
            let mut sub_dir = Vec::new();
            while i < not_ignored.len() {
                if not_ignored[i].starts_with(folder_name) {
                    // Remove this directory's name from the entry's path.
                    sub_dir.push(not_ignored[i][index + 1..].to_string());
                    i += 1;
                } else {
                    break;
                }
            }
            let filtered_sub_directory =
                __set_to_be_ignored(sub_dir, format!("{}{}/", prefix, folder_name))?;
            // add all ignored files in sub directory
            for path in &filtered_sub_directory {
                if !ignored.contains(path) {
                    ignored.push(path.clone());
                }
            }
        }

        i += 1;
    }

    Ok(ignored)
}

/// returns a list of all the files in the working directory that are set to be
/// ignored in any .gitignore file, that were not being already tracked
pub fn set_to_be_ignored() -> io::Result<Vec<String>> {
    let mut working_dir = parse_work_dir()?;
    working_dir.sort();
    __set_to_be_ignored(working_dir, "".to_string())
}

pub fn files_not_ignored() -> io::Result<Vec<String>> {
    let ignored_files = set_to_be_ignored()?;

    let index = read_index().unwrap_or_default();
    let working_dir = parse_work_dir()?;
    let mut res = vec![];

    for path in working_dir {
        if index.contains_key(&path) || !ignored_files.contains(&path) {
            res.push(path);
        }
    }

    Ok(res)
}
