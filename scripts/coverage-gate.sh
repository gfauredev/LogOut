#!/usr/bin/env bash

# coverage-gate.sh: Fail if coverage is below 90% for tested files in src/

if [ ! -f lcov.info ]; then
  echo "::error::lcov.info not found — coverage gate cannot be evaluated"
  exit 1
fi

# Find files that contain tests, but avoid those that only have tests that are empty
# This matches the current logic in ci.yml which looks for # [cfg(test)]
TESTED_FILES=$(grep -rl '#\[cfg(test)\]' src/ | sort || true)

if [ -z "$TESTED_FILES" ]; then
  echo "No tested files found."
  exit 0
fi

FAIL=0
echo "Evaluating coverage gate (>= 90% per file)..."

for f in $TESTED_FILES; do
  LH=0; LF=0; IN_FILE=0
  while IFS= read -r line; do
    case "$line" in
      SF:*"$f") IN_FILE=1 ;;
      SF:*) IN_FILE=0 ;;
      LH:*) [ "$IN_FILE" = 1 ] && LH="${line#LH:}" ;;
      LF:*) [ "$IN_FILE" = 1 ] && LF="${line#LF:}" ;;
    esac
  done < lcov.info

  if [ "$LF" -gt 0 ]; then
    PCT=$((LH * 100 / LF))
  else
    PCT=0
  fi

  if [ "$PCT" -lt 90 ]; then
    echo "::error::Coverage below 90% for $f: ${LH}/${LF} lines (${PCT}%)"
    FAIL=1
  else
    echo "✅ $f: $PCT% (${LH}/${LF} lines)"
  fi
done

exit $FAIL
