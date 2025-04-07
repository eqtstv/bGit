use crate::data::{GIT_DIR, Repository};
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
    let hash = repo.hash_object(data).unwrap();

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
    let hash = repo.hash_object(data).unwrap();

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
    let hash = repo.hash_object(original_data).unwrap();

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
    let hash = repo.hash_object(original_data).unwrap();

    // Corrupt the object file
    let (dir, file) = hash.split_at(2);
    let object_path = format!("{}/{}/objects/{}/{}", repo_path, GIT_DIR, dir, file);
    fs::write(&object_path, b"corrupted data").unwrap();

    // Try to retrieve the corrupted data
    let result = repo.get_object(&hash);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid object format"));
}
