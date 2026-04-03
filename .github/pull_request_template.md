This Pull Request…

## Engineering Principles

- [ ] PR only contains changes strictly related to the requested feature or fix,
      scope is focused (no unrelated dependency updates or formatting)
- [ ] This code totally respects [`README`](README.md)’s Engineering Principles

## CI/CD Readiness

- [ ] Branch follows Conventional Branch: `feat/…`, `fix/…`, `refactor/…`, …
- [ ] Code is formatted with `dx fmt; cargo fmt`
- [ ] All checks pass, `nix flake checks` succeeds without warnings
  - [ ] Code compiles, `dx build` with necessary platform flags succeeds
  - [ ] `cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic`
        produces zero warnings
  - [ ] All unit tests pass without warnings
        `cargo llvm-cov nextest --ignore-filename-regex '(src/components/|\.cargo/registry/|nix/store)'`
  - [ ] End-to-end tests pass `maestro test --headless maestro/web`
        `maestro test --headless maestro/android`
