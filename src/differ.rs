use crate::repository::{HEAD, ObjectType, Repository};
use std::collections::HashMap;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

type TreeComparisonResult = Vec<(String, Vec<Option<String>>)>;

pub struct Differ<'a> {
    repo: &'a Repository,
}

impl<'a> Differ<'a> {
    pub fn new(repo: &'a Repository) -> Self {
        Self { repo }
    }

    pub fn compare_trees(&self, trees: &[&str]) -> Result<TreeComparisonResult, String> {
        let mut entries: HashMap<String, Vec<Option<String>>> = HashMap::new();

        for (i, tree_hash) in trees.iter().enumerate() {
            if tree_hash.is_empty() {
                continue;
            }

            let tree_data = self.repo.get_tree_data(tree_hash)?;
            for (_, path, oid, obj_type) in tree_data {
                if matches!(obj_type, ObjectType::Tree) {
                    let sub_tree_data = self.repo.get_tree_data(&oid)?;
                    for (_, sub_path, sub_oid, _) in sub_tree_data {
                        let full_path = format!("{}/{}", path, sub_path);
                        if !entries.contains_key(&full_path) {
                            entries.insert(full_path.clone(), vec![None; trees.len()]);
                        }
                        entries.get_mut(&full_path).unwrap()[i] = Some(sub_oid);
                    }
                } else {
                    if !entries.contains_key(&path) {
                        entries.insert(path.clone(), vec![None; trees.len()]);
                    }
                    entries.get_mut(&path).unwrap()[i] = Some(oid);
                }
            }
        }

        let mut result: TreeComparisonResult = entries.into_iter().collect();
        result.sort_by(|a, b| a.0.cmp(&b.0));

        Ok(result)
    }

    pub fn diff_trees(&self, old_tree: &str, new_tree: &str) -> Result<Vec<u8>, String> {
        let mut output = Vec::new();
        let entries = self.compare_trees(&[old_tree, new_tree])?;

        for (path, oids) in entries {
            if oids[0] != oids[1] {
                let diff = self.diff_blobs(oids[0].as_deref(), oids[1].as_deref(), &path)?;
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
        let diff = self.diff_trees(&working_tree, &self.repo.get_commit(HEAD)?.tree)?;
        Ok(diff)
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
}
