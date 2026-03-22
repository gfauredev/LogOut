<!-- LTeX: language=en-GB -->

Check out the README and other documentation material. Respect coding
conventions or guidelines you found, especially about processes.

This project uses a development environment, defined in `flake.nix`: run a
command with all the needed dependencies and tools with `nix develop`.

Stick to modern Rust best practices and idiomatic patterns. Only produce the
most efficient and optimized code possible and report on suboptimal code you
notice. Remember to `dx fmt && cargo fmt --all`. Ensure `dx build`, `cargo test`
and `cargo clippy --all-targets -- -W clippy::all -W clippy::pedantic` succeed
without any warnings after your edits. `dx build` performs additional steps than
`cargo build`, it can fail even if `cargo test` suceeds; you HAVE to run
`dx build` as the first sanity check.
