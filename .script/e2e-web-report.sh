#!/usr/bin/env bash
# Build Maestro web E2E failure report and write REPORT_BODY to GITHUB_OUTPUT.
# Reads CLOUDINARY_CLOUD_NAME and CLOUDINARY_UPLOAD_PRESET from the environment.
set -e

report_body="### ❌ Maestro Web E2E Failures\n"

if [ -f maestro_console.log ]; then
    clean_log=$(sed 's/\x1b\[[0-9;]*[mG]//g' maestro_console.log | tail -n 100)
    report_body+="\n<details>\n<summary>📜 Maestro Console Output (Last 100 lines)</summary>\n\n\`\`\`text\n$clean_log\n\`\`\`\n</details>\n"
fi

if [ -n "${CLOUDINARY_CLOUD_NAME:-}" ] && [ -n "${CLOUDINARY_UPLOAD_PRESET:-}" ]; then
    report_body+="\n#### 📸 Screenshots\n"
    while IFS= read -r img; do
        echo "Uploading $img to Cloudinary…"
        result=$(curl -s -X POST \
            "https://api.cloudinary.com/v1_1/${CLOUDINARY_CLOUD_NAME}/image/upload" \
            -F "file=@$img" \
            -F "upload_preset=${CLOUDINARY_UPLOAD_PRESET}")
        img_url=$(echo "$result" | jq -r '.secure_url')
        if [ "$img_url" != "null" ]; then
            report_body+="\n- ![Screenshot]($img_url)"
        fi
    done < <(find ~/.maestro/tests/ -name "*.png")
fi

{
    echo "REPORT_BODY<<EOF"
    echo -e "$report_body"
    echo "EOF"
} >> "$GITHUB_OUTPUT"
