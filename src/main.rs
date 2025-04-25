mod cli;
mod differ;
mod repository;
mod visualizer;

use cli::Command;
use differ::Differ;
use repository::{MERGE_HEAD, ObjectType, Repository};
use std::fs;
use std::path::Path;
use visualizer::Visualizer;

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
        Command::Commit(message) => match repo.create_commit(&message) {
            Ok(hash) => println!("{}", hash),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Log => match repo.log() {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Checkout(commit_hash) => match repo.checkout(&commit_hash) {
            Ok(_) => println!("Checked out commit {}", commit_hash),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Tag(tag_name, commit_hash) => match repo.create_tag(&tag_name, &commit_hash) {
            Ok(_) => println!("Tag {} created successfully", tag_name),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Visualize => match Visualizer::new(repo).visualize() {
            Ok(_output) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::IterRefs => match repo.iter_refs("") {
            Ok(output) => println!("{:?}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Branch(branch_name) => {
            if let Some(branch_name) = branch_name {
                match repo.create_branch(&branch_name, None) {
                    Ok(_) => println!("Branch {} created successfully", branch_name),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match repo.iter_branch_names() {
                    Ok(names) => names.iter().for_each(|name| {
                        println!("{}", name);
                    }),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Command::Status => {
            let head = repo.get_oid_hash("@").unwrap();
            let branch = repo.get_branch_name().unwrap();
            let changed_files = Differ::new(&repo).iter_changed_files().unwrap();

            if branch.is_none() {
                println!("HEAD detached at {}", head);
            } else {
                println!("On branch {}", branch.unwrap());
            }

            let merge_head = repo.get_ref(MERGE_HEAD, true).unwrap();
            if !merge_head.value.is_empty() {
                println!("Merging with {}", merge_head.value);
            }

            println!("\nCurrent changes:");
            for file in changed_files {
                println!("{}", file);
            }
        }
        Command::Reset(commit_hash) => match repo.reset(&commit_hash) {
            Ok(_) => println!("Reset to commit {}", commit_hash),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Show(commit_hash) => match repo.show(&commit_hash) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Diff => match repo.diff() {
            Ok(diff) => {
                if diff.is_empty() {
                    println!("No changes");
                } else {
                    println!("{}", diff);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Merge(branch_name) => match repo.merge(&branch_name) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Rebase(branch_name) => match repo.rebase(&branch_name) {
            Ok(_) => (),
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
