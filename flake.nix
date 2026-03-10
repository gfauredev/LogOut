{
  description = "LogOut dev envs";
  nixConfig = {
    extra-substituters = [ "https://cache.garnix.io" ];
    extra-trusted-public-keys = [ "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" ];
  };
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux"
        # "aarch64-linux"
        "aarch64-darwin"
      ];
      nixpkgsFor = forAllSystems (
        system:
        import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
          config = {
            allowUnfree = true;
            android_sdk.accept_license = true;
          };
        }
      );
      sharedEnvFor =
        system:
        let
          pkgs = nixpkgsFor.${system};
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "llvm-tools-preview"
              "rust-src"
              "rust-analyzer"
              "clippy"
              "rustfmt"
            ];
            targets = [
              "wasm32-unknown-unknown"
              "aarch64-linux-android"
              "x86_64-linux-android"
              # "armv7-linux-androideabi"
              # "i686-linux-android"
            ];
          };
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
          androidComposition = pkgs.androidenv.composeAndroidPackages {
            platformVersions = [
              "33"
              "34"
              "35"
              "36"
            ]; # Target latest Android
            buildToolsVersions = [
              "34.0.0"
              "35.0.0"
              "36.0.0"
            ];
            includeNDK = true;
            includeEmulator = false; # Clean up unused
            includeSystemImages = false; # Clean up unused
            abiVersions = [
              "arm64-v8a"
              "x86_64"
            ];
          };
          commonNativeBuildInputs = with pkgs; [
            apksigner
            binaryen
            cargo-binutils
            cargo-deny
            cargo-llvm-cov
            cargo-mutants
            chromedriver
            dioxus-cli
            maestro
            pkg-config
            rustToolchain
            selenium-manager
            ungoogled-chromium
          ];
          commonBuildInputs = [
            pkgs.openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
        in
        {
          inherit
            pkgs
            rustToolchain
            rustPlatform
            androidComposition
            commonNativeBuildInputs
            commonBuildInputs
            ;
        };
    in
    {
      devShells = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
        in
        {
          default = env.pkgs.mkShell {
            packages = with env.pkgs; [
              biome
              patchelf
              sass
              scss-lint
              strace
              taplo # TOML LSP
              typescript-language-server # TS LSP
              vscode-langservers-extracted # HTML/CSS/JS(ON)
              yaml-language-server # YAML LSP
            ];
            nativeBuildInputs =
              env.commonNativeBuildInputs
              ++ (with env.pkgs; [
                cargo-ndk
                android-tools
                env.androidComposition.androidsdk
                env.androidComposition.ndk-bundle
                openjdk
              ]);
            buildInputs = env.commonBuildInputs;
            ANDROID_HOME = "${env.androidComposition.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_HOME = "${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle";
            GRADLE_USER_HOME = "$PWD/.gradle";
            LD_LIBRARY_PATH =
              with env.pkgs;
              lib.makeLibraryPath [
                stdenv.cc.cc.lib
                zlib
              ];
            shellHook = ''
              export SE_CACHE_PATH="$PWD/.selenium"
              # Patch aapt2 if in gradle cache or target dir (Android on Nix)
              find "$GRADLE_USER_HOME/caches" "$PWD/target" -name aapt2 -type f -executable 2>/dev/null | while read -r aapt2; do
                if ! patchelf --print-interpreter "$aapt2" >/dev/null 2>&1 || [[ "$(patchelf --print-interpreter "$aapt2")" == /lib* ]]; then
                  echo "🔧 Patching aapt2 at $aapt2"
                  chmod +x "$aapt2" # Just in case
                  patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$aapt2" || true
                  patchelf --set-rpath "$LD_LIBRARY_PATH" "$aapt2" || true
                fi
              done
              echo "💪 LogOut Dev Environment Ready"
              echo "- Rust $(rustc --version)"
              echo "- Dioxus CLI $(dx --version)"
              echo "- Android SDK $ANDROID_HOME"
              echo "- Android NDK $ANDROID_NDK_HOME"
            '';
          };
        }
      );
      packages = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
          # FOD (Fixed-Output Derivation) for Gradle/Maven dependencies.
          # Has network access to download all dependencies needed for the Android build.
          # The outputHash must be updated when Gradle dependencies change (e.g. Dioxus update).
          # To compute the correct hash, run: nix build .#android
          # Nix will report the expected vs actual hash on first build.
          androidGradleDeps = env.pkgs.stdenv.mkDerivation {
            name = "log-out-gradle-deps";
            src = self;
            nativeBuildInputs =
              env.commonNativeBuildInputs
              ++ (with env.pkgs; [
                cargo-ndk
                android-tools
                env.androidComposition.androidsdk
                env.androidComposition.ndk-bundle
                openjdk
                patchelf
              ]);
            buildInputs = env.commonBuildInputs;
            ANDROID_HOME = "${env.androidComposition.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_HOME = "${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle";
            LD_LIBRARY_PATH =
              with env.pkgs;
              lib.makeLibraryPath [
                stdenv.cc.cc.lib
                zlib
              ];
            buildPhase = ''
              export HOME=$TMPDIR/home
              export XDG_DATA_HOME=$HOME/.local/share
              export GRADLE_USER_HOME=$TMPDIR/gradle-home
              export CARGO_HOME=$TMPDIR/cargo-home
              mkdir -p $HOME $GRADLE_USER_HOME $CARGO_HOME
              export CARGO_TARGET_DIR=$PWD/target
              # Only target aarch64 for the dependency fetch build
              sed -i 's/targets = .*/targets = ["aarch64-linux-android"]/' Dioxus.toml
              # Pre-create the directory wry expects to avoid canonicalization failure
              export WRY_ANDROID_KOTLIN_FILES_OUT_DIR=$CARGO_TARGET_DIR/dx/log-out/release/android/app/app/src/main/kotlin/dev/dioxus/main
              mkdir -p $WRY_ANDROID_KOTLIN_FILES_OUT_DIR
              # Build to generate the Gradle project and download all dependencies
              dx build --android --release --target aarch64-linux-android --verbose
              # Patch aapt2 and ensure all Gradle dependencies are fully resolved
              APP_DIR=$(find target/dx -path "*/release/android/app" -type d 2>/dev/null | head -n 1)
              if [ -n "$APP_DIR" ] && [ -d "$APP_DIR" ]; then
                find "$GRADLE_USER_HOME/caches" -name aapt2 -type f -executable 2>/dev/null | while read -r aapt2; do
                  patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$aapt2" 2>/dev/null || true
                  patchelf --set-rpath "$LD_LIBRARY_PATH" "$aapt2" 2>/dev/null || true
                done
                pushd "$APP_DIR"
                # Supplementary Gradle invocations to ensure full dependency tree is cached
                ./gradlew --no-daemon dependencies || true
                ./gradlew --no-daemon assembleRelease || true
                popd
              fi
            '';
            installPhase = ''
              mkdir -p $out
              if [ -d "$TMPDIR/gradle-home" ]; then
                cp -r $TMPDIR/gradle-home/* $out/
              fi
              # Remove non-deterministic files for stable output hash
              find $out -name '*.lock' -delete 2>/dev/null || true
              find $out -name 'gc.properties' -delete 2>/dev/null || true
              find $out -name '*.log' -delete 2>/dev/null || true
              find $out -type d -name 'executionHistory' -exec rm -rf {} + 2>/dev/null || true
              find $out -type d -name 'buildOutputCleanup' -exec rm -rf {} + 2>/dev/null || true
              find $out -name 'file-access.properties' -delete 2>/dev/null || true
            '';
            outputHashAlgo = "sha256";
            outputHashMode = "recursive";
            outputHash = env.pkgs.lib.fakeHash;
          };
          mkWebPackage =
            {
              basePath ? "/",
            }:
            env.rustPlatform.buildRustPackage {
              pname = "log-out-web";
              version = "0.1.0";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
              nativeBuildInputs = env.commonNativeBuildInputs;
              buildInputs = env.commonBuildInputs;
              buildPhase = ''
                export HOME=$TMPDIR/fake-home
                export XDG_DATA_HOME=$HOME/.local/share
                mkdir -p $HOME
                export CARGO_TARGET_DIR=target
                dx build --web --release --base-path ${basePath}
              '';
              installPhase = ''
                mkdir -p $out
                cp -r target/dx/log-out/release/web/public/* $out/
              '';
              doCheck = true;
              preCheck = ''
                export HOME=$TMPDIR/fake-home
                export XDG_DATA_HOME=$HOME/.local/share
              '';
            };
        in
        {
          web = mkWebPackage { };
          pages = mkWebPackage { basePath = "LogOut"; };
          android = env.rustPlatform.buildRustPackage {
            pname = "log-out-android";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs =
              env.commonNativeBuildInputs
              ++ (with env.pkgs; [
                cargo-ndk
                android-tools
                env.androidComposition.androidsdk
                env.androidComposition.ndk-bundle
                openjdk
                strace
                patchelf
              ]);
            buildInputs = env.commonBuildInputs;
            postPatch = ''
              # Ensure the targets list is clean and only contains aarch64
              sed -i 's/targets = .*/targets = ["aarch64-linux-android"]/' Dioxus.toml
            '';
            ANDROID_HOME = "${env.androidComposition.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_HOME = "${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle";
            buildPhase = ''
              export HOME=$TMPDIR/fake-home
              export XDG_DATA_HOME=$HOME/.local/share
              export GRADLE_USER_HOME=$HOME/.gradle
              mkdir -p $HOME $GRADLE_USER_HOME
              # Use pre-downloaded Gradle dependencies from FOD
              cp -r ${androidGradleDeps}/* $GRADLE_USER_HOME/
              chmod -R u+w $GRADLE_USER_HOME
              # Set Gradle to offline mode (all deps are in the FOD cache)
              echo "org.gradle.offline=true" >> $GRADLE_USER_HOME/gradle.properties
              # Use absolute paths to avoid canonicalization issues in Nix sandbox
              export CARGO_TARGET_DIR=$PWD/target
              # Pre-create the directory wry expects to avoid canonicalization failure
              export WRY_ANDROID_KOTLIN_FILES_OUT_DIR=$CARGO_TARGET_DIR/dx/log-out/release/android/app/app/src/main/kotlin/dev/dioxus/main
              mkdir -p $WRY_ANDROID_KOTLIN_FILES_OUT_DIR
              # Build Android (Gradle uses offline cached dependencies from FOD)
              dx build --android --release --target aarch64-linux-android --verbose
              # Inject icons as per scripts/android-icon.sh logic
              APP_PROJECT_DIR=$(find target/dx -name "android" -type d | grep "release/android" | head -n 1)/app
              if [ -d "$APP_PROJECT_DIR" ]; then
                echo "🎨 Injecting Android icons into $APP_PROJECT_DIR"
                cp -r android/res "$APP_PROJECT_DIR/app/src/main/"
                pushd "$APP_PROJECT_DIR"
                # Patch aapt2 if it was downloaded/extracted by Gradle
                find "$GRADLE_USER_HOME/caches" -name aapt2 -type f -executable 2>/dev/null | while read -r aapt2; do
                  if ! patchelf --print-interpreter "$aapt2" >/dev/null 2>&1 || [[ "$(patchelf --print-interpreter "$aapt2")" == /lib* ]]; then
                    echo "🔧 Patching aapt2 at $aapt2"
                    chmod +x "$aapt2"
                    patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$aapt2" || true
                    patchelf --set-rpath "${
                      env.pkgs.lib.makeLibraryPath [
                        env.pkgs.stdenv.cc.cc.lib
                        env.pkgs.zlib
                      ]
                    }" "$aapt2" || true
                  fi
                done
                ./gradlew --offline assembleRelease
                popd
              fi
            '';
            installPhase = ''
              mkdir -p $out/bin
              find . -type f -name "*.apk" -exec cp {} $out/ \;
              echo "APK successfully copied to $out"
            '';
            doCheck = true;
            preCheck = ''
              export HOME=$TMPDIR/fake-home
              export XDG_DATA_HOME=$HOME/.local/share
            '';
          };
          default = env.pkgs.symlinkJoin {
            name = "log-out-all";
            paths = [
              self.packages.${system}.pages
              self.packages.${system}.android
            ];
          };
        }
      );
      checks = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
        in
        {
          fmt = env.pkgs.runCommand "cargo-fmt-check" { nativeBuildInputs = [ env.rustToolchain ]; } ''
            cd ${self}
            cargo fmt --all -- --check
            touch $out
          '';
          clippy = env.rustPlatform.buildRustPackage {
            pname = "log-out-clippy";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = env.commonNativeBuildInputs;
            buildInputs = env.commonBuildInputs;
            buildPhase = ''
              export HOME=$TMPDIR
              cargo clippy --all-targets -- -D warnings -W clippy::all -W clippy::pedantic
            '';
            installPhase = "touch $out";
            doCheck = false;
          };
          coverage = env.rustPlatform.buildRustPackage {
            pname = "log-out-coverage";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = env.commonNativeBuildInputs ++ [ env.pkgs.lcov ];
            buildInputs = env.commonBuildInputs;
            buildPhase = ''
              export HOME=$TMPDIR
              mkdir -p $out
              cargo llvm-cov --bin log-out \
                --ignore-filename-regex "src/components/" \
                --text > $out/coverage.txt
              cargo llvm-cov --bin log-out \
                --ignore-filename-regex "src/components/" \
                --json > $out/coverage.json
            '';
            installPhase = "true";
            doCheck = false;
          };
        }
      );
    };
}
