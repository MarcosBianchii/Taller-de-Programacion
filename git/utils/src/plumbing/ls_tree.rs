use crate::{
    io_err,
    object::object_db::{get_object_from_repo, get_object_with_offset},
};
use std::io::{self, BufRead, Read};

// Get hash from bits to hex String.
#[allow(dead_code)]
pub fn hash_to_str(hash: &[u8]) -> String {
    hash.iter().fold(String::new(), |mut acc, byte| {
        acc.push_str(&format!("{:02x}", byte));
        acc
    })
}

// Formats the given data to a tree object String.
// Follows this format: <mode> <name>\0<20_bit_hash>...
#[allow(dead_code)]
pub fn __ls_tree(mut data: &[u8]) -> io::Result<String> {
    let mut ret = String::new();

    let mut mode = vec![];
    let mut name = vec![];
    let mut hash = [0; 20];

    while !data.is_empty() {
        // Read entry's mode.
        data.read_until(b' ', &mut mode)?;

        // Convert mode from bytes
        // to String and write it.
        match &mode[..mode.len() - 1] {
            b"40000" => ret.push_str("040000 tree "),
            b"100644" => ret.push_str("100644 blob "),
            b"100755" => ret.push_str("100755 blob "),
            _ => {
                return Err(io_err!("Invalid mode in tree entry"));
            }
        }

        // Read entry's name and hash.
        data.read_until(b'\0', &mut name)?;
        data.read_exact(&mut hash)?;

        // Convert hash from bytes
        // to hex String and write it.
        let hash = hash_to_str(&hash);
        ret.push_str(&hash);

        // Write entry's name.
        ret.push('\t');
        ret.push_str(&String::from_utf8_lossy(&name));
        ret.push('\n');

        // Clear buffers.
        mode.clear();
        name.clear();
    }

    Ok(ret)
}

/// Returns a readable representation of given
/// tree object's data, represented by it's hash.
pub fn ls_tree(hash: &str) -> io::Result<String> {
    ls_tree_with_offset(hash, ".git")
}

pub fn ls_tree_with_offset(hash: &str, offset: &str) -> io::Result<String> {
    let (_, _, data) = get_object_with_offset(hash, offset)?;
    __ls_tree(&data)
}

pub fn ls_tree_from_repo(hash: &str, repo: &str) -> io::Result<String> {
    let (_, _, data) = get_object_from_repo(hash, repo)?;
    __ls_tree(&data)
}

// Parses an ls-tree line into a tuple of (mode, type, hash, name).
pub fn parse_ls_tree_entry(data: &str) -> (&str, &str, &str, String) {
    let data = data.split_whitespace().collect::<Vec<&str>>();
    let mode = data[0];
    let otype = data[1];
    let hash = data[2];
    let name = data[3].replace('\0', "");
    (mode, otype, hash, name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let diff = "100644 blob b0490c9675eac72a51abae693878d87ebae4dc23    Cargo.toml\0";

        assert_eq!(
            (
                "100644",
                "blob",
                "b0490c9675eac72a51abae693878d87ebae4dc23",
                "Cargo.toml".to_string()
            ),
            parse_ls_tree_entry(&diff)
        );
    }
}
