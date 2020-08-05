use std::env;
use std::fs;

use git_starter_rust::blob::read_blob;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" && args[2] == "-p" {
        if args.len() < 3 {
            println!("command usage: cat-file -p <blob-sha>")
        } else {
            read_blob(&args[3]);
        }
    }
}
