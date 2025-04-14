use crate::repository::{GIT_DIR, HEAD, ObjectType, RefValue, Repository};
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
    let gitignore_path = Path::new(&repo.worktree).join(".bgitignore");
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
    let target_dir = temp_dir.path().join("new_dir");
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
    let target_dir = temp_dir.path().join("new_dir");
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
fn test_empty_current_directory_with_bgitignore() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create .bgitignore file
    let gitignore_path = Path::new(&repo.worktree).join(".bgitignore");
    fs::write(&gitignore_path, "ignored.txt\n").unwrap();

    // Create test directory with ignored and non-ignored files
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("ignored.txt"), "Ignored content").unwrap();
    fs::write(test_dir.join("normal.txt"), "Normal content").unwrap();
    fs::write(test_dir.join(".bgitignore"), "ignored.txt\n").unwrap();
    fs::write(test_dir.join(".gitignore"), "ignored.txt\n").unwrap();
    fs::write(test_dir.join("settings.json"), "{}").unwrap();

    // Empty the directory
    assert!(repo.empty_current_directory(&test_dir).is_ok());

    // Verify ignored files is not deleted
    assert!(test_dir.join("ignored.txt").exists());

    // Verify normal files are deleted
    assert!(!test_dir.join("normal.txt").exists());

    // Verify hardcoded files are not deleted
    assert!(test_dir.join(".bgitignore").exists());
    assert!(test_dir.join(".gitignore").exists());
    assert!(test_dir.join("settings.json").exists());
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

#[test]
fn test_commit_success() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create some files
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Content 2").unwrap();

    // Create tree
    let tree_hash = repo.create_tree(&Path::new(&repo_path)).unwrap();

    // Create commit
    let commit_message = "Initial commit";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Verify commit hash format
    assert_eq!(commit_hash.len(), 40);
    assert!(commit_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify commit object exists
    let (dir, file) = commit_hash.split_at(2);
    let object_path = format!("{}/{}/objects/{}/{}", repo_path, GIT_DIR, dir, file);
    assert!(Path::new(&object_path).exists());

    // Verify commit content
    let commit_data = repo.get_object(&commit_hash).unwrap();
    let commit_str = String::from_utf8(commit_data).unwrap();
    println!("Commit string: {}", commit_str);
    assert!(commit_str.contains(&format!("tree {}", tree_hash)));
    assert!(commit_str.contains(commit_message));
    assert!(!commit_str.contains("parent")); // First commit shouldn't have parent
}

#[test]
fn test_commit_with_parent() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let first_commit_message = "First commit";
    let first_commit_hash = repo.create_commit(first_commit_message).unwrap();

    // Create second commit
    let second_commit_message = "Second commit";
    let second_commit_hash = repo.create_commit(second_commit_message).unwrap();

    // Verify second commit has parent
    let commit_data = repo.get_object(&second_commit_hash).unwrap();
    let commit_str = String::from_utf8(commit_data).unwrap();
    assert!(commit_str.contains(&format!("parent {}", first_commit_hash)));
}

#[test]
fn test_set_head() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create a commit
    let commit_message = "Test commit";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Set HEAD manually
    assert!(
        repo.set_ref(
            HEAD,
            RefValue {
                value: commit_hash.to_string(),
                is_symbolic: false,
            },
        )
        .is_ok()
    );

    // Verify HEAD content
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    let head_content = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_content.trim(), commit_hash);
}

#[test]
fn test_get_head() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create a commit
    let commit_message = "Test commit";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Get HEAD
    let head_hash = repo.get_ref(HEAD).unwrap();
    assert_eq!(head_hash.value, commit_hash);

    // Test getting HEAD when it doesn't exist
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    fs::remove_file(&head_path).unwrap();
    assert!(repo.get_ref(HEAD).is_err());
}

#[test]
fn test_commit_empty_message() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create empty directory
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    // Try to create commit with empty message
    let result = repo.create_commit("");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Commit message cannot be empty")
    );

    // Try to create commit with whitespace-only message
    let result = repo.create_commit("   ");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Commit message cannot be empty")
    );
}

