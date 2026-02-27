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
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
            targets = [
              "wasm32-unknown-unknown"
              "aarch64-linux-android"
              "armv7-linux-androideabi"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            # TODO Directly use packages’ inputs, keep DRY, keep SSOT
            packages = with pkgs; [
              strace
            ];
            nativeBuildInputs = with pkgs; [
              pkg-config
              dioxus-cli
              rustToolchain
              cargo-ndk
              android-tools
              androidenv.androidPkgs.ndk-bundle
              openjdk
            ];
            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
            OPENSSL_DIR = "${pkgs.openssl.dev}";
            OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
            shellHook = ''
              echo "💪 LogOut Dev Environment Ready"
              echo "- Rust $(rustc --version)"
              echo "- Dioxus CLI $(dx --version)"
            '';
          };
        }
      );
      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};
          rustToolchainWeb = pkgs.rust-bin.stable.latest.default.override {
            targets = [ "wasm32-unknown-unknown" ];
          };
          rustToolchainAndroid = pkgs.rust-bin.stable.latest.default.override {
            targets = [
              "aarch64-linux-android"
              "armv7-linux-androideabi"
            ];
          };
        in
        {
          web = pkgs.rustPlatform.buildRustPackage {
            pname = "log-workout-web";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [
              pkg-config
              dioxus-cli
              rustToolchainWeb
            ];
            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
            buildPhase = ''
              export CARGO_TARGET_DIR=target
              dx build --release --platform web
            '';
            installPhase = ''
              mkdir -p $out
              cp -r dist/* $out/
            '';
            doCheck = false; # TODO
          };

          android = pkgs.rustPlatform.buildRustPackage {
            pname = "log-workout-android";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = with pkgs; [
              pkg-config
              dioxus-cli
              rustToolchainAndroid
              cargo-ndk
              android-tools
              androidenv.androidPkgs.androidsdk # WARN Very large TODO Configure a subset of it
              androidenv.androidPkgs.ndk-bundle
              openjdk
            ];
            buildInputs =
              with pkgs;
              [
                openssl
              ]
              ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
                pkgs.darwin.apple_sdk.frameworks.Security
                pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
              ];
            buildPhase = ''
              export CARGO_TARGET_DIR=target
              # dx build --release --platform android # TODO When code OK
            '';
            installPhase = ''
              mkdir -p $out
              # TODO Copy android apk to $out
            '';
            doCheck = false; # TODO
          };
          default = pkgs.lib.attrValues {
            inherit (self.packages.${system}) web android;
          };
        }
      );
    };
}
