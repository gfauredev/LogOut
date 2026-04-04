#!/usr/bin/env bash
# Eventually publish timestamped pre-release
set -e

NOW=$(date -u +"%Y-%m-%d-%H")

CREATED_AT=$(gh release list --json publishedAt,isPrerelease \
  --jq '.[] | select(.isPrerelease == true) | .publishedAt' | head -n 1)

if [ -z "$CREATED_AT" ] ||
  [ "$(date -d "$CREATED_AT" +%s)" -le "$(date -d 'a hour ago' +%s)" ]; then
  LAST_PRE=$(git tag --sort=-creatordate | grep '^pre-' | head -n 1)
  NOTES=$(.script/changelog-notes.sh "$LAST_PRE")
  gh release create "pre-$NOW" *.apk --prerelease \
    --title "$NOW" --notes "${NOTES:-Automated pre-release for $NOW}"
fi
