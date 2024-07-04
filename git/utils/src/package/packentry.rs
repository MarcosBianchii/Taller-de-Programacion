use super::super::plumbing::hash_object::__hash_object;
use crate::{io_err, object::object_db::get_object_with_offset, pack_err};
use flate2::read::ZlibDecoder;
use libflate::zlib::Encoder;
use std::io::{self, BufRead, BufReader, Read, Write};

const STOPMASK: u8 = 0b10000000;
const TYPEMASK: u8 = 0b01110000;
const LENGMASK: u8 = 0b00001111;
const SIZEMASK: u8 = 0b01111111;

// Deltas.
const INSTMASK: u8 = 0b10000000;
const RESTMASK: u8 = 0b01111111;

/// Models an entry in a PACK file.
#[derive(Debug)]
pub struct PackEntry {
    pub otype: String,
    pub data: Vec<u8>,
}

impl PackEntry {
    /// Creates a new PackEntry.
    pub fn new(otype: &str, data: Vec<u8>) -> Self {
        Self {
            otype: otype.to_string(),
            data,
        }
    }

    // Acts as a dictionary for the object types.
    fn num_to_type(otype: u8) -> Option<String> {
        Some(match otype {
            1 => String::from("commit"),
            2 => String::from("tree"),
            3 => String::from("blob"),
            4 => String::from("tag"),
            6 => String::from("ofs_delta"),
            7 => String::from("ref_delta"),
            _ => return None,
        })
    }

    // Acts as a dictionary for the object types.
    fn type_to_num(otype: &str) -> Option<u8> {
        Some(match otype {
            "commit" => 1,
            "tree" => 2,
            "blob" => 3,
            "tag" => 4,
            "ofs_delta" => 6,
            "ref_delta" => 7,
            _ => return None,
        })
    }

    // Calculates the size of the object in pack format.
    fn obj_size_in_pack_format(mut size: usize, typenum: u8) -> Vec<u8> {
        let mut result = vec![];

        // Calculate and store first byte.
        let first_byte = STOPMASK | (typenum << 4) | (size as u8 & LENGMASK);
        result.push(first_byte);
        size >>= 4;

        while size > 0 {
            let byte = STOPMASK | (size as u8 & SIZEMASK);
            size >>= 7;
            result.push(byte);
        }

        // Turn off last bit of the last field.
        if let Some(last) = result.last_mut() {
            *last &= !STOPMASK;
        }

        result
    }

    // Turn a pack entry into a pack file format byte array.
    pub fn as_bytes(&self) -> io::Result<Vec<u8>> {
        let Self { otype, data } = self;
        let size = data.len();
        let mut entry = vec![];

        // Get object type in number format and caluclate size in pack format.
        let typenum = Self::type_to_num(otype).ok_or(io_err!("Invalid object type"))?;
        let size = Self::obj_size_in_pack_format(size, typenum);

        // Zip data.
        let mut encoder = Encoder::new(vec![])?;
        encoder.write_all(data)?;
        let zip = encoder.finish().into_result()?;

        // Build entry.
        entry.extend(size);
        entry.extend(zip);

        Ok(entry)
    }

    // Calculates part of the size in
    // bytes of the next loose object.
    fn calculate_size(bytes: &[u8], offset: usize) -> usize {
        let mut size = 0;
        for (i, byte) in bytes.iter().enumerate() {
            size |= (*byte as usize) << (offset + (i * 7));
        }

        size
    }

    // Processes a loose object.
    fn process_loose<R: Read>(
        reader: &mut BufReader<R>,
        otype: String,
        size: usize,
    ) -> io::Result<Self> {
        // Decompress data.
        let mut data = vec![0; size];
        let mut decoder = ZlibDecoder::new(reader.buffer());
        decoder.read_exact(&mut data)?;

        // Move reader.
        let moved = decoder.total_in();
        reader.fill_buf()?;
        reader.consume(moved as usize);
        Ok(Self { otype, data })
    }

    // Reads a variable length integer from reader not storing it.
    fn get_through_redundant_size<R: Read>(reader: &mut R) -> io::Result<()> {
        let mut byte = [0x80; 1];

        while (byte[0] & STOPMASK) != 0 {
            reader.read_exact(&mut byte)?;
        }
        Ok(())
    }

    // Reads a partial integer from the given reader until the last
    // bit is set to 0, marking it's the last byte of the integer.
    fn read_partial_int<R: Read>(reader: &mut R, n: u8, field: &mut u8) -> io::Result<usize> {
        let mut byte = [0; 1];
        let mut value = 0;

        for i in 0..n {
            if *field & 1 != 0 {
                reader.read_exact(&mut byte)?;
                value |= (byte[0] as usize) << (i * 8);
            }

            *field >>= 1;
        }

        Ok(value)
    }

