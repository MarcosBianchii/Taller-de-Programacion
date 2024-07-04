use super::packentry::PackEntry;
use crate::{
    io_err,
    object::object_db::get_object_with_offset,
    parse_tag,
    plumbing::{
        commit::{get_commit_root, get_parent_commits},
        ls_tree::{ls_tree_with_offset, parse_ls_tree_entry},
    },
};
use sha1::{Digest, Sha1};
use std::io::{self, BufReader, Read};

#[macro_export]
macro_rules! pack_err {
    ($msg:literal) => {
        Err(io::Error::new(io::ErrorKind::InvalidData, $msg))
    };
}

/// Models the contents of a PACK file.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Pack {
    version: u32,
    entries: Vec<PackEntry>,
}

impl Pack {
    /// Reads the packfile, storing it's entries in the object database.
    pub fn unpack<R: Read>(reader: BufReader<R>) -> io::Result<()> {
        Self::unpack_with_offset(reader, ".git")
    }

    /// Reads the packfile, storing it's entries in an object databse with the given offset.
    pub fn unpack_with_offset<R: Read>(mut reader: BufReader<R>, offset: &str) -> io::Result<()> {
        // P A C K line
        let mut buf = [0; 4];
        reader.read_exact(&mut buf)?;

        // Validate PACK file.
        if buf != *b"PACK" {
            return pack_err!("Invalid PACK file");
        }

        reader.read_exact(&mut buf)?;
        let version = u32::from_be_bytes(buf);

        // Validate 4-byte version number.
        if version != 2 {
            return pack_err!("Wrong version number");
        }

        // 4-byte number of objects contained in the pack (network byte order).
        reader.read_exact(&mut buf)?;
        let entries_len = u32::from_be_bytes(buf) as usize;

        // Read all the entries.
        for _ in 0..entries_len {
            PackEntry::unpack_with_offset(&mut reader, offset)?;
        }
        // 20-byte SHA-1 checksum of the packed content.

        Ok(())
    }

    fn make_pack_entries_with_offset(
        hash: String,
        entries: &mut Vec<PackEntry>,
        offset: &str,
    ) -> io::Result<()> {
        let (otype, _, data) = get_object_with_offset(hash.as_str(), offset)?;

        let entry = match otype.as_str() {
            "blob" => PackEntry::new(&otype, data),

            "tree" => {
                let tree = ls_tree_with_offset(&hash, offset)?;

                // Add tree items to entries.
                for line in tree.lines() {
                    let (_, _, hash, _) = parse_ls_tree_entry(line);
                    Self::make_pack_entries_with_offset(hash.to_string(), entries, offset)?;
                }

                PackEntry::new(&otype, data)
            }

            "commit" => {
                // Add commit tree to entries.
                let hash = get_commit_root(&data)?;
                Self::make_pack_entries_with_offset(hash, entries, offset)?;
                for parent in get_parent_commits(&data).unwrap_or_default() {
                    Self::make_pack_entries_with_offset(parent, entries, offset)?;
                }

                PackEntry::new(&otype, data)
            }

            "tag" => {
                // Add tag object to entries.
                let object = parse_tag(&data).map_err(|_| io_err!("Invalid tag object"))?;
                Self::make_pack_entries_with_offset(object, entries, offset)?;
                PackEntry::new(&otype, data)
            }

            _ => return Err(io_err!("Invalid object type")),
        };

        entries.push(entry);
        Ok(())
    }

    pub fn from_with_offset(refs: Vec<String>, offset: &str) -> io::Result<Self> {
        let mut entries = vec![];
        for hash in refs {
            Self::make_pack_entries_with_offset(hash, &mut entries, offset)?;
        }

        Ok(Self {
            version: 2,
            entries,
        })
    }

    /// Creates a new Pack from the given references.
    pub fn from(references: Vec<String>) -> io::Result<Self> {
        Self::from_with_offset(references, ".git")
    }

    /// Returns the bytes of the PACK file.
    pub fn as_bytes(&self) -> io::Result<Vec<u8>> {
        let mut pack_file = vec![];

        // P A C K line
        pack_file.extend(b"PACK");

        // 4-byte version number
        pack_file.extend(&self.version.to_be_bytes());

        // 4-byte number of objects contained in the pack (network byte order).
        pack_file.extend((self.entries.len() as u32).to_be_bytes());

        for entry in &self.entries {
            let entry = entry.as_bytes()?;
            pack_file.extend(entry);
        }

        // 20-byte SHA-1 checksum of the packed content.
        let hash = Sha1::digest(&pack_file);
        pack_file.extend(&hash);

        Ok(pack_file)
    }
}
