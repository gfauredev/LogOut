#!/usr/bin/env bash
# Cleanup old pre-releases
set -e

gh release list --json tagName,createdAt,isPrerelease --limit 100 \
  --jq '.[]|select(.isPrerelease == true)|"\(.tagName) \(.createdAt)"' |
  while read -r OLD_TAG CREATED_AT; do
    TIMESTAMP=$(date -d "$CREATED_AT" +%s)
    if [ "$TIMESTAMP" -lt "$(date -d '14 days ago' +%s)" ]; then
      gh release delete "$OLD_TAG" --yes --cleanup-tag
    fi
  done
