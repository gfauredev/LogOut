<!-- LTeX: language=en-GB -->

Check out the README and other documentation material. Respect coding
conventions or guidelines you found, especially about processes.

This project uses a development environment, defined in `flake.nix`: run a
command with all the needed dependencies and tools with `nix develop`.

Stick to modern Rust best practices and idiomatic patterns. Produce the most
efficient and optimized code possible and report on suboptimal code you notice.
Remember to `dx fmt && cargo fmt --all`. Ensure `dx build`, `cargo test` and
`cargo clippy --all-targets -- -W clippy::all -W clippy::pedantic` succeed
without any warnings after your edits. `dx build` performs additional steps than
`cargo build`, it can fail even if `cargo test` suceeds; you MUST run those
THREE tests to validate changes.

## Mandatory validation checklist – run ALL THREE before every commit

```sh
dx build                                                              # WASM front-end build – catches wasm32 target errors missed by cargo test
cargo test                                                            # unit + integration tests
cargo clippy --all-targets -- -D warnings -W clippy::all -W clippy::pedantic  # lint – zero warnings allowed
```

**`dx build` is not optional.** It compiles the crate for `wasm32-unknown-unknown`
(a separate target from the host), which can reject code that compiles fine for
the host target.  Skipping it has caused broken builds in the past.  Do **not**
use `cargo build` as a substitute.

`dx` may not be on `PATH` outside of `nix develop`; invoke it as
`nix develop --command dx build` if needed, or activate the dev-shell first.
