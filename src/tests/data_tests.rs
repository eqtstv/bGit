use crate::data::{GIT_DIR, ObjectType, Repository};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_repository_init_success() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    assert!(repo.init().is_ok());

    // Verify directory structure
    assert!(Path::new(&format!("{}/{}", repo_path, GIT_DIR)).exists());
    assert!(Path::new(&format!("{}/{}/objects", repo_path, GIT_DIR)).exists());
    assert!(Path::new(&format!("{}/{}/refs/heads", repo_path, GIT_DIR)).exists());
    assert!(Path::new(&format!("{}/{}/refs/tags", repo_path, GIT_DIR)).exists());
    assert!(Path::new(&format!("{}/{}/HEAD", repo_path, GIT_DIR)).exists());

    // Verify HEAD content
    let head_content = fs::read_to_string(format!("{}/{}/HEAD", repo_path, GIT_DIR)).unwrap();
    assert_eq!(head_content, "ref: refs/heads/master\n");
}

#[test]
fn test_repository_init_already_exists() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    // Initialize repository first time
    let repo = Repository::new(repo_path);
    assert!(repo.init().is_ok());

    // Try to initialize again
    let result = repo.init();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("already exists"));
}

#[test]
fn test_hash_object_success() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Test with simple data
    let data = b"Hello, world!";
    let hash = repo.hash_object(data, ObjectType::Blob).unwrap();

    // Verify hash format (40 hex characters)
    assert_eq!(hash.len(), 40);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify object file exists
    let (dir, file) = hash.split_at(2);
    let object_path = format!("{}/{}/objects/{}/{}", repo_path, GIT_DIR, dir, file);
    assert!(Path::new(&object_path).exists());
}

#[test]
fn test_hash_object_empty_data() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Test with empty data
    let data = b"";
    let hash = repo.hash_object(data, ObjectType::Blob).unwrap();

    // Verify hash format
    assert_eq!(hash.len(), 40);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_get_object_success() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Store some data
    let original_data = b"Test content for get_object";
    let hash = repo.hash_object(original_data, ObjectType::Blob).unwrap();

    // Retrieve the data
    let retrieved_data = repo.get_object(&hash).unwrap();

    // Verify the retrieved data matches the original
    assert_eq!(retrieved_data, original_data);
}

#[test]
fn test_get_object_invalid_hash() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Test with invalid hash format
    let result = repo.get_object("invalidhash");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid hash format"));

    // Test with non-existent hash
    let result = repo.get_object("a".repeat(40).as_str());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read object"));
}

#[test]
fn test_get_object_corrupted_data() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Store some data
    let original_data = b"Test content";
    let hash = repo.hash_object(original_data, ObjectType::Blob).unwrap();

    // Corrupt the object file
    let (dir, file) = hash.split_at(2);
    let object_path = format!("{}/{}/objects/{}/{}", repo_path, GIT_DIR, dir, file);
    fs::write(&object_path, b"corrupted data").unwrap();

    // Try to retrieve the corrupted data
    let result = repo.get_object(&hash);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid object format"));
}

#[test]
fn test_create_tree_success() {
    // Create a temporary directory for testing
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create test directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    // Create a file
    fs::write(test_dir.join("test.txt"), "Hello, world!").unwrap();

    // Create a subdirectory
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested.txt"), "Nested content").unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&test_dir).unwrap();

    // Verify hash format
    assert_eq!(tree_hash.len(), 40);
    assert!(tree_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify object exists
    let (dir, file) = tree_hash.split_at(2);
    let object_path = format!("{}/{}/objects/{}/{}", repo_path, GIT_DIR, dir, file);
    assert!(Path::new(&object_path).exists());
}

#[test]
fn test_create_tree_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create empty directory
    let empty_dir = temp_dir.path().join("empty_dir");
    fs::create_dir(&empty_dir).unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&empty_dir).unwrap();
    assert_eq!(tree_hash.len(), 40);
}

#[test]
fn test_is_ignored() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create test files and directories
    let test_file = temp_dir.path().join("test.txt");
    let ignored_dir = temp_dir.path().join("ignored_dir");
    let test_rs = temp_dir.path().join("test.rs");

    fs::write(&test_file, "test content").unwrap();
    fs::create_dir(&ignored_dir).unwrap();
    fs::write(&test_rs, "rust code").unwrap();

    // Create .bgitignore file
    let gitignore_path = Path::new(&repo.gitdir).join(".bgitignore");
    fs::write(&gitignore_path, "*.txt\nignored_dir/\n").unwrap();

    // Test ignored files
    assert!(repo.is_ignored(&test_file));
    assert!(repo.is_ignored(&ignored_dir));
    assert!(!repo.is_ignored(&test_rs));
}

