use sha1::{Digest, Sha1};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;

use crate::differ::Differ;

pub const GIT_DIR: &str = ".bgit";
pub const HEAD: &str = "HEAD";
pub const MERGE_HEAD: &str = "MERGE_HEAD";

#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub parents: Vec<String>,
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

        // Create master branch
        let master_branch = format!("{}/refs/heads/master", self.gitdir);
        fs::write(&master_branch, "")
            .map_err(|e| format!("Failed to create master branch: {}", e))?;

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

        // Merge HEAD
        if let Ok(merge_head) = self.get_ref(MERGE_HEAD, true) {
            commit_data.extend_from_slice(b"parent ");
            commit_data.extend_from_slice(merge_head.value.as_bytes());
            commit_data.extend_from_slice(b"\n");

            // Remove MERGE_HEAD
            self.delete_ref(MERGE_HEAD, false)?;
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

        if is_symbolic && deref {
            // Extract the target ref name and recursively resolve it
            let target_ref = content.strip_prefix("ref:").unwrap().trim();
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
    }

    pub fn delete_ref(&self, ref_name: &str, deref: bool) -> Result<(), String> {
        let ref_value = self.get_ref_internal(ref_name, deref)?;

        let ref_path = format!("{}/{}", self.gitdir, ref_value.0);

        fs::remove_file(&ref_path)
            .map_err(|e| format!("Failed to delete {} file: {}", ref_name, e))?;
        Ok(())
    }

    pub fn get_commit(&self, hash: &str) -> Result<Commit, String> {
        // Get the raw commit data
        let hash = self.get_oid_hash(hash)?;
        let commit_data = self.get_object(hash.as_str())?;
        let commit_str =
            String::from_utf8(commit_data).map_err(|_| "Invalid commit encoding".to_string())?;

        // Parse the commit data
        let mut tree = None;
        let mut parents = Vec::new();
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
                parents.push(rest.to_string());
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
            parents,
            timestamp,
            message,
        })
    }

    pub fn log(&self) -> Result<(), String> {
        // Get the current HEAD commit
        let head_hash = self
            .get_ref(HEAD, true)
            .map_err(|e| format!("No commits found: {}", e))?;

        let current_hash = head_hash.value;

        if current_hash.is_empty() {
            return Ok(());
        }

        let commits = self.iter_commits_and_parents(vec![current_hash])?;

        for hash in commits {
            let commit = self.get_commit(&hash)?;

            // Get all refs pointing to this commit
            let mut refs = Vec::new();

            // Check branches
            let branch_refs = self.iter_refs("refs/heads/")?;
            for (name, ref_hash) in branch_refs {
                if ref_hash == hash {
                    let branch_name = name.split("/").last().unwrap();
                    refs.push(format!("branch: {}", branch_name));
                }
            }

            // Check tags
            let tag_refs = self.iter_refs("refs/tags/")?;
            for (name, ref_hash) in tag_refs {
                if ref_hash == hash {
                    let tag_name = name.split("/").last().unwrap();
                    refs.push(format!("tag: {}", tag_name));
                }
            }

            // Print commit information
            println!();
            println!("\x1b[33mcommit {}\x1b[0m", hash);
            if !commit.parents.is_empty() {
                for parent in commit.parents {
                    println!("parents {}", parent);
                }
            }
            println!("tree {}", commit.tree);
            println!("Date:   {}", commit.timestamp);
            if !refs.is_empty() {
                println!("Refs:   {}", refs.join(", "));
            }
            println!();
            println!("    {}", commit.message);
            println!();
        }

        Ok(())
    }

    pub fn checkout(&self, value: &str) -> Result<(), String> {
        // Get oid hash
        let commit_hash = Self::get_oid_hash(self, value)?;

        // Validate hash format
        Self::validate_commit_hash(&commit_hash)?;

        // Get the commit from the hash
        let commit = self
            .get_commit(commit_hash.as_str())
            .map_err(|_| format!("Commit with hash: {} not found", commit_hash))?;

        // Read the commit tree
        self.read_tree(&commit.tree, Path::new(&self.worktree))?;

        // If the value is a branch, set the HEAD to the last commit of the branch
        // else set the HEAD to the commit hash
        let new_head = if self.is_branch(value)? {
            format!("refs/heads/{}", value)
        } else {
            commit_hash.clone()
        };

        // If the value is a branch, the ref is symbolic, else it is direct
        let new_is_symbolic = self.is_branch(value)?;

        // Set the HEAD to the new head
        self.set_ref(
            HEAD,
            RefValue {
                value: new_head,
                is_symbolic: new_is_symbolic,
            },
            false,
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
        let mut value_to_search = value;

        if value == "@" {
            value_to_search = "HEAD";
        }

        // First check if it's a direct hash
        if Self::is_hash(value_to_search)? {
            return Ok(value_to_search.to_string());
        }

        let refs_to_try = [
            value_to_search.to_string(),
            format!("refs/{}", value_to_search),
            format!("refs/tags/{}", value_to_search),
            format!("refs/heads/{}", value_to_search),
        ];

        for ref_to_try in refs_to_try {
            match self.get_ref(ref_to_try.as_str(), true) {
                Ok(ref_hash) => {
                    return Ok(ref_hash.value);
                }
                Err(_e) => continue,
            }
        }

        Err(format!("Oid hash not found for: {}", value_to_search))
    }

    pub fn iter_refs(&self, prefix: &str) -> Result<Vec<(String, String)>, String> {
        let ref_folder = "refs";
        let refs_dir = format!("{}/{}", self.gitdir, ref_folder);
        let mut refs = Vec::new();

        // Helper function to recursively collect refs
        fn collect_refs(
            path: &Path,
            ref_folder: &str,
            refs: &mut Vec<(String, String)>,
            prefix: &str,
        ) -> Result<(), String> {
            let files_to_ignore = [".DS_Store"];

            for entry in
                fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let entry_path = entry.path();

                if !format!("{}/", entry_path.to_string_lossy()).starts_with(prefix) {
                    continue;
                }

                if files_to_ignore
                    .iter()
                    .any(|f| f == &entry_path.file_name().unwrap().to_string_lossy())
                {
                    continue;
                }

                if entry_path.is_dir() {
                    collect_refs(&entry_path, ref_folder, refs, prefix)?;
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
        collect_refs(
            Path::new(&refs_dir),
            ref_folder,
            &mut refs,
            format!("{}/{}", self.gitdir, prefix).as_str(),
        )?;

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

            if !commit.parents.is_empty() {
                for parent in commit.parents {
                    queue.push_back(parent.clone());
                }
            }
        }

        Ok(result)
    }

    pub fn create_branch(
        &self,
        branch_name: &str,
        commit_hash: Option<String>,
    ) -> Result<(), String> {
        let hash = match commit_hash {
            Some(hash) => hash,
            None => {
                let (_, head_value) = self.get_ref_internal(HEAD, true)?;
                head_value.value
            }
        };

        self.set_ref(
            format!("refs/heads/{}", branch_name).as_str(),
            RefValue {
                value: hash,
                is_symbolic: false,
            },
            true,
        )
    }

    pub fn is_branch(&self, value: &str) -> Result<bool, String> {
        let ref_value: RefValue =
            match self.get_ref(format!("refs/heads/{}", value).as_str(), false) {
                Ok(value) => value,
                Err(_) => RefValue {
                    value: "".to_string(),
                    is_symbolic: false,
                },
            };

        if ref_value.value.is_empty() {
            return Ok(false);
        }

        Ok(true)
    }

    pub fn get_branch_name(&self) -> Result<Option<String>, String> {
        let head_ref: RefValue = self.get_ref(HEAD, false)?;

        if !head_ref.is_symbolic {
            Ok(None)
        } else {
            assert!(head_ref.value.starts_with("ref: refs/heads/"));
            Ok(Some(head_ref.value[16..].to_string()))
        }
    }

    pub fn iter_branch_names(&self) -> Result<Vec<String>, String> {
        let refs = self.iter_refs("refs/heads/")?;
        let current_branch = self.get_branch_name()?;

        let branch_names = refs
            .iter()
            .map(|(name, _hash)| {
                let branch_name = name.clone().split("/").last().unwrap().to_string();
                if let Some(current) = &current_branch {
                    if branch_name == *current {
                        format!("\x1b[32m* {}\x1b[0m", branch_name)
                    } else {
                        branch_name
                    }
                } else {
                    branch_name
                }
            })
            .collect();

        Ok(branch_names)
    }

    pub fn reset(&self, commit_hash: &str) -> Result<(), String> {
        // For now reset is working as --hard, so it will remove
        // all the changes in the working directory and set the HEAD to the commit hash

        // Check it the commit hash exists
        let commit = self
            .get_commit(commit_hash)
            .map_err(|_e| format!("Commit with hash: {} not found", commit_hash))?;

        // Update the working directory to match the commit
        self.read_tree(&commit.tree, Path::new(&self.worktree))?;

        // Set the HEAD to the commit hash
        self.set_ref(
            HEAD,
            RefValue {
                value: commit_hash.to_string(),
                is_symbolic: false,
            },
            true,
        )
        .map_err(|e| format!("Failed to reset to commit: {}", e))?;

        Ok(())
    }

    pub fn merge(&self, branch_name: &str) -> Result<(), String> {
        // Get refs
        let head_ref = self.get_ref(HEAD, true).unwrap();
        let branch_ref = self
            .get_ref(format!("refs/heads/{}", branch_name).as_str(), true)
            .unwrap();

        // Get head commits
        let curr_head_commit = self.get_commit(&head_ref.value).unwrap();
        let branch_head_commit = self.get_commit(&branch_ref.value).unwrap();
        let base_commit = self
            .get_commit(&self.get_merge_base(&curr_head_commit._oid, &branch_head_commit._oid)?)
            .unwrap();

        // Check if we can do a fast-forward merge
        // If the base commit is the same as the current HEAD, we can do a fast-forward merge
        if base_commit._oid == curr_head_commit._oid {
            // Update the working directory to match the branch head commit
            self.read_tree(&branch_head_commit.tree, Path::new(&self.worktree))?;

            // Update the HEAD to point to the branch head commit
            self.set_ref(
                HEAD,
                RefValue {
                    value: branch_ref.value,
                    is_symbolic: false,
                },
                false,
            )?;
            println!(
                "Successfully merged branch {} into current branch.\nFast-forward merge, no need to commit.",
                branch_name
            );
            return Ok(());
        }

        // If not a fast-forward merge, proceed with three-way merge
        // Set MERGE_HEAD
        self.set_ref(
            MERGE_HEAD,
            RefValue {
                value: branch_ref.value,
                is_symbolic: false,
            },
            false,
        )?;

        // Merge the trees
        self.read_tree_merged(
            &curr_head_commit.tree,
            &branch_head_commit.tree,
            Some(&base_commit.tree),
        )?;

        // TODO: Remove this later?
        // Remove MERGE_HEAD after successful merge
        self.delete_ref(MERGE_HEAD, false)?;

        println!(
            "Successfully merged branch {} into current branch.\nPlease commit the merge.",
            branch_name
        );

        Ok(())
    }

    pub fn read_tree_merged(
        &self,
        head_tree_oid: &str,
        other_tree_oid: &str,
        base_tree_oid: Option<&str>,
    ) -> Result<(), String> {
        // Empty the current directory first
        self.empty_current_directory(Path::new(&self.worktree))?;

        // Get the merged tree contents (path -> Result<Content, IsDirectoryMarker>)
        let differ = Differ::new(self);
        let merged_tree_result =
            differ.merge_trees(head_tree_oid, other_tree_oid, base_tree_oid)?;

        // Use a HashSet to track created directories to avoid redundant checks/creation attempts
        let mut created_dirs = std::collections::HashSet::new();

        // Sort paths to process parent directories before children implicitly
        // This ensures that `a/b` is processed before `a/b/c`
        let mut paths: Vec<_> = merged_tree_result.keys().cloned().collect();
        paths.sort();

        for path in paths {
            let full_path = Path::new(&self.worktree).join(&path);
            let result_entry = merged_tree_result.get(&path).unwrap();

            // Ensure parent directory exists
            if let Some(parent) = full_path.parent() {
                if !parent.starts_with(&self.worktree) {
                    // Avoid trying to create outside worktree
                    // This might happen with absolute paths, skip for safety
                    eprintln!(
                        "Warning: Skipping path outside worktree: {}",
                        full_path.display()
                    );
                    continue;
                }
                // Only create if not already tracked and exists
                if !created_dirs.contains(parent) && !parent.exists() {
                    if parent.is_file() {
                        return Err(format!(
                            "Merge conflict: trying to create directory {} where a file exists.",
                            parent.display()
                        ));
                    }
                    fs::create_dir_all(parent).map_err(|e| {
                        format!("Failed to create directory {}: {}", parent.display(), e)
                    })?;
                    created_dirs.insert(parent.to_path_buf()); // Track created dir
                }
            }

            match result_entry {
                Ok(content) => {
                    // It's a file
                    if full_path.is_dir() {
                        return Err(format!(
                            "Merge conflict: trying to write file {} where a directory exists.",
                            full_path.display()
                        ));
                    }
                    // Write the file content
                    fs::write(&full_path, content).map_err(|e| {
                        format!("Failed to write file {}: {}", full_path.display(), e)
                    })?;
                }
                Err(_) => {
                    // It's a directory marker
                    if full_path.is_file() {
                        return Err(format!(
                            "Merge conflict: trying to create directory {} where a file exists.",
                            full_path.display()
                        ));
                    }
                    // Ensure the directory itself exists if it wasn't created as a parent earlier
                    if !created_dirs.contains(&full_path) && !full_path.exists() {
                        fs::create_dir_all(&full_path).map_err(|e| {
                            format!("Failed to create directory {}: {}", full_path.display(), e)
                        })?;
                        created_dirs.insert(full_path.to_path_buf());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn print_commit(&self, commit_hash: &str) -> Result<(), String> {
        let commit = self
            .get_commit(commit_hash)
            .map_err(|_e| format!("Commit with hash: {} not found", commit_hash))?;

        println!("Commit: {}", commit_hash);
        println!("Tree: {}", commit.tree);
        if !commit.parents.is_empty() {
            for parent in commit.parents {
                println!("Parent: {}", parent);
            }
        } else {
            println!("Parent: None");
        }
        println!("Timestamp: {}", commit.timestamp);
        println!("Message: {}", commit.message);

        Ok(())
    }

    pub fn show(&self, commit_hash: &str) -> Result<(), String> {
        let commit = self
            .get_commit(commit_hash)
            .map_err(|_e| format!("Commit with hash: {} not found", commit_hash))?;

        self.print_commit(commit_hash)?;

        if !commit.parents.is_empty() {
            for parent in commit.parents {
                let parent_commit = self
                    .get_commit(&parent)
                    .map_err(|_e| format!("Commit with hash: {} not found", parent))?;

                let diff = Differ::new(self).diff_trees(&parent_commit.tree, &commit.tree)?;
                let colored_diff = Differ::colorize_diff(&diff);
                println!("{}", colored_diff);
            }
        }

        Ok(())
    }

    pub fn get_working_tree(&self) -> Result<String, String> {
        let tree = self.create_tree(Path::new(&self.worktree))?;
        Ok(tree)
    }

    pub fn diff(&self) -> Result<String, String> {
        // check if there is a HEAD
        let head = self.get_ref(HEAD, false)?;
        if head.value.is_empty() {
            return Err("No commits found".to_string());
        }

        let diff = Differ::new(self).diff_current_working_tree()?;
        let colored_diff = Differ::colorize_diff(&diff);
        Ok(colored_diff)
    }

    pub fn get_merge_base(&self, commit_hash1: &str, commit_hash2: &str) -> Result<String, String> {
        // Validate both commit hashes
        let commit1 = self.get_oid_hash(commit_hash1)?;
        let commit2 = self.get_oid_hash(commit_hash2)?;

        // If commits are the same, return that commit
        if commit1 == commit2 {
            return Ok(commit1);
        }

        // Get all ancestors of both commits
        let ancestors1 = self.get_commit_ancestors(&commit1)?;
        let ancestors2 = self.get_commit_ancestors(&commit2)?;

        // Find the first common ancestor
        for ancestor in &ancestors1 {
            if ancestors2.contains(ancestor) {
                return Ok(ancestor.clone());
            }
        }

        // If no common ancestor is found, return an error
        Err("No common ancestor found between commits".to_string())
    }

    pub fn get_commit_ancestors(&self, commit_hash: &str) -> Result<Vec<String>, String> {
        let mut ancestors = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(commit_hash.to_string());

        while let Some(current) = queue.pop_front() {
            // Skip if we've already processed this commit
            if ancestors.contains(&current) {
                continue;
            }

            // Add current commit to ancestors
            ancestors.push(current.clone());

            // Get the commit object
            let commit = self.get_commit(&current)?;

            // Add all parents to the queue
            for parent in commit.parents {
                queue.push_back(parent);
            }
        }

        Ok(ancestors)
    }

    pub fn rebase(&self, target: &str) -> Result<(), String> {
        // 1. Store original branch state (for potential abort)
        let original_head = self.get_ref(HEAD, true)?;
        let original_branch = self.get_branch_name()?;

        // 2. Get target commit and current HEAD
        let target_oid = self.get_oid_hash(target)?;
        let current_head_oid = self.get_oid_hash("HEAD")?;

        // 3. Find common ancestor
        let base_commit = self.get_merge_base(&current_head_oid, &target_oid)?;

        // 4. Collect commits to rebase (only those unique to our branch)
        let commits_to_rebase = self.iter_commits_and_parents(vec![current_head_oid.clone()])?;
        let base_commits = self.iter_commits_and_parents(vec![base_commit.clone()])?;
        let commits_to_rebase: Vec<String> = commits_to_rebase
            .into_iter()
            .filter(|commit| !base_commits.contains(commit))
            .collect();

        // 5. Switch to target commit (detached HEAD)
        self.checkout(&target_oid)?;

        // 6. Apply each commit on top of the target
        let mut new_base = target_oid;
        for commit_oid in commits_to_rebase.iter().rev() {
            let commit = self.get_commit(commit_oid)?;

            // Get the tree objects
            let target_tree = self.get_commit(&new_base)?.tree;
            let commit_tree = commit.tree;
            let base_tree = self.get_commit(&base_commit)?.tree;

            // Apply the changes from this commit
            self.read_tree_merged(&commit_tree, &target_tree, Some(&base_tree))?;

            // Create new commit using the existing function
            new_base = self.create_commit(&commit.message)?;
        }

        // 7. Update branch reference if we were on a branch
        if let Some(branch_name) = original_branch.clone() {
            self.set_ref(
                format!("refs/heads/{}", branch_name).as_str(),
                RefValue {
                    value: new_base.clone(),
                    is_symbolic: false,
                },
                true,
            )?;
        }

        // 8. Update HEAD
        if original_head.is_symbolic {
            self.set_ref(
                HEAD,
                RefValue {
                    value: format!("ref: refs/heads/{}", original_branch.unwrap()),
                    is_symbolic: true,
                },
                false,
            )?;
        } else {
            self.set_ref(
                HEAD,
                RefValue {
                    value: new_base,
                    is_symbolic: false,
                },
                false,
            )?;
        }

        println!("Successfully rebased branch onto target {}.", target);

        Ok(())
    }
}
