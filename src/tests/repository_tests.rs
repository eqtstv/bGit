use crate::repository::{GIT_DIR, HEAD, MERGE_HEAD, ObjectType, RefValue, Repository};
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tempfile::tempdir;

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
    assert!(result.unwrap_err().contains("Oid hash not found for"));

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
    assert!(result.unwrap_err().contains("Oid hash not found for"));

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
            true,
        )
        .is_ok()
    );

    // Verify HEAD should be a symbolic ref to master
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    let head_content = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_content.trim(), "ref: refs/heads/master");

    // Verify master branch points to the commit
    let master_path = format!("{}/{}/refs/heads/master", repo_path, GIT_DIR);
    let master_content = fs::read_to_string(&master_path).unwrap();
    assert_eq!(master_content.trim(), commit_hash);
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
    let head_hash = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_hash.value, commit_hash);

    // Test getting HEAD when it doesn't exist
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    fs::remove_file(&head_path).unwrap();
    assert!(repo.get_ref(HEAD, true).is_err());
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
    assert!(commit.parents.is_empty());
    assert!(!commit.timestamp.is_empty());
    assert!(!commit.tree.is_empty());

    // Create second commit
    let second_message = "Second commit";
    let second_hash = repo.create_commit(second_message).unwrap();

    // Get and verify second commit
    let second_commit = repo.get_commit(&second_hash).unwrap();
    assert_eq!(second_commit.message, second_message);
    assert_eq!(second_commit.parents[0], commit_hash);
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
    assert_eq!(third_commit.parents, vec![second_hash.clone()]);

    let second_commit = repo.get_commit(&second_hash).unwrap();
    assert_eq!(second_commit.message, second_message);
    assert_eq!(second_commit.parents, vec![first_hash.clone()]);

    let first_commit = repo.get_commit(&first_hash).unwrap();
    assert_eq!(first_commit.message, first_message);
    assert!(first_commit.parents.is_empty());
}

#[test]
fn test_log_empty_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Log should work on an empty repository
    let result = repo.log();
    assert!(result.is_ok());
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
    let head_hash = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_hash.value, first_hash);

    // Verify worktree is empty (first commit had no files)
    assert!(!test_dir.join("file1.txt").exists());
    assert!(!test_dir.join("file2.txt").exists());

    // Checkout second commit
    assert!(repo.checkout(&second_hash).is_ok());

    // Verify HEAD points to second commit
    let head_hash = repo.get_ref(HEAD, true).unwrap();
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
        result
            .as_ref()
            .unwrap_err()
            .contains("Oid hash not found for"),
        "Expected error message to contain 'Oid hash not found for', but got: {}",
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
    let result = repo.checkout(&first_hash);
    assert!(
        result.is_ok(),
        "Error checking out commit: {:?}",
        result.err().unwrap()
    );

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
    assert!(
        result.is_ok(),
        "Log should work on an empty repository. Got error: {}",
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

    // Verify HEAD should be a symbolic ref to master
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    let head_content = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_content.trim(), "ref: refs/heads/master");

    // Verify master branch points to the commit
    let master_path = format!("{}/{}/refs/heads/master", repo_path, GIT_DIR);
    let master_content = fs::read_to_string(&master_path).unwrap();
    assert_eq!(master_content.trim(), commit_hash);

    // Verify commit has no parent
    let commit = repo.get_commit(&commit_hash).unwrap();
    assert!(commit.parents.is_empty());
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

    // Verify HEAD should be a symbolic ref to master
    let head_after_first = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_after_first.trim(), "ref: refs/heads/master");

    // Verify master branch points to the commit
    let master_path = format!("{}/{}/refs/heads/master", repo_path, GIT_DIR);
    let master_content = fs::read_to_string(&master_path).unwrap();
    assert_eq!(master_content.trim(), first_commit_hash);

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit_hash = repo.create_commit("Second commit").unwrap();

    // Verify HEAD should be a symbolic ref to master
    let head_after_second = fs::read_to_string(&head_path).unwrap();
    assert_eq!(head_after_second.trim(), "ref: refs/heads/master");

    // Verify master branch points to the second commit
    let master_content = fs::read_to_string(&master_path).unwrap();
    assert_eq!(master_content.trim(), second_commit_hash);

    // Verify second commit has first commit as parent
    let second_commit = repo.get_commit(&second_commit_hash).unwrap();
    assert_eq!(second_commit.parents, vec![first_commit_hash.clone()]);
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
            .contains("Oid hash not found for: invalidhash")
    );

    // Test with non-existent reference
    let result = repo.get_oid_hash("refs/heads/nonexistent");
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Oid hash not found for: refs/heads/nonexistent")
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

