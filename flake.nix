{
  description = "LogOut dev envs";
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
        "aarch64-linux"
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
              "armv7-linux-androideabi"
              "i686-linux-android"
              "x86_64-linux-android"
            ];
          };
          androidComposition = pkgs.androidenv.composeAndroidPackages {
            platformVersions = [ "35" ]; # Target latest Android
            buildToolsVersions = [ "35.0.0" ];
            includeNDK = true;
            includeEmulator = false; # Clean up unused
            includeSystemImages = false; # Clean up unused
            abiVersions = [
              "x86_64"
              "arm64-v8a" # Add common real device ABI
            ];
          };
          commonNativeBuildInputs = with pkgs; [
            binaryen
            cargo-binutils
            cargo-llvm-cov
            chromedriver
            dioxus-cli
            maestro
            pkg-config
            rustToolchain
            selenium-manager
            ungoogled-chromium
            # xvfb-run
          ];
          commonBuildInputs =
            with pkgs;
            []
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
              # biome
              # bun # JS runtime, bundler, package manager
              sass
              scss-lint
              strace
              taplo # TOML LSP
              # typescript
              # typescript-language-server # TS LSP
              vscode-langservers-extracted # HTML/CSS/JS(ON)
              # yaml-language-server # YAML LSP

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

            shellHook = ''
              export SE_CACHE_PATH="$PWD/.selenium"
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
            pname = "log-workout-web";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = env.commonNativeBuildInputs;
            buildInputs = env.commonBuildInputs;
            buildPhase = ''
              export CARGO_TARGET_DIR=target
              dx build --release --platform web
            '';
            installPhase = ''
              mkdir -p $out
              cp -r target/dx/log-workout/release/web/public/* $out/
            '';
            doCheck = true;
          };
          android = env.pkgs.rustPlatform.buildRustPackage {
            pname = "log-workout-android";
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
              ]);
            buildInputs = env.commonBuildInputs;

            ANDROID_HOME = "${env.androidComposition.androidsdk}/libexec/android-sdk";
            ANDROID_NDK_HOME = "${env.androidComposition.ndk-bundle}/libexec/android-sdk/ndk-bundle";

            buildPhase = ''
              export CARGO_TARGET_DIR=target
              dx build --release --platform android
            '';
            installPhase = ''
              mkdir -p $out/bin
              find . -type f -name "*.apk" -exec cp {} $out/ \;
              echo "APK successfully copied to $out"
            '';
            doCheck = true;
          };
          default = env.pkgs.symlinkJoin {
            name = "log-workout-all";
            paths = [
              self.packages.${system}.web
              self.packages.${system}.android
            ];
          };
        }
      );
    };
}
