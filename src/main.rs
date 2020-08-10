use std::env;
use std::fs;

use git_starter_rust::blob::{hash_object, read_blob, read_tree_object, write_tree_object, create_sha_string};

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
            if let Some(content) = read_blob(&args[3]) {
                print!("{}", content);
            }
        }
    } else if args[1] == "hash-object" && args[2] == "-w" {
        if args.len() < 3 {
            println!("command usage: hash-object -w <file-name>")
        } else {
            if let Some(sha_val) = hash_object(&args[3]) {
                println!("{}", create_sha_string(&sha_val));
            }
        }
    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        if args.len() < 3 {
            println!("command usage: ls-tree --name-only <tree-sha>")
        } else {
            read_tree_object(&args[3]);
        }
    } else if args[1] == "write-tree" {
        let dir_path = env::current_dir().unwrap();
        if let Some(sha_val) = write_tree_object(dir_path.to_str().unwrap()) {
            println!("{}", create_sha_string(&sha_val));
        }
    }
}