#[test]
fn test_branch_creation() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit
    let commit_hash = repo.create_commit("Initial commit").unwrap();

    // Create a new branch
    repo.create_branch("feature", Some(commit_hash.clone()))
        .unwrap();

    // Verify branch exists and points to the correct commit
    let (ref_name, ref_value) = repo.get_ref_internal("refs/heads/feature", true).unwrap();
    assert_eq!(ref_name, "refs/heads/feature");
    assert_eq!(ref_value.value, commit_hash);
    assert!(!ref_value.is_symbolic);

    // Create another branch without specifying commit (should use HEAD)
    repo.create_branch("develop", None).unwrap();

    // Verify new branch points to the same commit as HEAD
    let (_, head_value) = repo.get_ref_internal("HEAD", true).unwrap();
    let (_, develop_value) = repo.get_ref_internal("refs/heads/develop", true).unwrap();
    assert_eq!(develop_value.value, head_value.value);
}

#[test]
fn test_checkout() {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create a file
    let file_path = temp_dir.path().join("test.txt");
    fs::write(&file_path, "initial content").unwrap();

    // Create second commit
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Create a branch at the first commit
    repo.create_branch("old_version", Some(first_commit.clone()))
        .unwrap();

    // Checkout the branch
    repo.checkout("old_version").unwrap();

    // Verify we're at the first commit
    let (_, head_value) = repo.get_ref_internal("HEAD", true).unwrap();
    assert_eq!(head_value.value, first_commit);

    // Verify the file doesn't exist (was created after first commit)
    assert!(!file_path.exists());

    // Checkout back to the second commit
    repo.checkout(&second_commit).unwrap();

    // Verify we're at the second commit
    let (_, head_value) = repo.get_ref_internal("HEAD", true).unwrap();
    assert_eq!(head_value.value, second_commit);

    // Verify the file exists again
    assert!(file_path.exists());
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "initial content");

    // Test checkout to non-existent branch
    assert!(repo.checkout("non-existent").is_err());

    // Test checkout to invalid commit hash
    assert!(repo.checkout("invalid-hash").is_err());
}

#[test]
fn test_master_branch_creation_and_updates() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);

    // Initialize the repository
    assert!(repo.init().is_ok());

    // Check that HEAD points to master
    let head_ref = repo.get_ref("HEAD", false).unwrap();
    assert!(head_ref.is_symbolic);
    assert_eq!(head_ref.value, "ref: refs/heads/master");

    // Check that master branch exists but is empty
    let master_ref = repo.get_ref("refs/heads/master", false);
    assert!(master_ref.is_ok());
    assert!(master_ref.unwrap().value.is_empty());

    // Create a test file and commit it
    let test_file_path = format!("{}/test.txt", repo_path);
    fs::write(&test_file_path, "test content").unwrap();

    // Create a commit
    let commit_hash = repo.create_commit("Initial commit").unwrap();

    // Check that HEAD still points to master
    let head_ref: RefValue = repo.get_ref("HEAD", false).unwrap();
    assert!(head_ref.is_symbolic);
    assert_eq!(head_ref.value, "ref: refs/heads/master");

    // Check that master branch now points to the commit
    let master_ref = repo.get_ref("refs/heads/master", false).unwrap();
    assert!(!master_ref.is_symbolic);
    assert_eq!(master_ref.value, commit_hash);

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_master_branch_dereferencing() {
    // Create a temporary directory for the test
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);

    // Initialize the repository
    assert!(repo.init().is_ok());

    // Create a test file and commit it
    let test_file_path = format!("{}/test.txt", repo_path);
    fs::write(&test_file_path, "test content").unwrap();
    let commit_hash = repo.create_commit("Initial commit").unwrap();

    // Check that HEAD dereferences to the commit hash
    let head_ref = repo.get_ref("HEAD", true).unwrap();
    assert!(!head_ref.is_symbolic);
    assert_eq!(head_ref.value, commit_hash);

    // Check that master branch points to the commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert!(!master_ref.is_symbolic);
    assert_eq!(master_ref.value, commit_hash);

    // Clean up
    temp_dir.close().unwrap();
}

