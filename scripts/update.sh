#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

REPO_DIR=$(pwd)
FORK_REMOTE="fork"
UPSTREAM_REMOTE="origin"
BIN_SRC="target/release/omatunes"
BIN_DST="$HOME/.local/bin/omatunes"

has_stashed=0
if ! git diff --quiet --ignore-submodules; then
    echo "Stashing uncommitted changes..."
    git stash push -m "auto-stash before update"
    has_stashed=1
fi

echo "Fetching upstream..."
git fetch "$UPSTREAM_REMOTE"

echo "Merging $UPSTREAM_REMOTE/master into $(git branch --show-current)..."
git merge "$UPSTREAM_REMOTE/master" --no-edit || {
    echo "Merge failed (conflicts). Resolve them, then run:"
    echo "  cargo build --release && cp $BIN_SRC $BIN_DST"
    exit 1
}

echo "Pushing to fork ($FORK_REMOTE)..."
git push "$FORK_REMOTE" "$(git branch --show-current)"

echo "Building release binary..."
cargo build --release

echo "Installing binary to $BIN_DST..."
cp "$BIN_SRC" "$BIN_DST"

if [ "$has_stashed" -eq 1 ]; then
    echo "Restoring stashed changes..."
    git stash pop
fi

echo "Done! omatunes is up to date."
