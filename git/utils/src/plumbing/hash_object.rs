use libflate::zlib::Encoder;
use sha1::{Digest, Sha1};
use std::{
    fs::{self, File},
    io::{self, Write},
};

/// Underlying implementation of hash_object.
pub fn __hash_object(
    data: &[u8],
    mut otype: &str,
    write: bool,
    offset: &str,
) -> io::Result<(Vec<u8>, String)> {
    if !["blob", "tree", "commit", "tag"].contains(&otype) {
        otype = "blob";
    }

    let mut object = vec![];
    // Write header and data.
    object.write_all(format!("{otype} {}\0", data.len()).as_bytes())?;
    object.write_all(data)?;

    // Hash object.
    let vhash = Sha1::digest(&object);
    let xhash = format!("{vhash:x}");
    //print!("hash: {}\n", xhash);

    if write {
        let mut path = format!("{offset}/objects/{}", &xhash[..2]);
        fs::create_dir_all(&path)?;

        // zip data.
        let mut encoder = Encoder::new(Vec::new())?;
        encoder.write_all(&object)?;
        let zip = encoder.finish().into_result()?;

        // write to database.
        path.push_str(&format!("/{}", &xhash[2..]));

        if let Ok(mut file) = File::create(path) {
            file.write_all(&zip)?;
        }
    }

    Ok((vhash.to_vec(), xhash))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_hash() {
        let path = "src/plumbing/hash_object.rs";
        let a = fs::read_to_string(path).unwrap();
        let b = fs::read_to_string(path).unwrap();

        let a = __hash_object(a.as_bytes(), "blob", false, ".git").unwrap();
        let b = __hash_object(b.as_bytes(), "blob", false, ".git").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn hash_as_git() {
        use std::fs;
        use std::process::Command;

        let path = "src/plumbing/hash_object.rs";
        let obj = fs::read_to_string(path).unwrap();
        let hash = __hash_object(obj.as_bytes(), "blob", false, ".git")
            .unwrap()
            .1;

        let out = Command::new("git")
            .args(["hash-object", path])
            .output()
            .unwrap();

        let git = &out.stdout[..40];
        assert_eq!(hash.as_bytes(), git);
    }
}
