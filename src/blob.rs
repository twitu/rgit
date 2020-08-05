use flate2::read::ZlibDecoder;
use std::fs::read;
use std::io::Read;
use std::path::PathBuf;

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
