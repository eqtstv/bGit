mod cli;
mod data;

use cli::Command;
use data::{ObjectType, Repository};
use std::fs;
use std::path::Path;

fn main() {
    let repo = Repository::new(".");

    match cli::parse_args() {
        Command::Init => {
            if let Err(e) = repo.init() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Command::HashObject(file_path) => match fs::read(&file_path) {
            Ok(data) => match repo.hash_object(&data, ObjectType::Blob) {
                Ok(hash) => println!("{}", hash),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            },
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                std::process::exit(1);
            }
        },
        Command::CatFile(hash) => {
            match repo.get_object(&hash) {
                Ok(data) => {
                    // Convert bytes to string and print
                    if let Ok(content) = String::from_utf8(data) {
                        print!("{}", content);
                    } else {
                        eprintln!("Error: Object content is not valid UTF-8");
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::WriteTree => match repo.create_tree(Path::new(&repo.worktree)) {
            Ok(hash) => println!("{}", hash),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::ReadTree(tree_oid) => {
            let worktree_path = Path::new(&repo.worktree);
            match repo.read_tree(&tree_oid, worktree_path) {
                Ok(_) => println!("Tree {} extracted successfully", tree_oid),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::GetTree(tree_oid) => match repo.get_tree_data(&tree_oid) {
            Ok(data) => {
                for (mode, name, hash, obj_type) in data {
                    println!("{} {:?} {} {}", mode, obj_type, name, hash);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Unknown(msg) => {
            eprintln!("Error: {}", msg);
            std::process::exit(1);
        }
    }
}
