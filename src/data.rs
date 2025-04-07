use std::fs;
use std::path::Path;

const GIT_DIR: &str = ".bgit";

pub struct Repository {
    worktree: String,
    gitdir: String,
}

impl Repository {
    pub fn new(path: &str) -> Repository {
        Repository {
            worktree: path.to_string(),
            gitdir: format!("{}/{}", path, GIT_DIR),
        }
    }

    pub fn init(&self) -> Result<(), String> {
        // Check if .bgit directory already exists
        if Path::new(&self.gitdir).exists() {
            return Err(format!("{} directory already exists", GIT_DIR));
        }

        // Create worktree directory if it doesn't exist
        if !Path::new(&self.worktree).exists() {
            fs::create_dir_all(&self.worktree)
                .map_err(|e| format!("Failed to create worktree directory: {}", e))?;
        }

        // Create .bgit directory
        fs::create_dir(&self.gitdir)
            .map_err(|e| format!("Failed to create {} directory: {}", GIT_DIR, e))?;

        // Create necessary subdirectories
        let subdirs = ["objects", "refs/heads", "refs/tags"];
        for dir in subdirs.iter() {
            let path = format!("{}/{}", self.gitdir, dir);
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
        }

        // Create HEAD file
        let head_path = format!("{}/HEAD", self.gitdir);
        fs::write(&head_path, "ref: refs/heads/master\n")
            .map_err(|e| format!("Failed to create HEAD file: {}", e))?;

        let full_path = fs::canonicalize(&self.gitdir)
            .map_err(|e| format!("Failed to get absolute path: {}", e))?;
        println!(
            "Initialized empty bGit repository in {}",
            full_path.display()
        );
        Ok(())
    }
}
