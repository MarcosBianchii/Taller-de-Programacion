//! # Implementation of the Blob Git Object variant.

use super::gitobject::{EntryMode, GitObject};
use sha1::{Digest, Sha1};
use std::{fs, io, os::unix::prelude::PermissionsExt, path::PathBuf};

/// Blob GitObject variant that holds
/// the contents of a text file.
pub struct Blob {
    content: String,
    hash: String,
    mode: &'static str,
}

impl EntryMode for Blob {
    // Returns self's mode.
    fn mode(&self) -> &'static str {
        self.mode
    }
}

impl GitObject for Blob {
    // Returns self's content.
    fn content(&self) -> &str {
        &self.content
    }

    // Returns self's sha1 key.
    fn hash(&self) -> &str {
        &self.hash
    }
}

impl Blob {
    // Returns true if this file has execute permissions.
    fn get_git_mode(path: &PathBuf) -> io::Result<&'static str> {
        let mode = fs::metadata(path)?.permissions().mode();
        if mode & 0o111 != 0 {
            // Executable file.
            Ok("100755")
        } else {
            // Non-executable file.
            Ok("100644")
        }
    }

    /// Method that creates a new Blob object from
    /// a text file's path, saving it's content,
    /// sha1 hash and file mode.
    pub fn new(path: &PathBuf) -> io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let content = format!("blob {}\0{}", content.len(), content);

        // Compute this blob's hash and mode.
        let hash = format!("{:x}", Sha1::digest(content.as_bytes()));
        let mode = Self::get_git_mode(path)?;

        Ok(Self {
            content,
            hash,
            mode,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create() {
        let path = PathBuf::from("src/objects/blob.rs");
        let blob = Blob::new(&path).unwrap();
        assert!(blob.content.contains(
            r#"
fn create() {
let path = PathBuf::from("src/objects/blob.rs\");
let blob = Blob::new(&path).unwrap();
}))"#
        ));
    }

    #[test]
    fn valid_hash() {
        let path = PathBuf::from("src/objects/blob.rs");
        let blob1 = Blob::new(&path).unwrap();
        let blob2 = Blob::new(&path).unwrap();
        assert_eq!(blob1.hash(), blob2.hash());
    }

    #[test]
    fn hash_as_git() {
        use std::process::Command;
        let blob_path = "src/objects/blob.rs";
        let path = PathBuf::from(blob_path);
        let hash = Blob::new(&path).unwrap().hash().to_string();
        let out = Command::new("git")
            .args(["hash-object", blob_path])
            .output()
            .unwrap();
        let git = &out.stdout[..40];
        assert_eq!(hash.as_bytes(), git);
    }

    #[test]
    #[ignore]
    fn exe_mode() {}

    #[test]
    fn non_exe_mode() {
        let path = PathBuf::from("src/objects/blob.rs");
        let blob = Blob::new(&path).unwrap();
        assert_eq!(blob.mode(), "100644");
    }

    #[test]
    #[ignore]
    // Este test funciona pero no es bueno, habria que cambiarlo.
    fn valid_save() {
        let path = PathBuf::from("src/objects/blob.rs");
        let blob = Blob::new(&path).unwrap();
        println!("{}", blob.hash());
        blob.save("src").unwrap();
    }
}
