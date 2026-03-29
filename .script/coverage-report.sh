#!/usr/bin/env bash
# Write llvm-cov output to GITHUB_OUTPUT
# Expects a coverage symlink/directory at ./coverage containing:
#   html/index.html, coverage.json, and nextest.log
set -e

# Fail if unit tests take longer than this many seconds
MAX_TEST_DURATION_SECS=10
REPORT=$(grep -o "<table>.*</table>" coverage/html/index.html)
PERCENT=$(jq '.data[0].totals.lines.percent' coverage/coverage.json)
COVERED=$(jq '.data[0].totals.lines.covered' coverage/coverage.json)
TOTAL=$(jq '.data[0].totals.lines.count' coverage/coverage.json)

# Parse nextest timing from the nextest log
DURATION=""
TESTS_COUNT=""
NEXTEST_LOG="coverage/nextest.log"
if [ -f "$NEXTEST_LOG" ]; then
  # Summary line example: "     Summary [   0.615s] 235 tests run: 235 passed, 0 skipped"
  DURATION=$(grep -oP 'Summary\s+\[\s*\K[0-9.]+(?=s\])' "$NEXTEST_LOG" | tail -1)
  TESTS_COUNT=$(grep -oP '\d+(?= tests run)' "$NEXTEST_LOG" | tail -1)
fi

{
  echo "REPORT<<EOF"
  echo -e "$REPORT"
  echo "EOF"
  echo "PERCENT=$PERCENT"
  echo "COVERED=$COVERED"
  echo "TOTAL=$TOTAL"
  echo "DURATION=$DURATION"
  echo "TESTS_COUNT=$TESTS_COUNT"
} >>"$GITHUB_OUTPUT"

echo "Coverage is $PERCENT% ($COVERED/$TOTAL lines)"

if (($(echo "$PERCENT < 80" | bc -l))); then
  echo "❌ Coverage below 80%"
  echo "FAILED=true" >>"$GITHUB_OUTPUT"
else
  echo "✅ Coverage above 80%"
fi

if [ -n "$DURATION" ]; then
  echo "Tests completed in ${DURATION}s ($TESTS_COUNT tests)"
  if awk "BEGIN {exit !($DURATION > $MAX_TEST_DURATION_SECS)}"; then
    echo "❌ Tests took ${DURATION}s (limit: ${MAX_TEST_DURATION_SECS}s)"
    echo "FAILED=true" >>"$GITHUB_OUTPUT"
  else
    echo "✅ Tests within ${MAX_TEST_DURATION_SECS}s limit"
  fi
fi
