mod database;

mod crc32;


use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ntfs_reader::{mft::Mft, volume::Volume};
use ntfs_reader::api::FIRST_NORMAL_RECORD;
use ntfs_reader::errors::{NtfsReaderError};
use ntfs_reader::file_info::FileInfo;


use crc32::{calculate_crc32};
fn main() {
    let path = std::path::Path::new("\\\\.\\C:");

    match Volume::new(path) {
        Ok(volume) => {
            let file_record_size = volume.file_record_size;
            println!("========= Volume file_record_size ========={}", file_record_size);
            let start = Instant::now();
            let mft: Mft = Mft::new(volume).unwrap();
            let file_vec = mft_iteration(&mft);
            update_database(file_vec);
            println!("Elapsed time: {:?}", start.elapsed())
        }
        Err(NtfsReaderError::ElevationError) => {
            println!("Error: Elevated permissions are required.");
        }
        Err(_) => {
            println!("Error: An unknown error occurred.");
        }
    }
}

fn mft_iteration(mft: &Mft) -> Vec<FileInfo>
{
    let mut file_vec: Vec<FileInfo> = Vec::new();
    for number in FIRST_NORMAL_RECORD..mft.max_record {
        if let Some(file) = mft.get_record(number) {
            let info = FileInfo::new(mft, &file);
            if info.size > 7283013 {
                continue;
            }
            if info.is_directory {
                continue;
            }
            file_vec.push(info);
        }
    }
    file_vec
}

struct FileTile {
    path: String,
    crc32: u32,
    date: String,
}

fn update_database(file_vec: Vec<FileInfo>) {
    let db = Arc::new(Mutex::new(database::NtfsDataBase::new()));

    file_vec.into_par_iter().for_each(|file| {
        let path = file.path.to_str().unwrap().to_string();
        let date: String = match file.modified {
            Some(date) => date.to_string(),
            None => String::from("0"),
        };

        let db = Arc::clone(&db);
        let db = db.lock().unwrap();

        match db.get_crc32(path.clone()).unwrap() {
            Some(_crc32) => {
                let date_now = db.get_date(path.clone()).unwrap().unwrap();
                if date_now != date {
                    let crc32_now = calculate_crc32(path.clone());
                    db.insert_path(crc32_now, path.clone()).unwrap();
                    db.insert_crc32(path.clone(), crc32_now).unwrap();
                    db.insert_date(path.clone(), date.clone()).unwrap();
                }
            }
            None => {
                let crc32 = calculate_crc32(path.clone());
                println!("------ Inserting path: {}, crc32: {} ------", &path, crc32);
                db.insert_path(crc32, path.clone()).unwrap();
                db.insert_crc32(path.clone(), crc32).unwrap();
                db.insert_date(path.clone(), date.clone()).unwrap();
            }
        }
    });
}
