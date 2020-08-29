use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{create_dir_all, read, File};
use std::io::Cursor;
use std::io::Read;
use std::path::PathBuf;
use std::time::SystemTime;

// hack to import both Writes
// https://stackoverflow.com/questions/59187608/can-i-use-write-and-file-write-all-in-same-fn
use std::fmt::Write as _;
use std::io::Write as _;

enum BlobType {
    BLOB,
    TREE,
    COMMIT,
}

struct Blob {
    blob_type: BlobType,
    sha_val: [u8; 20],
    content: Vec<u8>,
}

impl Blob {

    // create blob from data
    pub fn new_blob_from_data(data: Vec<u8>, blob_type: BlobType) -> Self {
        let mut header: Vec<u8> = match blob_type {
            BlobType::BLOB => format!("blob {}\x00", data.len().to_string()).into_bytes(),
            BlobType::TREE => format!("tree {}\x00", data.len().to_string()).into_bytes(),
            BlobType::COMMIT => format!("commit {}\x00", data.len().to_string()).into_bytes(),
        };
        header.extend(data);
        let with_header = header;

        // hash data with sha1
        let mut hasher = Sha1::new();
        hasher.update(&with_header);
        let sha_val: [u8; 20] = hasher.finalize().into();

        // compress data with zlib encoding
        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&with_header).unwrap();
        let content = z.finish().unwrap();

        Blob {
            blob_type,
            sha_val,
            content,
        }
    }

    // create blob from blob object
    pub fn new_blob_from_file(path: &PathBuf, blob_type: BlobType) -> Self {
        let bytes = read(path).unwrap();
        Blob::new_blob_from_data(bytes, blob_type)
    }

    // create blob from blob object
    pub fn new_blob_from_blob_file(blob_sha: &String, blob_type: BlobType) -> Self {
        let path = get_path(blob_sha);
        let bytes = read(path).unwrap();

        let mut z = ZlibDecoder::new(&bytes[..]);
        let mut content = Vec::new();
        z.read_to_end(&mut content).expect("cannot read blob");

        let mut hasher = Sha1::new();
        hasher.update(&content);
        let sha_val: [u8; 20] = hasher.finalize().into();

        Blob {
            blob_type,
            sha_val,
            content,
        }
    }

    // convert compressed sha to ascii string
    fn get_sha_string(&self) -> String {
        let mut sha_str = String::with_capacity(self.sha_val.len() * 2);
        for byte in &self.sha_val {
            write!(sha_str, "{:02x}", byte).unwrap();
        }

        sha_str
    }

    // create path and store blob content
    pub fn write_blob(&self) {
        let blob_path = get_path(&self.get_sha_string());
        create_dir_all(blob_path.parent().unwrap()).unwrap();
        let mut file = File::create(blob_path).unwrap();
        file.write_all(&self.content).unwrap();
        file.flush().unwrap();
    }
}

// get storage path for blob
fn get_path(sha: &str) -> PathBuf {
    let dir = &sha[0..2];
    let file = &sha[2..];
    let path: PathBuf = [".git", "objects", dir, file].iter().collect();
    path
}

// create blob from file contents
pub fn read_blob(blob_sha: &String) -> String {
    let blob = Blob::new_blob_from_blob_file(blob_sha, BlobType::BLOB);
    let data = String::from_utf8(blob.content).unwrap();

    // strip header before returning
    let i = data.find('\x00').unwrap();
    data[i + 1..].to_owned()
}

// create blob from file, write to disk and return sha1 hash
pub fn hash_object(file_path: &str) -> String {
    let path = PathBuf::from(file_path);
    let blob = Blob::new_blob_from_file(&path, BlobType::BLOB);
    blob.write_blob();
    blob.get_sha_string()
}

pub fn read_tree_object(tree_sha: &String) -> String {
    let blob = Blob::new_blob_from_blob_file(tree_sha, BlobType::TREE);
    let data = blob.content;
    let mut names: String = String::new();

    // skip meta data
    let mut cur_index = data.iter().position(|u| *u == '\x00' as u8).unwrap() + 1;

    // iterate over file names
    while let Some(next_index) = data[cur_index..].iter().position(|u| *u == '\x00' as u8) {
        let file_str = std::str::from_utf8(&data[cur_index..cur_index + next_index]).unwrap();
        let name = file_str.split(' ').last().unwrap();
        names.push_str(name);
        names.push('\n');

        cur_index = cur_index + next_index + 21; // skip sha
        if cur_index >= data.len() {
            break;
        }
    }

    names
}

pub fn create_tree_object(dir_path: &str) -> String {
    let path = PathBuf::from(dir_path);
    let blob = write_tree_object(&path).unwrap();  // dir should not be empty
    blob.get_sha_string()
}

struct TreeObject {
    is_file: bool,
    name: String,
    sha_val: [u8; 20],
}

fn write_tree_object(path: &PathBuf) -> Option<Blob> {
    let mut contents = Vec::<TreeObject>::new();

    // iterate over directory and write blobs for files and directories recursively
    for entry in path.read_dir().unwrap() {
        let dir_entry = entry.unwrap();
        let value = dir_entry.file_type().unwrap();
        let name = dir_entry.file_name().to_str().unwrap().to_string();
        let path = dir_entry.path();
        if name.starts_with(".") {
            continue;
        }

        if value.is_file() {
            let blob = Blob::new_blob_from_file(&path, BlobType::BLOB);
            let sha_val = blob.sha_val;
            blob.write_blob();

            contents.push(TreeObject {
                is_file: true,
                name,
                sha_val,
            });
        } else if value.is_dir() {
            if let Some(blob) = write_tree_object(&path) {
                let sha_val = blob.sha_val;
                blob.write_blob();

                contents.push(TreeObject {
                    is_file: false,
                    name,
                    sha_val,
                });
            }
        }
    }

    if contents.is_empty() {
        None
    } else {
        contents.sort_by(|o1, o2| o1.name.cmp(&o2.name));

        let mut blob_content: Vec<u8> = Vec::new();
        for content in contents {
            let line = if content.is_file {
                format!("100644 {}\x00", content.name)
            } else {
                // git writes 40000 as access mode
                // this is different from 040000 which is displayed on
                // running `cat-file`
                format!("40000 {}\x00", content.name)
            };
            let mut line = line.into_bytes();
            line.extend(content.sha_val.iter());
            blob_content.extend(line);
        }

        let blob = Blob::new_blob_from_data(blob_content, BlobType::TREE);
        blob.write_blob();
        Some(blob)
    }
}

pub fn create_commit(tree_sha: &String, parent_sha: &String, message: &String) -> String {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let offset = String::from("+0530");
    let content = if parent_sha.is_empty() {
        format!("tree {}\nauthor Alias Anon <a@a.com> {} {}\ncommiter Alias Anon <a@a.com> {} {}\n\n{}\n", tree_sha, time, offset, time, offset, message)
    } else {
        format!("tree {}\nparent {}\nauthor Alias Anon <a@a.com> {} {}\ncommiter Alias Anon <a@a.com> {} {}\n\n{}\n", tree_sha, parent_sha, time, offset, time, offset, message)
    };

    let content = content.into_bytes();
    let blob = Blob::new_blob_from_data(content, BlobType::COMMIT);
    blob.write_blob();
    blob.get_sha_string()
}
