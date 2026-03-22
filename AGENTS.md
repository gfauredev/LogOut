<!-- LTeX: language=en-GB -->

Check out the README and other documentation material. Respect coding
conventions or guidelines you found, especially about processes.

This project uses a development environment, defined in `flake.nix`, invoke
development tools preceded by `nix develop --command`, or enter dev-shell first.

Stick to modern Rust best practices and idiomatic patterns. Produce the most
efficient and optimized code possible and report on suboptimal code you notice.
Remember to `dx fmt && cargo fmt --all`.

## Mandatory THREE checks before every commit

```sh
dx build
cargo test
cargo clippy --all-targets -- -W clippy::all -W clippy::pedantic
```

**`dx build` is not optional.** It compiles the crate for
`wasm32-unknown-unknown` (a separate target from the host), which can reject
code that compiles fine for the host target. Skipping it has caused broken
builds in the past. Do **not** consider `cargo build` or `test` as substitutes.
