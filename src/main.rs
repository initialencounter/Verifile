mod database;

mod hash;


use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use ntfs_reader::{mft::Mft, volume::Volume};
use ntfs_reader::api::FIRST_NORMAL_RECORD;
use ntfs_reader::errors::{NtfsReaderError};
use ntfs_reader::file_info::FileInfo;


use hash::{calculate_blake2b512 as calculate_hash};
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
            if !info.name.ends_with(".pdf") {
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
    hash: u32,
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

        match db.get_hash(path.clone()).unwrap() {
            Some(_hash) => {
                let date_now = db.get_date(path.clone()).unwrap().unwrap();
                if date_now != date {
                    let hash_now = calculate_hash(path.clone());
                    db.insert_path(hash_now.clone(), path.clone()).unwrap();
                    db.insert_hash(path.clone(), hash_now).unwrap();
                    db.insert_date(path.clone(), date.clone()).unwrap();
                }
            }
            None => {
                let hash = calculate_hash(path.clone());
                println!("------ Inserting path: {}, hash: {} ------", &path, hash);
                db.insert_path(hash.clone(), path.clone()).unwrap();
                db.insert_hash(path.clone(), hash).unwrap();
                db.insert_date(path.clone(), date.clone()).unwrap();
            }
        }
    });
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_index_file_hash() {
        let db = database::NtfsDataBase::new();
        let path = String::from("\\\\.\\C:\\Users\\29115\\RustroverProjects\\Verifile\\test.txt");
        let hash = calculate_hash(path.clone());
        db.insert_hash(path.clone(), hash.clone()).unwrap();
        db.insert_path(hash.clone(), path.clone()).unwrap();
        db.insert_date(path.clone(), String::from("0")).unwrap();
        println!("{}", hash);
        let path = db.get_path(hash).unwrap().unwrap();
        println!("{}", path);
        assert_eq!(path, String::from("\\\\.\\C:\\Users\\29115\\RustroverProjects\\Verifile\\test.txt"));
    }
}

