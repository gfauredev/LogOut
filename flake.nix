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
          androidNativeBuildInputs = with pkgs; [
            aapt
            apksigner
            android-tools
            androidComposition.androidsdk
            androidComposition.ndk-bundle
            cargo-ndk
            openjdk
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
            env.rustPlatform.buildRustPackage {
              pname = "logout-web";
              version = "0.2.2";
              src = self;
              cargoLock.lockFile = ./Cargo.lock;
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
              preCheck = ''
                export HOME=$TMPDIR/fake-home
                export XDG_DATA_HOME=$HOME/.local/share
              '';
            };
        in
        {
          web = mkWebPackage { };
          pages = mkWebPackage { basePath = "LogOut"; };
          default = env.pkgs.symlinkJoin {
            name = "logout-all";
            paths = [
              self.packages.${system}.web
              self.packages.${system}.pages
            ];
          };
        }
      );
      apps = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
          pkgs = env.pkgs;
          # --no-sandbox required on non-NixOS CI runners where SUID sandbox
          # binary is absent, named so selenium-manager finds via PATH
          chromiumNoSandbox = pkgs.writeShellScript "google-chrome" ''
            exec "${pkgs.ungoogled-chromium}/bin/chromium" --no-sandbox "$@"
          '';
          # Serve the `pages` package at http://localhost:8080/LogOut/
          pagesScript = pkgs.writeShellApplication {
            name = "logout-pages";
            runtimeInputs = [ pkgs.python3 ];
            text = ''
              python3 -m http.server -d "${self.packages.${system}.pages}" 8080
            '';
          };
          # Start the pages server, then run Maestro web E2E tests against it
          e2eWebScript = pkgs.writeShellApplication {
            name = "logout-e2e-web";
            runtimeInputs = with pkgs; [
              chromedriver
              maestro
              chromiumNoSandbox
            ];
            text = ''
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
              sleep 2
              maestro test --headless "${self}/maestro/web"
            '';
          };
        in
        {
          pages = {
            type = "app";
            program = "${pagesScript}/bin/logout-pages";
          };
          e2e-web = {
            type = "app";
            program = "${e2eWebScript}/bin/logout-e2e-web";
          };
          default = {
            type = "app";
            program = "${pagesScript}/bin/logout-pages";
          };
        }
      );
      devShells = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
        in
        {
          default = env.pkgs.mkShell {
            packages = with env.pkgs; [
              # biome sass scss-lint
              python3
              strace
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
            cargo fmt --all -- --check
            touch $out
          '';
          clippy = env.rustPlatform.buildRustPackage {
            pname = "logout-clippy";
            version = "0.2.2";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = env.commonNativeBuildInputs;
            buildInputs = env.commonBuildInputs;
            buildPhase = ''
              export HOME=$TMPDIR
              cargo clippy --all-targets -- -D warnings -W clippy::all -W clippy::pedantic
            '';
            installPhase = "touch $out";
          };
          coverage = env.rustPlatform.buildRustPackage {
            pname = "logout-coverage";
            version = "0.2.2";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = env.commonNativeBuildInputs ++ [ env.pkgs.lcov ];
            buildInputs = env.commonBuildInputs;
            buildPhase = ''
              export HOME=$TMPDIR
              mkdir -p $out
              cargo llvm-cov --bin log-out \
                --ignore-filename-regex "src/components/" \
                --html --output-dir $out # /html auto added
              cargo llvm-cov --bin log-out \
                --ignore-filename-regex "src/components/" \
                --json > $out/coverage.json
            '';
            installPhase = "true";
          };
        }
      );
    };
}
