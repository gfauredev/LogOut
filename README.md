---
lang: en
---

<!--toc:start-->

- [Project Structure](#project-structure)
- [Tooling & Dependencies](#tooling-dependencies)
  - [Development & Testing Tools](#development-testing-tools)
- [Building & Running](#building-running)
  - [Building the PWA](#building-the-pwa)
  - [GitHub Pages deployment](#github-pages-deployment)
- [Code Quality & Conventions](#code-quality-conventions)
  - [Unit Testing](#unit-testing)
  - [End-to-End Testing](#end-to-end-testing)
  - [Documentation](#documentation)
  - [Other](#other)
- [TODO](#todo)
  - [Optimization & Technical](#optimization-technical)
    - [To consider](#to-consider)

<!--toc:end-->

# LogOut

> Turn off your computer, Log your workOut

A simple, efficient and cross-platform workout logging application with
[800+ exercises] built-in, by [Guilhem Fauré].

- 💪 Easily log workout sessions with sets, reps, weights, distances, durations
- 📊 Analytics panel with line charts to track progress over time
- 🏋️ Browse the 870+ included exercises with search functionality
  - Easily add your custom exercises or customize existing ones
- 📱 Mobile-first responsive design, bottom navigation bar, local-first

## Project Structure

The project follows a modular Rust structure for a Dioxus application:

```text
LogOut/
├─ src/
│  ├─ main.rs  App entry point, routing (Dioxus Router), global state management
│  ├─ models/  Data models (Exercise, WorkoutSession, Enums) and unit-safe types (Weight, Distance)
│  ├─ services/  Business logic and persistence layers
│  │  ├─ storage.rs          Unified storage interface (IndexedDB on web, SQLite on native)
│  │  ├─ exercise_db.rs      Exercise library management and search logic
│  │  ├─ exercise_loader.rs  Logic for loading exercise data from JSON
│  │  ├─ wake_lock.rs        Keeps the screen on during active workout sessions
│  │  └─ service_worker.rs   Integration logic for the PWA service worker
│  ├─ components/  Functional Dioxus UI components
│  │  ├─ active_session.rs  Complex "Active Session" view with timers and logging
│  │  ├─ home.rs            Main landing page with session history
│  │  ├─ analytics.rs       Progress tracking with interactive charts
│  │  └─ …                  Other modular UI components (BottomNav, ExerciseCard…)
│  └─ utils.rs  Pure, side-effect-free utility functions (formatting, timestamps, URLs)
├─ e2e/      Playwright end-to-end tests for progressive web app
├─ maestro/  Maestro end-to-end tests for native mobile app
├─ public/   PWA static assets required by the browser
│  ├─ manifest.json  Web app manifest for PWA installation
│  ├─ sw.js          JavaScript Service Worker for PWA
│  └─ 404.html       Fallback page for single-page app routing
├─ assets/       Application-wide static assets (styles.css…)
├─ Cargo.toml    Rust manifest (dependencies, features, targets)
├─ Dioxus.toml   Configuration for the Dioxus CLI (build, serve, platform options)
├─ flake.nix     Nix flake for reproducible development environments
└─ package.json  Node.js manifest for E2E testing tools (Playwright)
```

## Tooling & Dependencies

| Library     | Role                                                                            |
| ----------- | ------------------------------------------------------------------------------- |
| [Dioxus]    | Main UI framework for building reactive components with a Rust-native DSL       |
| [Serde]     | Serialization and deserialization framework for all data models and persistence |
| [IndexedDB] | Local-first browser storage for workouts and custom exercises (via [Rexie])     |
| [SQLite]    | Native-first storage for workout data on Android/Linux (via [Rusqlite])         |

| Library   | Role                                                                           |
| --------- | ------------------------------------------------------------------------------ |
| [Reqwest] | Asynchronous HTTP client for loading exercise data and external assets         |
| [Time]    | Type-safe date and time manipulation (UTC/Local offsets)                       |
| [Tokio]   | Async runtime for the native application target.                               |
| [Web-sys] | Low-level bindings to browser APIs (Service Worker, Notifications, Visibility) |

### Development & Testing Tools

| Function                      | Tool                   |
| ----------------------------- | ---------------------- |
| Rust compilation              | [rustc]                |
| Build system                  | [Cargo]                |
| Dependencies and environment  | [Nix]                  |
| Versionning and collaboration | [Git] hosted on GitHub |
| Unit tests                    | [Cargo test]           |
| End-to-end tests (PWA)        | [Playwright]           |
| End-to-end tests (Android)    | [Maestro]              |
| Code coverage                 | [cargo-llvm-cov]       |
| Rust language assistance      | [rust-analyzer] (LSP)  |
| Documentation from code       | [rustdoc]              |
| Rust formatting               | [rustfmt]              |
| Rust quality control          | [Clippy]               |
| Rust debugging                | [lldb]                 |
| Code edition                  | [Helix], [VS Code] …   |

## Building & Running

The project provides a [Nix] development shell with all required dependencies
(Rust, Dioxus CLI, Android SDK…). With Nix installed, enter the shell with
`nix develop`. Preferably, with Direnv installed, allow the automatic
development shell loading with `direnv allow`.

### Building the PWA

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

The project uses `rustdoc` for code documentation. To generate and open the
documentation in your browser:

```sh
cargo doc --open
```

This generates HTML documentation for all internal modules, models, and
services, providing a detailed view of the codebase's API.

### Other

- Simple, flat structures are always preffered, do not nest if not necessary
  - Especially in HTML, a node with only one child can be replaced by it
- Keep the HTML structure as simple as possible
- Class-light styling mainly based on HTML semantic hierarchy
- Same CSS rules for similarly looking components, don’t overcomplicate
- Never hardcode values (except 0, 1, 100%), use clearly named constants

## TODO

Check README for code conventions and guidelines.

- Unifiying: replace Playwright with Maestro (beta) web testing

Always ensure that all lints, end-to-end tests and unit tests pass.

### Optimization & Technical

- Sign Android app and make it properly installable
- Storing log by log rather than rewriting the whole session

#### To consider

- Improve indexedDB error handling with thiserror
- Reduce boilerplate by using strum crate for enums serialization

[800+ exercises]: https://github.com/yuhonas/free-exercise-db
[Cargo]: https://doc.rust-lang.org/cargo/
[cargo test]: https://doc.rust-lang.org/cargo/commands/cargo-test.html
[cargo-llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
[Clippy]: https://github.com/rust-lang/rust-clippy
[Dioxus]: https://dioxuslabs.com/
[direnv]: https://direnv.net/
[`direnv`]: https://direnv.net/
[free-exercise-db]: https://github.com/yuhonas/free-exercise-db
[Guilhem Fauré]: https://www.guilhemfau.re
[Git]: https://git-scm.com/
[Helix]: https://helix-editor.com/
[lcov]: https://github.com/linux-test-project/lcov
[lldb]: https://lldb.llvm.org/
[llvm-cov]: https://llvm.org/docs/CommandGuide/llvm-cov.html
[Maestro]: https://maestro.dev/
[Nix]: https://nixos.org/
[Node.js]: https://nodejs.org/
[Playwright]: https://playwright.dev/
[Rust]: https://www.rust-lang.org/
[rust-analyzer]: https://rust-analyzer.github.io/
[rust]: https://www.rust-lang.org/
[rustc]: https://doc.rust-lang.org/rustc/
[rustdoc]: https://doc.rust-lang.org/rustdoc/
[rustfmt]: https://github.com/rust-lang/rustfmt
[VS Code]: https://code.visualstudio.com/
[Serde]: https://serde.rs/
[IndexedDB]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
[Rexie]: https://github.com/wasmerio/rexie
[SQLite]: https://www.sqlite.org/index.html
[Rusqlite]: https://github.com/rusqlite/rusqlite
[Reqwest]: https://github.com/seanmonstar/reqwest
[Time]: https://github.com/time-rs/time
[Tokio]: https://tokio.rs/
[Web-sys]: https://rustwasm.github.io/wasm-bindgen/web-sys/index.html