#[test]
fn test_detached_head_and_branch_creation() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Checkout the first commit (detached HEAD)
    assert!(repo.checkout(&first_commit).is_ok());

    // Verify HEAD is detached (points directly to commit hash)
    let head_ref = repo.get_ref("HEAD", false).unwrap();
    assert!(!head_ref.is_symbolic);
    assert_eq!(head_ref.value, first_commit);

    // Verify that branch master points to the last commit
    let master_ref = repo.get_ref("refs/heads/master", false).unwrap();
    assert!(!master_ref.is_symbolic);
    assert_eq!(master_ref.value, second_commit);

    // Create a new branch from the first commit
    let new_branch_name = "feature-branch";
    assert!(
        repo.create_branch(new_branch_name, Some(first_commit.clone()))
            .is_ok()
    );

    // Checkout the new branch
    assert!(repo.checkout(new_branch_name).is_ok());

    // Verify HEAD is now on the new branch
    let head_ref = repo.get_ref("HEAD", false).unwrap();
    assert!(head_ref.is_symbolic);
    assert_eq!(
        head_ref.value,
        format!("ref: refs/heads/{}", new_branch_name)
    );

    // Verify the branch points to the first commit
    let branch_ref = repo
        .get_ref(&format!("refs/heads/{}", new_branch_name), true)
        .unwrap();
    assert_eq!(branch_ref.value, first_commit);
}

#[test]
fn test_get_branch_name() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Initially HEAD points to master
    let branch_name = repo.get_branch_name().unwrap();
    assert_eq!(branch_name, Some("master".to_string()));

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Now HEAD should point to master
    let branch_name = repo.get_branch_name().unwrap();
    assert_eq!(branch_name, Some("master".to_string()));

    // Create a new branch and checkout to it
    let new_branch = "feature-branch";
    assert!(
        repo.create_branch(new_branch, Some(first_commit.clone()))
            .is_ok()
    );
    assert!(repo.checkout(new_branch).is_ok());

    // Now HEAD should point to the new branch
    let branch_name = repo.get_branch_name().unwrap();
    assert_eq!(branch_name, Some(new_branch.to_string()));

    // Checkout to a commit hash (detached HEAD)
    assert!(repo.checkout(&first_commit).is_ok());
    let branch_name = repo.get_branch_name().unwrap();
    assert!(branch_name.is_none());
}

#[test]
fn test_iter_branch_names() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Initially only master exists
    let branch_names = repo.iter_branch_names().unwrap();
    assert_eq!(branch_names, vec!["\x1b[32m* master\x1b[0m"]);

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create multiple branches
    let branches = vec!["feature-1", "feature-2", "develop"];
    for branch in &branches {
        assert!(
            repo.create_branch(branch, Some(first_commit.clone()))
                .is_ok()
        );
    }

    // Get all branch names (should include master and new branches)
    let mut branch_names = repo.iter_branch_names().unwrap();
    branch_names.sort(); // Sort for consistent testing

    let mut expected = vec!["\x1b[32m* master\x1b[0m"];
    expected.extend(branches);
    expected.sort();

    assert_eq!(branch_names, expected);

    // Checkout to feature-1 and verify it's marked with *
    assert!(repo.checkout("feature-1").is_ok());
    let mut branch_names = repo.iter_branch_names().unwrap();
    branch_names.sort();

    let mut expected = vec![
        "\x1b[32m* feature-1\x1b[0m",
        "develop",
        "feature-2",
        "master",
    ];
    expected.sort();

    assert_eq!(branch_names, expected);

    // Checkout to a commit hash (detached HEAD) and verify no branch is marked with *
    assert!(repo.checkout(&first_commit).is_ok());
    let mut branch_names = repo.iter_branch_names().unwrap();
    branch_names.sort();

    let mut expected = vec!["develop", "feature-1", "feature-2", "master"];
    expected.sort();

    assert_eq!(branch_names, expected);
}

