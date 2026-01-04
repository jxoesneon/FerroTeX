#!/bin/bash
set -e

# Ensure we're in the repo root
cd "$(dirname "$0")/.."

UPSTREAM_DIR="upstream"
TECTORIC_DIR="$UPSTREAM_DIR/tectonic"
REPO_URL="https://github.com/tectonic-typesetting/tectonic.git"
BRANCH="local/ferrotex-vendor"

# Create upstream directory if it doesn't exist
if [ ! -d "$UPSTREAM_DIR" ]; then
    echo "Creating $UPSTREAM_DIR..."
    mkdir -p "$UPSTREAM_DIR"
fi

# Clone Tectonic if not present
if [ ! -d "$TECTORIC_DIR" ]; then
    echo "Cloning Tectonic from $REPO_URL..."
    git clone "$REPO_URL" "$TECTORIC_DIR"
else
    echo "Tectonic repo found at $TECTORIC_DIR."
fi

# Setup the vendored branch
echo "Setting up $BRANCH..."
cd "$TECTORIC_DIR"
git fetch origin

# Check if the branch exists locally, if not create/checkout it roughly matching our expectation
# Note: In a real scenario, this branch exists only locally on the user's machine right now.
# This script assumes the user might be restoring state or another dev might (eventually) pull it if we pushed `upstream`.
# Since we didn't push `upstream` (it's a massive repo), this script is mostly for local restoration.

if git show-ref --verify --quiet "refs/heads/$BRANCH"; then
    git checkout "$BRANCH"
else
    echo "Branch $BRANCH not found. Checking out 0.15.0/master as fallback."
    # Fallback to tag if local branch missing
    git checkout tectonic@0.15.0
fi

# Initialize submodules
echo "Initializing submodules..."
git submodule update --init --recursive

echo "Upstream setup complete. You can now build FerroTeX with [patch.crates-io] enabled."
