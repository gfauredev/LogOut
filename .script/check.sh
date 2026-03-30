dx build --web --verbose
cargo llvm-cov nextest --bin log-out --html \
  --ignore-filename-regex "(src/components/|\.cargo/registry/|nix/store)"
cargo clippy --all-targets -- -D warnings -W clippy::all -W clippy::pedantic