#[test]
fn test_reset_success() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("Initial commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Verify HEAD points to second commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, second_commit);

    // Verify master branch points to second commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert_eq!(master_ref.value, second_commit);

    // Reset to first commit
    assert!(
        repo.reset(&first_commit).is_ok(),
        "Failed to reset to commit, {}",
        repo.reset(&first_commit).unwrap_err()
    );

    // Verify HEAD points to first commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, first_commit);

    // Verify master branch points to first commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert_eq!(master_ref.value, first_commit);
}

#[test]
fn test_reset_invalid_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Try to reset to non-existent commit
    let result = repo.reset("a".repeat(40).as_str());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Commit with hash:"));
}

#[test]
fn test_reset_detached_head() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("Initial commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let _second_commit = repo.create_commit("Second commit").unwrap();

    // Reset to first commit
    assert!(repo.reset(&first_commit).is_ok());

    // Verify HEAD is in normal state
    let head_ref = repo.get_ref(HEAD, false).unwrap();
    assert!(head_ref.is_symbolic);
    assert_eq!(head_ref.value, "ref: refs/heads/master");

    // Verify master branch points to first commit
    let master_ref = repo.get_ref("refs/heads/master", false).unwrap();
    assert!(!master_ref.is_symbolic);
    assert_eq!(master_ref.value, first_commit);
}

#[test]
fn test_reset_multiple_commits() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Second content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Create third commit
    fs::write(&test_file, "Third content").unwrap();
    let _third_commit = repo.create_commit("Third commit").unwrap();

    // Create fourth commit
    fs::write(&test_file, "Fourth content").unwrap();
    let fourth_commit = repo.create_commit("Fourth commit").unwrap();

    // Verify HEAD points to fourth commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, fourth_commit);

    // Verify master branch points to fourth commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert_eq!(master_ref.value, fourth_commit);

    // Reset to second commit
    assert!(repo.reset(&second_commit).is_ok());

    // Verify HEAD points to second commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, second_commit);

    // Verify master branch points to second commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert_eq!(master_ref.value, second_commit);

    // Verify file content matches second commit
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "Second content");

    // Create a new branch at fourth commit
    let new_branch = "feature-branch";
    assert!(
        repo.create_branch(new_branch, Some(fourth_commit.clone()))
            .is_ok()
    );

    // Verify new branch points to fourth commit
    let branch_ref = repo
        .get_ref(&format!("refs/heads/{}", new_branch), true)
        .unwrap();
    assert_eq!(branch_ref.value, fourth_commit);

    // Reset to first commit
    assert!(repo.reset(&first_commit).is_ok());

    // Verify HEAD points to first commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, first_commit);

    // Verify master branch points to first commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert_eq!(master_ref.value, first_commit);

    // Verify file content matches first commit
    assert_eq!(fs::read_to_string(&test_file).unwrap(), "Initial content");

    // Verify feature branch still points to fourth commit
    let branch_ref = repo
        .get_ref(&format!("refs/heads/{}", new_branch), true)
        .unwrap();
    assert_eq!(branch_ref.value, fourth_commit);

    // Create a new commit after reset
    fs::write(&test_file, "New content after reset").unwrap();
    let new_commit = repo.create_commit("New commit after reset").unwrap();

    // Verify HEAD points to new commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, new_commit);

    // Verify master branch points to new commit
    let master_ref = repo.get_ref("refs/heads/master", true).unwrap();
    assert_eq!(master_ref.value, new_commit);

    // Verify file content matches new commit
    assert_eq!(
        fs::read_to_string(&test_file).unwrap(),
        "New content after reset"
    );

    // Create another branch at the new commit
    let another_branch = "another-branch";
    assert!(
        repo.create_branch(another_branch, Some(new_commit.clone()))
            .is_ok()
    );

    // Verify new branch points to new commit
    let branch_ref = repo
        .get_ref(&format!("refs/heads/{}", another_branch), true)
        .unwrap();
    assert_eq!(branch_ref.value, new_commit);

    // Verify feature branch still points to fourth commit
    let branch_ref = repo
        .get_ref(&format!("refs/heads/{}", new_branch), true)
        .unwrap();
    assert_eq!(branch_ref.value, fourth_commit);

    // Verify HEAD is symbolic and points to master
    let head_ref = repo.get_ref(HEAD, false).unwrap();
    assert!(head_ref.is_symbolic);
    assert_eq!(head_ref.value, "ref: refs/heads/master");
}

