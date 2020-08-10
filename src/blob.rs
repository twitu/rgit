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

pub fn create_sha_string(sha: &[u8]) -> String {
    let mut sha_str = String::with_capacity(sha.len() * 2);
    for byte in sha {
        write!(sha_str, "{:02x}", byte).unwrap();
    }

    sha_str
}

/// compress , hash and write blob to file
/// return sha1 hash of blob
fn write_blob(content: Vec<u8>) -> [u8; 20] {
    // hash data with sha1
    let mut hasher = Sha1::new();
    hasher.update(&content);
    let result: [u8; 20] = hasher.finalize().into();
    let sha_val = create_sha_string(&result);

    // compress data with zlib encoding
    let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
    z.write_all(&content).unwrap();
    let compressed = z.finish().unwrap();

    // create path for storing blob
    let blob_path = create_path_from_sha(&sha_val);
    create_dir_all(blob_path.parent().unwrap()).unwrap();
    let mut file = File::create(blob_path).unwrap();
    file.write_all(&compressed).unwrap();

    result
}

pub fn read_blob(blob_sha: &String) -> Option<String> {
    let path = create_path_from_sha(blob_sha);

    if let Ok(bytes) = read(path) {
        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut s = String::new();
        z.read_to_string(&mut s).expect("cannot read blob");

        // strip blob meta data about size
        // http://shafiul.github.io/gitbook/1_the_git_object_model.html
        if let Some(i) = s.find('\x00') {
            Some(s[i + 1..].to_owned())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn hash_object(file_path: &str) -> Option<[u8; 20]> {
    let path = PathBuf::from(file_path);
    if !path.is_file() {
        println!("file {} does not exist", file_path);
    }

    if let Ok(bytes) = read(path) {
        // add meta data to file content
        let mut data = format!("blob {}\x00", bytes.len().to_string()).into_bytes();
        data.extend(bytes);

        // hash data with sha1
        let sha_val = write_blob(data);
        Some(sha_val)
    } else {
        None
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

struct TreeObject {
    is_file: bool,
    name: String,
    sha_val: [u8; 20],
}

pub fn write_tree_object(dir_path: &str) -> Option<[u8; 20]> {
    let path = PathBuf::from(dir_path);
    let mut contents: Vec<TreeObject> = Vec::new();

    if path.is_dir() {
        for entry in path.read_dir().unwrap() {
            if let Ok(dir_entry) = entry {
                if let Ok(value) = dir_entry.file_type() {
                    let name = dir_entry.file_name().to_str().unwrap().to_string();
                    let path = dir_entry.path().to_str().unwrap().to_string();
                    if !name.starts_with(".") {
                        if value.is_file() {
                            if let Some(sha_val) = hash_object(&path) {
                                contents.push(TreeObject {
                                    is_file: true,
                                    name,
                                    sha_val
                                })
                            }
                        } else if value.is_dir() {
                            if let Some(sha_val) = write_tree_object(&path) {
                                contents.push(TreeObject {
                                    is_file: false,
                                    name,
                                    sha_val
                                })
                            }
                        }
                    }
                }
            }
        }
    }

    if contents.is_empty() {
        None
    } else {
        contents.sort_by(|o1, o2| o1.name.cmp(&o2.name));

        // TODO: sha is stored in hex representation
        let mut blob_content: Vec<u8> = Vec::new();
        for content in contents {
            let line = if content.is_file {
                format!("100644 {}\x00", content.name)
            } else {
                format!("40000 {}\x00", content.name)
            };
            let mut line = line.into_bytes();
            line.extend(content.sha_val.iter());
            blob_content.extend(line);
        }
        let meta_data = format!("tree {}\x00", blob_content.len().to_string());
        let mut blob_data = meta_data.into_bytes();
        blob_data.extend(blob_content);

        let sha_val = write_blob(blob_data);
        Some(sha_val)
    }
}
