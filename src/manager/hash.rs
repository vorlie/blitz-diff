use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use sha2::{Digest, Sha256};

pub const EMPTY_HASH: &str = "0";

pub fn calculate_reader_hash<R: Read>(mut reader: R) -> io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn calculate_bytes_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn calculate_file_hash(path: &Path) -> io::Result<String> {
    let file = File::open(path)?;
    calculate_reader_hash(BufReader::new(file))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        let h1 = calculate_bytes_hash(b"hello");
        let h2 = calculate_bytes_hash(b"hello");
        assert_eq!(h1, h2);
        assert_ne!(h1, EMPTY_HASH);
        assert_eq!(h1.len(), 64);
    }
}
