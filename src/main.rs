use std::env;
use std::fs;

use git_starter_rust::blob::{
    create_commit, hash_object, read_blob, read_tree_object, create_tree_object,
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args[1] == "init" {
        fs::create_dir(".git").unwrap();
        fs::create_dir(".git/objects").unwrap();
        fs::create_dir(".git/refs").unwrap();
        fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
        println!("Initialized git directory")
    } else if args[1] == "cat-file" && args[2] == "-p" {
        // "command usage: cat-file -p <blob-sha>"
        let content = read_blob(&args[3]);
        print!("{}", content);
    } else if args[1] == "hash-object" && args[2] == "-w" {
        // "command usage: hash-object -w <file-name>"
        let sha_hash = hash_object(&args[3]);
        println!("{}", sha_hash);
    } else if args[1] == "ls-tree" && args[2] == "--name-only" {
        // "command usage: ls-tree --name-only <tree-sha>"
        let tree_files = read_tree_object(&args[3]);
        print!("{}", tree_files);
    } else if args[1] == "write-tree" {
        let dir_path = env::current_dir().unwrap();
        let sha_val = create_tree_object(dir_path.to_str().unwrap());
        println!("{}", sha_val);
    } else if args[1] == "commit-tree" && args[3] == "-p" && args[5] == "-m" {
        let tree_sha = args[2].clone();
        let parent_sha = args[4].clone();
        let message = args[6].clone();
        let sha_val = create_commit(&tree_sha, &parent_sha, &message);
        println!("{}", sha_val);
    } else if args[1] == "clone" {
        // "command usage: clone https://github.com/blah/blah <some_dir>"
        let url = args[2].clone();
        let dir = args[3].clone();
        // clone(&url);
    }
}
