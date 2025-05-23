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

#[test]
fn test_diff_file_rename() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create and commit initial file
    let old_file = temp_dir.path().join("oldname.txt");
    fs::write(&old_file, "Rename me").unwrap();
    repo.create_commit("Initial commit").unwrap();

    // Rename file
    let new_file = temp_dir.path().join("newname.txt");
    fs::rename(&old_file, &new_file).unwrap();

    // Get diff
    let diff = repo.diff().unwrap();
    // Should show removal of old file and addition of new file
    assert!(diff.contains("--- a/oldname.txt"));
    assert!(diff.contains("+++ b/newname.txt"));
    assert!(diff.contains("-Rename me"));
    assert!(diff.contains("+Rename me"));
}

#[test]
fn test_diff_ignored_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Add .bgitignore to ignore *.log files
    let gitignore = temp_dir.path().join(".bgitignore");
    fs::write(&gitignore, "*.log\n").unwrap();
    repo.create_commit("Initial commit").unwrap();

    // Add a .log file and a normal file
    let log_file = temp_dir.path().join("debug.log");
    let txt_file = temp_dir.path().join("visible.txt");
    fs::write(&log_file, "Should be ignored").unwrap();
    fs::write(&txt_file, "Should be visible").unwrap();

    // Get diff
    let diff = repo.diff().unwrap();
    // Should not contain the log file
    assert!(!diff.contains("debug.log"));
    // Should contain the visible file
    assert!(diff.contains("visible.txt"));
}

#[test]
fn test_diff_subdirectory_changes() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create subdirectory and file, commit
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file = subdir.join("file.txt");
    fs::write(&file, "Subdir content").unwrap();
    repo.create_commit("Initial commit").unwrap();

    // Modify file in subdir
    fs::write(&file, "Changed content").unwrap();
    // Add another file in subdir
    let new_file = subdir.join("new.txt");
    fs::write(&new_file, "New file").unwrap();
    // Remove the original file
    fs::remove_file(&file).unwrap();

    // Get diff
    let diff = repo.diff().unwrap();
    // Should show removal of file.txt and addition of new.txt
    assert!(diff.contains("--- a/subdir/file.txt"));
    assert!(diff.contains("+++ b/subdir/new.txt"));
    assert!(diff.contains("+New file"));
}

#[test]
fn test_merge_trees_simple_add_new_line() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file and commit
    let test_file = temp_dir.path().join("test.txt");
    fs::write(&test_file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify file and commit
    fs::write(&test_file, "Initial content\nNew line added").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(&first_commit_obj.tree, &second_commit_obj.tree, None)
        .unwrap();

    // Verify merged content contains both versions with proper merge markers
    let merged_content = String::from_utf8_lossy(merged["test.txt"].as_ref().unwrap());
    assert!(merged_content.contains("<<<<<<<"));
    assert!(merged_content.contains("======="));
    assert!(merged_content.contains(">>>>>>>"));
    assert!(merged_content.contains("Initial content\nNew line added"));
    assert!(merged_content.contains("Initial content"));
}

#[test]
fn test_merge_trees_python_code() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file and commit
    let test_file = temp_dir.path().join("main.py");
    let initial_content = r#"
        def main():
            print("This function is cool")
            print("It prints stuff")
            print("It can even return a number:")
            return 7
"#;
    fs::write(&test_file, initial_content).unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify file and commit
    let modified_content = r#"
        def main():
            print("1 + 1 = 2")
            print("This function is cool")
            print("It prints stuff")
"#;
    fs::write(&test_file, modified_content).unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(&first_commit_obj.tree, &second_commit_obj.tree, None)
        .unwrap();

    // Verify merged content contains both versions with proper merge markers
    let merged_content = String::from_utf8_lossy(merged["main.py"].as_ref().unwrap());
    assert!(merged_content.contains("<<<<<<<"));
    assert!(merged_content.contains("======="));
    assert!(merged_content.contains(">>>>>>>"));
    assert!(merged_content.contains(
        r#"
        def main():
            print("1 + 1 = 2")
            print("This function is cool")
            print("It prints stuff")
"#
    ));
    assert!(merged_content.contains(
        r#"
        def main():
            print("This function is cool")
            print("It prints stuff")
            print("It can even return a number:")
            return 7
"#
    ));
}

