use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::{read, File, create_dir_all};
use std::io::Read;
use std::path::PathBuf;

// hack to import both Writes
// https://stackoverflow.com/questions/59187608/can-i-use-write-and-file-write-all-in-same-fn
use std::io::Write as _;
use std::fmt::Write as _;

pub fn read_blob(blob_sha: &String) -> () {
    let dir = &blob_sha[0..2];
    let file = &blob_sha[2..];
    let path: PathBuf = [".git", "objects", dir, file].iter().collect();

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

pub fn hash_file(file_path: &String) -> () {
    let path = PathBuf::from(file_path);
    if !path.is_file() {
        println!("file {} does not exist", file_path);
    }

    if let Ok(bytes) = read(path) {
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        let result: [u8; 20] = hasher.finalize().into();
        let mut sha_val = String::with_capacity(result.len() * 2);
        for byte in &result {
            write!(sha_val, "{:02x}", byte);
        }

        let meta_data = format!("blob {}", bytes.len().to_string());
        let mut data = meta_data.into_bytes();
        data.push(0); // append end string character
        data.extend(bytes);

        let blob_dir = &sha_val[0..2];
        let blob_file = &sha_val[2..];
        let blob_path: PathBuf = [".git", "objects", blob_dir, blob_file].iter().collect();

        let mut z = ZlibEncoder::new(Vec::new(), Compression::default());
        z.write_all(&data);
        let compressed = z.finish().unwrap();

        create_dir_all(blob_path.parent().unwrap());
        let mut file = File::create(blob_path).unwrap();
        file.write_all(&compressed);

        // print sha to std out
        println!("{}", sha_val);
    }
}
