use std::io::{self, Write};
use utils::index_file::index_entry::IndexEntry;
use utils::plumbing::hash_object::__hash_object;

// Underlying implementation of `git write-tree`.
pub fn __write_tree(entries: &mut [IndexEntry], save: bool) -> io::Result<Vec<u8>> {
    // String to store this tree's entries.
    let mut tree_entries = Vec::new();

    let mut i = 0;
    // Iterate over entries.
    while i < entries.len() {
        let path = entries[i].get_path().to_string();
        // Check if path contains '/'. In that case, there should
        // be a Tree object for this exact directory.
        if let Some(index) = path.find('/') {
            let name = &path[..index];

            let from = i;
            // Iterate over entries until we find one that is
            // not in the same sub-directory.
            while i < entries.len() {
                let path = entries[i].get_path().to_string();
                if entries[i].get_path().starts_with(name) {
                    // Remove this directory's name from the entry's path.
                    entries[i].set_path(&path[index + 1..]);
                    i += 1;
                } else {
                    break;
                }
            }

            // Recursively call __write_tree to get this sub-tree's hash.
            let hash = __write_tree(&mut entries[from..i], save)?;
            tree_entries.write_all(format!("40000 {name}\0").as_bytes())?;
            tree_entries.write_all(&hash)?;
        } else {
            // If there is no '/', then the Tree object
            // containing this blob has already been created.
            let mode = entries[i].get_mode();
            let name = entries[i].get_path();
            let hash = entries[i].get_hash();
            tree_entries.write_all(format!("{mode} {name}\0").as_bytes())?;
            tree_entries.write_all(hash)?;
            i += 1;
        }
    }

    Ok(__hash_object(&tree_entries, "tree", save, ".git")?.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    #[allow(unreachable_code)]
    fn test() {
        let mut entries = vec![];
        entries.push(IndexEntry::new("src/objects/blob.rs", false, false).unwrap());
        entries.push(IndexEntry::new("src/objects/tree.rs", false, false).unwrap());
        entries.push(IndexEntry::new("src/objects/gitobject.rs", false, false).unwrap());

        for entry in &entries {
            println!(
                "{}: {:?}",
                entry.get_path(),
                entry
                    .get_hash()
                    .iter()
                    .map(|b| format!("{:02x}", b))
                    .collect::<String>()
            );
        }

        println!("\n\n");

        let tree = __write_tree(&mut entries, false).unwrap();
        println!("{:?}", tree);
    }
}
