{
  description = "LogOut dev envs";
  nixConfig = {
    extra-substituters = [
      "https://cache.garnix.io"
      "https://gfauredev.cachix.org"
    ];
    extra-trusted-public-keys = [
      "cache.garnix.io:CTFPyKSLcx5RMJKfLo5EEPUObbA78b0YQ2DTCJXqr9g="
      "gfauredev.cachix.org-1:mGOZ5I0bDVatgwLhbuTasIiWpVjgCyMFjfIZEPjmQfM="
    ];
  };
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      crane,
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux" # "aarch64-linux"
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
            ];
          };
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };
          craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
          commonNativeBuildInputs = with pkgs; [
            binaryen
            cargo-binutils
            cargo-deny
            cargo-llvm-cov
            cargo-mutants
            chromedriver
            dioxus-cli
            wasm-bindgen-cli_0_2_114
            maestro
            patchelf
            pkg-config
            rustToolchain
            selenium-manager
            ungoogled-chromium
            unzip
          ];
          commonBuildInputs = [
            pkgs.openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          cargoArtifacts = craneLib.buildDepsOnly {
            src = craneLib.cleanCargoSource (craneLib.path ./.);
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
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
          androidNativeBuildInputs = with pkgs; [
            aapt
            apksigner
            android-tools
            androidComposition.androidsdk
            androidComposition.ndk-bundle
            cargo-ndk
            openjdk
          ];
        in
        {
          projectVersion = "0.2.4";
          inherit
            pkgs
            rustToolchain
            rustPlatform
            craneLib
            cargoArtifacts
            androidComposition
            commonNativeBuildInputs
            androidNativeBuildInputs
            commonBuildInputs
            ;
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
          mkWebPackage =
            {
              basePath ? "/",
            }:
            env.craneLib.buildPackage {
              inherit (env) cargoArtifacts;
              pname = "logout-web";
              version = env.projectVersion;
              src = env.craneLib.path ./.;
              nativeBuildInputs = env.commonNativeBuildInputs;
              buildInputs = env.commonBuildInputs;
              buildPhase = ''
                export HOME=$TMPDIR/fake-home
                export XDG_DATA_HOME=$HOME/.local/share
                mkdir -p $HOME
                export CARGO_TARGET_DIR=target
                dx build --web --release --base-path ${env.pkgs.lib.escapeShellArg basePath}
              '';
              installPhase = ''
                mkdir -p $out
                cp -r target/dx/log-out/release/web/public/${
                  # Let me break a line
                  if basePath == "/" then "* $out/" else " $out/${basePath}"
                }
              '';
              doCheck = false;
            };
          # --no-sandbox on non-NixOS CI runners where SUID sandbox
          # binary is absent, named so chromedriver finds via PATH
          chromiumWrapper = env.pkgs.writeShellScriptBin "google-chrome" ''
            exec "${env.pkgs.ungoogled-chromium}/bin/chromium" --no-sandbox "$@"
          '';
          mkAndroidBuilder =
            {
              target ? "aarch64-linux-android",
            }:
            env.pkgs.writeShellApplication {
              name = "logout-android-builder";
              runtimeInputs = env.commonNativeBuildInputs ++ env.androidNativeBuildInputs;
              # LD_LIBRARY_PATH = with env.pkgs; lib.makeLibraryPath [ stdenv.cc.cc.lib zlib ];
              text = ''
                unset ANDROID_SDK_ROOT # Set in GitHub Runners conflict with Home
                export ANDROID_HOME="${env.androidComposition.androidsdk}/libexec/android-sdk"
                export ANDROID_NDK_HOME="${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle"
                export GRADLE_USER_HOME="''${GRADLE_USER_HOME:-$PWD/.gradle}" 
                export HOME="''${HOME:-$TMPDIR}"
                # Patch aapt2 if in gradle cache or target dir (Android on Nix)
                # find "$GRADLE_USER_HOME/caches" "$PWD/target" -name aapt2 -type f -executable 2>/dev/null | while read -r aapt2; do
                #   if ! patchelf --print-interpreter "$aapt2" >/dev/null 2>&1 || [[ "$(patchelf --print-interpreter "$aapt2")" == /lib* ]]; then
                #     echo "🔧 Patching aapt2 at $aapt2"
                #     chmod +x "$aapt2" # Just in case
                #     patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$aapt2" || true
                #     patchelf --set-rpath "$LD_LIBRARY_PATH" "$aapt2" || true
                #   fi
                # done
                echo "💪 LogOut Build Environment Ready"
                echo "- Rust $(rustc --version)"
                echo "- Dioxus CLI $(dx --version)"
                echo "- Android SDK $ANDROID_HOME"
                echo "- Android NDK $ANDROID_NDK_HOME"
                dx build --android --release --target ${target}
                "${self}/.script/android-sign.sh"
              '';
            };
        in
        {
          web = mkWebPackage { };
          pages = mkWebPackage { basePath = "LogOut"; };
          pagesServer = env.pkgs.writeShellApplication {
            name = "logout-pages";
            runtimeInputs = [ env.pkgs.python3 ];
            text = ''
              python3 -m http.server -d "${self.packages.${system}.pages}" 8080
            '';
          };
          pagesE2eTester = env.pkgs.writeShellApplication {
            name = "logout-pages-e2e-tester";
            runtimeInputs = with env.pkgs; [
              curl
              chromedriver
              maestro
              chromiumWrapper
            ];
            text = ''
              # Tell Selenium Manager to use the nixpkgs Chromium wrapper instead of
              # detecting the system Chrome via hardcoded paths (e.g. /usr/bin/google-chrome)
              export SE_CHROME_PATH="${chromiumWrapper}/bin/google-chrome"
              SERVER_PID=""
              cleanup() {
                if [ -n "$SERVER_PID" ]; then
                  kill "$SERVER_PID" 2>/dev/null || true
                fi
              }
              trap cleanup EXIT
              # Serve the pages package at http://localhost:8080/LogOut/
              ${self.apps.${system}.pages.program} &
              SERVER_PID=$!
              # Wait until the server is ready (max 60 seconds)
              timeout 60 bash -c 'until curl -sf http://localhost:8080/LogOut/ > /dev/null 2>&1; do sleep 1; done'
              maestro test --headless "${self}/maestro/web"
            '';
          };
          androidBuilder = mkAndroidBuilder { };
          androidE2eTester = env.pkgs.writeShellApplication {
            name = "logout-android-e2e-tester";
            runtimeInputs = [ env.pkgs.maestro ];
            text = ''
              maestro test --headless "${self}/maestro/android"
            '';
          };
          sandbox =
            let
              pkgs = env.pkgs;
              devShellExecutable = pkgs.writeShellScriptBin "logout-devshell" ''
                exec ${pkgs.nix}/bin/nix develop "path:$PWD" "$@"
              '';
            in
            pkgs.dockerTools.buildLayeredImage {
              name = "logout-sandbox";
              tag = "latest";
              contents = [
                pkgs.nix
                devShellExecutable
                pkgs.cacert
              ];
              config = {
                Cmd = [ "${devShellExecutable}/bin/logout-devshell" ];
                Env = [
                  "PATH=/bin"
                  "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                  "NIX_CONFIG=experimental-features = nix-command flakes"
                  "NIX_PAGER=cat"
                  "SHELL=${pkgs.bashInteractive}/bin/bash"
                ];
              };
            };
          default = env.pkgs.symlinkJoin {
            name = "logout-all";
            paths = [
              self.packages.${system}.web
              self.packages.${system}.pages
            ];
          };
        }
      );
      apps = forAllSystems (system: rec {
        pages = {
          type = "app";
          program = "${self.packages.${system}.pagesServer}/bin/logout-pages";
          meta.description = "Serve PWA";
        };
        pagesE2eTest = {
          type = "app";
          program = "${self.packages.${system}.pagesE2eTester}/bin/logout-pages-e2e-tester";
          meta.description = "Run Maestro E2E tests against PWA";
        };
        androidBuild = {
          type = "app";
          program = "${self.packages.${system}.androidBuilder}/bin/logout-android-builder";
          meta.description = "Build and sign Android APK";
        };
        androidE2eTest = {
          type = "app";
          program = "${self.packages.${system}.androidE2eTester}/bin/logout-android-e2e-tester";
          meta.description = "Run Maestro E2E tests against Android App";
        };
        default = pages;
      });
      devShells = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
        in
        {
          default = env.pkgs.mkShell {
            packages = with env.pkgs; [
              # biome sass scss-lint python3 strace
              cachix
              taplo # TOML LSP
              typescript-language-server # TS LSP
              vscode-langservers-extracted # HTML/CSS/JS(ON)
              yaml-language-server # YAML LSP
            ];
            nativeBuildInputs = env.commonNativeBuildInputs ++ env.androidNativeBuildInputs;
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
              unset ANDROID_SDK_ROOT # Set in GitHub Runners conflict with Home
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
      checks = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
        in
        {
          fmt = env.pkgs.runCommand "cargo-fmt-check" { nativeBuildInputs = [ env.rustToolchain ]; } ''
            cd ${self}
            dx fmt --check >> $out
            echo >> $out
            cargo fmt --all -- --check >> $out
          '';
          clippy = env.craneLib.cargoClippy {
            inherit (env) cargoArtifacts;
            pname = "logout";
            version = env.projectVersion;
            src = env.craneLib.path ./.;
            nativeBuildInputs = env.commonNativeBuildInputs;
            buildInputs = env.commonBuildInputs;
            cargoClippyExtraArgs = "--all-targets -- -D warnings -W clippy::all -W clippy::pedantic";
          };
          coverage = env.craneLib.buildPackage {
            inherit (env) cargoArtifacts;
            pname = "logout-coverage";
            version = env.projectVersion;
            src = env.craneLib.path ./.;
            nativeBuildInputs = env.commonNativeBuildInputs ++ [ env.pkgs.lcov ];
            buildInputs = env.commonBuildInputs;
            buildPhase = ''
              export HOME=$TMPDIR
              mkdir -p $out
              cargo llvm-cov --bin log-out \
                --ignore-filename-regex "(src/components/|\.cargo/registry/|/rustc/)" \
                --html --output-dir $out # /html auto added
              cargo llvm-cov --bin log-out \
                --ignore-filename-regex "(src/components/|\.cargo/registry/|/rustc/)" \
                --json > $out/coverage.json
            '';
            installPhase = "true";
            doCheck = false;
          };
        }
      );
    };
}
