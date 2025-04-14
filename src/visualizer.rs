use crate::repository::Repository;
use graphviz_rust::dot_generator::*;
use graphviz_rust::dot_structures::*;
use graphviz_rust::printer::{DotPrinter, PrinterContext};
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::process::Command;
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

pub struct Visualizer {
    repo: Repository,
}

impl Visualizer {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }

    pub fn visualize(&self) -> Result<(), String> {
        let mut graph = graph!(di id!("commit_graph"));

        // Add graph attributes for vertical layout
        graph.add_stmt(Stmt::Attribute(attr!("rankdir", "TB")));
        graph.add_stmt(Stmt::Attribute(attr!("nodesep", "0.5")));
        graph.add_stmt(Stmt::Attribute(attr!("ranksep", "0.5")));
        graph.add_stmt(Stmt::Attribute(attr!("splines", "ortho")));

        // Get current HEAD
        let head_hash = self.repo.get_ref("HEAD", true)?;

        let refs = self.repo.iter_refs("")?;
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
                .map(|(ref_name, _)| {
                    ref_name
                        .split('/')
                        .next_back()
                        .unwrap_or(ref_name)
                        .to_string()
                })
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
            let is_head = commit_hash == &head_hash.value;
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

        // Create a temporary file for the DOT output
        let mut dot_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        dot_file
            .write_all(dot_output.as_bytes())
            .map_err(|e| e.to_string())?;
        let dot_path = dot_file.path().to_path_buf();

        // Create a temporary file for the SVG output
        let svg_file = NamedTempFile::new().map_err(|e| e.to_string())?;
        let svg_path = svg_file.path().to_path_buf();

        // Convert DOT to SVG using Graphviz
        let output = Command::new("dot")
            .arg("-Tsvg")
            .arg(dot_path)
            .arg("-o")
            .arg(&svg_path)
            .output()
            .map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(format!(
                "Failed to generate SVG: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        // Read the SVG content
        let svg_content = fs::read_to_string(&svg_path).map_err(|e| e.to_string())?;

        // Wrap SVG content with interactive controls
        let interactive_svg = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Git Commit Graph</title>
    <style>
        body {{ margin: 0; overflow: hidden; }}
        #svg-container {{ 
            width: 100vw; 
            height: 100vh; 
            overflow: hidden;
            cursor: grab;
        }}
        #svg-container:active {{ cursor: grabbing; }}
    </style>
</head>
<body>
    <div id="svg-container">
        {}
    </div>
    <script>
        const container = document.getElementById('svg-container');
        let scale = 1;
        let isPanning = false;
        let startPoint = {{ x: 0, y: 0 }};
        let transform = {{ x: 0, y: 0 }};

        // Zoom functionality
        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            
            // Get mouse position relative to container
            const rect = container.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            
            // Calculate the point to zoom around
            const x = (mouseX - transform.x) / scale;
            const y = (mouseY - transform.y) / scale;
            
            // Apply zoom
            scale *= delta;
            scale = Math.min(Math.max(0.1, scale), 5);
            
            // Adjust transform to zoom around mouse position
            transform.x = mouseX - x * scale;
            transform.y = mouseY - y * scale;
            
            updateTransform();
        }});

        // Pan functionality
        container.addEventListener('mousedown', (e) => {{
            isPanning = true;
            startPoint = {{ x: e.clientX - transform.x, y: e.clientY - transform.y }};
        }});

        container.addEventListener('mousemove', (e) => {{
            if (!isPanning) return;
            
            transform.x = e.clientX - startPoint.x;
            transform.y = e.clientY - startPoint.y;
            
            updateTransform();
        }});

        container.addEventListener('mouseup', () => {{
            isPanning = false;
        }});

        container.addEventListener('mouseleave', () => {{
            isPanning = false;
        }});

        function updateTransform() {{
            container.style.transform = `translate(${{transform.x}}px, ${{transform.y}}px) scale(${{scale}})`;
        }}
    </script>
</body>
</html>"#,
            svg_content
        );

        // Start a simple HTTP server
        let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        let url = format!("http://127.0.0.1:{}", port);

        println!("Opening visualization in browser at {}...", url);

        // Open the browser in a separate thread
        let url_clone = url.clone();
        thread::spawn(move || {
            if let Err(e) = webbrowser::open(&url_clone) {
                eprintln!("Failed to open browser: {}", e);
            }
        });

        // Serve the interactive SVG content
        if let Some(stream) = listener.incoming().next() {
            match stream {
                Ok(mut stream) => {
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
                        interactive_svg.len(),
                        interactive_svg
                    );
                    stream
                        .write_all(response.as_bytes())
                        .map_err(|e| e.to_string())?;
                }
                Err(e) => return Err(format!("Failed to accept connection: {}", e)),
            }
        }

        // Keep the server running for a while
        println!("Visualization opened. The server will be kept alive for 5 seconds...");
        thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}
