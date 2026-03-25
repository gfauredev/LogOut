<!-- LTeX: language=en-GB -->

Check out the README and other documentation material. Respect coding
conventions or guidelines you found, especially engineering principles.

This project uses a development environment, defined in `flake.nix`, invoke
development tools preceded by `nix develop --command`, or enter dev-shell first.

Stick to modern Rust best practices and idiomatic patterns. Produce the most
efficient and optimized code possible, remember to `dx fmt && cargo fmt --all`.

**Mandatory** ensure `nix flake check` passes before every commit.

> Note: Compilation for `wasm32-unknown-unknown` (our goal) with `dx build` can
> reject code that compiles fine for the host target and passes `cargo test`.
