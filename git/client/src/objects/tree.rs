//! # Implementation of the Tree Git Object variant.

use super::{
    blob::Blob,
    gitobject::{EntryMode, GitObject},
};
use crate::io_err;
use sha1::{Digest, Sha1};
use std::{fs, io, path::PathBuf};

/// Tree GitObject variant that holds a
/// file name and the file which can be
/// either another Tree or Blob GitObject.
pub struct Tree {
    content: String,
    hash: String,
}

impl EntryMode for Tree {
    // Returns self's mode.
    fn mode(&self) -> &'static str {
        "040000"
    }
}

impl GitObject for Tree {
    fn content(&self) -> &str {
        &self.content
    }

    // Returns self's sha1 key.
    fn hash(&self) -> &str {
        &self.hash
    }
}

#[allow(dead_code)]
impl Tree {
    // Formats the files and subdirectories of the Tree object.
    fn file_fmt(files: Vec<(String, Box<dyn EntryMode>)>) -> String {
        let mut data = vec![];
        // Iterates through names and objects
        // adding them to data.
        for (name, obj) in files {
            let hash = obj.hash();
            let mode = obj.mode();

            data.push(format!("{} {}\0{}", mode, name, hash));
        }

        let mut data: String = data.join(""); // por ahi hay que cambiar el separador
        data.insert_str(0, &format!("tree {}\0", data.len()));
        data
    }

    /// Creates a new Tree object by iterating through
    /// the directory's files and subdirectories. It
    /// creates it's respective
    pub fn new(dir: &PathBuf) -> io::Result<Tree> {
        let mut files: Vec<(String, Box<dyn EntryMode>)> = vec![];
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let name = match entry.file_name().to_str() {
                None => return Err(io_err!("Invalid name for tree object")),
                Some(name) => String::from(name),
            };

            files.push(if path.is_dir() {
                (name, Box::new(Self::new(&path)?))
            } else {
                (name, Box::new(Blob::new(&path)?))
            });
        }

        let content = Self::file_fmt(files);
        let hash = format!("{:x}", Sha1::digest(content.as_bytes()));
        Ok(Self { content, hash })
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn hash() {
        let dir = PathBuf::from("src");
        let tree1 = Tree::new(&dir).unwrap();
        let tree2 = Tree::new(&dir).unwrap();
        assert_eq!(tree1.hash(), tree2.hash());
    }
}
