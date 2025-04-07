use std::env;

pub enum Command {
    Init,
    Unknown(String),
}

impl Command {
    pub fn from_args(args: &[String]) -> Command {
        if args.is_empty() {
            return Command::Unknown("No command provided".to_string());
        }

        match args[0].as_str() {
            "init" => Command::Init,
            cmd => Command::Unknown(format!("Unknown command: {}", cmd)),
        }
    }
}

pub fn parse_args() -> Command {
    let args: Vec<String> = env::args().skip(1).collect();
    Command::from_args(&args)
}
