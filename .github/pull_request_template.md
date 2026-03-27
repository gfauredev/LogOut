# PR Title

Pull Request Description

## Related Issue

Resolves #ENTER_ISSUE_NUMBER

## Engineering Principles

- [ ] PR only contains changes strictly related to the requested feature or fix,
      scope is focused (no unrelated dependency updates or formatting)
- [ ] This code totally respects [`README`](README.md)’s Engineering Principles

## CI/CD Readiness

- [ ] Branch follows Conventional Branch: `feat/…`, `fix/…`, `refactor/…`, …
- [ ] Code is formatted with `dx fmt` AND `cargo fmt --all`
- [ ] All checks pass, `nix flake checks` succeeds without warnings
  - [ ] Code compiles, `dx build` succeeds (with necessary platform flags)
  - [ ] NO `cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic`
        warnings
  - [ ] All unit tests pass without warnings (`cargo test`)
  - [ ] End-to-end tests pass (`maestro test --headless maestro/{web,android}`)
