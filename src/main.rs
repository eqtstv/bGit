mod cli;
mod data;

use cli::Command;
use data::Repository;

fn main() {
    match cli::parse_args() {
        Command::Init => {
            let repo = Repository::new(".");
            if let Err(e) = repo.init() {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Command::Unknown(msg) => {
            eprintln!("Error: {}", msg);
            std::process::exit(1);
        }
    }
}
