use super::index_entry::IndexEntry;
use crate::io_err;
use sha1::{Digest, Sha1};
use std::io::{self, Read, Seek, Write};

const HEADER_TYPE: &[u8; 4] = b"DIRC";
const VERSION: u32 = 2;

fn valid_header(buf: [u8; 4]) -> bool {
    buf == *HEADER_TYPE
}

fn valid_version(buf: [u8; 4]) -> bool {
    u32::from_be_bytes(buf) == VERSION
}

/// Underlying implementation of read_index.
pub fn __read_index<R: Read + Seek>(mut file: R) -> io::Result<Vec<IndexEntry>> {
    let mut buf = [0u8; 4];

    file.read_exact(&mut buf)?;
    if !valid_header(buf) {
        return Err(io_err!("invalid index header"));
    }

    file.read_exact(&mut buf)?;
    if !valid_version(buf) {
        return Err(io_err!("invalid index version"));
    }

    file.read_exact(&mut buf)?;
    let entries_count = u32::from_be_bytes(buf) as usize;
    let mut entries = Vec::with_capacity(entries_count);

    for _ in 0..entries_count {
        entries.push(IndexEntry::from_index(&mut file)?);
    }

    // Aca seguido iria el hash del index.
    // Todavia no se para que se puede llegar a usar
    // ni si hace falta leerlo y guardarlo.

    Ok(entries)
}

/// Underlying implementation of write_index.
pub fn __write_index<W: Write>(entries: Vec<IndexEntry>, mut output: W) -> io::Result<()> {
    let mut out = vec![];

    // 4-bytes: 'D', 'I', 'R', 'C'.
    out.write_all(HEADER_TYPE)?;

    // 4-bytes: version.
    out.write_all(&VERSION.to_be_bytes())?;

    // 4-bytes: Amount of entries.
    out.write_all(&(entries.len() as u32).to_be_bytes())?;

    for entry in entries {
        out.write_all(&entry.as_bytes()?)?;
    }

    // write index hash.
    let hash = Sha1::digest(&out);
    out.write_all(&hash)?;
    output.write_all(&out)
}

mod tests {
    use super::*;

    struct IndexEntryMock {
        bytes: Vec<u8>,
        iterpos: i64,
    }

    impl Seek for IndexEntryMock {
        fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
            self.iterpos = match pos {
                io::SeekFrom::Start(n) => n as i64,
                io::SeekFrom::Current(n) => self.iterpos + n,
                io::SeekFrom::End(n) => self.bytes.len() as i64 + n,
            };

            Ok(self.iterpos as u64)
        }
    }

    impl Read for IndexEntryMock {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            let mut bytes_read = 0;

            for byte in buf {
                *byte = self.bytes[self.iterpos as usize];
                bytes_read += 1;
                self.iterpos += 1;
            }

            Ok(bytes_read)
        }
    }

    impl Write for IndexEntryMock {
        fn flush(&mut self) -> io::Result<()> {
            self.iterpos = 0;
            Ok(())
        }

        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.bytes.append(&mut buf.to_vec());
            self.iterpos += buf.len() as i64;
            Ok(buf.len())
        }

        fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
            self.bytes.append(&mut buf.to_vec());
            self.iterpos += buf.len() as i64;
            Ok(())
        }
    }

    #[test]
    #[ignore]
    fn write_index() {
        let mut entries = vec![];
        entries.push(IndexEntry::new("src/plumbing/index_file/commands.rs", false, false).unwrap());

        let mut mock = IndexEntryMock {
            bytes: vec![],
            iterpos: 0,
        };

        __write_index(entries, &mut mock).unwrap();
        println!("{:?}", mock.bytes)
    }

    #[test]
    fn read_index() {
        let mut entries = vec![];
        let entry = IndexEntry::new("src/index_file/index_entry.rs", false, false).unwrap();
        entries.push(entry.clone());
        entries.push(IndexEntry::new("src/index_file/commands.rs", false, false).unwrap());
        entries.push(IndexEntry::new("src/index_file/mod.rs", false, false).unwrap());

        let mut mock = IndexEntryMock {
            bytes: vec![],
            iterpos: 0,
        };

        __write_index(entries, &mut mock).unwrap();
        mock.flush().unwrap();

        let entries = __read_index(mock).unwrap();
        assert_eq!(entries[0], entry);
    }
}
