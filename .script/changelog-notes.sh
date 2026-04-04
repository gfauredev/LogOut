#!/usr/bin/env bash
# Print commit messages since a given tag, formatted as a Markdown list.
# Usage: changelog-notes.sh [from-tag [to-ref]]
#   from-tag  optional starting tag; when omitted all commits up to to-ref are listed
#   to-ref    optional end ref (default: HEAD)
set -e

FROM="${1:-}"
TO="${2:-HEAD}"

if [ -n "$FROM" ]; then
  git log "$FROM..$TO" --pretty=format:'- %s' --no-merges
else
  git log "$TO" --pretty=format:'- %s' --no-merges
fi
