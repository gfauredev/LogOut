#!/bin/sh
set -e
# OUT=release-signed.apk
APK_PATH=$1
KEYSTORE_PATH=${ANDROID_KEYSTORE_PATH:-"android/secrets/logout.jks"}
KEY_ALIAS=${ANDROID_KEY_ALIAS:-"logout-key"}
if [ -z "$APK_PATH" ]; then
  APK_PATH=$(find target/dx/log-out/release/android/ -name "*.apk" | head -n 1)
fi
if [ ! -f "$APK_PATH" ]; then
  echo "Error: File $APK_PATH not found."
  exit 1
fi
# Ensure required environment variables are set
if [ -z "$ANDROID_KEYSTORE_PASS" ]; then
  if [ -z "$ANDROID_KEY_PASSWORD" ]; then
    echo "Error: ANDROID_KEYSTORE_PASS or ANDROID_KEY_PASSWORD needed"
    exit 1
  else
    export ANDROID_KEYSTORE_PASS=$ANDROID_KEY_PASSWORD
  fi
else
  if [ -z "$ANDROID_KEY_PASSWORD" ]; then
    export ANDROID_KEY_PASSWORD=$ANDROID_KEYSTORE_PASS
  fi
fi
# Ensure apksigner is in PATH
if ! command -v apksigner >/dev/null 2>&1; then
  if [ -n "$ANDROID_HOME" ]; then
    APKSIGNER=$(find "$ANDROID_HOME/build-tools" -name apksigner | sort -r | head -n 1)
    if [ -n "$APKSIGNER" ]; then
      export PATH="$(dirname "$APKSIGNER"):$PATH"
    else
      echo "Error: apksigner not found in $ANDROID_HOME/build-tools"
      exit 1
    fi
  else
    echo "Error: apksigner not found and ANDROID_HOME not set"
    exit 1
  fi
fi
echo "🖋️ Signing $APK_PATH..."
apksigner sign --ks "$KEYSTORE_PATH" \
  --ks-key-alias "$KEY_ALIAS" \
  --ks-pass "env:ANDROID_KEYSTORE_PASS" \
  --key-pass "env:ANDROID_KEY_PASSWORD" "$APK_PATH" # --out "$OUT"
echo "✅ Successfully signed $APK_PATH"              # to $OUT"
echo "To install on device, use: adb install -r $APK_PATH"
