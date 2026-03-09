{
  description = "LogOut dev envs";
  # nixConfig = {
  #   extra-substituters = [ "https://cache.garnix.io" ];
  #   extra-trusted-public-keys = [ "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g=" ];
  # };
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
            ];
            targets = [
              "wasm32-unknown-unknown"
              "aarch64-linux-android"
              "x86_64-linux-android"
              # "armv7-linux-androideabi"
              # "i686-linux-android"
            ];
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
          commonBuildInputs =
            [ ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
        in
        {
          inherit
            pkgs
            rustToolchain
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
        in
        {
          web = env.pkgs.rustPlatform.buildRustPackage {
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
              dx build --web --release
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
          android = env.pkgs.rustPlatform.buildRustPackage {
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
              mkdir -p $HOME
              # Use absolute paths to avoid canonicalization issues in Nix sandbox
              export CARGO_TARGET_DIR=$PWD/target
              # Pre-create the directory wry expects to avoid canonicalization failure
              # Dioxus CLI uses the binary name 'log-out' for this path.
              export WRY_ANDROID_KOTLIN_FILES_OUT_DIR=$CARGO_TARGET_DIR/dx/log-out/release/android/app/app/src/main/kotlin/dev/dioxus/main
              mkdir -p $WRY_ANDROID_KOTLIN_FILES_OUT_DIR
              dx build --android --release --target aarch64-linux-android --verbose
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
              self.packages.${system}.web
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
          clippy =
            env.pkgs.runCommand "cargo-clippy-check" { nativeBuildInputs = env.commonNativeBuildInputs; }
              ''
                cd ${self}
                export HOME=$TMPDIR
                cargo clippy --all-targets -- -D warnings -W clippy::all -W clippy::pedantic
                touch $out
              '';
          coverage =
            env.pkgs.runCommand "cargo-coverage"
              {
                nativeBuildInputs = env.commonNativeBuildInputs ++ [ env.pkgs.lcov ];
              }
              ''
                cd ${self}
                export HOME=$TMPDIR
                cp -r . /tmp/src # Writable directory for cargo
                cd /tmp/src
                chmod -R +w .
                cargo llvm-cov --bin log-out \
                  --ignore-filename-regex "src/components/" \
                  --fail-under-functions 90 \
                  --fail-under-lines 80 \
                  --show-missing-lines
                touch $out
              '';
        }
      );
    };
}
