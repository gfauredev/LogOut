---
lang: en
---

<!--toc:start-->

- [Project Structure](#project-structure)
- [Tooling](#tooling)
- [Building & Running](#building-running)
  - [GitHub Pages deployment](#github-pages-deployment)
- [Code Quality Conventions](#code-quality-conventions)
  - [Unit Testing](#unit-testing)
  - [End-to-End Testing](#end-to-end-testing)
  - [Documentation](#documentation)
- [TODO](#todo)
  - [Optimization & Technical](#optimization-technical)

<!--toc:end-->

# LogOut

> Turn off your computer, Log your workOut

A simple, efficient and cross-platform workout logging application with
[800+ exercises] built-in, by [Guilhem Fauré].

- 💪 Easily log workout sessions with sets, reps, weights, distances, durations
- 📊 **Analytics panel** with line charts to track progress over time
- 🏋️ Browse the 870+ included exercises with search functionality
  - Easily add your custom exercises or customize existing ones
- 📱 Mobile-first responsive design, bottom navigation bar, local-first

## Project Structure

<!-- TODO Update -->

```
src/
  main.rs       # Application entry point and routing
  models/       # Data models: exercises, sessions, sets, enums
  services/     # Business: exercise DB, storage (IndexedDB), service worker
  components/   # UI components: home, exercise list, session view, analytics
  utils.rs      # Pure utility functions (date formatting, etc.)
e2e/app.spec.ts   # Playwright end-to-end tests
assets/styles.css # Application stylesheet
public/
  manifest.json # PWA manifest
  sw.js         # Service worker (JavaScript, required by the browser SW spec)
```

## Tooling

| Function                      | Tool                   |
| ----------------------------- | ---------------------- |
| Rust compilation              | [rustc]                |
| Build system                  | [Cargo]                |
| Dependencies and environment  | [Nix]                  |
| Versionning and collaboration | [Git] hosted on GitHub |
| Unit tests                    | [Cargo test]           |
| End-to-end tests (PWA)        | [Playwright]           |
| End-to-end tests (Android)    | [Maestro]              |
| Code coverage                 | [cargo llvm-cov]       |
| Rust language assistance      | [rust-analyzer] (LSP)  |
| Documentation from code       | [rustdoc]              |
| Rust formatting               | [rustfmt]              |
| Rust quality control          | [Clippy]               |
| Rust debugging                | [lldb]                 |
| Code edition                  | [Helix], [VS Code] …   |

## Building & Running

To build for web as a PWA, run

```sh
dx build --platform web --release
```

Output is written to `target/dx/log-workout/release/web/public/`.

To serve the PWA locally with hot-reload during development, run

```sh
dx serve # Serves at http://localhost:8080
```

### GitHub Pages deployment

The PWA is deployed automatically on every push to `main` by
`.github/workflows/cd.yml` on `https://gfauredev.github.io/LogOut`.

## Code Quality & Conventions

Every pull request is validated by `.github/workflows/ci.yml`, ensuring that all
five jobs pass before a PR can be merged.

| Job            | Command                       | Requirement                                                 |
| -------------- | ----------------------------- | ----------------------------------------------------------- |
| **Formatting** | `cargo fmt --check`           | Code must match `rustfmt` style exactly                     |
| **Linting**    | `cargo clippy -- -D warnings` | Zero Clippy warnings                                        |
| **Unit tests** | `cargo llvm-cov …`            | All unit tests pass, covering more than 90% of the codebase |
| **E2E tests**  | `npx playwright test`         | All Playwright tests pass                                   |
| **PageSpeed**  | Lighthouse CLI                | Performance scores posted as PR comment                     |

You can run them locally with the commands

```sh
cargo fmt --check # Formatting
cargo clippy -- -D warnings # Linting
cargo test # Unit tests
npx playwright test # E2E tests (starts dev server)
```

### Unit Testing

Unit tests cover pure-Rust model functions (formatting, parsing, serialization),
service stubs, and utility helpers. They compile and run on the native target —
no browser or WASM toolchain required.

```sh
cargo test
```

The `main` branch must always pass `100%` of unit tests, covering more than
`90%` of the codebase.

They can be run with `cargo llvm-cov` (might need to be installed).

```sh
cargo llvm-cov --bin log-workout # Summary inline
```

```sh
cargo llvm-cov --bin log-workout --lcov --output-path lcov.info # LCOV report
```

### End-to-End Testing

End-to-end tests exercise the full application running with [Playwright]. They
start `dx serve` automatically before the test run.

```sh
npx playwright test # Run the tests (headless by default)
```

```sh
npx playwright test --headed # Run tests within a visible browser
```

The `main` branch must always pass `100%` of E2E tests. When a test fails,
Playwright captures a screenshot automatically and saves it to `test-results/`.

### Documentation

<!-- TODO -->

### Other

- Simple, flat structures are always preffered, do not nest if not necessary
  - Especially in HTML, a node with only one child can be replaced by it

## TODO

Always prefer simpler, leaner structure with less nesting.

- Notifications
  - Fix notifications don’t fire on mobile, don’t do any sound even when allowed
  - Request notification only by a direct user click on the warning toast
    instead of aggressively at launch to respect browsers
    - Indicate in the toast that the user should click it
- Inform user of uncatched failures / errors (especially storage) with toasts
- Clamp timestamps to 0 before casting
- Use SQLite (rusqlite) for native storage instead of inneficiant JSON files

Ensure that all edits respect code conventions and pass all checks.

### Optimization & Technical

- Maestro End-to-End Tests
  - Make native Android tests pass
  - Unifiying: consider replacing Playwright with Maestro (beta) web testing
  - Use `extendedWaitUntil` commands to dynamically wait for the app's first
    render instead of hardcoded 60 seconds sleep
- Sign Android app and make it properly installable
- HTML structure, CSS
  - Prefer HTML semantic hierarchy over classes
  - Keep similar items styled by the same CSS
  - Remove unused (dead) CSS
- Remove any magic number, making then into clearly named constants
  - In Rust and (especially) in CSS (:root variables)
- Reduce allocations to the heap (clone), especially in search loops
- Split the SessionView god component into modular child components
- Improve indexedDB error handling with thiserror
- Reduce boilerplate by using strum crate for enums serialization

[800+ exercises]: https://github.com/yuhonas/free-exercise-db
[Cargo]: https://rust-lang.org
[cargo test]: https://rust-lang.org
[cargo llvm-cov]: https://llvm.org/docs/CommandGuide/llvm-cov.html
[Clippy]: https://rust-lang.org
[Dioxus]: https://dioxuslabs.com
[direnv]: https://direnv.net
[`direnv`]: https://direnv.net
[free-exercise-db]: https://github.com/yuhonas/free-exercise-db
[Guilhem Fauré]: https://www.guilhemfau.re
[Git]: https://git-scm.com
[Helix]: https://helix-editor.com
[lcov]: https://github.com/linux-test-project/lcov
[lldb]: https://lldb.llvm.org
[llvm-cov]: https://llvm.org/docs/CommandGuide/llvm-cov.html
[Maestro]: https://maestro.dev
[Nix]: https://nixos.org
[Node.js]: https://nodejs.org
[Playwright]: https://playwright.dev
[Rust]: https://rust-lang.org
[rust-analyzer]: https://rust-lang.org
[rust]: https://rust-lang.org
[rustc]: https://rust-lang.org
[rustdoc]: https://rust-lang.org
[rustfmt]: https://rust-lang.org
[VS Code]: https://code.visualstudio.com