#[test]
fn test_read_tree_success() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create test directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    // Create files and subdirectories
    fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Content 2").unwrap();
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested.txt"), "Nested content").unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&test_dir).unwrap();

    // Create target directory for reading tree
    let target_dir = temp_dir.path().join("target_dir");
    fs::create_dir(&target_dir).unwrap();

    // Read tree into target directory
    assert!(repo.read_tree(&tree_hash, &target_dir).is_ok());

    // Verify structure
    assert!(target_dir.join("file1.txt").exists());
    assert!(target_dir.join("file2.txt").exists());
    assert!(target_dir.join("subdir").exists());
    assert!(target_dir.join("subdir/nested.txt").exists());

    // Verify content
    assert_eq!(
        fs::read_to_string(target_dir.join("file1.txt")).unwrap(),
        "Content 1"
    );
    assert_eq!(
        fs::read_to_string(target_dir.join("subdir/nested.txt")).unwrap(),
        "Nested content"
    );
}

#[test]
fn test_read_tree_with_existing_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create test directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("test.txt"), "New content").unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&test_dir).unwrap();

    // Create target directory with existing files
    let target_dir = temp_dir.path().join("target_dir");
    fs::create_dir(&target_dir).unwrap();
    fs::write(target_dir.join("old.txt"), "Old content").unwrap();
    fs::write(target_dir.join("test.txt"), "Old test content").unwrap();

    // Read tree into target directory
    assert!(repo.read_tree(&tree_hash, &target_dir).is_ok());

    // Verify old files are gone and new structure is correct
    assert!(!target_dir.join("old.txt").exists());
    assert!(target_dir.join("test.txt").exists());
    assert_eq!(
        fs::read_to_string(target_dir.join("test.txt")).unwrap(),
        "New content"
    );
}

#[test]
fn test_empty_current_directory() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create test directory with various files and subdirectories
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Content 2").unwrap();
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested.txt"), "Nested content").unwrap();

    // Empty the directory
    assert!(repo.empty_current_directory(&test_dir).is_ok());

    // Verify everything is gone except .bgit directory
    assert!(!test_dir.join("file1.txt").exists());
    assert!(!test_dir.join("file2.txt").exists());
    assert!(!test_dir.join("subdir").exists());
    assert!(Path::new(&repo.gitdir).exists());
}

#[test]
fn test_empty_current_directory_with_gitignore() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create .bgitignore file
    let gitignore_path = Path::new(&repo.gitdir).join(".bgitignore");
    fs::write(&gitignore_path, "ignored.txt\n").unwrap();

    // Create test directory with ignored and non-ignored files
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("ignored.txt"), "Ignored content").unwrap();
    fs::write(test_dir.join("normal.txt"), "Normal content").unwrap();

    // Empty the directory
    assert!(repo.empty_current_directory(&test_dir).is_ok());

    // Verify everything is gone
    assert!(!test_dir.join("ignored.txt").exists());
    assert!(!test_dir.join("normal.txt").exists());
}

#[test]
fn test_get_tree_data_simple() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create a simple directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "Hello, world!").unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&test_dir).unwrap();

    // Get tree data
    let entries = repo.get_tree_data(&tree_hash).unwrap();

    // Verify entries
    assert_eq!(entries.len(), 1);
    let (mode, name, hash, obj_type) = &entries[0];
    assert_eq!(mode, "100644");
    assert_eq!(name, "file.txt");
    assert_eq!(hash.len(), 40);
    assert!(matches!(obj_type, ObjectType::Blob));
}

#[test]
fn test_get_tree_data_complex() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create a complex directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    // Create files
    fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Content 2").unwrap();

    // Create subdirectory
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested.txt"), "Nested content").unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&test_dir).unwrap();

    // Get tree data
    let entries = repo.get_tree_data(&tree_hash).unwrap();

    // Verify entries
    assert_eq!(entries.len(), 3);

    // Sort entries by name for consistent testing
    let mut entries = entries;
    entries.sort_by(|a, b| a.1.cmp(&b.1));

    // Verify file1.txt
    let (mode, name, hash, obj_type) = &entries[0];
    assert_eq!(mode, "100644");
    assert_eq!(name, "file1.txt");
    assert_eq!(hash.len(), 40);
    assert!(matches!(obj_type, ObjectType::Blob));

    // Verify file2.txt
    let (mode, name, hash, obj_type) = &entries[1];
    assert_eq!(mode, "100644");
    assert_eq!(name, "file2.txt");
    assert_eq!(hash.len(), 40);
    assert!(matches!(obj_type, ObjectType::Blob));

    // Verify subdir
    let (mode, name, hash, obj_type) = &entries[2];
    assert_eq!(mode, "40000");
    assert_eq!(name, "subdir");
    assert_eq!(hash.len(), 40);
    assert!(matches!(obj_type, ObjectType::Tree));
}

#[test]
fn test_get_tree_data_empty() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create empty directory
    let empty_dir = temp_dir.path().join("empty_dir");
    fs::create_dir(&empty_dir).unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&empty_dir).unwrap();

    // Get tree data
    let entries = repo.get_tree_data(&tree_hash).unwrap();

    // Verify empty tree
    assert!(entries.is_empty());
}

#[test]
fn test_get_tree_data_invalid_hash() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Test with invalid hash
    let result = repo.get_tree_data("invalidhash");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid hash format"));

    // Test with non-existent hash
    let result = repo.get_tree_data(&"a".repeat(40));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read object"));
}
