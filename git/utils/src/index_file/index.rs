use crate::plumbing::hash_object::__hash_object;

use super::index_entry::IndexEntry;
use std::{
    collections::HashMap,
    fs, io,
    ops::{Deref, DerefMut},
};

/// git index file representation.
// K: Path, V: Entry.
#[derive(Default, Debug)]
pub struct Index {
    entries: HashMap<String, IndexEntry>,
}

impl Deref for Index {
    type Target = HashMap<String, IndexEntry>;
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}

impl DerefMut for Index {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entries
    }
}

impl Index {
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new index with the given entries.
    pub fn with(entries: HashMap<String, IndexEntry>) -> Self {
        Self { entries }
    }

    /// Returns a vector of the entries in the index.
    pub fn get_entries(self) -> Vec<IndexEntry> {
        let mut entries: Vec<IndexEntry> = self.entries.into_values().collect();
        entries.sort_by_key(|e| e.get_path().to_string());
        entries
    }

    fn hash_file(path: &str) -> io::Result<Vec<u8>> {
        let data = fs::read(path)?;
        Ok(__hash_object(&data, "blob", false, ".git")?.0)
    }

    /// Adds a new entry to the index.
    pub fn add(&mut self, file: String, stage: bool, db: bool) -> io::Result<()> {
        if let Some(entry) = self.entries.remove(&file) {
            // Compare the hashes of the file.
            let hash = entry.get_hash();
            let new_hash = Self::hash_file(&file)?;

            // If file has not changed then
            // don't mark it as staged.
            if hash == new_hash {
                self.insert(file, entry);
                return Ok(());
            }
        }

        let entry = IndexEntry::new(&file, stage, db)?;
        self.insert(file, entry);
        Ok(())
    }

    pub fn add_from_repo(
        &mut self,
        path: String,
        repo: &str,
        mode: &str,
        data: Vec<u8>,
        write: bool,
    ) -> io::Result<()> {
        let entry = IndexEntry::new_from_repo(&path, repo, mode, data, write)?;
        self.insert(path, entry);
        Ok(())
    }

    /// Removes an entry from the index.
    pub fn remove(&mut self, file: &str) {
        self.entries.remove(file);
    }

    pub fn unstage_all(&mut self) {
        for (_, entry) in self.entries.iter_mut() {
            entry.unstage();
        }
    }

    /// Merges two indexes. The entries in self take priority.
    pub fn merge(self, other: &mut Self) {
        for (path, entry) in self.entries {
            other.entries.insert(path, entry);
        }
    }
}
