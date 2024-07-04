use crate::plumbing::hash_object::__hash_object;
use std::{
    fs::{self, Metadata},
    io::{self, Read, Seek, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct IndexEntry {
    ctime_secs: u32, // ctime seconds, the last time a file's metadata changed.
    ctime_nano: u32, // nanosecond fractions.
    mtime_secs: u32, // the last time a file's data changed.
    mtime_nano: u32, // nanosecond fractions.
    dev: u32,        // dev.
    ino: u32,        // ino.
    mode: u32,       // split into (high to low bits).
    // OBJECT TYPE: 4-bit binary 1000 (regular file), 1010 (symbolic link) and 1110 (gitlink).
    // UNUSED: 3-bit unused.
    // UNIX-PERMISSIONS: 9-bit. Only 0755 and 0644 are valid for regular files. Symbolic links and gitlinks have value 0.
    uid: u32,           // uid.
    gid: u32,           // gid.
    file_size: u32,     // This is the on-disk size from stat(2), truncated to 32-bit.
    sha_hash: [u8; 20], // 160-bit SHA-1 key for the represented object.
    flags: u16,         // 'flags' field split into (high to low bits).
    // 1-bit assume-valid flag. La usamos para marcar si un entry esta en stage.
    // 1-bit extended flag (must be zero in version 2).
    // 2-bit stage (during merge).
    // 12-bit name length if the length is less than 0xFFF; otherwise 0xFFF is stored in this field.
    path_name: String, // Entry path name (variable length) relative to top level directory (without leading slash).
} // 1-8 nul bytes as necessary to pad the entry to a multiple of eight bytes while keeping the name NUL-terminated.

impl IndexEntry {
    // Returns the mode field of the IndexEntry struct
    // in the format git exepects it.
    fn get_git_mode(metadata: &Metadata) -> u32 {
        let file_type = metadata.file_type();

        if file_type.is_file() {
            // Checks if the file has executable permissions.
            let permissions = match metadata.permissions().mode() & 0o111 {
                // mode: 0000000000000000 | 0000 | 000 | 000000000.
                0 => 0o644, // Non-executable file.
                _ => 0o755, // Executable file.
            };

            // Regular or Executable file.
            0x8000 | permissions
        } else if file_type.is_symlink() {
            // Symlink.
            0xA000
        } else {
            // Gitlink.
            0xE000
        }
    }

    // Returns the flag field of the IndexEntry struct
    // in the format git expects it.
    fn get_git_flags(path: &str) -> u16 {
        u16::min(0xFFF, path.len() as u16)
    }

    pub fn set_path(&mut self, path: &str) {
        self.path_name = path.to_string();
    }

    pub fn get_stage(&self) -> u16 {
        (self.flags & 0x3000) >> 12
    }

    pub fn get_mode(&self) -> &'static str {
        match self.mode >> 12 {
            // Symbolic link.
            0b1010 => "120000",
            // Gitlink.
            0b1110 => "160000",
            _ => match self.mode >> 9 & 0b111 {
                // Non-executable file.
                0 => "100644",
                // Executable file.
                _ => "100755",
            },
        }
    }

    pub fn get_hash(&self) -> &[u8] {
        &self.sha_hash
    }

    pub fn get_path(&self) -> &str {
        &self.path_name
    }

    pub fn unstage(&mut self) {
        self.flags &= !(1 << 15);
    }

    pub fn is_staged(&self) -> bool {
        self.flags & (1 << 15) != 0
    }

    /// Creates a new entry based on a file
    pub fn new(path: &str, stage: bool, db: bool) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        let mut sha_hash = [0; 20];

        let content = fs::read_to_string(path)?;
        let hash = __hash_object(content.as_bytes(), "blob", db, ".git")?.0;
        sha_hash.copy_from_slice(&hash);

        // Mark this entry as a stage entry.
        let for_stage = if stage { 1 << 15 } else { 0 };

        Ok(Self {
            ctime_secs: metadata.ctime() as u32,
            ctime_nano: metadata.ctime_nsec() as u32,
            mtime_secs: metadata.mtime() as u32,
            mtime_nano: metadata.mtime_nsec() as u32,
            dev: metadata.dev() as u32,
            ino: metadata.ino() as u32,
            mode: Self::get_git_mode(&metadata),
            uid: metadata.uid(),
            gid: metadata.gid(),
            file_size: metadata.size() as u32,
            flags: Self::get_git_flags(path) | for_stage,
            sha_hash,
            path_name: path.to_string(),
        })
    }

    /// Creates an existing entry from the index file.
    pub fn from_index<R: Read + Seek>(file: &mut R) -> io::Result<Self> {
        let mut buf_2 = [0; 2];
        let mut buf_4 = [0; 4];

        file.read_exact(&mut buf_4)?;
        let ctime_secs = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let ctime_nano = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let mtime_secs = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let mtime_nano = u32::from_be_bytes(buf_4);

        file.read_exact(&mut buf_4)?;
        let dev = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let ino = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let mode = u32::from_be_bytes(buf_4);

        file.read_exact(&mut buf_4)?;
        let uid = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let gid = u32::from_be_bytes(buf_4);
        file.read_exact(&mut buf_4)?;
        let file_size = u32::from_be_bytes(buf_4);

        let mut sha_hash = [0; 20];
        file.read_exact(&mut sha_hash)?;

        file.read_exact(&mut buf_2)?;
        let flags = u16::from_be_bytes(buf_2);

        // Read path name.
        let path_len = flags & 0xFFF;
        let mut path_name = vec![0; path_len as usize];
        file.read_exact(&mut path_name)?;
        let path_name = String::from_utf8_lossy(&path_name).to_string();

        // Get through padding.
        let padding = 8 - (62 + path_len) % 8;
        file.read_exact(&mut vec![0; padding as usize])?;

        Ok(Self {
            ctime_secs,
            ctime_nano,
            mtime_secs,
            mtime_nano,
            dev,
            ino,
            mode,
            uid,
            gid,
            file_size,
            sha_hash,
            flags,
            path_name,
        })
    }

    pub fn as_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = vec![];

        // ctime & mtime.
        bytes.write_all(&self.ctime_secs.to_be_bytes())?;
        bytes.write_all(&self.ctime_nano.to_be_bytes())?;
        bytes.write_all(&self.mtime_secs.to_be_bytes())?;
        bytes.write_all(&self.mtime_nano.to_be_bytes())?;

        bytes.write_all(&self.dev.to_be_bytes())?;
        bytes.write_all(&self.ino.to_be_bytes())?;

        // mode with OBJECT TYPE and UNIX PERMISSIONS.
        bytes.write_all(&self.mode.to_be_bytes())?;

        bytes.write_all(&self.uid.to_be_bytes())?;
        bytes.write_all(&self.gid.to_be_bytes())?;
        bytes.write_all(&self.file_size.to_be_bytes())?;

        bytes.write_all(&self.sha_hash)?;
        bytes.write_all(&self.flags.to_be_bytes())?;

        // Pathname.
        bytes.write_all(self.path_name.as_bytes())?;

        // Padding.
        let padding = 8 - (62 + self.path_name.len()) % 8;
        bytes.write_all(&vec![0; padding])?;

        Ok(bytes)
    }

    pub fn new_from_repo_with_hash(path: &str, hash: &str) -> io::Result<Self> {
        let mut sha_hash = [0; 20];
        sha_hash.copy_from_slice(hash.as_bytes());

        Ok(Self {
            sha_hash,
            path_name: path.to_string(),
            ..Default::default()
        })
    }

    pub fn new_from_repo(
        path: &str,
        repo: &str,
        mode: &str,
        data: Vec<u8>,
        write: bool,
    ) -> io::Result<Self> {
        let mut sha_hash = [0; 20];
        let hash = __hash_object(&data, "blob", write, repo)?.0;
        sha_hash.copy_from_slice(&hash);

        // Convert mode to u32.
        let mode = match mode {
            "100755" => 0o100755,
            "120000" => 0o120000,
            "160000" => 0o160000,
            _ => 0o100644,
        };

        Ok(Self {
            sha_hash,
            mode,
            path_name: path.to_string(),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort() {
        let mut entries = vec![];
        entries.push(IndexEntry::new("src/plumbing/commit.rs", false, false).unwrap());
        entries.push(IndexEntry::new("src/plumbing/mod.rs", false, false).unwrap());
        entries.push(IndexEntry::new("src/plumbing/hash_object.rs", false, false).unwrap());

        entries.sort_by_key(|entry| entry.path_name.clone());

        assert_eq!(entries[0].path_name, "src/plumbing/commit.rs");
        assert_eq!(entries[1].path_name, "src/plumbing/hash_object.rs");
        assert_eq!(entries[2].path_name, "src/plumbing/mod.rs");
    }
}
