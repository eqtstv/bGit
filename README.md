# bgit

A minimal Git implementation in Rust.

## Commands

### Initialize a repository

```bash
cargo run -- init
```

Creates a new `.bgit` directory with the necessary structure for version control.

## Project Structure

```
bgit/
├── src/
│   ├── cli.rs     # Command-line interface implementation
│   ├── data.rs    # Core Git functionality implementation
│   └── main.rs    # Entry point and command handling
└── Cargo.toml     # Project configuration and dependencies
```
