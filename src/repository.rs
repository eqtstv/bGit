use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

pub const GIT_DIR: &str = ".bgit";

#[derive(Debug)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug)]
pub struct Commit {
    pub _tree: String,
    pub parent: Option<String>,
    pub timestamp: String,
    pub message: String,
}

impl ObjectType {
    fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
        }
    }
}

pub struct Repository {
    pub worktree: String,
    pub gitdir: String,
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

        // Create subdirectories
        let subdirs = ["objects", "refs/heads", "refs/tags"];

        // Create subdirectories
        for dir in subdirs.iter() {
            let path = format!("{}/{}", self.gitdir, dir);
            fs::create_dir_all(&path)
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
        }

        // Create HEAD file
        let head_path = format!("{}/HEAD", self.gitdir);
        fs::write(&head_path, "ref: refs/heads/master\n")
            .map_err(|e| format!("Failed to create HEAD file: {}", e))?;

        // Create settings file
        let full_path = fs::canonicalize(&self.gitdir)
            .map_err(|e| format!("Failed to get absolute path: {}", e))?;

        // Print success message
        println!(
            "Initialized empty bGit repository in {}",
            full_path.display()
        );

        Ok(())
    }

    pub fn hash_object(&self, data: &[u8], obj_type: ObjectType) -> Result<String, String> {
        // Create header: "{type} {size}\0"
        let header = format!("{} {}\0", obj_type.as_str(), data.len());

        // Combine header and data
        let mut object_data = Vec::new();
        object_data.extend_from_slice(header.as_bytes());
        object_data.extend_from_slice(data);

        // Create a new SHA-1 hasher
        let mut hasher = Sha1::new();

        // Update the hasher with the data
        hasher.update(&object_data);

        // Finalize the hasher and get the hash
        let hash = hasher.finalize();

        // Encode the hash as a hex string
        let hash_str = hex::encode(hash);

        // Create object path
        let (dir, file) = hash_str.split_at(2);
        let object_dir = format!("{}/objects/{}", self.gitdir, dir);
        let object_path = format!("{}/{}", object_dir, file);

        // Create directory if it doesn't exist
        fs::create_dir_all(&object_dir)
            .map_err(|e| format!("Failed to create object directory: {}", e))?;

        // Write the object data to the file
        fs::write(&object_path, &object_data)
            .map_err(|e| format!("Failed to write object file: {}", e))?;

        Ok(hash_str)
    }

    pub fn get_object(&self, hash: &str) -> Result<Vec<u8>, String> {
        // Validate hash format (should be 40 hex characters)
        if hash.len() != 40 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("Invalid hash format".to_string());
        }

        // Create object path
        let (dir, file) = hash.split_at(2);
        let object_path = format!("{}/objects/{}/{}", self.gitdir, dir, file);

        // Read the object file
        let object_data =
            fs::read(&object_path).map_err(|e| format!("Failed to read object: {}", e))?;

        // Parse the header
        let header_end = object_data
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| "Invalid object format: missing null byte".to_string())?;

        // Convert the header to a string
        let header = String::from_utf8(object_data[..header_end].to_vec())
            .map_err(|_| "Invalid header encoding".to_string())?;

        // Split the header into parts
        let mut parts = header.split_whitespace();

        // Get the object type
        let _obj_type = parts
            .next()
            .ok_or_else(|| "Missing object type".to_string())?;

        // Get the object size
        let _size = parts
            .next()
            .ok_or_else(|| "Missing object size".to_string())?;

        // Return the actual content (everything after the header)
        Ok(object_data[header_end + 1..].to_vec())
    }

    pub fn create_tree(&self, path: &Path) -> Result<String, String> {
        let mut entries = Vec::new();

        // Read the directory
        for entry in fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let entry_path = entry.path();
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid file name".to_string())?;

            // Ignore .bgit directory and ignored files
            if name == GIT_DIR || self.is_ignored(&entry_path) {
                continue;
            }

            let metadata = entry
                .metadata()
                .map_err(|e| format!("Failed to get metadata: {}", e))?;

            if metadata.is_file() {
                // For files, create a blob object
                let content =
                    fs::read(&entry_path).map_err(|e| format!("Failed to read file: {}", e))?;
                let hash = self.hash_object(&content, ObjectType::Blob)?;

                // Format: "100644 {name}\0{hash}"
                let mut entry_data = format!("100644 {}\0", name).into_bytes();
                entry_data.extend_from_slice(&hex::decode(hash).unwrap());
                entries.push(entry_data);
            } else if metadata.is_dir() {
                // For directories, recursively create tree objects
                let hash = self.create_tree(&entry_path)?;

                // Format: "40000 {name}\0{hash}"
                let mut entry_data = format!("40000 {}\0", name).into_bytes();
                entry_data.extend_from_slice(&hex::decode(hash).unwrap());
                entries.push(entry_data);
            }
        }

        // Sort entries by name
        entries.sort();

        // Combine all entries
        let mut tree_data = Vec::new();
        for entry in entries {
            tree_data.extend_from_slice(&entry);
        }

        // Create tree object
        self.hash_object(&tree_data, ObjectType::Tree)
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let gitignore_path = Path::new(&self.gitdir).join(".bgitignore");
        if !gitignore_path.exists() {
            return false;
        }
        let gitignore_content = match fs::read_to_string(gitignore_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Failed to read .gitignore file: {}", e);
                return false;
            }
        };

        // Get the relative path from the worktree
        let path_str = match path.strip_prefix(&self.worktree) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => path.to_string_lossy().to_string(),
        };

        let lines = gitignore_content.lines();
        for line in lines {
            if line.is_empty() || line.starts_with("#") {
                continue;
            }

            let pattern = line.trim();

            // Handle directory patterns (ending with /)
            if pattern.ends_with('/') {
                let dir_pattern = pattern.strip_suffix('/').unwrap_or(pattern);
                if path_str.contains(dir_pattern) && path.is_dir() {
                    return true;
                }
                continue;
            }

            // Handle wildcard patterns
            if pattern.contains('*') {
                let regex_pattern = pattern.replace(".", "\\.").replace("*", ".*");
                if let Ok(re) = regex::Regex::new(&regex_pattern) {
                    if re.is_match(&path_str) {
                        return true;
                    }
                }
                continue;
            }

            // Simple contains check for non-wildcard patterns
            if path_str.contains(pattern) {
                return true;
            }
        }
        false
    }

    pub fn read_tree(&self, tree_oid: &str, path: &Path) -> Result<(), String> {
        // Empty the current directory first
        self.empty_current_directory(path)?;

        // Get the tree object
        let tree_data = self.get_object(tree_oid)?;

        // Parse tree entries
        let mut pos = 0;
        while pos < tree_data.len() {
            // Find the null byte that separates the mode+name from the hash
            let null_pos = tree_data[pos..]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| "Invalid tree format: missing null byte".to_string())?;

            // Parse mode and name
            let mode_name = &tree_data[pos..pos + null_pos];
            let mode_name_str = String::from_utf8(mode_name.to_vec())
                .map_err(|_| "Invalid mode/name encoding".to_string())?;

            let mut parts = mode_name_str.split_whitespace();
            let mode = parts.next().ok_or_else(|| "Missing mode".to_string())?;
            let name = parts.next().ok_or_else(|| "Missing name".to_string())?;

            // Get the hash (20 bytes after the null byte)
            let hash_start = pos + null_pos + 1;
            let hash_end = hash_start + 20;
            if hash_end > tree_data.len() {
                return Err("Invalid tree format: truncated hash".to_string());
            }

            let hash = hex::encode(&tree_data[hash_start..hash_end]);

            // Create the full path
            let entry_path = path.join(name);

            if mode == "100644" {
                // It's a file - create a blob
                let content = self.get_object(&hash)?;
                fs::write(&entry_path, content)
                    .map_err(|e| format!("Failed to write file {}: {}", name, e))?;
            } else if mode == "40000" {
                // It's a directory - create it and recurse
                fs::create_dir_all(&entry_path)
                    .map_err(|e| format!("Failed to create directory {}: {}", name, e))?;
                self.read_tree(&hash, &entry_path)?;
            } else {
                return Err(format!("Unsupported mode: {}", mode));
            }

            // Move to the next entry
            pos = hash_end;
        }

        Ok(())
    }

    pub fn empty_current_directory(&self, path: &Path) -> Result<(), String> {
        // Read all entries in the directory
        for entry in fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))? {
            let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
            let entry_path = entry.path();
            let name = entry_path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid file name".to_string())?;

            // Skip .bgit directory
            if name == GIT_DIR {
                continue;
            }

            // Remove the entry
            if entry_path.is_dir() {
                fs::remove_dir_all(&entry_path)
                    .map_err(|e| format!("Failed to remove directory {}: {}", name, e))?;
            } else {
                fs::remove_file(&entry_path)
                    .map_err(|e| format!("Failed to remove file {}: {}", name, e))?;
            }
        }

        Ok(())
    }

    pub fn get_tree_data(
        &self,
        tree_oid: &str,
    ) -> Result<Vec<(String, String, String, ObjectType)>, String> {
        // Get the raw object data
        let tree_data = self.get_object(tree_oid)?;

        let mut entries = Vec::new();
        let mut pos = 0;

        while pos < tree_data.len() {
            // Find the null byte that separates the mode+name from the hash
            let null_pos = tree_data[pos..]
                .iter()
                .position(|&b| b == 0)
                .ok_or_else(|| "Invalid tree format: missing null byte".to_string())?;

            // Parse mode and name
            let mode_name = &tree_data[pos..pos + null_pos];
            let mode_name_str = String::from_utf8(mode_name.to_vec())
                .map_err(|_| "Invalid mode/name encoding".to_string())?;

            let mut parts = mode_name_str.split_whitespace();
            let mode = parts.next().ok_or_else(|| "Missing mode".to_string())?;
            let name = parts.next().ok_or_else(|| "Missing name".to_string())?;

            // Get the hash (20 bytes after the null byte)
            let hash_start = pos + null_pos + 1;
            let hash_end = hash_start + 20;
            if hash_end > tree_data.len() {
                return Err("Invalid tree format: truncated hash".to_string());
            }

            let hash = hex::encode(&tree_data[hash_start..hash_end]);

            // Determine object type based on mode
            let obj_type = match mode {
                "100644" => ObjectType::Blob,
                "40000" => ObjectType::Tree,
                _ => return Err(format!("Unsupported mode: {}", mode)),
            };

            // Add the entry to the result
            entries.push((mode.to_string(), name.to_string(), hash, obj_type));

            // Move to the next entry
            pos = hash_end;
        }

        Ok(entries)
    }

    pub fn create_commit(&self, message: &str) -> Result<String, String> {
        if message.trim().is_empty() {
            return Err("Commit message cannot be empty".to_string());
        }

        let mut commit_data = Vec::new();

        // Create tree from worktree
        let tree_oid = self.create_tree(Path::new(&self.worktree))?;

        // Add tree hash
        commit_data.extend_from_slice(b"tree ");
        commit_data.extend_from_slice(tree_oid.as_bytes());
        commit_data.extend_from_slice(b"\n");

        // Add parent commit if HEAD exists and contains a valid commit hash
        if let Ok(parent_hash) = self.get_head() {
            // Only add parent if it's a valid commit hash (40 hex characters)
            if parent_hash.len() == 40 && parent_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                commit_data.extend_from_slice(b"parent ");
                commit_data.extend_from_slice(parent_hash.as_bytes());
                commit_data.extend_from_slice(b"\n");
            }
        }

        // Add datetime
        let datetime = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        commit_data.extend_from_slice(b"timestamp ");
        commit_data.extend_from_slice(datetime.as_bytes());

        // 2 new lines
        commit_data.extend_from_slice(b"\n");
        commit_data.extend_from_slice(b"\n");

        // Add commit message
        commit_data.extend_from_slice(message.as_bytes());
        commit_data.extend_from_slice(b"\n");

        let hash = self.hash_object(&commit_data, ObjectType::Commit)?;

        // Set HEAD to point to the new commit
        self.set_head(&hash)?;

        Ok(hash)
    }

    pub fn set_head(&self, commit_hash: &str) -> Result<(), String> {
        let head_path = format!("{}/HEAD", self.gitdir);
        fs::write(&head_path, commit_hash)
            .map_err(|e| format!("Failed to update HEAD file: {}", e))?;
        Ok(())
    }

    pub fn get_head(&self) -> Result<String, String> {
        let head_path = format!("{}/HEAD", self.gitdir);
        fs::read_to_string(&head_path)
            .map_err(|e| format!("Failed to read HEAD file: {}", e))
            .map(|content| content.trim().to_string())
    }

    pub fn get_commit(&self, hash: &str) -> Result<Commit, String> {
        // Get the raw commit data
        let commit_data = self.get_object(hash)?;
        let commit_str =
            String::from_utf8(commit_data).map_err(|_| "Invalid commit encoding".to_string())?;

        // Parse the commit data
        let mut tree = None;
        let mut parent = None;
        let mut timestamp = None;
        let mut message = String::new();
        let mut in_message = false;

        for line in commit_str.lines() {
            if in_message {
                message.push_str(line);
                message.push('\n');
                continue;
            }

            if line.is_empty() {
                in_message = true;
                continue;
            }

            if let Some(rest) = line.strip_prefix("tree ") {
                tree = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("parent ") {
                parent = Some(rest.to_string());
            } else if let Some(rest) = line.strip_prefix("timestamp ") {
                timestamp = Some(rest.to_string());
            }
        }

        // Validate required fields
        let tree = tree.ok_or_else(|| "Missing tree hash in commit".to_string())?;
        let timestamp = timestamp.ok_or_else(|| "Missing timestamp in commit".to_string())?;

        // Remove trailing newline from message
        message = message.trim_end().to_string();

        Ok(Commit {
            _tree: tree,
            parent,
            timestamp,
            message,
        })
    }

    pub fn log(&self) -> Result<(), String> {
        // Get the current HEAD commit
        let head_hash = self.get_head()?;
        let mut current_hash = Some(head_hash);

        while let Some(hash) = current_hash {
            let commit = self.get_commit(&hash)?;

            // Print commit information
            println!();
            println!("\x1b[33mcommit {}\x1b[0m", hash);
            if let Some(parent) = &commit.parent {
                println!("parent {}", parent);
            }
            println!("Date:   {}", commit.timestamp);
            println!();
            println!("    {}", commit.message);
            println!();

            // Move to parent commit
            current_hash = commit.parent;
        }

        Ok(())
    }
}
