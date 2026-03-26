#!/usr/bin/env bash
# Bump SemVer in Cargo.toml and flake.nix based on conventional commits
# Increments MAJOR for breaking changes, MINOR for feat:, PATCH otherwise
# Prompts to create a git tag if running in an interactive terminal
set -e

cargo_toml="${1:-Cargo.toml}"
flake_nix="${2:-flake.nix}"

# Read current version from Cargo.toml
current=$(grep '^version = ' "$cargo_toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
major=$(echo "$current" | cut -d. -f1)
minor=$(echo "$current" | cut -d. -f2)
patch=$(echo "$current" | cut -d. -f3)

# Collect commits between origin/main and HEAD (fall back to last 20 commits)
if git rev-parse --verify origin/main >/dev/null 2>&1; then
  commits=$(git log origin/main..HEAD --pretty=format:"%s%n%b" 2>/dev/null)
else
  commits=$(git log --pretty=format:"%s%n%b" HEAD~20..HEAD 2>/dev/null || true)
fi

# Determine bump level
if echo "$commits" | grep -qE "(BREAKING[[:space:]]CHANGE|!:)"; then
  major=$((major + 1))
  minor=0
  patch=0
elif echo "$commits" | grep -qE "^feat(\([^)]*\))?:"; then
  minor=$((minor + 1))
  patch=0
else
  patch=$((patch + 1))
fi

new="$major.$minor.$patch"
echo "Bumping version: $current → $new"

# Update Cargo.toml – target only the first occurrence to avoid touching dependencies
sed -i "0,/^version = \"$current\"/{s/^version = \"$current\"/version = \"$new\"/}" "$cargo_toml"

# Update flake.nix projectVersion
sed -i "s/projectVersion = \"$current\"/projectVersion = \"$new\"/" "$flake_nix"

echo "Updated $cargo_toml and $flake_nix to $new"

# Prompt to tag if running interactively
if [ -t 0 ] && [ -t 1 ]; then
  read -r -p "Create git tag v$new? [y/N] " response
  if [[ "$response" =~ ^[Yy]$ ]]; then
    cargo update logout
    git commit --all
    git tag --annotate "v$new" --message "Release v$new"
    echo "Created tag v$new"
  fi
fi
