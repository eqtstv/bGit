use std::env;

pub enum Command {
    Init,
    HashObject(String),
    CatFile(String),
    WriteTree,
    ReadTree(String),
    GetTree(String),
    Commit(String),
    Log,
    Checkout(String),
    Tag(String, String),
    Visualize,
    IterRefs,
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
            "write-tree" => {
                if args.len() > 1 {
                    return Command::Unknown("write-tree does not take any arguments".to_string());
                }
                Command::WriteTree
            }
            "read-tree" => {
                if args.len() < 2 {
                    return Command::Unknown("No tree hash provided for read-tree".to_string());
                }
                Command::ReadTree(args[1].clone())
            }
            "get-tree" => {
                if args.len() < 2 {
                    return Command::Unknown("No tree hash provided for get-tree".to_string());
                }
                Command::GetTree(args[1].clone())
            }
            "commit" => {
                if args.len() < 2 {
                    return Command::Unknown("No commit message provided for commit".to_string());
                }
                Command::Commit(args[1].clone())
            }
            "log" => Command::Log,
            "checkout" => {
                if args.len() < 2 {
                    return Command::Unknown("No commit hash provided for checkout".to_string());
                }
                Command::Checkout(args[1].clone())
            }
            "tag" => {
                if args.len() < 3 {
                    return Command::Unknown("No commit hash provided for tag".to_string());
                }
                Command::Tag(args[1].clone(), args[2].clone())
            }
            "iter-refs" => {
                if args.len() > 1 {
                    return Command::Unknown("iter-refs does not take any arguments".to_string());
                }
                Command::IterRefs
            }
            "visualize" => Command::Visualize,
            cmd => Command::Unknown(format!("Unknown command: {}", cmd)),
        }
    }
}

pub fn parse_args() -> Command {
    let args: Vec<String> = env::args().skip(1).collect();
    Command::from_args(&args)
}
