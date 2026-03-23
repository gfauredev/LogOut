{
  description = "LogOut full development system & tooling";
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
    crane.url = "github:ipetkov/crane";
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
          assetFilter =
            path: type:
            builtins.match ".*(/public/.*|/assets/.*|Dioxus\\.toml|index\\.html|logo\\.png|schema2\\.json)$" path
            != null;
          sourceFilter = path: type: (assetFilter path type) || (craneLib.filterCargoSources path type);
          filteredSrc = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = sourceFilter;
          };
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
          cargoArtifactsHost = craneLib.buildDepsOnly {
            src = filteredSrc;
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
            doCheck = false;
          };
          cargoArtifactsWeb = craneLib.buildDepsOnly {
            src = filteredSrc;
            cargoExtraArgs = "--target wasm32-unknown-unknown";
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
            doCheck = false;
          };
          androidComposition = pkgs.androidenv.composeAndroidPackages {
            platformVersions = [
              "33"
              "34"
              "35"
              "36"
            ];
            buildToolsVersions = [
              "34.0.0"
              "35.0.0"
              "36.0.0"
            ];
            includeNDK = true;
            includeEmulator = false;
            includeSystemImages = false;
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
            filteredSrc
            cargoArtifactsHost
            cargoArtifactsWeb
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
              cargoArtifacts = env.cargoArtifactsWeb;
              src = env.filteredSrc;
              pname = "logout-web";
              version = env.projectVersion;
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
                  if basePath == "/" then "* $out/" else " $out/${basePath}"
                }
              '';
              doCheck = false;
            };
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
                unset ANDROID_SDK_ROOT # Conflicts with Home in GitHub Runners
                export ANDROID_HOME="${env.androidComposition.androidsdk}/libexec/android-sdk"
                export ANDROID_NDK_HOME="${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle"
                export GRADLE_USER_HOME="''${GRADLE_USER_HOME:-$PWD/.gradle}" 
                export HOME="''${HOME:-$TMPDIR}"
                echo "🤖 LogOut Build Environment Ready"
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
          web = mkWebPackage { basePath = "LogOut"; }; # Needed for GitHub Pages
          server = env.pkgs.writeShellApplication {
            name = "logout-serve";
            runtimeInputs = [ env.pkgs.python3 ];
            text = ''
              python3 -m http.server -d "${self.packages.${system}.web}" 8080
            '';
          };
          webE2eTester = env.pkgs.writeShellApplication {
            name = "logout-web-e2e-tester";
            runtimeInputs = with env.pkgs; [
              curl
              chromedriver
              maestro
              chromiumWrapper
            ];
            text = ''
              export SE_CHROME_PATH="${chromiumWrapper}/bin/google-chrome"
              SERVER_PID=""
              cleanup() {
                if [ -n "$SERVER_PID" ]; then
                  kill "$SERVER_PID" 2>/dev/null || true
                fi
              }
              trap cleanup EXIT
              ${self.apps.${system}.web.program} &
              SERVER_PID=$!
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
              self.packages.${system}.server
              self.packages.${system}.webE2eTester
              self.packages.${system}.androidBuilder
              self.packages.${system}.androidE2eTester
            ];
          };
        }
      );
      apps = forAllSystems (system: rec {
        web = {
          type = "app";
          program = "${self.packages.${system}.server}/bin/logout-web";
          meta.description = "Serve PWA";
        };
        webTest = {
          type = "app";
          program = "${self.packages.${system}.webE2eTester}/bin/logout-web-e2e-tester";
          meta.description = "Run Maestro E2E tests against PWA";
        };
        androidBuild = {
          type = "app";
          program = "${self.packages.${system}.androidBuilder}/bin/logout-android-builder";
          meta.description = "Build and sign Android APK";
        };
        androidTest = {
          type = "app";
          program = "${self.packages.${system}.androidE2eTester}/bin/logout-android-e2e-tester";
          meta.description = "Run Maestro E2E tests against Android App";
        };
        default = web;
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
              unset ANDROID_SDK_ROOT # Conflicts with Home in GitHub Runners
              export SE_CACHE_PATH="$PWD/.selenium"
              find "$GRADLE_USER_HOME/caches" "$PWD/target" -name aapt2 -type f -executable 2>/dev/null | while read -r aapt2; do
                if ! patchelf --print-interpreter "$aapt2" >/dev/null 2>&1 || [[ "$(patchelf --print-interpreter "$aapt2")" == /lib* ]]; then
                  echo "🔧 Patching aapt2 at $aapt2"
                  chmod +x "$aapt2" 
                  patchelf --set-interpreter "$(cat $NIX_CC/nix-support/dynamic-linker)" "$aapt2" || true
                  patchelf --set-rpath "$LD_LIBRARY_PATH" "$aapt2" || true
                fi
              done
              echo "✅ LogOut Dev Environment Ready"
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
          fmt =
            env.pkgs.runCommand "cargo-fmt-check"
              {
                nativeBuildInputs = env.commonNativeBuildInputs;
              }
              ''
                cd ${self}
                dx fmt --check >> $out
                echo >> $out
                cargo fmt --all -- --check >> $out
              '';
          build = self.packages.${system}.default;
          clippy = env.craneLib.cargoClippy {
            cargoArtifacts = env.cargoArtifactsHost;
            src = env.filteredSrc;
            pname = "logout"; # -clippy auto added by craneLib.cargoClippy
            version = env.projectVersion;
            nativeBuildInputs = env.commonNativeBuildInputs;
            buildInputs = env.commonBuildInputs;
            cargoClippyExtraArgs = "--all-targets -- -D warnings -W clippy::all -W clippy::pedantic";
          };
          coverage = env.craneLib.buildPackage {
            cargoArtifacts = env.cargoArtifactsHost;
            src = env.filteredSrc;
            pname = "logout-coverage";
            version = env.projectVersion;
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
