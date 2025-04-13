use crate::repository::Repository;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};

pub struct Visualizer {
    repo: Repository,
}

impl Visualizer {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub fn visualize(&self) -> Result<String, String> {
        let mut graph = graph!(di id!("commit_graph"));

        // Add graph attributes for vertical layout
        graph.add_stmt(Stmt::Attribute(attr!("rankdir", "TB")));
        graph.add_stmt(Stmt::Attribute(attr!("nodesep", "0.5")));
        graph.add_stmt(Stmt::Attribute(attr!("ranksep", "0.5")));
        graph.add_stmt(Stmt::Attribute(attr!("splines", "ortho")));

        // Get current HEAD
        let head_hash = self.repo.get_ref("HEAD")?;

        let refs = self.repo.iter_refs()?;
        let commits = self
            .repo
            .iter_commits_and_parents(refs.iter().map(|(_, hash)| hash.clone()).collect())?;

        // Create nodes for all commits
        for commit_hash in &commits {
            let commit = self.repo.get_commit(commit_hash)?;
            let short_hash = &commit_hash[..7];
            let first_line = commit
                .message
                .lines()
                .next()
                .unwrap_or("")
                .replace('"', "\\\"");

            // Find any tags pointing to this commit
            let tags: Vec<String> = refs
                .iter()
                .filter(|(ref_name, hash)| {
                    ref_name.starts_with("refs/tags/") && hash == commit_hash
                })
                .map(|(ref_name, _)| ref_name.split('/').last().unwrap_or(ref_name).to_string())
                .collect();

            // Create label with hash, tags, and commit message
            let mut label = format!("\"{}\\n{}\"", short_hash, first_line);
            if !tags.is_empty() {
                label = format!(
                    "\"{}\\n{}\\ntag: {}\"",
                    short_hash,
                    first_line,
                    tags.join(", ")
                );
            }

            let node_id = format!("\"{}\"", commit_hash);

            // Style HEAD commit differently
            let is_head = commit_hash == &head_hash;
            let node_style = if is_head {
                node!(node_id;
                    attr!("label", label),
                    attr!("shape", "box"),
                    attr!("style", "filled"),
                    attr!("fillcolor", "gold"),
                    attr!("penwidth", "2")
                )
            } else {
                node!(node_id;
                    attr!("label", label),
                    attr!("shape", "box"),
                    attr!("style", "filled"),
                    attr!("fillcolor", "lightblue")
                )
            };

            graph.add_stmt(Stmt::Node(node_style));
        }

        // Add edges for parent relationships
        for commit_hash in &commits {
            let commit = self.repo.get_commit(commit_hash)?;
            if let Some(parent) = &commit.parent {
                let from_id = format!("\"{}\"", commit_hash);
                let to_id = format!("\"{}\"", parent);
                graph.add_stmt(Stmt::Edge(edge!(node_id!(from_id) => node_id!(to_id);
                    attr!("arrowhead", "normal")
                )));
            }
        }

        // Add branch refs as special nodes
        for (ref_name, commit_hash) in refs {
            // Skip tags since they're now shown in the commit node
            if ref_name.starts_with("refs/tags/") {
                continue;
            }

            let ref_label = ref_name.split('/').next_back().unwrap_or(&ref_name);
            let ref_label = format!("\"{}\"", ref_label);
            let ref_id = format!("\"{}\"", ref_name);
            let commit_id = format!("\"{}\"", commit_hash);

            // Style ref nodes
            let node_style = node!(ref_id;
                attr!("label", ref_label),
                attr!("shape", "box"),
                attr!("style", "filled"),
                attr!("fillcolor", "lightgreen")
            );

            graph.add_stmt(Stmt::Node(node_style));
            graph.add_stmt(Stmt::Edge(edge!(node_id!(ref_id) => node_id!(commit_id);
                attr!("style", "dashed")
            )));
        }

        // Generate DOT format output
        let dot_output = graph.print(&mut PrinterContext::default());
        Ok(dot_output)
    }
}
