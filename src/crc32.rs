use std::fs::File;
use std::io::{BufReader, Read};
use crc32fast::Hasher;
use blake2::{Blake2b512, Digest};

pub fn calculate_crc32(path: String) -> u32 {
    let file = File::open(&path);
    return match file {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut hasher = Hasher::new();
            let mut buffer = [0u8; 1024];
            loop {
                let n = reader.read(&mut buffer).unwrap_or_else(|_| 0);
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            hasher.finalize()
        }
        Err(_) => {
            0
        }
    }
}


pub fn calculate_blake2b512(path: String) -> String {
    let file = File::open(&path);
    return match file {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut hasher = Blake2b512::new();
            let mut buffer = [0u8; 1024];
            loop {
                let n = reader.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                hasher.update(&buffer[..n]);
            }
            hex::encode(hasher.finalize())
        }
        Err(_) => String::from("0"),
    };
}
