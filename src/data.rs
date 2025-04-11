use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

pub const GIT_DIR: &str = ".bgit";

#[derive(Debug)]
enum ObjectType {
    Blob,
    Tree,
}

impl ObjectType {
    fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
        }
    }
}

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

    pub fn hash_object(&self, data: &[u8]) -> Result<String, String> {
        // Create header: "blob {size}\0"
        let header = format!("{} {}\0", ObjectType::Blob.as_str(), data.len());

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
        let obj_type = parts
            .next()
            .ok_or_else(|| "Missing object type".to_string())?;

        // Get the object size
        let _size = parts
            .next()
            .ok_or_else(|| "Missing object size".to_string())?;

        // For now, we only support blobs
        if obj_type != "blob" {
            return Err(format!("Unsupported object type: {}", obj_type));
        }

        // Return the actual content (everything after the header)
        Ok(object_data[header_end + 1..].to_vec())
    }
}