#[test]
fn test_show_simple() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let _first_commit = repo.create_commit("First commit").unwrap();

    // Modify file
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Show second commit
    assert!(repo.show(&second_commit).is_ok());
}

#[test]
fn test_show_nested_changes() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create nested directory structure
    let nested_dir = temp_dir.path().join("src").join("nested");
    fs::create_dir_all(&nested_dir).unwrap();
    let nested_file = nested_dir.join("file.txt");
    fs::write(&nested_file, "Initial content").unwrap();
    let _first_commit = repo.create_commit("First commit").unwrap();

    // Modify nested file
    fs::write(&nested_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Show second commit
    assert!(repo.show(&second_commit).is_ok());
}

#[test]
fn test_show_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create multiple files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "File 1 initial").unwrap();
    fs::write(&file2, "File 2 initial").unwrap();
    let _first_commit = repo.create_commit("First commit").unwrap();

    // Modify both files
    fs::write(&file1, "File 1 updated").unwrap();
    fs::write(&file2, "File 2 updated").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Show second commit
    assert!(repo.show(&second_commit).is_ok());
}

#[test]
fn test_show_added_removed_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file
    let file1 = temp_dir.path().join("file1.txt");
    fs::write(&file1, "File 1 content").unwrap();
    let _first_commit = repo.create_commit("First commit").unwrap();

    // Remove file1 and add file2
    fs::remove_file(&file1).unwrap();
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file2, "File 2 content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Show second commit
    assert!(repo.show(&second_commit).is_ok());
}

#[test]
fn test_show_invalid_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Try to show non-existent commit
    let result = repo.show("a".repeat(40).as_str());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Commit with hash:"));
}

#[test]
fn test_show_first_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create first commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Show first commit (should work even though it has no parent)
    assert!(repo.show(&first_commit).is_ok());
}

#[test]
fn test_delete_ref_branch() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let commit_hash = repo.create_commit("First commit").unwrap();

    // Create a branch
    repo.create_branch("test-branch", Some(commit_hash))
        .unwrap();

    // Verify branch exists
    assert!(repo.is_branch("test-branch").unwrap());

    // Delete the branch
    repo.delete_ref("refs/heads/test-branch", false).unwrap();

    // Verify branch is deleted
    assert!(!repo.is_branch("test-branch").unwrap());
}

#[test]
fn test_delete_ref_tag() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let commit_hash = repo.create_commit("First commit").unwrap();

    // Create a tag
    repo.create_tag("v1.0", &commit_hash).unwrap();

    // Verify tag exists
    let refs = repo.iter_refs("refs/tags/").unwrap();
    assert!(refs.iter().any(|(name, _)| name == "refs/tags/v1.0"));

    // Delete the tag
    repo.delete_ref("refs/tags/v1.0", false).unwrap();

    // Verify tag is deleted
    let refs = repo.iter_refs("refs/tags/").unwrap();
    assert!(!refs.iter().any(|(name, _)| name == "refs/tags/v1.0"));
}

#[test]
fn test_delete_ref_head() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let _commit_hash = repo.create_commit("First commit").unwrap();

    // Verify HEAD exists
    let head_ref = repo.get_ref("HEAD", false).unwrap();
    assert!(head_ref.value.starts_with("ref: refs/heads/"));

    // Delete HEAD
    repo.delete_ref("HEAD", false).unwrap();

    // Verify HEAD is deleted
    assert!(repo.get_ref("HEAD", false).is_err());
}

#[test]
fn test_delete_ref_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Try to delete a non-existent branch
    let result = repo.delete_ref("refs/heads/nonexistent", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Failed to read refs/heads/nonexistent file")
    );

    // Try to delete a non-existent tag
    let result = repo.delete_ref("refs/tags/nonexistent", false);
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .contains("Failed to read refs/tags/nonexistent file")
    );
}

#[test]
fn test_get_merge_base() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create two branches from first commit
    repo.create_branch("branch1", Some(first_commit.clone()))
        .unwrap();
    repo.create_branch("branch2", Some(first_commit.clone()))
        .unwrap();

    // Switch to branch1 and make changes
    repo.checkout("branch1").unwrap();
    fs::write(&test_file, "Branch1 content").unwrap();
    let branch1_commit = repo.create_commit("Branch1 commit").unwrap();

    // Switch to branch2 and make changes
    repo.checkout("branch2").unwrap();
    fs::write(&test_file, "Branch2 content").unwrap();
    let branch2_commit = repo.create_commit("Branch2 commit").unwrap();

    // Find merge base between branch1 and branch2
    let merge_base = repo
        .get_merge_base(&branch1_commit, &branch2_commit)
        .unwrap();
    assert_eq!(merge_base, first_commit);
}

