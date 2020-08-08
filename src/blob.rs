use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{create_dir_all, read, File};
use std::io::Read;
use std::path::PathBuf;

// hack to import both Writes
// https://stackoverflow.com/questions/59187608/can-i-use-write-and-file-write-all-in-same-fn
use std::fmt::Write as _;
use std::io::Write as _;

fn create_path_from_sha(sha: &String) -> PathBuf {
    let dir = &sha[0..2];
    let file = &sha[2..];
    let path: PathBuf = [".git", "objects", dir, file].iter().collect();
    path
}

pub fn read_blob(blob_sha: &String) {
    let path = create_path_from_sha(blob_sha);

    if let Ok(bytes) = read(path) {
        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut s = String::new();
        z.read_to_string(&mut s).expect("cannot read blob");

        // strip blob meta data about size
        // http://shafiul.github.io/gitbook/1_the_git_object_model.html
        if let Some(i) = s.find('\x00') {
            print!("{}", &s[i + 1..]);
        } else {
            print!("{}", s);
        }
    } else {
        println!("blob {} does not exist.", blob_sha);
    }
}

pub fn hash_object(file_path: &String) -> () {
    let path = PathBuf::from(file_path);
    if !path.is_file() {
        println!("file {} does not exist", file_path);
    }

    if let Ok(bytes) = read(path) {
        // add meta data to file content
        let meta_data = format!("blob {}", bytes.len().to_string());
        let mut data = meta_data.into_bytes();
        data.push(0); // append end string character
        data.extend(bytes);

        // hash data with sha1
        let mut hasher = Sha1::new();
        hasher.update(&data);
        let result: [u8; 20] = hasher.finalize().into();
        let mut sha_val = String::with_capacity(result.len() * 2);
        for byte in &result {
            write!(sha_val, "{:02x}", byte).unwrap();
        }

        // compress data with zlib encoding
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&data).unwrap();
        let compressed = z.finish().unwrap();

        // create path for storing blob
        let blob_path = create_path_from_sha(&sha_val);
        create_dir_all(blob_path.parent().unwrap()).unwrap();
        let mut file = File::create(blob_path).unwrap();
        file.write_all(&compressed).unwrap();

        // print sha to std out
        print!("{}", sha_val);
    }
}

pub fn read_tree_object(tree_sha: &String) -> () {
    let tree_path = create_path_from_sha(tree_sha);

    if let Ok(bytes) = read(tree_path) {
        // split bytes by null terminating characters
        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut data: Vec<u8> = Vec::new();
        z.read_to_end(&mut data).unwrap();

        // skip meta data
        let mut cur_index = data.iter().position(|u| *u == '\x00' as u8).unwrap() + 1;

        // iterate over file names
        while let Some(next_index) = data[cur_index..].iter().position(|u| *u == '\x00' as u8) {
            let file_str = std::str::from_utf8(&data[cur_index..cur_index + next_index]).unwrap();
            let name = file_str.split(' ').last().unwrap();
            println!("{}", name);

            cur_index = cur_index + next_index + 21;  // skip sha
            if cur_index >= data.len() {
                break
            }
        }
    } else {
        println!("Could not find object for sha {}", tree_sha);
    }
}
