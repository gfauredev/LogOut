#!/bin/sh
set -e
NAME=LogOut
AAB_PATH=$1
KEYSTORE_PATH=${ANDROID_KEYSTORE_PATH:-"android/secrets/logout.jks"}
KEY_ALIAS=${ANDROID_KEY_ALIAS:-"logout-key"}
if [ -z "$AAB_PATH" ]; then
  AAB_PATH=$(find target/dx/log-out/release/android/ -name "*.aab" | head -n 1)
fi
if [ ! -f "$AAB_PATH" ]; then
  echo "Error: File $AAB_PATH not found."
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
# Ensure jarsigner is in PATH (provided by any JDK)
if ! command -v jarsigner >/dev/null 2>&1; then
  if [ -n "$JAVA_HOME" ]; then
    export PATH="$JAVA_HOME/bin:$PATH"
  fi
  if ! command -v jarsigner >/dev/null 2>&1; then
    echo "Error: jarsigner not found – ensure a JDK is installed and JAVA_HOME is set"
    exit 1
  fi
fi
OUTPUT="$NAME.aab"
cp "$AAB_PATH" "$OUTPUT"
echo "🖋️ Signing $AAB_PATH → $OUTPUT …"
# Google Play requires v1 (JAR) signing via jarsigner.
# SHA-256 digest and SHA256withRSA signature algorithm are required for
# compliance with modern Android signing requirements.
jarsigner \
  -verbose \
  -sigalg SHA256withRSA \
  -digestalg SHA-256 \
  -keystore "$KEYSTORE_PATH" \
  -storepass "$ANDROID_KEYSTORE_PASS" \
  -keypass "$ANDROID_KEY_PASSWORD" \
  "$OUTPUT" "$KEY_ALIAS"
echo "✅ Successfully signed $AAB_PATH to $OUTPUT"
echo "Upload $OUTPUT to Google Play Console as an App Bundle"