#[test]
fn test_commit_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create complex directory structure
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
    let tree_hash = repo.create_tree(&Path::new(&repo_path)).unwrap();

    // Create commit
    let commit_message = "Commit with multiple files";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Verify commit hash format
    assert_eq!(commit_hash.len(), 40);
    assert!(commit_hash.chars().all(|c| c.is_ascii_hexdigit()));

    // Verify commit content
    let commit_data = repo.get_object(&commit_hash).unwrap();
    let commit_str = String::from_utf8(commit_data).unwrap();
    assert!(commit_str.contains(&format!("tree {}", tree_hash)));
    assert!(commit_str.contains(commit_message));
}

#[test]
fn test_get_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let commit_message = "Initial commit";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Get and verify commit
    let commit = repo.get_commit(&commit_hash).unwrap();
    assert_eq!(commit.message, commit_message);
    assert!(commit.parent.is_none());
    assert!(!commit.timestamp.is_empty());
    assert!(!commit.tree.is_empty());

    // Create second commit
    let second_message = "Second commit";
    let second_hash = repo.create_commit(second_message).unwrap();

    // Get and verify second commit
    let second_commit = repo.get_commit(&second_hash).unwrap();
    assert_eq!(second_commit.message, second_message);
    assert_eq!(second_commit.parent.unwrap(), commit_hash);
    assert!(!second_commit.timestamp.is_empty());
    assert!(!second_commit.tree.is_empty());

    // Test invalid commit hash
    let result = repo.get_commit("invalidhash");
    assert!(result.is_err());
}

#[test]
fn test_log() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create first commit
    let first_message = "First commit";
    let first_hash = repo.create_commit(first_message).unwrap();

    // Create second commit
    let second_message = "Second commit";
    let second_hash = repo.create_commit(second_message).unwrap();

    // Create third commit
    let third_message = "Third commit";
    let third_hash = repo.create_commit(third_message).unwrap();

    // Get all commits in order
    let commits = repo
        .iter_commits_and_parents(vec![third_hash.clone()])
        .unwrap();

    // Verify commits are in correct order
    assert_eq!(commits.len(), 3);
    assert_eq!(commits[0], third_hash);
    assert_eq!(commits[1], second_hash);
    assert_eq!(commits[2], first_hash);

    // Verify commit data
    let third_commit = repo.get_commit(&third_hash).unwrap();
    assert_eq!(third_commit.message, third_message);
    assert_eq!(third_commit.parent.unwrap(), second_hash);

    let second_commit = repo.get_commit(&second_hash).unwrap();
    assert_eq!(second_commit.message, second_message);
    assert_eq!(second_commit.parent.unwrap(), first_hash);

    let first_commit = repo.get_commit(&first_hash).unwrap();
    assert_eq!(first_commit.message, first_message);
    assert!(first_commit.parent.is_none());
}

#[test]
fn test_log_empty_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Log should fail when there are no commits
    let result = repo.log();
    assert!(result.is_err());
}

#[test]
fn test_checkout_success() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let first_message = "First commit";
    let first_hash = repo.create_commit(first_message).unwrap();

    // Create some files for second commit
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
    fs::write(test_dir.join("file2.txt"), "Content 2").unwrap();

    // Create second commit
    let second_message = "Second commit";
    let second_hash = repo.create_commit(second_message).unwrap();

    // Checkout first commit
    assert!(repo.checkout(&first_hash).is_ok());

    // Verify HEAD points to first commit
    let head_hash = repo.get_ref(HEAD).unwrap();
    assert_eq!(head_hash.value, first_hash);

    // Verify worktree is empty (first commit had no files)
    assert!(!test_dir.join("file1.txt").exists());
    assert!(!test_dir.join("file2.txt").exists());

    // Checkout second commit
    assert!(repo.checkout(&second_hash).is_ok());

    // Verify HEAD points to second commit
    let head_hash = repo.get_ref(HEAD).unwrap();
    assert_eq!(head_hash.value, second_hash);

    // Verify worktree has the files from second commit
    assert!(test_dir.join("file1.txt").exists());
    assert!(test_dir.join("file2.txt").exists());
    assert_eq!(
        fs::read_to_string(test_dir.join("file1.txt")).unwrap(),
        "Content 1"
    );
    assert_eq!(
        fs::read_to_string(test_dir.join("file2.txt")).unwrap(),
        "Content 2"
    );
}

