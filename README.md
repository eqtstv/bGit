# bGit - A Git Implementation in Rust

`bGit` is a learning project implementing core concepts of the Git version control system using the Rust programming language. It aims to provide a functional subset of Git's features, focusing on understanding the underlying object model and commands.

## Core Concepts Implemented

- **Repository Initialization:** Creates the `.bgit` directory structure (`objects`, `refs`, `HEAD`).
- **Object Model:**
  - **Blobs:** Stores file content.
  - **Trees:** Represents directory structures, referencing blobs and other trees.
  - **Commits:** Records snapshots of the project tree, linking to parent commits.
- **Content-Addressable Storage:** Objects are stored based on the SHA-1 hash of their content.
- **Refs:** Manages pointers like branches (`refs/heads/*`) and tags (`refs/tags/*`).
- **HEAD:** Points to the currently checked-out commit or branch.
- **Branching:** Supports creating and checking out branches.
- **Merging:** Implements three-way merging using `diff3`.
- **Diffing:** Shows differences between commits or the working tree.
- **Ignoring Files:** Basic support for `.bgitignore` (similar to `.gitignore`).

## Getting Started

### Prerequisites

- Rust toolchain (latest stable recommended)
- `diffutils` (provides the `diff3` command used for merging)
  - On macOS: Usually included or installable via Homebrew (`brew install diffutils`)
  - On Debian/Ubuntu: `sudo apt update && sudo apt install diffutils`

### Building

```bash
cargo build
```

### Running Commands

Use `cargo run -- <command> [arguments...]` to execute bGit commands.

```bash
# Example: Initialize a repository in the current directory
cargo run -- init

# Example: Commit changes
echo "Hello World" > file.txt
cargo run -- commit "Add initial file"
```

## CLI Commands

The following commands are available:

- `init`

  - Initializes a new, empty bGit repository in the current directory by creating the `.bgit` structure.
  - Usage: `cargo run -- init`

- `hash-object <file_path>`

  - Reads the content of the specified file, creates a blob object, stores it in the object database (`.bgit/objects`), and prints the resulting SHA-1 hash.
  - Usage: `cargo run -- hash-object path/to/your/file.txt`

- `cat-file <object_hash>`

  - Retrieves and prints the content of a Git object (blob, tree, or commit) given its SHA-1 hash.
  - Usage: `cargo run -- cat-file <sha1_hash>`

- `write-tree`

  - Creates a tree object representing the current state of the working directory (respecting `.bgitignore`), stores it, and prints its SHA-1 hash.
  - Usage: `cargo run -- write-tree`

- `read-tree <tree_hash>`

  - Reads the tree object specified by `<tree_hash>` and updates the working directory to match the state represented by that tree. Warning: This overwrites uncommitted changes.
  - Usage: `cargo run -- read-tree <tree_sha1_hash>`

- `get-tree <tree_hash>`

  - Retrieves and prints the formatted entries (mode, type, name, hash) of a tree object.
  - Usage: `cargo run -- get-tree <tree_sha1_hash>`

- `commit <message>`

  - Creates a new commit object. It generates a tree from the current working directory, finds the current HEAD commit to use as a parent, and combines them with the provided commit message and timestamp. Prints the new commit hash.
  - Usage: `cargo run -- commit "Your descriptive commit message"`

- `log`

  - Displays the commit history starting from the current HEAD, showing commit hashes, parents, dates, and messages.
  - Usage: `cargo run -- log`

- `checkout <commit_or_branch>`

  - Updates the working directory to match the state of the specified commit hash or branch name. Updates the HEAD pointer accordingly.
  - Usage (commit): `cargo run -- checkout <commit_sha1_hash>`
  - Usage (branch): `cargo run -- checkout <branch_name>`

- `tag <tag_name> <commit_hash>`

  - Creates a tag (a reference in `refs/tags/`) pointing to the specified commit hash.
  - Usage: `cargo run -- tag v1.0 <commit_sha1_hash>`

- `branch [branch_name]`

  - With no argument: Lists all local branches, highlighting the current one.
  - With `<branch_name>`: Creates a new branch pointing to the current HEAD commit.
  - Usage (list): `cargo run -- branch`
  - Usage (create): `cargo run -- branch <new_branch_name>`

- `status`

  - Shows the status of the working directory - changed files, untracked files etc.
  - Usage: `cargo run -- status`

- `reset <commit_hash>`

  - Resets the current branch HEAD to the specified `<commit_hash>` and updates the working directory to match (hard reset behavior).
  - Usage: `cargo run -- reset <commit_sha1_hash>`

- `show <commit_hash>`

  - Displays information about a specific commit (metadata and diff against its parent(s)).
  - Usage: `cargo run -- show <commit_sha1_hash>`

- `diff`

  - Shows the differences between the current working directory and the HEAD commit.
  - Usage: `cargo run -- diff`

- `merge <branch_name>`

  - Performs a three-way merge of the specified `<branch_name>` into the current branch (HEAD). Uses the external `diff3` command. Creates merge commit parents if applicable.
  - Usage: `cargo run -- merge <other_branch_name>`

- `iter-refs`

  - (Likely internal or debug command) Iterates and prints all references found in the `.bgit/refs` directory.
  - Usage: `cargo run -- iter-refs`

- `visualize`
  - (Likely internal or debug command) Potentially generates a visualization of the commit graph.
  - Usage: `cargo run -- visualize`

## Project Structure

```
bgit/
├── .github/workflows/ci.yaml # GitHub Actions CI configuration
├── src/
│   ├── cli.rs        # Command-line interface parsing
│   ├── differ.rs     # Diffing and Merging logic
│   ├── repository.rs # Core Git object model and repository operations
│   ├── visualizer.rs # Commit graph visualization
│   └── main.rs       # Entry point, command dispatch
├── tests/            # Integration and unit tests
│   └── ...
├── Cargo.toml        # Project configuration and dependencies
├── Cargo.lock        # Dependency lock file
└── README.md         # This file
```
