use std::env;

pub enum Command {
    Init,
    HashObject(String),
    CatFile(String),
    Unknown(String),
}

impl Command {
    pub fn from_args(args: &[String]) -> Command {
        if args.is_empty() {
            return Command::Unknown("No command provided".to_string());
        }

        match args[0].as_str() {
            "init" => Command::Init,
            "hash-object" => {
                if args.len() < 2 {
                    return Command::Unknown("No file path provided for hash-object".to_string());
                }
                Command::HashObject(args[1].clone())
            }
            "cat-file" => {
                if args.len() < 2 {
                    return Command::Unknown("No hash provided for cat-file".to_string());
                }
                Command::CatFile(args[1].clone())
            }
            cmd => Command::Unknown(format!("Unknown command: {}", cmd)),
        }
    }
}

pub fn parse_args() -> Command {
    let args: Vec<String> = env::args().skip(1).collect();
    Command::from_args(&args)
}
