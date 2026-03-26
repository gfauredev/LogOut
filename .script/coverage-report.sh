#!/usr/bin/env bash
# Write llvm-cov output to GITHUB_OUTPUT
# Expects a coverage symlink/directory at ./coverage containing:
#   html/index.html  and  coverage.json
set -e

REPORT=$(grep -o "<table>.*</table>" coverage/html/index.html)
PERCENT=$(jq '.data[0].totals.lines.percent' coverage/coverage.json)
COVERED=$(jq '.data[0].totals.lines.covered' coverage/coverage.json)
TOTAL=$(jq '.data[0].totals.lines.count' coverage/coverage.json)

{
  echo "REPORT<<EOF"
  echo -e "$REPORT"
  echo "EOF"
  echo "PERCENT=$PERCENT"
  echo "COVERED=$COVERED"
  echo "TOTAL=$TOTAL"
} >>"$GITHUB_OUTPUT"

echo "Coverage is $PERCENT% ($COVERED/$TOTAL lines)"

if (($(echo "$PERCENT < 80" | bc -l))); then
  echo "❌ Coverage below 80%"
  echo "FAILED=true" >>"$GITHUB_OUTPUT"
else
  echo "✅ Coverage above 80%"
fi
