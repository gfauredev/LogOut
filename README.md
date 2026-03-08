---
lang: en
---

# LogOut

[![Get it on GitHub](https://img.shields.io/badge/Get_it_on-GitHub-black?style=for-the-badge&logo=github)](https://github.com/gfauredev/LogOut/releases/latest)
[![Get it on Obtainium](https://img.shields.io/badge/Get_it_on-Obtainium-black?style=for-the-badge&logo=obtainium)](https://apps.obtainium.imranr.dev/redirect.html?r=obtainium://add/https://github.com/gfauredev/LogOut)

<!--toc:start-->

- [Project Structure](#project-structure)
- [Tooling & Dependencies](#tooling-dependencies)
- [Building & Running](#building-running)
  - [Building the PWA](#building-the-pwa)
  - [GitHub Pages deployment](#github-pages-deployment)
- [Code Quality & Conventions](#code-quality-conventions)
  - [Unit Testing](#unit-testing)
  - [End-to-End Testing](#end-to-end-testing)
  - [Documentation](#documentation)
  - [Other](#other)
- [TODO](#todo)

<!--toc:end-->

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
├ src/
│ ├ main.rs  App entry point, routing (Dioxus Router), global state management
│ ├ models/  Data models (Exercise, WorkoutSession, Enums) and unit-safe types
│ ├ services/      Business logic and persistence layers
│ │ ├ storage.rs   Unified storage interface (IndexedDB Web, SQLite native)
│ │ ├ exercise_db.rs      Exercise library management and search logic
│ │ ├ exercise_loader.rs  Logic for loading exercise data from JSON
│ │ ├ wake_lock.rs        Keeps the screen on during active workout sessions
│ │ └ service_worker.rs   Integration logic for the PWA service worker
│ ├ components/           Functional Dioxus UI components
│ │ ├ active_session.rs   Complex "Active Session" view with timers and logging
│ │ ├ home.rs             Main landing page with session history
│ │ ├ analytics.rs        Progress tracking with interactive charts
│ │ └ …                   Other modular UI components (BottomNav, ExerciseCard…)
│ └ utils.rs   Pure, side-effect-free utility functions (format, timestamps…)
├ maestro/          Maestro end-to-end tests
│ ├ web/            Web browser PWA tests (order-independent, each self-contained)
│ │ └ _flows/       Reusable subflows (navigation, session setup, etc.)
│ └ android/        Android native app
├ public/           PWA static assets required by the browser
│ ├ manifest.json   Web app manifest for PWA installation
│ ├ sw.js           JavaScript Service Worker for PWA
│ └ 404.html        Fallback page for single-page app routing
├ assets/       Application-wide static assets (SCSS…)
├ Cargo.toml    Rust manifest (dependencies, features, targets)
├ Dioxus.toml   Configuration for Dioxus CLI (build, serve, platform options)
└ flake.nix     Nix flake for reproducible development environments
```

## Tooling & Dependencies

| Role                                                                            | Library     |
| ------------------------------------------------------------------------------- | ----------- |
| Main UI framework for building reactive components with a Rust-native DSL       | [Dioxus]    |
| Serialization and deserialization framework for all data models and persistence | [Serde]     |
| Local-first browser storage for workouts and custom exercises (via [Rexie])     | [IndexedDB] |
| Native-first storage for workout data on Android/Linux (via [Rusqlite])         | [SQLite]    |

| Role                                                                           | Library   |
| ------------------------------------------------------------------------------ | --------- |
| Asynchronous HTTP client for loading exercise data and external assets         | [Reqwest] |
| Type-safe date and time manipulation (UTC/Local offsets)                       | [Time]    |
| Async runtime for the native application target.                               | [Tokio]   |
| Low-level bindings to browser APIs (Service Worker, Notifications, Visibility) | [Web-sys] |

| Function                      | Tool                   |
| ----------------------------- | ---------------------- |
| Rust compilation              | [rustc]                |
| Build system                  | [Cargo]                |
| Dependencies and environment  | [Nix]                  |
| Versionning and collaboration | [Git] hosted on GitHub |
| Unit tests                    | [Cargo test]           |
| End-to-end tests (PWA)        | [Maestro] (beta web)   |
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
dx build --web --release
```

Output is written to `target/dx/log-out/release/web/public/`.

To serve the PWA locally with hot-reload during development, run

```sh
dx serve # Serves at http://localhost:8080
```

### GitHub Pages deployment

The PWA is deployed automatically on every push to `main` by
`.github/workflows/cd.yml` on `https://gfauredev.github.io/LogOut`.

### Building the Android App

To build for Android as APK, run

```sh
dx build --android --release --target aarch64-linux-android
```

Dioxus `0.7` don’t yet supports signing (it does, but keys have to be in clear
in Dioxus.toml) the APK and adding icon(s) to it. So two scripts allow that.

Run `scripts/android-icon.sh` to add `android/res` icons to the app.

Run `scripts/android-sign.sh` to generate a signed `release-signed.apk` APK.

## Code Quality & Conventions

Ensure that all lints, end-to-end and unit tests pass before merging a pull
request, or `.github/workflows/ci.yml` will reject it.

| Job               | Command                         | Requirement                                                 |
| ----------------- | ------------------------------- | ----------------------------------------------------------- |
| **Formatting**    | `cargo fmt --check`             | Code must match `rustfmt` style exactly                     |
| **Linting**       | `cargo clippy -- -D warnings`   | Zero Clippy warnings                                        |
| **Unit tests**    | `cargo llvm-cov …`              | All unit tests pass, covering more than 90% of the codebase |
| **E2E (web)**     | `maestro test --platform web …` | All Maestro web tests pass                                  |
| **E2E (Android)** | `maestro test maestro/android/` | All Maestro Android tests pass                              |
| **PageSpeed**     | Lighthouse CLI                  | Performance scores posted as PR comment                     |

You can run them locally with the commands

```sh
cargo fmt --check # Formatting
cargo clippy -- -D warnings # Linting
cargo test # Unit tests
maestro test --platform web maestro/web/ # Web E2E tests (requires built PWA served on localhost:8080)
maestro test maestro/android/ # Android E2E tests (requires running emulator)
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
cargo llvm-cov --bin log-out # Summary inline
```

```sh
cargo llvm-cov --bin log-out --lcov --output-path lcov.info # LCOV report
```

### End-to-End Testing

End-to-end tests exercise the full application using [Maestro]. All
[user stories](./USER_STORIES.md) are covered with two test flows each: one for
the PWA and one for native Android. Tests are numbered `01`–`20` matching the
user story order, so each test can rely on state from the previous ones when run
as a full suite.

```sh
# Build and serve the PWA
dx serve --open false --interactive false --web --release
 # In a second terminal, run all web E2E tests (order-independent)
maestro test maestro/web/
# Or run a single test file
maestro test maestro/web/full_workout_session.yaml
# Or run the tests headless
maestro test --headless maestro/web/
```

> [!NOTE]
> The first run of tests that touch the exercise browser may take up to 30
> seconds while the exercise database is downloaded from the remote URL.

Replace `web` by `android` in the above commands to run the Android E2E tests.
An emulator must be running, or a physical device must be connected via `ADB`.

> [!TIP]
> Each test is self-contained and independent — no specific run order is
> required. Tests that need pre-existing state (e.g. a completed session) set it
> up via reusable subflows in `maestro/web/_flows/`. Use `maestro studio` to
> debug a failing test interactively.

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
- Always ensure that all lints, end-to-end and unit tests pass.

## TODO

- Make android app background black at up, currently briefly flashing white
- Fix android app showing OS error 30, unable to read custom exercises
- Fix database URL change, as currently reloading the app displays the default
  exercises again (but interestingly, images don’t load after changing)
- Fix web `?dl_session` link seemingly not working, add tests to make sure
- DRY E2E tests to only one par user story, they can still runFlow others
- Mock exercise database with public/exercises.json for E2E tests, so they can
  load faster and not rely on external network requests
- Harmonize Rest timer text color when background when rest is due
- Make layout harmonious but minimal, efficient, look at spacings, sizes
- Consider using Dioxus Components https://dioxuslabs.com/components

[LogOut]: https://gfauredev.github.io/LogOut
[800+ exercises]: https://github.com/yuhonas/free-exercise-db
[Cargo]: https://doc.rust-lang.org/cargo
[cargo test]: https://doc.rust-lang.org/cargo/commands/cargo-test.html
[cargo-llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
[Clippy]: https://github.com/rust-lang/rust-clippy
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
[Rust]: https://www.rust-lang.org
[rust-analyzer]: https://rust-analyzer.github.io
[rust]: https://www.rust-lang.org
[rustc]: https://doc.rust-lang.org/rustc
[rustdoc]: https://doc.rust-lang.org/rustdoc
[rustfmt]: https://github.com/rust-lang/rustfmt
[VS Code]: https://code.visualstudio.com
[Serde]: https://serde.rs
[IndexedDB]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
[Rexie]: https://github.com/wasmerio/rexie
[SQLite]: https://www.sqlite.org/index.html
[Rusqlite]: https://github.com/rusqlite/rusqlite
[Reqwest]: https://github.com/seanmonstar/reqwest
[Time]: https://github.com/time-rs/time
[Tokio]: https://tokio.rs
[Web-sys]: https://rustwasm.github.io/wasm-bindgen/web-sys/index.html