#[test]
fn test_get_merge_base_same_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let commit = repo.create_commit("First commit").unwrap();

    // Find merge base between same commit
    let merge_base = repo.get_merge_base(&commit, &commit).unwrap();
    assert_eq!(merge_base, commit);
}

#[test]
fn test_get_merge_base_ancestor() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Create second commit
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Find merge base between first and second commit
    let merge_base = repo.get_merge_base(&first_commit, &second_commit).unwrap();
    assert_eq!(merge_base, first_commit);
}

#[test]
fn test_get_merge_base_invalid_commit() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let commit = repo.create_commit("First commit").unwrap();

    // Try to find merge base with invalid commit
    let result = repo.get_merge_base(&commit, "invalidhash");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Oid hash not found for"));
}

#[test]
fn test_get_merge_base_different_branches() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let initial_commit = repo.create_commit("Initial commit").unwrap();

    // Create and checkout branch1
    repo.create_branch("branch1", Some(initial_commit.clone()))
        .unwrap();
    repo.checkout("branch1").unwrap();

    // Make changes and commit in branch1
    fs::write(&test_file, "Branch1 content").unwrap();
    let commit1 = repo.create_commit("Branch1 commit").unwrap();

    // Create and checkout branch2
    repo.create_branch("branch2", Some(initial_commit.clone()))
        .unwrap();
    repo.checkout("branch2").unwrap();

    // Make changes and commit in branch2
    fs::write(&test_file, "Branch2 content").unwrap();
    let commit2 = repo.create_commit("Branch2 commit").unwrap();

    // Find merge base between the two commits
    let merge_base = repo.get_merge_base(&commit1, &commit2).unwrap();
    assert_eq!(merge_base, initial_commit);
}

#[test]
fn test_merge_success() {
    let temp_dir = tempdir().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit
    fs::write(temp_dir.path().join("file1.txt"), "initial content").unwrap();
    repo.create_commit("Initial commit").unwrap();

    // Create and checkout a new branch
    repo.create_branch("feature", None).unwrap();
    repo.checkout("feature").unwrap();

    // Make changes in feature branch
    fs::write(temp_dir.path().join("file1.txt"), "feature content").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "new file in feature").unwrap();
    repo.create_commit("Feature changes").unwrap();

    // Switch back to master and make changes
    repo.checkout("master").unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "master content").unwrap();
    fs::write(temp_dir.path().join("file3.txt"), "new file in master").unwrap();
    repo.create_commit("Master changes").unwrap();

    // Merge feature branch into master
    repo.merge("feature").unwrap();

    // Verify the merged state
    let file1_content = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
    let file2_content = fs::read_to_string(temp_dir.path().join("file2.txt")).unwrap();
    let file3_content = fs::read_to_string(temp_dir.path().join("file3.txt")).unwrap();

    // The merge should have resolved conflicts and kept all files
    assert!(file1_content.contains("master content") || file1_content.contains("feature content"));
    assert_eq!(file2_content, "new file in feature");
    assert_eq!(file3_content, "new file in master");

    // Verify MERGE_HEAD was removed
    assert!(!Path::new(&format!("{}/{}", repo.gitdir, MERGE_HEAD)).exists());
}

