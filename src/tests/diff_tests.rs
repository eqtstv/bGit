use crate::differ::Differ;

#[test]
fn test_diff_trees_simple() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify file
    fs::write(&test_file, "Updated content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and get diff
    let differ = Differ::new(&repo);
    let diff = differ
        .diff_trees(&first_commit_obj.tree, &second_commit_obj.tree)
        .unwrap();

    // Convert diff to string for easier testing
    let diff_str = String::from_utf8_lossy(&diff);

    // Verify diff contains expected content
    assert!(diff_str.contains("--- a/test.txt"), "Found: {}", diff_str);
    assert!(diff_str.contains("+++ b/test.txt"), "Found: {}", diff_str);
    assert!(diff_str.contains("-Initial content"));
    assert!(diff_str.contains("+Updated content"));
}

#[test]
fn test_diff_trees_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "File 1 initial").unwrap();
    fs::write(&file2, "File 2 initial").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify files
    fs::write(&file1, "File 1 updated").unwrap();
    fs::write(&file2, "File 2 updated").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and get diff
    let differ = Differ::new(&repo);
    let diff = differ
        .diff_trees(&first_commit_obj.tree, &second_commit_obj.tree)
        .unwrap();

    // Convert diff to string for easier testing
    let diff_str = String::from_utf8_lossy(&diff);

    // Verify diff contains changes for both files
    assert!(diff_str.contains("-File 1 initial"));
    assert!(diff_str.contains("+File 1 updated"));
    assert!(diff_str.contains("-File 2 initial"));
    assert!(diff_str.contains("+File 2 updated"));
}

#[test]
fn test_diff_trees_added_removed_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file
    let file1 = temp_dir.path().join("file1.txt");
    fs::write(&file1, "File 1 content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Remove file1 and add file2
    fs::remove_file(&file1).unwrap();
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file2, "File 2 content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and get diff
    let differ = Differ::new(&repo);
    let diff = differ
        .diff_trees(&first_commit_obj.tree, &second_commit_obj.tree)
        .unwrap();

    // Convert diff to string for easier testing
    let diff_str = String::from_utf8_lossy(&diff);

    // Verify diff shows removed and added files
    assert!(diff_str.contains("-File 1 content"));
    assert!(diff_str.contains("+File 2 content"));
}

#[test]
fn test_diff_modified_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file and commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    repo.create_commit("First commit").unwrap();

    // Modify file
    fs::write(&test_file, "Updated content").unwrap();

    // Get diff
    let diff = repo.diff().unwrap();

    // Verify diff contains expected content
    assert!(diff.contains("--- a/test.txt"));
    assert!(diff.contains("+++ b/test.txt"));
    assert!(diff.contains("-Initial content"), "{}", diff);
    assert!(diff.contains("+Updated content"), "{}", diff);
}

#[test]
fn test_diff_added_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    repo.create_commit("First commit").unwrap();

    // Add new file
    let new_file = temp_dir.path().join("new.txt");
    fs::write(&new_file, "New file content").unwrap();

    // Get diff
    let diff = repo.diff().unwrap();

    // Verify diff contains new file
    assert!(diff.contains("--- a/new.txt"));
    assert!(diff.contains("+++ b/new.txt"));
    assert!(diff.contains("+New file content"));
}

#[test]
fn test_diff_removed_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial files and commit
    let test_file = temp_dir.path().join("test.txt");
    let other_file = temp_dir.path().join("other.txt");
    fs::write(&test_file, "Test content").unwrap();
    fs::write(&other_file, "Other content").unwrap();
    repo.create_commit("First commit").unwrap();

    // Remove file
    fs::remove_file(&other_file).unwrap();

    // Get diff
    let diff = repo.diff().unwrap();

    // Verify diff shows removed file
    assert!(diff.contains("--- a/other.txt"));
    assert!(diff.contains("+++ b/other.txt"));
    assert!(diff.contains("-Other content"));
}

#[test]
fn test_diff_no_changes() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file and commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    repo.create_commit("First commit").unwrap();

    // Get diff (should be empty)
    let diff = repo.diff().unwrap();
    assert!(diff.is_empty());
}

#[test]
fn test_diff_no_head() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Remove HEAD file
    let head_path = format!("{}/{}/HEAD", repo_path, GIT_DIR);
    fs::remove_file(&head_path).unwrap();

    // Get diff should fail
    let result = repo.diff();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read HEAD file"));
}

#[test]
fn test_diff_invalid_repository() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();

    let repo = Repository::new(repo_path);
    // Don't initialize repository

    // Get diff should fail
    let result = repo.diff();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Failed to read HEAD file"));
}
