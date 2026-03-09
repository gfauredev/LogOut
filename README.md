---
lang: en
---

[![built with garnix](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fgarnix.io%2Fapi%2Fbadges%2Fgfauredev%2FLogOut)](https://garnix.io/repo/gfauredev/LogOut)

[![Get it on GitHub](https://img.shields.io/badge/Get_it_on-GitHub-black?style=for-the-badge&logo=github)](https://github.com/gfauredev/LogOut/releases/latest)

[![Get it on Obtainium](https://img.shields.io/badge/Get_it_on-Obtainium-black?style=for-the-badge&logo=obtainium)](https://apps.obtainium.imranr.dev/redirect.html?r=obtainium://add/https://github.com/gfauredev/LogOut)

# LogOut

<!--toc:start-->

- [Project Structure](#project-structure)
- [Tooling & Dependencies](#tooling-dependencies)
- [Building & Running](#building-running)
  - [Building the PWA](#building-the-pwa)
  - [Building the Android App](#building-the-android-app)
- [Code Conventions & Contributing](#code-conventions-contributing)
- [Continuous Integration (CI)](#continuous-integration-ci)
- [Continuous Deployment (CD)](#continuous-deployment-cd)
- [Nightly Deep Checks](#nightly-deep-checks)
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

### Building the Android App

To build for Android as APK, run

```sh
dx build --android --release --target aarch64-linux-android
```

Dioxus `0.7` don’t yet supports signing (it does, but keys have to be in clear
in Dioxus.toml) the APK and adding icon(s) to it. So two scripts allow that.

Run `scripts/android-icon.sh` to add `android/res` icons to the app.

Run `scripts/android-sign.sh` to generate a signed `release-signed.apk` APK.

## Code Conventions & Contributing

- Functions, structs… must be documented with `rustdoc`
  - To generate and open the documentation `cargo doc --open`
- Every change must pass through a PR
- PRs must pass [CI checks](#continuous-integration-ci) to be merged
- Simple, flat structures are always preffered, do not nest if not necessary
  - Especially in HTML, a node with only one child can be replaced by it
- Keep the HTML structure as simple as possible
- Class-light styling mainly based on HTML semantic hierarchy
- Same CSS rules for similarly looking components, don’t overcomplicate
- Never hardcode values (except 0, 1, 100%), use clearly named constants
- Always ensure that all lints, end-to-end and unit tests pass

## Continuous Integration (CI)

[LogOut] keep high standards of code quality and reliability. Every change must
pass through a pull-request (PR), and every below check (that runs on pushes on
PRs) must pass (for some, at a certain level) for it to be merged into `main`.

- Run isolated in Garnix via (`flake.nix`)[./flake.nix], for every push on PR
  - Check if the code is properly **formated** `cargo fmt --all -- --check`
  - **Lint** `cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic`
  - **Unit test** while measuring **coverage** with `cargo llvm-cov`
  - Optimized **production build** for Web `dx build --web --release`
  - Optimized **production build** for Android `dx build --android --release`
  - At each step, cache outputs to avoid redundant work (automatic in Garnix)
- Run in standard Linux or macOS runners once necessary outputs are available
  - Check that more than `80%` of code (excluding `model` files) is covered,
    publish the full coverage summary table as a PR comment
  - Slower checks, only if above pass _and_ branch is up-to-date with `main`
    - **PageSpeed** Lighthouse audit on PWA, publish report as a PR comment
    - Web Maestro **end-to-end tests** with `maestro test maestro/web`
    - Publish a report with screenshots of failed E2E tests as a PR comment

## Continuous Deployment (CD)

[LogOut] stays continuously fresh and up-to-date thanks to its automated
deployment pipeline running at every push on `main` branch (coming only from
validated PRs), on standard Linux runners.

- Deploy the _production_ **Progressive Web App** to GitHub Pages
- Deploy **Android APK** in a “Rolling” timestamped GitHub (pre-)Release
  - Sign it with GitHub secrets and `scripts/android-sign.sh`
  - Only if the last release is from the previous (UTC) day, to avoid spamming
  - Remove the previous “Rolling” pre-releases older than a week

CD also runs when a [SemVer] `vMAJOR.MINOR.PATCH` **tag** is pushed, publishing
a “Stable” GitHub Release with a production Android APK buit on this `tag`.

## Nightly Deep Checks

[LogOut] ensures high quality code while with additional ressource intensive
checks that run every night at 2:00 AM (UTC) on the `main` branch.

- Run Android **end-to-end tests** in emulator `maestro test maestro/android`
- Analyze dependencies for vulnerabilities or deprecations with `cargo deny`
  - Automatically open PRs to update dependencies and flake with `renovate`
- Test the tests’ comprehensiveness by introducing bugs they should catch
  - **Mutation testing** with `cargo-mutants`
- Publish report(s) of the above checks accessible via the forge

## TODO

- DRY E2E tests to only one par user story, they can still runFlow others
- Mock exercise database with public/exercises.json for E2E tests, so they can
  load faster and not rely on external network requests
- Make layout and palette harmonious but minimal, efficient
  - Equalize spacings, sizes, net but not wasteful
  - Harmonize Rest timer text color with background when rest is due

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
[SemVer]: https://semver.org
[Serde]: https://serde.rs
[IndexedDB]: https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API
[Rexie]: https://github.com/wasmerio/rexie
[SQLite]: https://www.sqlite.org/index.html
[Rusqlite]: https://github.com/rusqlite/rusqlite
[Reqwest]: https://github.com/seanmonstar/reqwest
[Time]: https://github.com/time-rs/time
[Tokio]: https://tokio.rs
[Web-sys]: https://rustwasm.github.io/wasm-bindgen/web-sys/index.html
