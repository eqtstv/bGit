use sha1::{Digest, Sha1};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

pub const GIT_DIR: &str = ".bgit";
pub const HEAD: &str = "HEAD";

#[derive(Debug)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
}

#[derive(Debug)]
pub struct RefValue {
    pub value: String,
    pub is_symbolic: bool,
}

#[derive(Debug)]
pub struct Commit {
    pub _oid: String,
    pub tree: String,
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

    fn validate_commit_hash(hash: &str) -> Result<(), String> {
        if hash.len() != 40 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(format!("Invalid hash format: {}", hash));
        }
        Ok(())
    }

    fn is_hash(value: &str) -> Result<bool, String> {
        if value.len() != 40 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(false);
        }
        Ok(true)
    }

    pub fn get_object(&self, hash: &str) -> Result<Vec<u8>, String> {
        // Validate hash format
        let hash = self.get_oid_hash(hash)?;

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

            // Ignore ignored files and directories
            if self.is_ignored(&entry_path) {
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
        let paths_to_ignore = [
            // bGit directories
            ".bgit",
            ".bgitignore",
            // Git directories
            ".git",
            ".gitignore",
            // Other
            "settings.json",
            ".DS_Store",
            ".vscode",
        ];

        for ignore_path in paths_to_ignore {
            if path.to_string_lossy().contains(ignore_path) {
                return true;
            }
        }

        // Look for .bgitignore in the root of the repository (worktree) instead of in .bgit
        let gitignore_path = Path::new(&self.worktree).join(".bgitignore");
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

        let tree_oid = self.get_oid_hash(tree_oid)?;

        // Get the tree object
        let tree_data = self.get_object(tree_oid.as_str())?;

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

            // Skip ignored files
            if self.is_ignored(&entry_path) {
                pos = hash_end;
                continue;
            }

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

            // Skip ignored files
            if self.is_ignored(&entry_path) {
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
        let tree_oid = self.get_oid_hash(tree_oid)?;
        let tree_data = self.get_object(tree_oid.as_str())?;

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
        if let Ok(parent_hash) = self.get_ref(HEAD, true) {
            // Only add parent if it's a valid commit hash (40 hex characters)
            if Self::is_hash(&parent_hash.value)? {
                commit_data.extend_from_slice(b"parent ");
                commit_data.extend_from_slice(parent_hash.value.as_bytes());
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
        self.set_ref(
            HEAD,
            RefValue {
                value: hash.clone(),
                is_symbolic: false,
            },
            true,
        )?;

        Ok(hash)
    }

    pub fn set_ref(&self, ref_name: &str, ref_value: RefValue, deref: bool) -> Result<(), String> {
        // Ability to set a symbolic ref
        let new_value = if ref_value.is_symbolic {
            // Set a symbolic ref
            format!("ref: {}", ref_value.value)
        } else {
            // Set a direct ref
            ref_value.value.clone()
        };

        // Try to get the actual reference, but if it doesn't exist, use the original name
        let ref_path = match self.get_ref_internal(ref_name, deref) {
            Ok((deref_name, _)) => format!("{}/{}", self.gitdir, deref_name),
            Err(_) => format!("{}/{}", self.gitdir, ref_name),
        };

        fs::write(&ref_path, new_value)
            .map_err(|e| format!("Failed to update {} file: {}", ref_name, e))?;
        Ok(())
    }

    pub fn get_ref(&self, ref_name: &str, deref: bool) -> Result<RefValue, String> {
        let (_, ref_value) = self.get_ref_internal(ref_name, deref)?;
        Ok(ref_value)
    }

    pub fn get_ref_internal(
        &self,
        ref_name: &str,
        deref: bool,
    ) -> Result<(String, RefValue), String> {
        // Get the ref path
        let ref_path = format!("{}/{}", self.gitdir, ref_name);

        // Read the ref file
        let content = fs::read_to_string(&ref_path)
            .map_err(|e| format!("Failed to read {} file: {}", ref_name, e))?;

        // Trim the content
        let content = content.trim();

        let is_symbolic = content.starts_with("ref:");

        if is_symbolic {
            // Extract the target ref name and recursively resolve it
            let target_ref = content.strip_prefix("ref:").unwrap().trim();
            if deref {
                self.get_ref_internal(target_ref, deref)
            } else {
                Ok((
                    ref_name.to_string(),
                    RefValue {
                        value: content.to_string(),
                        is_symbolic,
                    },
                ))
            }
        } else {
            Ok((
                ref_name.to_string(),
                RefValue {
                    value: content.to_string(),
                    is_symbolic,
                },
            ))
        }
    }

    pub fn get_commit(&self, hash: &str) -> Result<Commit, String> {
        // Get the raw commit data
        let hash = self.get_oid_hash(hash)?;
        let commit_data = self.get_object(hash.as_str())?;
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
            _oid: hash,
            tree,
            parent,
            timestamp,
            message,
        })
    }

    pub fn log(&self) -> Result<(), String> {
        // Get the current HEAD commit
        let head_hash = self
            .get_ref(HEAD, true)
            .map_err(|e| format!("No commits found: {}", e))?;

        let current_hash = Some(head_hash.value);

        let commits = self.iter_commits_and_parents(vec![current_hash.clone().unwrap()])?;

        for hash in commits {
            let commit = self.get_commit(&hash)?;

            // Print commit information
            println!();
            println!("\x1b[33mcommit {}\x1b[0m", hash);
            if let Some(parent) = &commit.parent {
                println!("parent {}", parent);
            }
            println!("tree {}", commit.tree);
            println!("Date:   {}", commit.timestamp);
            println!();
            println!("    {}", commit.message);
            println!();
        }

        Ok(())
    }

    pub fn checkout(&self, commit_hash: &str) -> Result<(), String> {
        // Validate hash format
        let commit_hash = Self::get_oid_hash(self, commit_hash)?;

        // Get the commit from the hash
        let commit = self
            .get_commit(commit_hash.as_str())
            .map_err(|_| format!("Commit with hash: {} not found", commit_hash))?;

        // Read the commit tree
        self.read_tree(&commit.tree, Path::new(&self.worktree))?;

        // Set HEAD to point to the new commit
        self.set_ref(
            HEAD,
            RefValue {
                value: commit_hash.clone(),
                is_symbolic: false,
            },
            true,
        )?;

        Ok(())
    }

    pub fn create_tag(&self, tag_name: &str, commit_hash: &str) -> Result<(), String> {
        // Validate hash format
        Self::validate_commit_hash(commit_hash)?;

        self.set_ref(
            format!("refs/tags/{}", tag_name).as_str(),
            RefValue {
                value: commit_hash.to_string(),
                is_symbolic: false,
            },
            true,
        )
    }

    pub fn get_oid_hash(&self, value: &str) -> Result<String, String> {
        if value == "@" {
            return self.get_ref(HEAD, true).map(|ref_value| ref_value.value);
        }

        // First check if it's a direct hash
        if Self::is_hash(value)? {
            return Ok(value.to_string());
        }

        let refs_to_try = [
            value.to_string(),
            format!("refs/{}", value),
            format!("refs/tags/{}", value),
            format!("refs/heads/{}", value),
        ];

        for ref_to_try in refs_to_try {
            match self.get_ref(ref_to_try.as_str(), true) {
                Ok(ref_hash) => {
                    return Ok(ref_hash.value);
                }
                Err(_e) => continue,
            }
        }

        Err(format!("Invalid hash format: {}", value))
    }

    pub fn iter_refs(&self) -> Result<Vec<(String, String)>, String> {
        let ref_folder = "refs";
        let refs_dir = format!("{}/{}", self.gitdir, ref_folder);
        let mut refs = Vec::new();

        // Helper function to recursively collect refs
        fn collect_refs(
            path: &Path,
            ref_folder: &str,
            refs: &mut Vec<(String, String)>,
        ) -> Result<(), String> {
            let files_to_ignore = [".DS_Store"];

            for entry in
                fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let entry_path = entry.path();

                if files_to_ignore
                    .iter()
                    .any(|f| f == &entry_path.file_name().unwrap().to_string_lossy())
                {
                    continue;
                }

                if entry_path.is_dir() {
                    collect_refs(&entry_path, ref_folder, refs)?;
                } else {
                    let content = fs::read_to_string(&entry_path).map_err(|e| {
                        format!(
                            "Failed to read ref file {}: {}",
                            entry_path.to_string_lossy(),
                            e
                        )
                    })?;
                    let hash = content.trim();

                    // Get the relative path from refs directory
                    let ref_name = entry_path
                        .strip_prefix(path.parent().unwrap())
                        .map_err(|e| format!("Failed to get relative path: {}", e))?
                        .to_string_lossy()
                        .to_string();

                    refs.push((format!("{}/{}", ref_folder, ref_name), hash.to_string()));
                }
            }
            Ok(())
        }

        // Start collecting refs from the refs directory
        collect_refs(Path::new(&refs_dir), ref_folder, &mut refs)?;

        Ok(refs)
    }

    pub fn iter_commits_and_parents(&self, oids: Vec<String>) -> Result<Vec<String>, String> {
        let mut visited: Vec<String> = Vec::new();
        let mut queue: VecDeque<String> = VecDeque::new();
        let mut result = Vec::new();

        for oid in oids {
            queue.push_back(oid);
        }

        while let Some(oid) = queue.pop_back() {
            if visited.contains(&oid) {
                continue;
            }

            visited.push(oid.clone());

            let oid_str = oid.clone();
            result.push(oid_str.clone());

            let commit = self.get_commit(&oid_str)?;

            if let Some(parent) = &commit.parent {
                queue.push_back(parent.clone());
            }
        }

        Ok(result)
    }

    pub fn create_branch(
        &self,
        branch_name: &str,
        commit_hash: Option<String>,
    ) -> Result<(), String> {
        self.set_ref(
            format!("refs/heads/{}", branch_name).as_str(),
            RefValue {
                value: commit_hash.unwrap_or("@".to_string()),
                is_symbolic: false,
            },
            true,
        )
    }
}
