# Pure Nix build for the Android APK
# Usage: nix build .#android
#
# Uses a Fixed-Output Derivation (FOD) to pre-fetch Gradle/Maven
# dependencies, then builds the APK in offline mode.
#
# When Gradle dependencies change (e.g. Dioxus or Android SDK update),
# update gradleDeps.outputHash:
#   1. Set it to lib.fakeHash
#   2. Run: nix build .#android
#   3. Copy the hash from the error message
{
  pkgs,
  lib,
  rustPlatform,
  commonNativeBuildInputs,
  commonBuildInputs,
  androidComposition,
  self,
}:

let
  androidSdk = androidComposition.androidsdk;
  ndkBundle = androidComposition.ndk-bundle;
  androidHome = "${androidSdk}/libexec/android-sdk";
  androidNdkHome = "${ndkBundle}/libexec/android-sdk/ndk-bundle";
  jdk = pkgs.openjdk;

  ldLibraryPath = lib.makeLibraryPath [
    pkgs.stdenv.cc.cc.lib
    pkgs.zlib
  ];

  # All native build inputs needed for Android builds
  androidNativeBuildInputs = commonNativeBuildInputs ++ [
    pkgs.cargo-ndk
    androidSdk
    ndkBundle
    jdk
  ];

  # Patch aapt2 binaries downloaded by Gradle for NixOS compatibility.
  # On NixOS, ELF binaries from Maven need their dynamic linker patched.
  patchAapt2 = ''
    find "''${GRADLE_USER_HOME:-/nonexistent}" "''${CARGO_TARGET_DIR:-target}" \
      -name aapt2 -type f -executable 2>/dev/null | while read -r aapt2; do
      if ! patchelf --print-interpreter "$aapt2" >/dev/null 2>&1 || \
         [[ "$(patchelf --print-interpreter "$aapt2")" == /lib* ]]; then
        echo "Patching aapt2: $aapt2"
        chmod +x "$aapt2"
        if ! patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$aapt2" || \
           ! patchelf --set-rpath "${ldLibraryPath}" "$aapt2"; then
          echo "Error: failed to patch aapt2 binary: $aapt2" >&2
          exit 1
        fi
      fi
    done
  '';

  # Fixed-Output Derivation to pre-fetch Gradle and Maven dependencies.
  # FODs get network access during build, producing a fixed output hash.
  # Update outputHash when Gradle dependencies change.
  gradleDeps = pkgs.stdenv.mkDerivation {
    name = "logout-android-gradle-deps";
    src = self;

    nativeBuildInputs = androidNativeBuildInputs;
    buildInputs = commonBuildInputs;

    # FOD: allows network access, output verified by hash
    outputHashAlgo = "sha256";
    outputHashMode = "recursive";
    outputHash = "sha256-zkSmUpOmnp35/FAPqNpA0w9VAeniOCnlUFct39tRd0k=";

    ANDROID_HOME = androidHome;
    ANDROID_NDK_HOME = androidNdkHome;
    LD_LIBRARY_PATH = ldLibraryPath;

    buildPhase = ''
      export HOME=$TMPDIR/fake-home
      mkdir -p $HOME
      export XDG_DATA_HOME=$HOME/.local/share
      export GRADLE_USER_HOME=$TMPDIR/gradle-home
      mkdir -p $GRADLE_USER_HOME

      # Use absolute path so dx derives absolute paths for WRY env vars.
      # Cargo build scripts run from the dependency source dir, so
      # relative paths from dx cannot be resolved.
      export CARGO_TARGET_DIR=$(pwd)/target

      # First attempt: downloads Cargo + Gradle deps, may fail at aapt2
      dx build --android --release || true

      # Patch aapt2 binaries for NixOS (Gradle downloads them from Maven)
      ${patchAapt2}

      # Second attempt: aapt2 patched, Cargo/Gradle cached
      dx build --android --release
    '';

    installPhase = ''
      mkdir -p $out

      # Copy only the downloaded artifact caches and Gradle wrapper,
      # skip daemon logs, build outputs, and other non-deterministic data.
      if [ -d "$GRADLE_USER_HOME/caches" ]; then
        cp -r "$GRADLE_USER_HOME/caches" "$out/caches"
      fi
      if [ -d "$GRADLE_USER_HOME/wrapper" ]; then
        cp -r "$GRADLE_USER_HOME/wrapper" "$out/wrapper"
      fi

      # Normalize for deterministic output:
      # Remove Gradle lock files, journals, and non-deterministic metadata
      find $out -name "*.lock" -type f -delete
      find $out -name "gc.properties" -type f -delete
      find $out -name "file-access.properties" -type f -delete
      find $out -name "resource-usage.json" -type f -delete
      find $out -name "last-build.bin" -type f -delete

      # Remove non-deterministic directories
      find $out -type d -name "journal-*" -exec rm -rf {} + 2>/dev/null || true
      find $out -type d -name "build-scan-data" -exec rm -rf {} + 2>/dev/null || true
      find $out -type d -name "daemon" -exec rm -rf {} + 2>/dev/null || true
      find $out -type d -name "build-cache-*" -exec rm -rf {} + 2>/dev/null || true
      find $out -type d -name "file-changes" -exec rm -rf {} + 2>/dev/null || true

      # Remove non-deterministic Gradle transform metadata
      find $out -name "*.bin" -path "*/transforms-*/*" -type f -delete 2>/dev/null || true

      # Remove Gradle version-specific non-deterministic data
      find $out -type d -name "executionHistory" -exec rm -rf {} + 2>/dev/null || true
      find $out -type d -name "fileChanges" -exec rm -rf {} + 2>/dev/null || true
      find $out -type d -name "fileHashes" -exec rm -rf {} + 2>/dev/null || true

      # Remove empty directories
      find $out -type d -empty -delete 2>/dev/null || true

      # Set all timestamps to epoch for reproducibility
      find $out -exec touch -h -d @0 {} +
    '';
  };

in
rustPlatform.buildRustPackage {
  pname = "logout-android";
  version = "0.1.0";
  src = self;
  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = androidNativeBuildInputs;
  buildInputs = commonBuildInputs;

  ANDROID_HOME = androidHome;
  ANDROID_NDK_HOME = androidNdkHome;
  LD_LIBRARY_PATH = ldLibraryPath;

  buildPhase = ''
    export HOME=$TMPDIR/fake-home
    mkdir -p $HOME
    export XDG_DATA_HOME=$HOME/.local/share

    # Use absolute path so dx derives absolute paths for WRY env vars.
    # Cargo build scripts run from the dependency source dir, so
    # relative paths from dx cannot be resolved.
    export CARGO_TARGET_DIR=$(pwd)/target

    # Restore pre-fetched Gradle dependencies
    export GRADLE_USER_HOME=$TMPDIR/gradle-home
    mkdir -p $GRADLE_USER_HOME
    cp -r ${gradleDeps}/* $GRADLE_USER_HOME/
    chmod -R u+w $GRADLE_USER_HOME

    # Patch aapt2 for NixOS
    ${patchAapt2}

    # Build APK with offline Gradle (deps already cached)
    dx build --android --release --offline
  '';

  installPhase = ''
    mkdir -p $out
    find target/dx/log-out/release/android -name "*.apk" -exec cp {} $out/ \;
  '';

  doCheck = false;
}