#[test]
fn test_checkout_invalid_hash() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Try to checkout with invalid hash format
    let result = repo.checkout("not40chars");
    assert!(result.is_err());
    assert!(
        result.as_ref().unwrap_err().contains("Invalid hash format"),
        "Expected error message to contain 'Invalid hash format', but got: {}",
        result.unwrap_err()
    );

    // Try to checkout non-existent commit with valid hash format
    let result = repo.checkout("0000000000000000000000000000000000000000");
    assert!(result.is_err());
    let error = result.unwrap_err();
    println!("Actual error message: {}", error);
    assert!(error.contains("Commit with hash: 0000000000000000000000000000000000000000 not found"));
}

#[test]
fn test_checkout_with_existing_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit with some files
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file1.txt"), "Initial content").unwrap();
    let first_hash = repo.create_commit("First commit").unwrap();

    // Create some additional files not in the commit
    fs::write(test_dir.join("file2.txt"), "Uncommitted content").unwrap();
    fs::write(test_dir.join("file3.txt"), "Another uncommitted file").unwrap();

    // Checkout the same commit (should work and preserve uncommitted files)
    assert!(repo.checkout(&first_hash).is_ok());

    // Verify committed file exists with correct content
    assert!(test_dir.join("file1.txt").exists());
    assert_eq!(
        fs::read_to_string(test_dir.join("file1.txt")).unwrap(),
        "Initial content"
    );

    // Verify uncommitted files are not present
    assert!(!test_dir.join("file2.txt").exists());
    assert!(!test_dir.join("file3.txt").exists());
}

#[test]
fn test_checkout_directory_structure() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit with nested directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("nested.txt"), "Nested content").unwrap();
    let first_hash = repo.create_commit("First commit").unwrap();

    // Create second commit with different structure
    fs::remove_dir_all(&test_dir).unwrap();
    fs::create_dir(&test_dir).unwrap();
    fs::write(test_dir.join("file.txt"), "New content").unwrap();
    let second_hash = repo.create_commit("Second commit").unwrap();

    // Checkout first commit
    assert!(repo.checkout(&first_hash).is_ok());

    // Verify directory structure from first commit
    assert!(test_dir.join("subdir").exists());
    assert!(test_dir.join("subdir/nested.txt").exists());
    assert_eq!(
        fs::read_to_string(test_dir.join("subdir/nested.txt")).unwrap(),
        "Nested content"
    );
    assert!(!test_dir.join("file.txt").exists());

    // Checkout second commit
    assert!(repo.checkout(&second_hash).is_ok());

    // Verify directory structure from second commit
    assert!(!test_dir.join("subdir").exists());
    assert!(test_dir.join("file.txt").exists());
    assert_eq!(
        fs::read_to_string(test_dir.join("file.txt")).unwrap(),
        "New content"
    );
}

#[test]
fn test_ignore_directories() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create test directory structure
    let test_dir = temp_dir.path().join("test_dir");
    fs::create_dir(&test_dir).unwrap();

    // Create a directory that should be ignored
    let ignored_dir = test_dir.join("node_modules");
    fs::create_dir(&ignored_dir).unwrap();
    fs::write(ignored_dir.join("package.json"), "{}").unwrap();

    // Create a .git directory that should be ignored
    let git_dir = test_dir.join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(git_dir.join("config"), "{}").unwrap();

    // Create .bgitignore file with directory pattern
    fs::write(
        Path::new(&repo.worktree).join(".bgitignore"),
        "node_modules/\n",
    )
    .unwrap();

    // Create some normal files and directories
    fs::write(test_dir.join("file1.txt"), "Content 1").unwrap();
    let normal_dir = test_dir.join("src");
    fs::create_dir(&normal_dir).unwrap();
    fs::write(normal_dir.join("main.rs"), "fn main() {}").unwrap();

    // Empty the directory
    assert!(repo.empty_current_directory(&test_dir).is_ok());

    // Verify ignored directory still exists with its contents
    assert!(ignored_dir.exists());
    assert!(ignored_dir.join("package.json").exists());

    // Verify .git directory still exists with its contents
    assert!(git_dir.exists());
    assert!(git_dir.join("config").exists());

    // Verify normal files and directories are removed
    assert!(!test_dir.join("file1.txt").exists());
    assert!(!normal_dir.exists());
    assert!(!normal_dir.join("main.rs").exists());
}

