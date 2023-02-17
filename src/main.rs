use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path};
use blake3::{Hasher};
use data_encoding::HEXUPPER;

fn hash_file_blake3(file_path: &Path) -> Result<String, std::io::Error> {
    let file = fs::File::open(file_path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Hasher::new();
    let mut buffer = [0; 1024];

    while let Ok(bytes_read) = reader.read(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(HEXUPPER.encode(hasher.finalize().as_bytes()))
}

fn get_files_in_all_subdirectories(root: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut files = Vec::new();

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            files.extend(get_files_in_all_subdirectories(&path)?);
        } else {
            files.push(path.to_string_lossy().to_string());
        }
    }

    Ok(files)
}

fn find_duplicates(root: &Path) -> Result<HashMap<String, Vec<String>>, std::io::Error> {
    let mut file_hashes: HashMap<String, Vec<String>> = HashMap::new();

    for file in get_files_in_all_subdirectories(root)? {
        let path: &Path = Path::new(&file);
        let hash = hash_file_blake3(&path)?;
        file_hashes.entry(hash).or_default().push(file);
    }

    let duplicates = file_hashes
        .into_iter()
        .filter_map(|(_, files)| {
            if files.len() > 1 {
                Some((files[0].clone(), files[1..].to_vec()))
            } else {
                None
            }
        })
        .collect();

    Ok(duplicates)
}

fn main() -> Result<(), std::io::Error> {
    let path = Path::new("./src/text_files");
    println!("Finding duplicates in {:?}...", path);

    let duplicates = find_duplicates(path)?;

    if duplicates.is_empty() {
        println!("No duplicates found.");
    }

    for (file, duplicates) in duplicates {
        println!("{} is duplicated in:", file);
        for duplicate in duplicates {
            println!("\t{}", duplicate);
        }
    }

    Ok(())
}
