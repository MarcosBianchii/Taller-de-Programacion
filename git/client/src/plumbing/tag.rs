use utils::object::object_db::get_object;

use crate::{commands::TagCommand, io_err, plumbing::commands::get_head};
use std::{fs, io, path::PathBuf};

use super::{
    commands::{get_userconfig, hash_object},
    commit::get_time_fmt,
};

const TAGS_DIR: &str = ".git/refs/tags";

/// Returns an iterator over tuples (hash, tag_name) for every tag in the repo.
pub fn get_tags_with_offset(offset: &str) -> io::Result<impl Iterator<Item = (String, String)>> {
    let dir = fs::read_dir(format!("{offset}/{TAGS_DIR}"))?;
    let mut tags = vec![];

    for entry in dir.flatten() {
        let path = entry.path();
        let name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        let hash = fs::read_to_string(path)?.trim().to_string();
        tags.push((hash, name));
    }

    Ok(tags.into_iter())
}

/// Returns an iterator over tuples (hash, tag_name) for every tag in the repo.
pub fn get_tags() -> io::Result<impl Iterator<Item = (String, String)>> {
    get_tags_with_offset(".")
}

fn build_tag(hash: &str, msg: &str, otype: &str, name: &str) -> io::Result<String> {
    let tagger = get_userconfig()?.to_string();
    let time = get_time_fmt(chrono::Local::now());

    let object =
        format!("object {hash}\ntype {otype}\ntag {name}\ntagger {tagger} {time}\n\n{msg}\n");

    hash_object(object.as_bytes(), "tag", true)
}

/// Underlying implementation of `git tag`.
pub fn __tag(cmd: TagCommand) -> io::Result<Option<Vec<String>>> {
    use TagCommand::*;
    match cmd {
        List => Ok(Some(get_tags()?.map(|(_, name)| name).collect())),

        Add { name, hash, msg } => {
            let path = format!("{TAGS_DIR}/{name}");

            // If the tag already exists, exit.
            if fs::metadata(path).is_ok() {
                return Err(io_err!("tag already exists"));
            }

            __tag(AddForce { name, hash, msg })
        }

        AddForce { name, hash, msg } => {
            let hash = match hash {
                Some(hash) => hash,
                None => get_head().ok_or(io_err!("no commits yet"))?,
            };

            // Check if the hash is valid.
            let otype = match get_object(&hash) {
                Ok((otype, _, _)) => otype,
                Err(_) => return Err(io_err!("invalid hash")),
            };

            let path = format!("{TAGS_DIR}/{name}");

            // Create the path until the tag file.
            match PathBuf::from(&path).parent() {
                Some(parent) => fs::create_dir_all(parent)?,
                None => return Err(io_err!("invalid tag name")),
            }

            let content = match msg {
                None => format!("{hash}\n"),
                Some(msg) => build_tag(&hash, &msg, &otype, &name)?,
            };

            // Write the file.
            fs::write(path, content)?;
            Ok(None)
        }

        Del { name } => {
            let path = format!("{TAGS_DIR}/{name}");

            match fs::remove_file(path) {
                Ok(_) => Ok(None),
                Err(_) => Err(io_err!("tag does not exist")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn get_tags() {
        let tags = get_tags_with_offset("..").unwrap();
        for tag in tags {
            println!("{tag:?}");
        }
    }
}
