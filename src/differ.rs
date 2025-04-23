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
        let diff = self.diff_trees(&self.repo.get_commit(HEAD)?.tree, &working_tree)?;
        Ok(diff)
    }

    pub fn iter_changed_files(&self) -> Result<Vec<String>, String> {
        let working_tree = self.repo.get_working_tree()?;
        let entries = self.compare_trees(&[&working_tree, &self.repo.get_commit(HEAD)?.tree])?;

        Ok(entries
            .into_iter()
            .filter(|(_, oids)| oids[0] != oids[1])
            .map(|(path, oids)| {
                match (oids[0].as_ref(), oids[1].as_ref()) {
                    (Some(_), None) => format!("\x1b[32m{}\x1b[0m", path), // Green for added files
                    (None, Some(_)) => format!("\x1b[31m{}\x1b[0m", path), // Red for deleted files
                    (Some(_), Some(_)) => format!("\x1b[33m{}\x1b[0m", path), // Yellow for modified files
                    _ => path, // Should never happen due to filter
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

    pub fn merge_trees(
        &self,
        t_head: &str,
        t_other: &str,
        t_base: Option<&str>,
    ) -> Result<HashMap<String, Vec<u8>>, String> {
        let mut tree = HashMap::new();

        let entries = if let Some(base) = t_base {
            // Three-way merge
            self.compare_trees(&[base, t_head, t_other])?
        } else {
            // Two-way merge
            self.compare_trees(&[t_head, t_other])?
        };

        for (path, oids) in entries {
            let merged = if let Some(_base) = t_base {
                // Three-way merge
                self.merge_blobs_three_way(
                    oids[0].as_deref(), // base
                    oids[1].as_deref(), // head
                    oids[2].as_deref(), // other
                )?
            } else {
                // Two-way merge
                self.merge_blobs(oids[0].as_deref(), oids[1].as_deref())?
            };
            tree.insert(path, merged);
        }

        Ok(tree)
    }

    fn merge_blobs_three_way(
        &self,
        o_base: Option<&str>,
        o_head: Option<&str>,
        o_other: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        // If all OIDs are the same, just return the content
        if o_base == o_head && o_head == o_other {
            if let Some(oid) = o_base {
                return Ok(self.repo.get_object(oid).unwrap());
            }
            return Ok(Vec::new());
        }

        // If head and other are the same, return that content
        if o_head == o_other {
            if let Some(oid) = o_head {
                return Ok(self.repo.get_object(oid).unwrap());
            }
            return Ok(Vec::new());
        }

        // If base and head are the same, return other's content
        if o_base == o_head {
            if let Some(oid) = o_other {
                return Ok(self.repo.get_object(oid).unwrap());
            }
            return Ok(Vec::new());
        }

        // If base and other are the same, return head's content
        if o_base == o_other {
            if let Some(oid) = o_head {
                return Ok(self.repo.get_object(oid).unwrap());
            }
            return Ok(Vec::new());
        }

        // Get content from all three versions
        let base_content = if let Some(oid) = o_base {
            self.repo.get_object(oid)?
        } else {
            Vec::new()
        };
        let head_content = if let Some(oid) = o_head {
            self.repo.get_object(oid)?
        } else {
            Vec::new()
        };
        let other_content = if let Some(oid) = o_other {
            self.repo.get_object(oid)?
        } else {
            Vec::new()
        };

        // Convert to strings for easier manipulation
        let base_str = String::from_utf8_lossy(&base_content);
        let head_str = String::from_utf8_lossy(&head_content);
        let other_str = String::from_utf8_lossy(&other_content);

        // Split into lines
        let base_lines: Vec<&str> = base_str.lines().collect();
        let head_lines: Vec<&str> = head_str.lines().collect();
        let other_lines: Vec<&str> = other_str.lines().collect();

        // Find common lines at the start and end
        let mut start_common = 0;
        while start_common < head_lines.len()
            && start_common < other_lines.len()
            && head_lines[start_common] == other_lines[start_common]
        {
            start_common += 1;
        }

        let mut end_common = 0;
        while end_common < head_lines.len()
            && end_common < other_lines.len()
            && head_lines[head_lines.len() - 1 - end_common]
                == other_lines[other_lines.len() - 1 - end_common]
        {
            end_common += 1;
        }

        // Combine the content
        let mut merged = Vec::new();

        // Add common start
        for line in head_lines.iter().take(start_common) {
            merged.extend_from_slice(line.as_bytes());
            merged.push(b'\n');
        }

        // Process the middle section
        let head_middle = &head_lines[start_common..head_lines.len() - end_common];
        let other_middle = &other_lines[start_common..other_lines.len() - end_common];
        let base_middle = if base_lines.len() > start_common + end_common {
            &base_lines[start_common..base_lines.len() - end_common]
        } else {
            &[]
        };

        // Compare each line with the base version
        let mut i = 0;
        while i < head_middle.len() || i < other_middle.len() {
            let head_line = head_middle.get(i);
            let other_line = other_middle.get(i);
            let base_line = base_middle.get(i);

            match (head_line, other_line, base_line) {
                // If both versions changed the same line differently, take the head version
                (Some(h), Some(o), Some(b)) if h != b && o != b && h != o => {
                    merged.extend_from_slice(h.as_bytes());
                    merged.push(b'\n');
                }
                // If only head changed from base, take head
                (Some(h), Some(o), Some(b)) if h != b && o == b => {
                    merged.extend_from_slice(h.as_bytes());
                    merged.push(b'\n');
                }
                // If only other changed from base, take other
                (Some(h), Some(o), Some(b)) if h == b && o != b => {
                    merged.extend_from_slice(o.as_bytes());
                    merged.push(b'\n');
                }
                // If both versions are the same, take either
                (Some(h), Some(o), _) if h == o => {
                    merged.extend_from_slice(h.as_bytes());
                    merged.push(b'\n');
                }
                // If base is missing but both versions exist, take head
                (Some(h), Some(_o), None) => {
                    merged.extend_from_slice(h.as_bytes());
                    merged.push(b'\n');
                }
                // If all versions exist but don't match any condition, take head
                (Some(h), Some(_), Some(_)) => {
                    merged.extend_from_slice(h.as_bytes());
                    merged.push(b'\n');
                }
                // If only head has content, take it
                (Some(h), None, _) => {
                    merged.extend_from_slice(h.as_bytes());
                    merged.push(b'\n');
                }
                // If only other has content, take it
                (None, Some(o), _) => {
                    merged.extend_from_slice(o.as_bytes());
                    merged.push(b'\n');
                }
                // If neither has content, skip
                (None, None, _) => {} // If base exists but neither head nor other exists, skip
            }
            i += 1;
        }

        // Add common end
        for line in head_lines.iter().skip(head_lines.len() - end_common) {
            merged.extend_from_slice(line.as_bytes());
            merged.push(b'\n');
        }

        Ok(merged)
    }

    pub fn merge_blobs(
        &self,
        o_head: Option<&str>,
        o_other: Option<&str>,
    ) -> Result<Vec<u8>, String> {
        // If both OIDs are the same, just return the content
        if o_head == o_other {
            if let Some(oid) = o_head {
                return Ok(self.repo.get_object(oid).unwrap());
            }
            return Ok(Vec::new());
        }

        let mut head_file =
            NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;
        let mut other_file =
            NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;

        if let Some(oid) = o_head {
            let content = self.repo.get_object(oid)?;
            head_file
                .write_all(&content)
                .map_err(|e| format!("Failed to write to temp file: {}", e))?;
        }
        if let Some(oid) = o_other {
            let content = self.repo.get_object(oid)?;
            other_file
                .write_all(&content)
                .map_err(|e| format!("Failed to write to temp file: {}", e))?;
        }

        head_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;
        other_file
            .flush()
            .map_err(|e| format!("Failed to flush temp file: {}", e))?;

        let output = Command::new("diff")
            .args([
                "-DHEAD",
                head_file.path().to_str().unwrap(),
                other_file.path().to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to run diff command: {}", e))?;

        Ok(output.stdout)
    }
}
