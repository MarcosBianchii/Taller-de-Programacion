use crate::io_err;
use libflate::zlib::Decoder;
use std::{
    fs,
    io::{self, Read},
    vec,
};

// Splits header between type and size.
pub fn header_split(header: &[&str]) -> (String, String) {
    let otype = header[0];
    let osize = header[1];
    (otype.to_string(), osize.to_string())
}

// Looks for the object in the database associated
// to the given hash and returns its content.
fn __get_object(hash: &str) -> io::Result<Vec<u8>> {
    __get_object_with_offset(hash, ".git")
}

fn __get_object_with_offset(hash: &str, offset: &str) -> io::Result<Vec<u8>> {
    //println!("offset: {}", offset);
    let path = format!("{offset}/objects/{}/{}", &hash[..2], &hash[2..]);
    println!("path: {}", path);
    fs::read(path).map_err(|_| io_err!("Could not find object"))
}

// decompresses the object and returns its content as an array of bytes
pub fn deflate_object(data: &[u8]) -> io::Result<Vec<u8>> {
    let mut decoder = Decoder::new(data)?;
    let mut decoded_data = vec![];
    decoder
        .read_to_end(&mut decoded_data)
        .map_err(|_| io_err!("Decoder err"))?;
    Ok(decoded_data)
}

// Tries to find the object in the database, if it finds it, it deflates it
// and returns a tuple of its header and content
pub fn get_object(hash: &str) -> io::Result<(String, String, Vec<u8>)> {
    get_object_with_offset(hash, ".git")
}

// Completes an incomplete hash by looking for it in the database.
fn complete_hash(hash: &str, offset: &str) -> io::Result<String> {
    let dir_hash = &hash[..2];
    let file_hash = &hash[2..];
    let path = format!("{offset}/objects/{dir_hash}");

    let mut files = vec![];
    for file in fs::read_dir(path)?.flatten() {
        let file_name = match file.file_name().into_string() {
            Ok(name) => name,
            Err(_) => continue,
        };

        if file_name.starts_with(file_hash) {
            // If there are more than one file
            // with the same hash, it is ambiguous.
            if !files.is_empty() {
                return Err(io_err!("Ambiguous hash"));
            }

            files.push(file_name);
        }
    }

    if files.is_empty() {
        return Err(io_err!("No hash found"));
    }

    Ok(dir_hash.to_string() + &files.remove(0))
}

/// Looks for an object in a database with a path offset.
pub fn get_object_with_offset(hash: &str, offset: &str) -> io::Result<(String, String, Vec<u8>)> {
    let hash = if hash.len() < 2 || hash.len() > 40 {
        return Err(io_err!("Invalid hash"));
    } else if hash.len() < 40 {
        complete_hash(hash, offset)?.to_string()
    } else {
        hash.to_string()
    };

    let data = __get_object_with_offset(&hash, offset)?;
    let data = deflate_object(&data)?;

    // Find separator between header and data.
    let sep = match data.iter().position(|&x| x == b'\0') {
        Some(i) => i,
        None => return Err(io_err!("No separator found")),
    };

    // Get header and split it in type and size.
    let header = String::from_utf8_lossy(&data[..sep]);
    let header: Vec<&str> = header.split(' ').collect();
    let (otype, osize) = header_split(&header);

    // Validate object type.
    if !["blob", "tree", "commit", "tag"].contains(&header[0]) {
        return Err(io_err!("Invalid object type"));
    }

    let data = data[sep + 1..].to_vec();
    Ok((otype, osize, data))
}

// header: <type> <len>\0<data>
// blob: <data>
// tree: <mode> <name>\0<hash_20_bit>...

pub fn get_object_from_repo(hash: &str, repo: &str) -> io::Result<(String, String, Vec<u8>)> {
    get_object_with_offset(hash, repo)
}
