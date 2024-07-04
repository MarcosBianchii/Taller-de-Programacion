use super::super::commands::ls_tree;
use crate::io_err;
use std::io;
use utils::index_file::index_entry::IndexEntry;
use utils::plumbing::ls_tree::parse_ls_tree_entry;

// Underlying implementation of read_tree.
pub fn __read_tree(hash: &str, entries: &mut Vec<IndexEntry>, path: String) -> io::Result<()> {
    // Get a String representation of tree.
    let tree = ls_tree(hash)?;

    // Iterate over the objects in the tree.
    for line in tree.lines() {
        // Get data about the object.

        let (_, otype, hash, name) = parse_ls_tree_entry(line);
        let path = match path.as_str() {
            "" => name.to_string(),
            _ => path.clone() + "/" + &name,
        };
        //println!("path: {}", path);
        match otype {
            "blob" => entries.push(IndexEntry::new(&path, false, false)?),
            "tree" => __read_tree(hash, entries, path)?,
            _ => return Err(io_err!("invalid object type")),
        }
    }

    Ok(())
}