#[test]
fn test_is_ignored_with_directory_pattern() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // Initialize a repository
    let repo = Repository::new(repo_path.to_str().unwrap());
    repo.init().unwrap();

    // Create a .bgitignore file with a directory pattern
    let gitignore_path = Path::new(&repo.worktree).join(".bgitignore");
    fs::write(&gitignore_path, "target\n").unwrap();

    // Create a target directory
    let target_dir = repo_path.join("target");
    fs::create_dir_all(&target_dir).unwrap();

    // Create some files in the target directory
    fs::write(target_dir.join("file1.txt"), "test").unwrap();
    fs::write(target_dir.join("file2.txt"), "test").unwrap();

    // Test that the target directory is ignored
    assert!(repo.is_ignored(&target_dir));

    // Test that files inside the target directory are also ignored
    assert!(repo.is_ignored(&target_dir.join("file1.txt")));
    assert!(repo.is_ignored(&target_dir.join("file2.txt")));

    // Test that other directories are not ignored
    let other_dir = repo_path.join("src");
    fs::create_dir_all(&other_dir).unwrap();
    assert!(!repo.is_ignored(&other_dir));

    // Test that files in other directories are not ignored
    let other_file = other_dir.join("main.rs");
    fs::write(&other_file, "test").unwrap();
    assert!(!repo.is_ignored(&other_file));
}

#[test]
fn test_head_content_after_init() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Verify HEAD content after initialization
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    let head_content = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_content, "ref: refs/heads/master\n");

    // Verify log fails with appropriate error message
    let result = repo.log();
    assert!(result.is_err());
    assert!(
        result.as_ref().unwrap_err().contains("No commits found"),
        "Expected error message to contain 'No commits found', but got: {}",
        result.unwrap_err()
    );
}

#[test]
fn test_first_commit_handling() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create a file for the first commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();

    // Create first commit
    let commit_message = "First commit";
    let commit_hash = repo.create_commit(commit_message).unwrap();

    // Verify HEAD now points to the commit hash
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    let head_content = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_content.trim(), commit_hash);

    // Verify commit has no parent
    let commit = repo.get_commit(&commit_hash).unwrap();
    assert!(commit.parent.is_none());
    assert!(!commit.tree.is_empty());

    // Verify log works now
    assert!(repo.log().is_ok());
}

#[test]
fn test_empty_to_committed_transition() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Verify initial HEAD content
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    let initial_head = fs::read_to_string(&head_path).unwrap();
    assert_eq!(initial_head, "ref: refs/heads/master\n");

    // Create first commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    let first_commit_hash = repo.create_commit("First commit").unwrap();

    // Verify HEAD now points to first commit
    let head_after_first = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_after_first.trim(), first_commit_hash);

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit_hash = repo.create_commit("Second commit").unwrap();

    // Verify HEAD now points to second commit
    let head_after_second = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_after_second.trim(), second_commit_hash);

    // Verify second commit has first commit as parent
    let second_commit = repo.get_commit(&second_commit_hash).unwrap();
    assert_eq!(second_commit.parent.unwrap(), first_commit_hash);
}

#[test]
fn test_create_tag() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create a commit to tag
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    let commit_hash = repo.create_commit("Initial commit").unwrap();

    // Create a tag
    let tag_name = "v1.0.0";
    assert!(repo.create_tag(tag_name, &commit_hash).is_ok());

    // Verify tag file exists and contains correct hash
    let tag_path = format!("{}/{}/refs/tags/{}", repo_path, GIT_DIR, tag_name);
    let tag_content = fs::read_to_string(&tag_path).unwrap();
    assert_eq!(tag_content.trim(), commit_hash);
}

