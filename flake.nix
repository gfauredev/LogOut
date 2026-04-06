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
    agents-jail.url = "github:gfauredev/nix-agents-jail";
  };
  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      agents-jail,
      crane,
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-linux" # "aarch64-linux" # "aarch64-darwin"
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
          # Assets used by every target (web, server, Android).
          assetFilter =
            path: type:
            builtins.match ".*(/public/.*|/assets/.*|/icon/.*|Dioxus\\.toml|index\\.html|.*\\.png)$" path
            != null;
          # Extra assets only needed by the server build: the bundled example
          # exercise database served alongside the binary.
          serverAssetFilter =
            path: type:
            (assetFilter path type)
            || (builtins.match ".*/database\\.example(/.*)?$" path != null);
          sourceFilter = path: type: (assetFilter path type) || (craneLib.filterCargoSources path type);
          # Source tree used by web / Android builds.
          filteredSrc = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = sourceFilter;
          };
          # Source tree for the server build: includes the example exercise database.
          filteredSrcServer = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            filter = path: type: (serverAssetFilter path type) || (craneLib.filterCargoSources path type);
          };
          wasm-bindgen-cli = rustPlatform.buildRustPackage rec {
            pname = "wasm-bindgen-cli";
            version = "0.2.117";
            src = pkgs.fetchCrate {
              inherit pname version;
              hash = "sha256-vtDQXL8FSgdutqXG7/rBUWgrYCtzdmeVQQkWkjasvZU=";
            };
            cargoHash = "sha256-eKe7uwneUYxejSbG/1hKqg6bSmtL0KQ9ojlazeqTi88=";
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [
              pkgs.openssl
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Security
              pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
            ];
          };
          commonNativeBuildInputs = with pkgs; [
            binaryen
            cargo-binutils
            cargo-deny
            cargo-llvm-cov
            cargo-nextest
            cargo-mutants
            clang
            dioxus-cli
            patchelf
            pkg-config
            rustToolchain
            unzip
          ];
          webNativeBuildInputs = [
            # pkgs.esbuild
            wasm-bindgen-cli
          ];
          commonBuildInputs = [
            pkgs.openssl
          ]
          ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];
          webTestInputs = with pkgs; [
            curl
            chromedriver
            maestro
            selenium-manager
            ungoogled-chromium
          ];
          cargoArtifactsHost = craneLib.buildDepsOnly {
            src = filteredSrc;
            nativeBuildInputs = commonNativeBuildInputs;
            buildInputs = commonBuildInputs;
            doCheck = false;
          };
          cargoArtifactsServer = craneLib.buildDepsOnly {
            src = filteredSrc;
            cargoExtraArgs = "--features server-platform";
            nativeBuildInputs = commonNativeBuildInputs ++ webNativeBuildInputs;
            buildInputs = commonBuildInputs;
            doCheck = false;
          };
          cargoArtifactsWeb = craneLib.buildDepsOnly {
            src = filteredSrc;
            cargoExtraArgs = "--target wasm32-unknown-unknown";
            nativeBuildInputs = commonNativeBuildInputs ++ webNativeBuildInputs;
            buildInputs = commonBuildInputs;
            doCheck = false;
          };
          androidComposition = pkgs.androidenv.composeAndroidPackages {
            platformVersions = [ "36" ];
            buildToolsVersions = [ "34.0.0" ];
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
          projectVersion = "0.4.3";
          inherit
            pkgs
            rustToolchain
            rustPlatform
            craneLib
            filteredSrc
            filteredSrcServer
            cargoArtifactsHost
            cargoArtifactsServer
            cargoArtifactsWeb
            androidComposition
            commonNativeBuildInputs
            webNativeBuildInputs
            androidNativeBuildInputs
            webTestInputs
            commonBuildInputs
            ;
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
          mkLogOut =
            {
              basePath ? "LogOut", # Needed for GitHub Pages
              platform ? "web",
              cargoArtifacts ? (if platform == "server" then env.cargoArtifactsServer else env.cargoArtifactsWeb),
            }:
            let
              target =
                if platform == "web" then
                  "target/dx/log-out/release/${platform}/public/*"
                else if platform == "server" then
                  "target/dx/log-out/release/web/*" # Server bin plus assets
                else
                  "target/dx/log-out/release/${platform}/*";
              out =
                if platform == "server" then
                  "$out/bin/" # \n
                else
                  "$out/${basePath}"; # WARN May need to deduplicate /
            in
            env.craneLib.buildPackage {
              inherit cargoArtifacts;
              # The server build needs the example exercise database in its source tree
              # so it can be copied into the output alongside the binary.
              src = if platform == "server" then env.filteredSrcServer else env.filteredSrc;
              pname = "logout-${platform}";
              version = env.projectVersion;
              nativeBuildInputs = env.commonNativeBuildInputs ++ env.webNativeBuildInputs;
              buildInputs = env.commonBuildInputs;
              buildPhase = ''
                export HOME=$TMPDIR/fake-home
                export XDG_DATA_HOME=$HOME/.local/share
                mkdir -p $HOME
                export CARGO_TARGET_DIR=target
                dx build --${platform} --release --base-path ${env.pkgs.lib.escapeShellArg basePath}
              '';
              installPhase = ''
                mkdir --parents --verbose ${out}
                cp --recursive --verbose ${target} ${out}
              '' + env.pkgs.lib.optionalString (platform == "server") ''
                # Bundle the example exercise database with the server so it can be
                # used as a self-hosted default (serve from ./database.example/).
                cp --recursive --verbose database.example ${out}
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
              name = "logout-android-build-${env.projectVersion}";
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
                # Stash web-only public assets so they are not bundled into the APK.
                # These files (service worker, web icons, 404 page) are irrelevant on
                # the Android platform and would only inflate the APK size.
                _web_stash=$(mktemp -d)
                for _f in public/sw.js public/404.html public/icon-*.png; do
                  [ -f "$_f" ] && mv "$_f" "$_web_stash/"
                done
                # Ensure web-only assets are always restored, even on failure.
                _restore_web_assets() {
                  mv "$_web_stash"/* public/ 2>/dev/null || true
                  rmdir "$_web_stash" 2>/dev/null || true
                }
                trap _restore_web_assets EXIT
                dx build --android --release --target ${target}
                _restore_web_assets
                trap - EXIT
                "${self}/.script/apk-sign.sh"
              '';
            };
        in
        {
          web = mkLogOut { };
          preWeb = mkLogOut { basePath = "LogOut/preview"; };
          server = mkLogOut { platform = "server"; };
          webE2eTest = env.pkgs.writeShellApplication {
            name = "logout-web-e2e-test-${env.projectVersion}";
            runtimeInputs = env.webTestInputs ++ [
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
              ${self.packages.${system}.server}/bin/server &
              SERVER_PID=$!
              timeout 60 bash -c 'until curl -sf http://localhost:8080/LogOut/ > /dev/null 2>&1; do sleep 1; done'
              maestro test --headless "${self}/maestro/web"
            '';
          };
          androidBuild = mkAndroidBuilder { };
          androidE2eTest = env.pkgs.writeShellApplication {
            name = "logout-android-e2e-test-${env.projectVersion}";
            runtimeInputs = [ env.pkgs.maestro ];
            # TODO Android emulator…
            text = ''
              maestro test --headless "${self}/maestro/android"
            '';
          };
          default = env.pkgs.symlinkJoin {
            name = "logout-all-${env.projectVersion}";
            paths = [
              self.packages.${system}.androidBuild
              # self.packages.${system}.androidE2eTest
              self.packages.${system}.preWeb
              self.packages.${system}.server
              # self.packages.${system}.web
              self.packages.${system}.webE2eTest
            ];
          };
        }
      );
      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.server}/bin/server";
          meta.description = "Serve the LogOut Progressive Web App with Axum Server";
        };
      });
      devShells = forAllSystems (
        system:
        let
          env = sharedEnvFor system;
          devTools = with env.pkgs; [
            # biome python3 sass strace
            cachix # Nix binary cache
            fastlane # Mobile app publishing automation TODO
            kotlin-language-server # Kotlin LSP
            lightningcss # CSS linter & optimizer
            scss-lint # SCSS linter
            taplo # TOML LSP
            typescript-language-server # TypeScript LSP
            vscode-langservers-extracted # HTML/CSS/JS(ON)
            yaml-language-server # YAML LSP
          ];
        in
        {
          default = env.pkgs.mkShell {
            packages = devTools ++ [
              # TODO: Uncomment when sourcehut is back up (jail-nix dependency)
              # (agents-jail.lib.${system}.mkCrush {
              #   extraPkgs =
              #     devTools ++ env.commonNativeBuildInputs ++ env.webNativeBuildInputs ++ env.androidNativeBuildInputs;
              # })
              # (agents-jail.lib.${system}.mkOpencode {
              #   extraPkgs =
              #     devTools ++ env.commonNativeBuildInputs ++ env.webNativeBuildInputs ++ env.androidNativeBuildInputs;
              # })
            ];
            nativeBuildInputs =
              env.commonNativeBuildInputs ++ env.webNativeBuildInputs ++ env.androidNativeBuildInputs;
            buildInputs = env.commonBuildInputs;
            ANDROID_HOME = "${env.androidComposition.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_HOME = "${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle";
            LD_LIBRARY_PATH =
              with env.pkgs;
              lib.makeLibraryPath [
                stdenv.cc.cc.lib
                zlib
              ];
            shellHook = ''
              unset ANDROID_SDK_ROOT # Conflicts with Home in GitHub Runners
              # Use an absolute path so Java subprocesses (Gradle wrapper) resolve it correctly.
              export GRADLE_USER_HOME="$PWD/.gradle"
              export SE_CACHE_PATH="$PWD/.selenium"
              # If the system has a Temurin/Adoptium JDK with a broader CA trust store,
              # point the nix JVM at it so the Gradle wrapper can reach services.gradle.org.
              for _cacerts in \
                /usr/lib/jvm/temurin-21-jdk-amd64/lib/security/cacerts \
                /usr/lib/jvm/temurin-17-jdk-amd64/lib/security/cacerts \
                /usr/lib/jvm/temurin-8-jdk-amd64/jre/lib/security/cacerts \
                /etc/ssl/certs/java/cacerts; do
                if [ -f "$_cacerts" ]; then
                  export JAVA_TOOL_OPTIONS="-Djavax.net.ssl.trustStore=$_cacerts -Djavax.net.ssl.trustStorePassword=changeit"
                  break
                fi
              done
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
          format =
            env.pkgs.runCommand "logout-fmt-${env.projectVersion}"
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
          lint = env.craneLib.cargoClippy {
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
            # FIXME Flaky DB tests sometimes fail depending on their concurrency
            buildPhase = ''
              export HOME=$TMPDIR
              mkdir -p $out
              cargo llvm-cov nextest --bin log-out \
                --ignore-filename-regex "(src/components/|\.cargo/registry/|nix/store)" \
                --html --output-dir $out 2>&1 | tee $out/nextest.log
              cargo llvm-cov report \
                --ignore-filename-regex "(src/components/|\.cargo/registry/|nix/store)" \
                --json > $out/coverage.json
            '';
            installPhase = "true";
            doCheck = false;
          };
          default = env.pkgs.linkFarm "logout-quick-checks" [
            {
              name = "format";
              path = self.checks.${system}.format; # dx fmt + cargo fmt
            }
            {
              name = "coverage";
              path = self.checks.${system}.coverage; # LLVM Cov + Nextest
            }
            {
              name = "lint";
              path = self.checks.${system}.lint; # Clippy
            }
          ];
        }
      );
    };
}