#[test]
fn test_merge_complex_structure() {
    let temp_dir = tempdir().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial structure
    fs::create_dir_all(temp_dir.path().join("src/modules")).unwrap();
    fs::create_dir_all(temp_dir.path().join("tests/unit")).unwrap();
    fs::create_dir_all(temp_dir.path().join("docs/api")).unwrap();

    // Create initial files
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Hello\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("src/modules/utils.rs"),
        "pub fn helper() { }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("tests/unit/basic.rs"),
        "#[test] fn test_basic() { }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("docs/api/README.md"),
        "# API Documentation",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        "[package]\nname = \"test\"\nversion = \"0.1.0\"",
    )
    .unwrap();

    // Initial commit
    repo.create_commit("Initial commit with complex structure")
        .unwrap();

    // Create and checkout feature branch
    repo.create_branch("feature", None).unwrap();
    repo.checkout("feature").unwrap();

    // Make changes in feature branch
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Hello from feature\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("src/modules/new_feature.rs"),
        "pub fn new_feature() { }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("tests/unit/feature_test.rs"),
        "#[test] fn test_feature() { }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("docs/api/feature.md"),
        "# Feature Documentation",
    )
    .unwrap();
    repo.create_commit("Feature branch changes").unwrap();

    // Switch back to master and make different changes
    repo.checkout("master").unwrap();
    fs::write(
        temp_dir.path().join("src/main.rs"),
        "fn main() { println!(\"Hello from master\"); }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("src/modules/master_feature.rs"),
        "pub fn master_feature() { }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("tests/unit/master_test.rs"),
        "#[test] fn test_master() { }",
    )
    .unwrap();
    fs::write(
        temp_dir.path().join("docs/api/master.md"),
        "# Master Documentation",
    )
    .unwrap();
    repo.create_commit("Master branch changes").unwrap();

    // Merge feature branch into master
    repo.merge("feature").unwrap();

    // Verify the merged state
    let main_content = fs::read_to_string(temp_dir.path().join("src/main.rs")).unwrap();
    let utils_content = fs::read_to_string(temp_dir.path().join("src/modules/utils.rs")).unwrap();
    let new_feature_content =
        fs::read_to_string(temp_dir.path().join("src/modules/new_feature.rs")).unwrap();
    let master_feature_content =
        fs::read_to_string(temp_dir.path().join("src/modules/master_feature.rs")).unwrap();
    let feature_test_content =
        fs::read_to_string(temp_dir.path().join("tests/unit/feature_test.rs")).unwrap();
    let master_test_content =
        fs::read_to_string(temp_dir.path().join("tests/unit/master_test.rs")).unwrap();
    let feature_doc_content =
        fs::read_to_string(temp_dir.path().join("docs/api/feature.md")).unwrap();
    let master_doc_content =
        fs::read_to_string(temp_dir.path().join("docs/api/master.md")).unwrap();

    // Verify all files exist and have correct content
    assert!(
        main_content.contains("Hello from master") || main_content.contains("Hello from feature")
    );
    assert_eq!(utils_content, "pub fn helper() { }");
    assert_eq!(new_feature_content, "pub fn new_feature() { }");
    assert_eq!(master_feature_content, "pub fn master_feature() { }");
    assert_eq!(feature_test_content, "#[test] fn test_feature() { }");
    assert_eq!(master_test_content, "#[test] fn test_master() { }");
    assert_eq!(feature_doc_content, "# Feature Documentation");
    assert_eq!(master_doc_content, "# Master Documentation");

    // Verify directory structure is maintained
    assert!(temp_dir.path().join("src/modules").is_dir());
    assert!(temp_dir.path().join("tests/unit").is_dir());
    assert!(temp_dir.path().join("docs/api").is_dir());
}

#[test]
fn test_merge_fast_forward() {
    let temp_dir = tempdir().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit
    fs::write(temp_dir.path().join("file1.txt"), "initial content").unwrap();
    let initial_commit = repo.create_commit("Initial commit").unwrap();

    // Create and checkout a new branch
    repo.create_branch("feature", Some(initial_commit.clone()))
        .unwrap();
    repo.checkout("feature").unwrap();

    // Make changes in feature branch
    fs::write(temp_dir.path().join("file1.txt"), "feature content").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "new file in feature").unwrap();
    let feature_commit = repo.create_commit("Feature changes").unwrap();

    // Switch back to master
    repo.checkout("master").unwrap();

    // Merge feature branch into master (should be fast-forward)
    repo.merge("feature").unwrap();

    // Verify the merged state
    let file1_content = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
    let file2_content = fs::read_to_string(temp_dir.path().join("file2.txt")).unwrap();

    assert_eq!(file1_content, "feature content");
    assert_eq!(file2_content, "new file in feature");

    // Verify HEAD points to the feature commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, feature_commit);
}

