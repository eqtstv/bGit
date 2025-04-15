#!/bin/bash

# Initialize the repository
cargo run -- init

# Create the initial commit and capture its hash
COMMIT_HASH=$(cargo run -- commit "initial-commit" | tail -n 1)

# Create a tag for the initial commit
cargo run -- tag init "$COMMIT_HASH"

# Create second commit
echo "Second commit content" > test.txt
COMMIT_HASH=$(cargo run -- commit "commit-1" | tail -n 1)
cargo run -- tag c1 "$COMMIT_HASH"

# Create third commit
echo "Third commit content" > test.txt
COMMIT_HASH=$(cargo run -- commit "commit-2" | tail -n 1)
cargo run -- tag c2 "$COMMIT_HASH"

# Checkout to initial commit
cargo run -- checkout init

# Create first commit in new branch
echo "Branch 2 commit 1 content" > test.txt
COMMIT_HASH=$(cargo run -- commit "branch2-commit-1" | tail -n 1)
cargo run -- tag b2c1 "$COMMIT_HASH"

# Create second commit in new branch
echo "Branch 2 commit 2 content" > test.txt
COMMIT_HASH=$(cargo run -- commit "branch2-commit-2" | tail -n 1)
cargo run -- tag b2c2 "$COMMIT_HASH"

echo "Done"
