use super::super::commands::ls_tree;
use crate::io_err;
use std::{
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};
use utils::{object::object_db::get_object, plumbing::ls_tree::parse_ls_tree_entry};

// Underlying implementation of parse_work_dir.
fn __parse_work_dir(path: &PathBuf, cwdlen: usize) -> io::Result<Vec<String>> {
    let mut paths = vec![];

    for entry in fs::read_dir(path)?.flatten() {
        let path = entry.path();

        if path.is_dir() {
            match path.file_name() {
                Some(name) if name != ".git" => paths.append(&mut __parse_work_dir(&path, cwdlen)?),
                _ => {}
            }
        } else {
            let path = path.to_string_lossy();
            let path = match path.get(cwdlen..) {
                Some(path) => path,
                None => continue,
            };

            let path = path.trim_start_matches('/');
            paths.push(path.to_string());
        }
    }

    Ok(paths)
}

/// Returns a vector of paths to every file in the working directory.
pub fn parse_work_dir() -> io::Result<Vec<String>> {
    let path = std::env::current_dir()?;
    __parse_work_dir(&path, path.to_string_lossy().len())
}

/// Adds all the entries inside this hash's tree to the path.
pub fn directify_tree(hash: &str, path: &str) -> io::Result<()> {
    let tree = ls_tree(hash)?;

    for line in tree.lines() {
        let (_, otype, hash, name) = parse_ls_tree_entry(line);
        let path = format!("{}/", path) + &name;
        match otype {
            "blob" => {
                let mut file = File::create(&path)?;
                let (_, _, data) = get_object(hash)?;
                file.write_all(&data)?;
            }

            "tree" => {
                fs::create_dir(&path)?;
                directify_tree(hash, &path)?;
            }

            _ => return Err(io_err!("Invalid object type")),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn dir_parsing() {
        let path = std::env::current_dir().unwrap();
        let cwdlen = path.to_string_lossy().len();
        let files = __parse_work_dir(&path, cwdlen).unwrap();
        println!("{:?}", files);
    }
}
