
use ntfs_reader::{mft::Mft, volume::Volume};
use ntfs_reader::errors::NtfsReaderError;

fn main() {
    let path = std::path::Path::new(r"\\.\D:");

    match Volume::new(path) {
        Ok(volume) => {
            println!("Volume created successfully!");
            let _mft:Mft = Mft::new(volume).unwrap();
            // parse_records(mft);
        },
        Err(NtfsReaderError::ElevationError) => {
            println!("Error: Elevated permissions are required.");
        },
        Err(_) => {
            println!("Error: An unknown error occurred.");
        },
    }
}