    // Build the delta object executing the instructions of the delta.
    fn build_delta_obj<R: Read>(base: Vec<u8>, delta: &mut R) -> io::Result<Vec<u8>> {
        // Read delta header.
        let mut byte = [0; 1];
        let mut data = vec![];

        while delta.read_exact(&mut byte).is_ok() {
            // 1-bit instruction + 7-lowest bits.
            let instr = (byte[0] & INSTMASK) >> 7;
            let value = byte[0] & RESTMASK;

            match instr {
                0 => {
                    // Append.
                    let mut buf = vec![0; value as usize];
                    delta.read_exact(&mut buf)?;
                    data.extend_from_slice(&buf);
                }

                1 => {
                    // Copy.
                    let offset = Self::read_partial_int(delta, 4, &mut byte[0])?;
                    let mut nbytes = Self::read_partial_int(delta, 3, &mut byte[0])?;
                    if nbytes == 0 {
                        nbytes = 0x10000;
                    }

                    data.extend_from_slice(&base[offset..offset + nbytes]);
                }

                _ => {} // unreachable.
            }
        }

        Ok(data)
    }

    // Processes a delta object.
    fn process_delta<R: Read>(
        reader: &mut BufReader<R>,
        otype: String,
        offset: &str,
    ) -> io::Result<Self> {
        if otype == "ofs_delta" {
            return pack_err!("ofs_delta not implemented");
        }

        // Take hash of base object.
        let mut hash = [0; 20];
        reader.read_exact(&mut hash)?;
        let hash = hash.iter().fold(String::new(), |mut acc, byte| {
            acc.push_str(&format!("{:02x}", byte));
            acc
        });

        // Get base object.
        let (otype, _, base) = get_object_with_offset(&hash, offset)?;

        // Read delta data.
        let mut delta = ZlibDecoder::new(reader.buffer());

        // Get through the base object's size
        // and the resultant delta object's size.
        Self::get_through_redundant_size(&mut delta)?;
        Self::get_through_redundant_size(&mut delta)?;

        // Build.
        let data = Self::build_delta_obj(base, &mut delta)?;

        // Move reader.
        let moved = delta.total_in();
        reader.consume(moved as usize);

        Ok(Self { otype, data })
    }

    /// Saves the entry to the object database.
    pub fn unpack<R: Read>(reader: &mut BufReader<R>) -> io::Result<()> {
        Self::unpack_with_offset(reader, ".git")
    }

    /// Saves the entry to the object database using an offset for the .git folder.
    pub fn unpack_with_offset<R: Read>(reader: &mut BufReader<R>, offset: &str) -> io::Result<()> {
        let mut byte = [0; 1];
        let mut size = vec![];

        // Read first byte.
        reader.read_exact(&mut byte)?;
        let typenum = (byte[0] & TYPEMASK) >> 4;
        let otype = match Self::num_to_type(typenum) {
            None => return pack_err!("Invalid object type")?,
            Some(otype) => otype,
        };

        print!("otype: {otype}, ");
        // Store first size part.
        size.push(byte[0] & LENGMASK);

        // Read size.
        while (byte[0] & STOPMASK) != 0 {
            reader.read_exact(&mut byte)?;
            size.push(byte[0] & SIZEMASK);
        }

        // Calculate size.
        let size = size[0] as usize | Self::calculate_size(&size[1..], 4);

        println!("size: {size} ");
        // Get object data.
        let entry = match typenum {
            1..=4 => Self::process_loose(reader, otype, size),
            6 | 7 => Self::process_delta(reader, otype, offset),
            _ => pack_err!("Invalid object type")?,
        }?;

        entry.dump(offset)?;
        Ok(())
    }

    /// Writes self in object database.
    pub fn dump(&self, offset: &str) -> io::Result<()> {
        __hash_object(&self.data, &self.otype, true, offset)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_size<R: Read>(mut reader: R) -> io::Result<usize> {
        let mut byte = [0; 1];
        let mut size = vec![];

        // Read first byte.
        reader.read_exact(&mut byte)?;

        // Store first size part.
        size.push(byte[0] & LENGMASK);

        // Read size.
        while (byte[0] & STOPMASK) != 0 {
            reader.read_exact(&mut byte)?;
            size.push(byte[0] & SIZEMASK);
        }

        // Calculate size.
        let size = size[0] as usize | PackEntry::calculate_size(&size[1..], 4);
        Ok(size)
    }

    #[test]
    fn test() {
        let num = 123512;
        let typenum = 7;

        let size = PackEntry::obj_size_in_pack_format(num, typenum);
        let size = read_size(size.as_slice()).unwrap();

        assert_eq!(num, size);
    }

    #[test]
    #[ignore]
    fn test1() {
        use libflate::zlib::Decoder;

        fn deflate_object(data: &[u8]) -> io::Result<Vec<u8>> {
            let mut decoder = Decoder::new(data)?;
            let mut decoded_data = vec![];
            decoder
                .read_to_end(&mut decoded_data)
                .map_err(|_| io_err!("Decoder err"))?;
            Ok(decoded_data)
        }

        let file =
            std::fs::read("../client/.git/objects/e6/9de29bb2d1d6434b8b29ae775ad8c2e48c5391")
                .unwrap();

        println!("pre-deflate: {file:?}");

        let data = deflate_object(&file).unwrap();
        println!("data: {data:?}");

        let data = String::from_utf8_lossy(&data);
        println!("str: {data:?}")
    }
}
