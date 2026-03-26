#!/usr/bin/env bash
# Build a Lighthouse audit summary and write it to GITHUB_OUTPUT as BODY.
# Usage: .script/lighthouse-report.sh '<links-json>'
# The optional first argument is the JSON object from treosh/lighthouse-ci-action outputs.links.
set -e

LINKS="${1:-}"
TMPFILE=$(mktemp)

echo "### 🚀 Lighthouse Audit" > "$TMPFILE"

if [ -f .lighthouseci/manifest.json ]; then
    {
        echo ""
        echo "| URL | Performance | Accessibility | Best Practices | SEO | PWA |"
        echo "| --- | --- | --- | --- | --- | --- |"
        jq -r '.[] |
          (.url | sub("http://localhost:[0-9]*/"; "")) as $url |
          (.summary.performance // 0 | . * 100 | floor) as $perf |
          (.summary.accessibility // 0 | . * 100 | floor) as $a11y |
          (.summary["best-practices"] // 0 | . * 100 | floor) as $bp |
          (.summary.seo // 0 | . * 100 | floor) as $seo |
          (.summary.pwa // 0 | . * 100 | floor) as $pwa |
          "| `\($url)` | \($perf) | \($a11y) | \($bp) | \($seo) | \($pwa) |"
        ' .lighthouseci/manifest.json
    } >> "$TMPFILE"
fi

if [ -n "$LINKS" ] && echo "$LINKS" | jq empty 2>/dev/null; then
    {
        echo ""
        echo "**Full reports:**"
        echo "$LINKS" | jq -r \
            'to_entries[] | "- [\(.key | sub("http://localhost:[0-9]*/"; ""))](\(.value))"'
    } >> "$TMPFILE"
fi

{
    echo "BODY<<EOF"
    cat "$TMPFILE"
    echo "EOF"
} >> "$GITHUB_OUTPUT"
