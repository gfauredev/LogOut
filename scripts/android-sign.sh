set -e
APK_PATH=$1
KEYSTORE_PATH=${ANDROID_KEYSTORE_PATH:-"android/secrets/logout.jks"}
KEY_ALIAS=${ANDROID_KEY_ALIAS:-"logout-key"}
if [ -z "$APK_PATH" ]; then
  dx build --android --release
  APK_PATH=$(find target/dx/log-workout/release/android/ -name "*.apk" | head -n 1)
fi
if [ ! -f "$APK_PATH" ]; then
  echo "Error: File $APK_PATH not found."
  exit 1
fi
# Ensure required environment variables are set
if [ -z "$ANDROID_KEYSTORE_PASS" ]; then
  if [ -z "$ANDROID_KEY_PASS" ]; then
    echo "Error: ANDROID_KEYSTORE_PASS or ANDROID_KEY_PASS needed"
    exit 1
  else
    export ANDROID_KEYSTORE_PASS=$ANDROID_KEY_PASS
  fi
else
  if [ -z "$ANDROID_KEY_PASS" ]; then
    export ANDROID_KEY_PASS=$ANDROID_KEYSTORE_PASS
  fi
fi
echo "🖋️ Signing $APK_PATH..."
apksigner sign --ks "$KEYSTORE_PATH" \
  --ks-key-alias "$KEY_ALIAS" \
  --ks-pass "env:ANDROID_KEYSTORE_PASS" \
  --key-pass "env:ANDROID_KEY_PASS" \
  "$APK_PATH"
echo "✅ Successfully signed $APK_PATH"
echo "To install on device, use the command:"
echo "  adb install -r $APK_PATH"