#[test]
fn test_merge_fast_forward_with_multiple_commits() {
    let temp_dir = tempdir().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit
    fs::write(temp_dir.path().join("file1.txt"), "initial content").unwrap();
    let initial_commit = repo.create_commit("Initial commit").unwrap();

    // Create and checkout a new branch
    repo.create_branch("feature", Some(initial_commit.clone()))
        .unwrap();
    repo.checkout("feature").unwrap();

    // Make multiple commits in feature branch
    fs::write(temp_dir.path().join("file1.txt"), "feature content 1").unwrap();
    let _commit1 = repo.create_commit("Feature commit 1").unwrap();

    fs::write(temp_dir.path().join("file2.txt"), "new file in feature").unwrap();
    let _commit2 = repo.create_commit("Feature commit 2").unwrap();

    fs::write(temp_dir.path().join("file3.txt"), "another new file").unwrap();
    let final_commit = repo.create_commit("Feature commit 3").unwrap();

    // Switch back to master
    repo.checkout("master").unwrap();

    // Merge feature branch into master (should be fast-forward)
    repo.merge("feature").unwrap();

    // Verify the merged state
    let file1_content = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
    let file2_content = fs::read_to_string(temp_dir.path().join("file2.txt")).unwrap();
    let file3_content = fs::read_to_string(temp_dir.path().join("file3.txt")).unwrap();

    assert_eq!(file1_content, "feature content 1");
    assert_eq!(file2_content, "new file in feature");
    assert_eq!(file3_content, "another new file");

    // Verify HEAD points to the final feature commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, final_commit);
}

#[test]
fn test_merge_fast_forward_with_deleted_files() {
    let temp_dir = tempdir().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit with multiple files
    fs::write(temp_dir.path().join("file1.txt"), "initial content 1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "initial content 2").unwrap();
    fs::write(temp_dir.path().join("file3.txt"), "initial content 3").unwrap();
    let initial_commit = repo.create_commit("Initial commit").unwrap();

    // Create and checkout a new branch
    repo.create_branch("feature", Some(initial_commit.clone()))
        .unwrap();
    repo.checkout("feature").unwrap();

    // Delete some files and modify others in feature branch
    fs::remove_file(temp_dir.path().join("file2.txt")).unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "modified content").unwrap();
    let feature_commit = repo
        .create_commit("Feature changes with deletions")
        .unwrap();

    // Switch back to master
    repo.checkout("master").unwrap();

    // Merge feature branch into master (should be fast-forward)
    repo.merge("feature").unwrap();

    // Verify the merged state
    let file1_content = fs::read_to_string(temp_dir.path().join("file1.txt")).unwrap();
    assert_eq!(file1_content, "modified content");
    assert!(!temp_dir.path().join("file2.txt").exists());
    assert!(temp_dir.path().join("file3.txt").exists());

    // Verify HEAD points to the feature commit
    let head_ref = repo.get_ref(HEAD, true).unwrap();
    assert_eq!(head_ref.value, feature_commit);
}

#[test]
fn test_rebase_simple() {
    let temp_dir = tempdir().unwrap();
    let repo = Repository::new(temp_dir.path().to_str().unwrap());
    repo.init().unwrap();

    // Create initial commit
    fs::write(temp_dir.path().join("file1.txt"), "initial content").unwrap();
    let initial_commit = repo.create_commit("Initial commit").unwrap();

    // Create and checkout a new branch
    repo.create_branch("feature", Some(initial_commit.clone()))
        .unwrap();
    repo.checkout("feature").unwrap();

    // Make changes in feature branch
    fs::write(temp_dir.path().join("feature_file.txt"), "feature content").unwrap();
    let feature_commit = repo.create_commit("Feature changes").unwrap();

    // Switch back to master and make changes
    repo.checkout("master").unwrap();
    fs::write(temp_dir.path().join("master_file.txt"), "master content").unwrap();
    let _master_commit = repo.create_commit("Master changes").unwrap();

    // Rebase feature onto master
    repo.checkout("feature").unwrap();
    repo.rebase("master").unwrap();

    // Verify the rebased state
    let feature_content = fs::read_to_string(temp_dir.path().join("feature_file.txt")).unwrap();
    let master_content = fs::read_to_string(temp_dir.path().join("master_file.txt")).unwrap();

    assert_eq!(feature_content, "feature content");
    assert_eq!(master_content, "master content");

    // Verify feature branch points to new commit
    let feature_ref = repo.get_ref("refs/heads/feature", true).unwrap();
    assert_ne!(feature_ref.value, feature_commit);
}