#[test]
fn test_create_multiple_tags() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    let first_commit = repo.create_commit("Initial commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Create tags for both commits
    assert!(repo.create_tag("v1.0.0", &first_commit).is_ok());
    assert!(repo.create_tag("v1.1.0", &second_commit).is_ok());

    // Verify both tags exist with correct hashes
    let tag1_path = format!("{}/{}/refs/tags/v1.0.0", repo_path, GIT_DIR);
    let tag2_path = format!("{}/{}/refs/tags/v1.1.0", repo_path, GIT_DIR);

    let tag1_content = fs::read_to_string(&tag1_path).unwrap();
    let tag2_content = fs::read_to_string(&tag2_path).unwrap();

    assert_eq!(tag1_content.trim(), first_commit);
    assert_eq!(tag2_content.trim(), second_commit);
}

#[test]
fn test_create_tag_invalid_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Try to create tag with invalid commit hash
    let result = repo.create_tag("v1.0.0", "invalidhash");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: invalidhash")
    );
}

#[test]
fn test_create_tag_overwrite() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit and tag
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    let first_commit = repo.create_commit("Initial commit").unwrap();
    assert!(repo.create_tag("v1.0.0", &first_commit).is_ok());

    // Create second commit and overwrite tag
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();
    assert!(repo.create_tag("v1.0.0", &second_commit).is_ok());

    // Verify tag now points to second commit
    let tag_path = format!("{}/{}/refs/tags/v1.0.0", repo_path, GIT_DIR);
    let tag_content = fs::read_to_string(&tag_path).unwrap();
    assert_eq!(tag_content.trim(), second_commit);
}

#[test]
fn test_hash_validation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Test get_object with invalid hash
    let result = repo.get_object("invalidhash");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: invalidhash")
    );

    // Test checkout with invalid hash
    let result = repo.checkout("invalidhash");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: invalidhash")
    );

    // Test create_tag with invalid hash
    let result = repo.create_tag("v1.0.0", "invalidhash");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: invalidhash")
    );

    // Test set_ref with invalid hash
    let result = repo.set_ref(
        HEAD,
        RefValue {
            value: "invalidhash".to_string(),
            is_symbolic: false,
        },
    );

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: invalidhash")
    );

    // Test with hash that's too short
    let result = repo.get_object("123");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid hash format: 123"));

    // Test with hash that's too long
    let result = repo.get_object(&"a".repeat(41));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid hash format"));

    // Test with hash containing non-hex characters
    let result = repo.get_object(&"g".repeat(40));
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid hash format"));
}

#[test]
fn test_get_oid_hash() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Test with direct hash
    let result = repo.get_oid_hash(&first_commit);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), first_commit);

    // Test with HEAD reference
    let result = repo.get_oid_hash("HEAD");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), second_commit);

    // Test with tag reference
    assert!(repo.create_tag("v1.0", &first_commit).is_ok());
    let result = repo.get_oid_hash("refs/tags/v1.0");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), first_commit);

    // Test with invalid hash
    let result = repo.get_oid_hash("invalidhash");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: invalidhash")
    );

    // Test with non-existent reference
    let result = repo.get_oid_hash("refs/heads/nonexistent");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Invalid hash format: refs/heads/nonexistent")
    );

    // Test with reference to reference (HEAD -> refs/heads/master)
    let result = repo.get_oid_hash("HEAD");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), second_commit);
}

#[test]
fn test_get_oid_hash_reference_chain() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Test content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Test resolving HEAD through reference chain
    let result = repo.get_oid_hash("HEAD");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), first_commit);

    // Create a tag pointing to the branch
    assert!(repo.create_tag("feature-tag", &first_commit).is_ok());

    // Test resolving tag through reference chain
    let result = repo.get_oid_hash("refs/tags/feature-tag");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), first_commit);
}
