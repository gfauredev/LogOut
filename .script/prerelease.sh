#!/usr/bin/env bash
# Eventually publish timestamped pre-release
set -e

NOW=$(date -u +"%Y-%m-%d")

CREATED_AT=$(gh release list --json publishedAt,isPrerelease \
  --jq '.[] | select(.isPrerelease == true) | .publishedAt' | head -n 1)

if [ -z "$CREATED_AT" ] ||
  [ "$(date -d "$CREATED_AT" +%s)" -le "$(date -d '2 days ago' +%s)" ]; then
  gh release create "pre-$NOW" *.apk --prerelease \
    --title "$NOW" --notes "Automated pre-release for $NOW"
fi
