use crate::repository::{HEAD, ObjectType, Repository};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

// Define the result structure for compare_trees
// (path, type, list_of_oids_across_compared_trees)
pub type TreeComparisonResult = Vec<(String, ObjectType, Vec<Option<String>>)>;

pub struct Differ<'a> {
    repo: &'a Repository,
}

impl<'a> Differ<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    // Refactored compare_trees using BFS
    pub fn compare_trees(&self, trees: &[&str]) -> Result<TreeComparisonResult, String> {
        let num_trees = trees.len();
        // Store path -> (ObjectType, Vec<Option<String>>)
        let mut entries: HashMap<String, (ObjectType, Vec<Option<String>>)> = HashMap::new();
        // Keep track of visited tree OIDs for each version to avoid redundant processing
        let mut visited_trees: Vec<HashSet<String>> = vec![HashSet::new(); num_trees];
        // Queue for BFS: (tree_index, tree_oid, path_prefix)
        let mut queue: VecDeque<(usize, String, String)> = VecDeque::new();

        // Initial population of the queue with root trees
        for (i, &tree_oid) in trees.iter().enumerate() {
            if !tree_oid.is_empty() {
                queue.push_back((i, tree_oid.to_string(), "".to_string()));
                visited_trees[i].insert(tree_oid.to_string());
            }
        }

        while let Some((tree_index, current_tree_oid, prefix)) = queue.pop_front() {
            // Get data for the current tree OID
            let tree_data = match self.repo.get_tree_data(&current_tree_oid) {
                Ok(data) => data,
                Err(e) => {
                    // Handle case where tree OID might be invalid or unreadable
                    eprintln!(
                        "Warning: Could not read tree data for OID {} at index {}: {}. Skipping.",
                        current_tree_oid, tree_index, e
                    );
                    continue; // Skip this tree and proceed
                }
            };

            for (_, name, oid, obj_type) in tree_data {
                let path = if prefix.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", prefix, name)
                };

                // --- Entry Management ---
                let (entry_type, oids) = entries
                    .entry(path.clone())
                    .or_insert_with(|| (obj_type.clone(), vec![None; num_trees]));

                // Type conflict resolution (prefer Tree)
                if *entry_type != obj_type && obj_type == ObjectType::Tree {
                    *entry_type = ObjectType::Tree;
                }

                // Update OID for the current tree index
                if oids.len() > tree_index {
                    oids[tree_index] = Some(oid.clone());
                } else {
                    return Err(format!(
                        "Logic error: OID vector index out of bounds for path {}",
                        path
                    ));
                }
                // --- End Entry Management ---

                // If it's a tree and not visited yet for this index, add to queue
                if obj_type == ObjectType::Tree && visited_trees[tree_index].insert(oid.clone()) {
                    queue.push_back((tree_index, oid.clone(), path.clone()));
                }
            }
        }

        // Convert HashMap to Vec and sort
        let mut result: TreeComparisonResult = entries
            .into_iter()
            .map(|(path, (obj_type, oids))| (path, obj_type, oids))
            .collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
    }

    pub fn diff_trees(&self, old_tree: &str, new_tree: &str) -> Result<Vec<u8>, String> {
        let mut output = Vec::new();
        // diff_trees now needs adapting if its caller relies on old compare_trees output format
        // compare_trees now returns Vec<(String, ObjectType, Vec<Option<String>>)>
        // We only want to diff blobs
        let entries = self.compare_trees(&[old_tree, new_tree])?;

        for (path, obj_type, oids) in entries {
            if obj_type == ObjectType::Blob
                && oids.first().unwrap_or(&None) != oids.get(1).unwrap_or(&None)
            {
                // Correctly get Option<&str> for diff_blobs
                let oid1 = oids.first().unwrap_or(&None).as_deref();
                let oid2 = oids.get(1).unwrap_or(&None).as_deref();
                let diff = self.diff_blobs(oid1, oid2, &path)?;
                output.extend_from_slice(&diff);
            }
        }

        Ok(output)
    }

    fn diff_blobs(
        &self,
        from_oid: Option<&str>,
        to_oid: Option<&str>,
        path: &str,
    ) -> Result<Vec<u8>, String> {
        let mut from_file =
            NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;
        let mut to_file =
            NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;

        if let Some(oid) = from_oid {
            let content = self.repo.get_object(oid)?;
            from_file
                .write_all(&content)
                .map_err(|e| format!("Failed to write to temp file: {}", e))?;
        }
        if let Some(oid) = to_oid {
            let content = self.repo.get_object(oid)?;
            to_file
                .write_all(&content)
                .map_err(|e| format!("Failed to write to temp file: {}", e))?;
        }

        from_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;
        to_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;

        let output = Command::new("diff")
            .args([
                "--unified",
                "--show-c-function",
                "--label",
                &format!("a/{}", path),
                from_file.path().to_str().unwrap(),
                "--label",
                &format!("b/{}", path),
                to_file.path().to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to run diff command: {}", e))?;

        Ok(output.stdout)
    }

    pub fn diff_current_working_tree(&self) -> Result<Vec<u8>, String> {
        let working_tree = self.repo.get_working_tree()?;
        let head_tree = match self.repo.get_commit(HEAD) {
            Ok(commit) => commit.tree,
            Err(_) => "".to_string(), // Use empty string for empty tree if no commits
        };
        let diff = self.diff_trees(&head_tree, &working_tree)?;
        Ok(diff)
    }

    pub fn iter_changed_files(&self) -> Result<Vec<String>, String> {
        let working_tree = self.repo.get_working_tree()?;
        let head_tree = match self.repo.get_commit(HEAD) {
            Ok(commit) => commit.tree,
            Err(_) => "".to_string(), // Use empty string for empty tree if no commits
        };

        // Compare working tree (index 0) vs head tree (index 1)
        let entries = self.compare_trees(&[&working_tree, &head_tree])?;

        Ok(entries
            .into_iter()
            // Filter where OID at index 0 (worktree) differs from index 1 (HEAD)
            .filter(|(_path, _obj_type, oids)| {
                oids.first().unwrap_or(&None) != oids.get(1).unwrap_or(&None)
            })
            .map(|(path, _obj_type, oids)| {
                // Correctly destructure tuple
                // Determine status based on presence/absence of OIDs
                match (oids.first().unwrap_or(&None), oids.get(1).unwrap_or(&None)) {
                    (Some(_), None) => format!("\x1b[32m{}[0m", path), // Added
                    (None, Some(_)) => format!("\x1b[31m{}[0m", path), // Deleted
                    (Some(_), Some(_)) => format!("\x1b[33m{}[0m", path), // Modified
                    (None, None) => path, // Should not happen due to filter
                }
            })
            .collect())
    }

    pub fn colorize_diff(diff: &[u8]) -> String {
        let mut colored = String::new();
        let diff_str = String::from_utf8_lossy(diff);

        for line in diff_str.lines() {
            if line.starts_with('+') && !line.starts_with("+++") {
                colored.push_str("\x1b[32m"); // Green
                colored.push_str(line);
                colored.push_str("\x1b[0m"); // Reset
            } else if line.starts_with('-') && !line.starts_with("---") {
                colored.push_str("\x1b[31m"); // Red
                colored.push_str(line);
                colored.push_str("\x1b[0m"); // Reset
            } else {
                colored.push_str(line);
            }
            colored.push('\n');
        }

        colored
    }

    // Refactored merge_trees
    pub fn merge_trees(
        &self,
        t_head: &str,
        t_other: &str,
        t_base: Option<&str>,
        // Return type distinguishes files (Ok) from directories (Err)
    ) -> Result<HashMap<String, Result<Vec<u8>, ()>>, String> {
        let mut tree: HashMap<String, Result<Vec<u8>, ()>> = HashMap::new();
        let base = t_base.unwrap_or(""); // Use empty string for None base

        // Get comparison result including object types
        let entries = self.compare_trees(&[base, t_head, t_other])?;

        for (path, obj_type, oids) in entries {
            let base_oid = oids.first().unwrap_or(&None);
            let head_oid = oids.get(1).unwrap_or(&None);
            let other_oid = oids.get(2).unwrap_or(&None);

            match obj_type {
                ObjectType::Blob => {
                    // Handle blobs: merge content
                    let merged_content = self.merge_blobs_three_way(
                        base_oid.as_deref(),
                        head_oid.as_deref(),
                        other_oid.as_deref(),
                    )?;

                    // Check if file was deleted in both branches relative to base
                    if base_oid.is_some() && head_oid.is_none() && other_oid.is_none() {
                        // Deleted in both, skip.
                    } else if head_oid.is_none() && other_oid.is_none() {
                        // Not present in base, head, or other (shouldn't happen due to compare_trees logic?)
                        // Or, if base was None, means it was only added then deleted - skip.
                    } else {
                        tree.insert(path, Ok(merged_content));
                    }
                }
                ObjectType::Tree => {
                    // Handle trees: check existence
                    let base_oid = oids.first().unwrap_or(&None);
                    let head_oid = oids.get(1).unwrap_or(&None);
                    let other_oid = oids.get(2).unwrap_or(&None);
                    // Keep directory if it exists in head or other, *unless* it was
                    // present in base but deleted in *both* head and other.
                    if base_oid.is_some() && head_oid.is_none() && other_oid.is_none() {
                        // Deleted in both relative to base, skip.
                    } else if head_oid.is_some() || other_oid.is_some() {
                        // Exists in head or other (and not deleted in both relative to base)
                        tree.insert(path, Err(())); // Mark as directory
                    }
                    // If only in base (deleted in both), skip.
                    // If not in base, head, or other, skip.
                }
                ObjectType::Commit => {
                    // This shouldn't happen within a tree comparison
                    return Err(format!(
                        "Unexpected Commit object type found for path {}",
                        path
                    ));
                }
            }
        }
        Ok(tree)
    }

    fn merge_blobs_three_way(
        &self,
        o_base: Option<&str>,
        o_head: Option<&str>,
        o_other: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        // Handle the trivial case where all inputs are None
        if o_base.is_none() && o_head.is_none() && o_other.is_none() {
            return Ok(Vec::new());
        }

        // Get content or empty vec
        let base_content = match o_base {
            Some(oid) => self.repo.get_object(oid)?,
            None => Vec::new(),
        };
        let head_content = match o_head {
            Some(oid) => self.repo.get_object(oid)?,
            None => Vec::new(),
        };
        let other_content = match o_other {
            Some(oid) => self.repo.get_object(oid)?,
            None => Vec::new(),
        };

        // Use helper function to create temp files
        let base_file = self.create_temp_file_with_content(&base_content, "base")?;
        let head_file = self.create_temp_file_with_content(&head_content, "head")?;
        let other_file = self.create_temp_file_with_content(&other_content, "other")?;

        // Call diff3 without labels
        let output = Command::new("diff3")
            .args([
                "-m", // Merge output with conflict markers
                "-L",
                "HEAD",                             // Label for HEAD version
                head_file.path().to_str().unwrap(), // My file (HEAD)
                "-L",
                "BASE",                             // Label for BASE version
                base_file.path().to_str().unwrap(), // Older file (BASE)
                "-L",
                "MERGE_HEAD",                        // Label for OTHER version
                other_file.path().to_str().unwrap(), // Your file (OTHER)
            ])
            .output()
            .map_err(|e| {
                format!(
                    "Failed to run diff3 command: {}. Ensure diffutils is installed.",
                    e
                )
            })?;

        // diff3 returns status 1 for conflicts, 0 for success.
        // We accept both as valid merge results (stdout contains markers on conflict).
        // Status > 1 indicates an error.
        if output.status.success() || output.status.code() == Some(1) {
            Ok(output.stdout)
        } else {
            Err(format!(
                "diff3 command failed with status {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ))
        }
    }

    // Helper function to create, write, and flush a temp file
    fn create_temp_file_with_content(
        &self,
        content: &[u8],
        label: &str, // For error messages
    ) -> Result<NamedTempFile, String> {
        let mut temp_file = NamedTempFile::new()
            .map_err(|e| format!("Failed to create temp file ({}): {}", label, e))?;
        temp_file
            .write_all(content)
            .map_err(|e| format!("Failed to write temp file ({}): {}", label, e))?;
        temp_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file ({}): {}", label, e))?;
        Ok(temp_file)
    }
}