#[test]
fn test_merge_trees_python_code_three_way() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file and commit
    let test_file = temp_dir.path().join("main.py");
    let initial_content = r#"
        def main():
            print("This function is cool")
            print("It prints stuff")
            print("It can even return a number:")
            return 7
"#;
    fs::write(&test_file, initial_content).unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify file and commit
    let modified_content = r#"
        def main():
            print("1 + 1 = 2")
            print("This function is cool")
            print("It prints stuff")
"#;
    fs::write(&test_file, modified_content).unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();
    let base_commit_obj = repo
        .get_commit(&repo.get_merge_base(&first_commit, &second_commit).unwrap())
        .unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(
            &first_commit_obj.tree,
            &second_commit_obj.tree,
            Some(&base_commit_obj.tree),
        )
        .unwrap();

    // Verify merged content contains both versions with proper merge markers
    let merged_content = String::from_utf8_lossy(merged["main.py"].as_ref().unwrap());
    let expected = r#"def main():
            print("1 + 1 = 2")
            print("It prints stuff")
            return 7"#;
    assert_eq!(merged_content.trim(), expected);
}

#[test]
fn test_merge_trees_multiple_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial files and commit
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "File 1 initial").unwrap();
    fs::write(&file2, "File 2 initial").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify files and commit
    fs::write(&file1, "File 1 modified").unwrap();
    fs::write(&file2, "File 2 modified").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(&first_commit_obj.tree, &second_commit_obj.tree, None)
        .unwrap();

    // Verify merged content for both files
    let file1_content = String::from_utf8_lossy(merged["file1.txt"].as_ref().unwrap());
    let file2_content = String::from_utf8_lossy(merged["file2.txt"].as_ref().unwrap());

    assert!(file1_content.contains("File 1 initial"));
    assert!(file1_content.contains("File 1 modified"));
    assert!(file2_content.contains("File 2 initial"));
    assert!(file2_content.contains("File 2 modified"));
}

#[test]
fn test_merge_trees_added_removed_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial files and commit
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "File 1 content").unwrap();
    fs::write(&file2, "File 2 content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Remove file1 and add file3
    fs::remove_file(&file1).unwrap();
    let file3 = temp_dir.path().join("file3.txt");
    fs::write(&file3, "File 3 content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(&first_commit_obj.tree, &second_commit_obj.tree, None)
        .unwrap();

    // Verify merged content
    assert!(merged.contains_key("file1.txt")); // Removed file should be marked as deleted
    assert!(merged.contains_key("file2.txt")); // Unchanged file should be present
    assert!(merged.contains_key("file3.txt")); // New file should be present

    let file2_content = String::from_utf8_lossy(merged["file2.txt"].as_ref().unwrap());
    let file3_content = String::from_utf8_lossy(merged["file3.txt"].as_ref().unwrap());
    assert!(file2_content.contains("File 2 content"));
    assert!(file3_content.contains("File 3 content"));
}

#[test]
fn test_merge_trees_empty_files() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create empty file and commit
    let empty_file = temp_dir.path().join("empty.txt");
    fs::write(&empty_file, "").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Add content to file and commit
    fs::write(&empty_file, "New content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(&first_commit_obj.tree, &second_commit_obj.tree, None)
        .unwrap();

    // Verify merged content
    let merged_content = String::from_utf8_lossy(merged["empty.txt"].as_ref().unwrap());
    assert!(merged_content.contains("New content"));
}

#[test]
fn test_merge_trees_subdirectories() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create subdirectory and file, commit
    let subdir = temp_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file = subdir.join("file.txt");
    fs::write(&file, "Initial content").unwrap();
    let first_commit = repo.create_commit("First commit").unwrap();

    // Modify file in subdir and add new file
    fs::write(&file, "Modified content").unwrap();
    let new_file = subdir.join("new.txt");
    fs::write(&new_file, "New file content").unwrap();
    let second_commit = repo.create_commit("Second commit").unwrap();

    // Get commits
    let first_commit_obj = repo.get_commit(&first_commit).unwrap();
    let second_commit_obj = repo.get_commit(&second_commit).unwrap();

    // Create differ and merge trees
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(&first_commit_obj.tree, &second_commit_obj.tree, None)
        .unwrap();

    // Verify merged content
    let file_content = String::from_utf8_lossy(merged["subdir/file.txt"].as_ref().unwrap());
    let new_file_content = String::from_utf8_lossy(merged["subdir/new.txt"].as_ref().unwrap());
    assert!(file_content.contains("Initial content"));
    assert!(file_content.contains("Modified content"));
    assert!(new_file_content.contains("New file content"));
}

