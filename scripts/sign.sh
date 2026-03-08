#!/usr/bin/env bash
# scripts/sign.sh
set -e

APK_PATH=$1
KEYSTORE_PATH=${ANDROID_KEYSTORE_PATH:-"android/secrets/logout.jks"}
KEY_ALIAS=${ANDROID_KEY_ALIAS:-"logout-key"}

if [ -z "$APK_PATH" ]; then
    echo "Usage: $0 <path-to-apk>"
    exit 1
fi

if [ ! -f "$APK_PATH" ]; then
    echo "Error: File $APK_PATH not found."
    exit 1
fi

# Ensure environment variables are set
if [ -z "$ANDROID_KEYSTORE_PASS" ] || [ -z "$ANDROID_KEY_PASS" ]; then
    echo "Error: ANDROID_KEYSTORE_PASS and ANDROID_KEY_PASS must be set."
    echo "Tip: Add them to your .envrc or export them in your shell."
    exit 1
fi

echo "🖋️ Signing $APK_PATH..."
apksigner sign --ks "$KEYSTORE_PATH" 
  --ks-key-alias "$KEY_ALIAS" 
  --ks-pass "env:ANDROID_KEYSTORE_PASS" 
  --key-pass "env:ANDROID_KEY_PASS" 
  "$APK_PATH"

echo "✅ Successfully signed $APK_PATH"
