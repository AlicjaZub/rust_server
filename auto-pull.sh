#!/bin/bash

REPO_DIR="rust-repo"
BRANCH="main"

cd "$REPO_DIR" || exit 1

git fetch origin "$BRANCH"

LOCAL_HASH=$(git rev-parse "$BRANCH")
REMOTE_HASH=$(git rev-parse "origin/$BRANCH")

if [ "$LOCAL_HASH" != "$REMOTE_HASH" ]; then
  echo "New changes detected. Pulling latest..."
  git reset --hard "origin/$BRANCH"

else
  echo "No changes detected."
fi
