//! GitObject trait for Blob, Tree, Tag and Commit.

use libflate::zlib::Encoder;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
};

/// Trait to use polymorphism between
/// GitObjects that have a mode. Them
/// being Blob and Tree.
pub trait EntryMode: GitObject {
    // Returns self's file mode.
    fn mode(&self) -> &'static str;
}

/// Trait to use polymorphism between
/// the different GitObject variants:
/// Blob, Tree, Commit and Tag.
pub trait GitObject {
    // Returns self's file content.
    fn content(&self) -> &str;

    /// Returns self's hash key.
    fn hash(&self) -> &str;

    // Uses zlib to compress self's content.
    fn zip(&self) -> io::Result<Vec<u8>> {
        // Create encoder and zip the content.
        let mut encoder = Encoder::new(Vec::new())?;
        encoder.write_all(self.content().as_bytes())?;
        let mut zip = vec![];

        // Collect the compressed data into zip.
        encoder
            .finish()
            .into_result()?
            .as_slice()
            .read_to_end(&mut zip)?;

        Ok(zip)
    }

    // Saves the object to the database.
    fn save(&self, path: &str) -> io::Result<()> {
        let (dir, file) = self.hash().split_at(2);
        let dir = format!("{}/{}", path, dir);

        // Check if the directory already exists.
        if fs::metadata(&dir).is_err() {
            fs::create_dir(&dir)?;
        }

        let file_path = format!("{}/{}", dir, file);
        if fs::metadata(&file_path).is_err() {
            // Open the file and write encoded data.
            let mut file = File::create(&file_path)?;
            file.write_all(self.zip()?.as_slice())?;
        }

        Ok(())
    }
}
