mod cli;
mod data;

use cli::Command;
use data::Repository;
use std::fs;

fn main() {
    match cli::parse_args() {
        Command::Init => {
            let repo = Repository::new(".");
            if let Err(e) = repo.init() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Command::HashObject(file_path) => {
            let repo = Repository::new(".");
            match fs::read(&file_path) {
                Ok(data) => match repo.hash_object(&data) {
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
            }
        }
        Command::CatFile(hash) => {
            let repo = Repository::new(".");
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
        Command::Unknown(msg) => {
            eprintln!("Error: {}", msg);
            std::process::exit(1);
        }
    }
}