#[test]
fn test_merge_trees_three_way_merge() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // Create initial file with common ancestor content
    let test_file = temp_dir.path().join("animals.py");
    let common_ancestor = r#"def be_a_cat():
    print("Meow")
    return True

def be_a_dog():
    print("Bark!")
    return False"#;
    fs::write(&test_file, common_ancestor).unwrap();
    let base_commit = repo.create_commit("Base commit").unwrap();

    // Create version A (changes be_a_cat)
    let version_a = r#"def be_a_cat():
    print("Sleep")
    return True

def be_a_dog():
    print("Bark!")
    return False"#;
    fs::write(&test_file, version_a).unwrap();
    let commit_a = repo.create_commit("Version A commit").unwrap();

    // Create version B (changes be_a_dog)
    let version_b = r#"def be_a_cat():
    print("Meow")
    return True

def be_a_dog():
    print("Eat homework")
    return False"#;
    fs::write(&test_file, version_b).unwrap();
    let commit_b = repo.create_commit("Version B commit").unwrap();

    // Get commits
    let base_commit_obj = repo.get_commit(&base_commit).unwrap();
    let commit_a_obj = repo.get_commit(&commit_a).unwrap();
    let commit_b_obj = repo.get_commit(&commit_b).unwrap();

    // Create differ and merge trees using three-way merge
    let differ = Differ::new(&repo);
    let merged = differ
        .merge_trees(
            &commit_a_obj.tree,
            &commit_b_obj.tree,
            Some(&base_commit_obj.tree),
        )
        .unwrap();

    // Verify merged content contains combined changes from both versions
    let merged_content = String::from_utf8_lossy(merged["animals.py"].as_ref().unwrap());
    let expected = r#"def be_a_cat():
    print("Sleep")
    return True

def be_a_dog():
    print("Eat homework")
    return False"#;
    assert_eq!(merged_content.trim(), expected);
}

#[test]
fn test_merge_trees_three_way_no_conflict() {
    let temp_dir = TempDir::new().unwrap();
    let repo_path = temp_dir.path().to_str().unwrap();
    let repo = Repository::new(repo_path);
    repo.init().unwrap();

    // 1. Base commit
    let test_file = temp_dir.path().join("file.txt");
    fs::write(&test_file, "Line 1\n\n").unwrap();
    let base_commit = repo.create_commit("Base commit").unwrap();

    // 2. Feature branch: Add line 2
    repo.create_branch("feature", Some(base_commit.clone()))
        .unwrap();
    repo.checkout("feature").unwrap();
    fs::write(&test_file, "Line 1\n\nLine 2 feature\n").unwrap();
    let _feature_commit = repo.create_commit("Feature commit").unwrap();

    // 3. Master branch: Modify line 1
    repo.checkout("master").unwrap(); // Should be at base_commit
    fs::write(&test_file, "Line master 1\n\n").unwrap();
    let _master_commit = repo.create_commit("Master commit").unwrap();

    // 4. Merge feature into master
    // This merge should be clean as changes are on different lines
    let merge_result = repo.merge("feature");
    assert!(
        merge_result.is_ok(),
        "Merge failed: {:?}",
        merge_result.err()
    );

    // 5. Verify the merged content in the working directory
    let final_content = fs::read_to_string(&test_file).unwrap();
    let expected_content = "Line master 1\n\nLine 2 feature\n";
    assert_eq!(final_content, expected_content);
}
