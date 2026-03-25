<!-- LTeX: language=en-GB -->

[![Get it on Obtainium](https://img.shields.io/badge/Get_it_on-Obtainium-black?style=for-the-badge&logo=obtainium)](https://apps.obtainium.imranr.dev/redirect.html?r=obtainium://add/https://github.com/gfauredev/LogOut)
[![Get it on GitHub](https://img.shields.io/badge/Get_it_on-GitHub-black?style=for-the-badge&logo=github)](https://github.com/gfauredev/LogOut/releases/latest)
[![Garnix CI Status](https://img.shields.io/endpoint.svg?url=https%3A%2F%2Fgarnix.io%2Fapi%2Fbadges%2Fgfauredev%2FLogOut)](https://garnix.io/repo/gfauredev/LogOut)

# LogOut

<!--toc:start-->

- [Project Structure](#project-structure)
- [Tooling and Dependencies](#tooling-and-dependencies)
- [Building and Running](#building-and-running)
  - [Android Native APK](#android-native-apk)
- [Engineering Principles and Contributing](#engineering-principles-and-contributing)
- [Continuous Integration and Deployment (CI/CD)](#continuous-integration-and-deployment-cicd)
  - [Weekly Deep Checks](#weekly-deep-checks)
- [Licensing and Credits](#licensing-and-credits)

<!--toc:end-->

> Close your laptop, *l*og your work*o*ut

A simple, efficient and cross-platform workout logging application with
[800+ exercises] built-in, by [Guilhem Fauré].

- 💪 Easily log workout sessions with sets, reps, weights, distances, durations
- 🏋️ Use the 870+ included exercises with images and instructions
  - 📝 Easily add your custom exercises or customize existing ones
- 🔍 Easily search them with powerful text search and attribute based filtering
- 📊 Track your progress over time on several metrics and exercises in analytics
- 📱 Responsive design, ergonomic navigation, local-first, performant

<p float="left">
  <img src=".screenshot/search.png" width="32%" alt="Screenshot of LogOut exerices list page, with search terms entered">
  <img src=".screenshot/home.png" width="32%" alt="Screenshot of LogOut home page, showing completed past sessions">
  <img src=".screenshot/analytics.png" width="32%" alt="Screenshot of LogOut analytics page, showing evolution of Pull-Up weight and reps">
</p>

## Project Structure

The project follows a modular [Rust] structure for a [Dioxus] application:

```text
LogOut/
├ Cargo.toml    Rust manifest (dependencies, features, targets)
├ Dioxus.toml   Configuration for Dioxus CLI (build, serve, platform options)
├ STORIES.md    User stories, serve as a basis for end-to-end tests
├ android/      Android native app static assets and configuration files
├ assets/       Application-wide static assets
├ flake.nix     Nix flake: reproducible development environment, builds, checks
├ maestro/      Maestro end-to-end tests (order-independent, self-contained)
│ ├ android/    Android native app tests
│ └ web/        Web browser PWA tests
├ public/       PWA static assets required by the browser
└ src/
  ├ main.rs     App entry point, routing (Dioxus Router), global state 
  ├ utils.rs    Pure, side-effect-free utility functions (format, timestamps…)
  ├ models/     Data models (Exercise, WorkoutSession, Enums), unit-safe types
  ├ services/   Business logic and persistence layers
  └ components/ Functional Dioxus UI components
```

## Tooling and Dependencies

| Purpose             | Methodology            |
| ------------------- | ---------------------- |
| Project versionning | [SemVer]               |
| Commit messages     | [Conventional Commits] |
| Branch naming       | [Conventional Branch]  |
| Branching model     | [GitHub Flow]          |
| Changes submission  | GitHub Pull Requests   |
| Issue tracking      | GitHub Issues          |

| Purpose                       | Tool                                         |
| ----------------------------- | -------------------------------------------- |
| Rust compilation              | [rustc]                                      |
| Build system                  | [Cargo]                                      |
| Dependencies and environment  | [Nix]                                        |
| Versionning and collaboration | [Git] hosted on GitHub                       |
| Unit tests                    | [Cargo test]                                 |
| End-to-end tests (PWA)        | [Maestro] (beta web)                         |
| End-to-end tests (Android)    | [Maestro]                                    |
| Code coverage                 | [cargo-llvm-cov]                             |
| Rust language assistance      | [rust-analyzer] (LSP)                        |
| Documentation from code       | [rustdoc]                                    |
| Rust formatting               | [rustfmt] and [dx] fmt                       |
| Rust quality control          | [Clippy]                                     |
| Rust debugging                | [lldb]                                       |
| Code edition                  | Allows modern Rust dev ([Helix], [VS Code]…) |

| Purpose                                                 | Library    |
| ------------------------------------------------------- | ---------- |
| Main UI reactive framework                              | [Dioxus]   |
| (De)Serialization, data models and persistence          | [Serde]    |
| PWA Workouts and custom exercises storage ([IndexedDB]) | [Rexie]    |
| Native Workouts and custom exercises storage ([SQLite]) | [Rusqlite] |
| Asynchronous HTTP client                                | [Reqwest]  |
| Date and time manipulation (UTC/Local offsets)          | [Time]     |
| Async runtime for the native application target.        | [Tokio]    |
| Bindings to browser APIs (Service Worker…)              | [Web-sys]  |

## Building and Running

The project uses [Nix] to download all (proper versions of) required
dependencies, configure the development environment (shell) and build the
application, reproducibly. The [Nix] environment and tooling is defined in
[`flake.nix`](flake.nix), enable it with `nix develop` or automatically with an
allowed [`.envrc`](.envrc) (`direnv allow`) via [`direnv`] (recommended).

For release builds, we prefer pure reproducible `nix build`, but for development
speed, it is recommended to use the hot-reloading `dx serve` ([PWA] by default).

### Android Native APK

We currently don’t support pure `nix build` for Android. To build the native
Android APK, run the following from an activated development shell:

```sh
dx build --android --release --target aarch64-linux-android # Or desired arch
```

> APK is signed with [`apk-sign.sh`](.script/apk-sign.sh) after the build, to
> keep it reproducible and because Dioxus requires secrets in clear in VCS

## Engineering Principles and Contributing

Sometimes, we need to make tradeoffs between different positives outcomes.
LogOut follows that priority order:

1. User Experience
   1. Maximize data integrity, never lose or corrupt user data
      - ⚠️ WARNING Ignored until v1.0.0, expect breaking data model changes
   2. Maximize extensibility, easily give users features they need
   3. Maximize correctness and stability, work as the user expects, reliably
   4. Minimize computational complexity, be snappy, pleasant to use
2. Developer Experience
   1. Maximize code readability and maintenability, make it easy to understand
   2. Maximize simplicity, minimize complexity, avoid nesting, over-engineering
   3. Maximize testability and iteration speed, isolated units, fast compile
3. Device Friendlyness
   1. Minimize battery usage, don’t kill mobile devices
   2. Minimize memory footprint, run smoothly on low-end devices
   3. Minimize binary and stored data size, be unnoticed, load quickly
   4. Minimize network usage, work offline, don’t waste users’ data plans

That doesn’t means lower order items are not important, this list is just for
when tradeoffs are strictly necessary. If possible, maximize all outcomes.
Follow these general engineering principles:

- **Decouple core logic** from **platform specificities** (storage, OS
  integrations, UI frameworks, network…)
  - Abstract these boundaries behind traits or interfaces to keep the
    application testable and portable
- Enforce a **Single Source of Truth**, derive component or local state directly
  from a centralized global state to prevent desynchronization
  - Confine all state mutations to atomic, centralized functions
- Bind external resources to **strict lifecycles** (RAII), guarantee their
  cleanup by tying their lifecycles directly to object scope
  - Event listeners, database transactions, browser object URLs…
- **Never block main/UI thread**, strictly offload synchronous, I/O-heavy, or
  CPU-bound operations to background threads
  - Ensure state is passed safely across async boundaries, and use cancellable
    primitives instead of hanging tasks
- **Lazily load and evaluate**, optimize memory, assume datasets will grow large
  - Defer loading heavy data, historical records, and binary assets until the
    exact moment they are needed
  - Use reference counting (`Rc`/`Arc`) for heavy objects in memory to avoid
    expensive deep copies
- **Gracefully fail** (especially at system boundaries), anticipate failures
  whenever interacting with the network, file system, foreign functions (FFI)…
  - Handle errors explicitly without crashing the app, surface them gracefully
    to the user via managed queues, and never swallow them silently
  - Generaly, avoid panicking methods and macros such as `.unwrap()` and
    `.expect()` variants, direct `vec[i]`, `.borrow()` variants, `panic!()`…
- **Avoid or optimize I/O**, disk reads and network requests are expensive
  - Cache assets, strictly normalize data, optimize database queries…
- **Don’t reinvent the wheel**, use battle-tested abstractions, standardized,
  widely adopted crates/libraries (URL encoding, timezone parsing, unit…)
- **Explicit over implicit**, for developer experience, cache management…
- Avoid "**magic**" hardcoded values, use clearly named constants
  - Except where it really makes sense, like usually 0, 1, 100%…
- Properly **document** what you do (functions, structs… with `rustdoc`)
- Avoid nesting, avoid complexity; generally, avoid things with only one child
- **Style** class-light, mainly based on semantic hierarchy and types
- Ensure code is properly **formatted** with `dx fmt` and `cargo fmt --all`
- Ensure code **compiles** with `dx build` plus eventual platform flags
- Ensure all **unit tests** `cargo test` pass without warning
- **No** `cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic` warns
- Ensure all **end-to-end tests** `maestro test` (`--headless`) pass

Follow this contribution process, based on [GitHub Flow], [Conventional Branch]:

1. Before writing any code, **open an issue** to discuss it with the maintainer
2. Create a **branch** for your change with a clear [Conventional Branch] name:
   - `feat/my-new-feature`
   - `fix/my-bug-fix`
   - `refactor/my-consequent-refactor`
   - …
3. Open a **Pull Request (PR)** as soon as your code compiles and checks
   - Avoid touching things not strictly related to your desired changes, e.g.
     updating dependencies
4. Fulfill the **PR** template checks before marking it ready for review
5. Fix your code if it don’t pass [CI checks](#continuous-integration-ci)

## Continuous Integration and Deployment (CI/CD)

[LogOut] keep high standards of code quality and reliability. Every change must
pass through a pull-request (PR), and every below check (that runs on pushes on
PRs) must pass (for some, at a certain level) for it to be merged into `main`.

- Run isolated in Garnix via (`flake.nix`)[flake.nix], for every push on PR
  - Check if the code is properly **formated** `cargo fmt --all -- --check`
  - **Lint** `cargo clippy -- -D warnings -W clippy::all -W clippy::pedantic`
  - **Unit test** while measuring **coverage** with `cargo llvm-cov`
  - **Build production** release for Web `dx build --web --release`
  - At each step, cache outputs to avoid redundant work (automatic in Garnix)
- Run in standard Linux or macOS runners once necessary outputs are available
  - Check that more than `80%` of code (excluding `components`) is covered,
    publish the full coverage summary table as a PR comment
  - Slower checks, only if above pass _and_ branch is up-to-date with `main`
    - **PageSpeed** Lighthouse audit on PWA, publish report as a PR comment
    - Web Maestro **end-to-end tests** with `maestro test maestro/web`
    - Publish a report with screenshots of failed E2E tests as a PR comment

[LogOut] stays continuously fresh and up-to-date thanks to its automated
deployment pipeline running at every push on `main` branch (coming only from
validated PRs), on standard Linux runners.

- Deploy the _production_ **Progressive Web App** to GitHub Pages
- **Build production** release for Android `dx build --android --release`
  - Deploy **Android APK** in a “Rolling” timestamped GitHub (pre-)Release
  - Sign it with GitHub secrets and `scripts/apk-sign.sh`
  - Only if the last release is from the previous (UTC) day, to avoid spamming
  - Remove the previous “Rolling” pre-releases older than a week

CD also runs when a [SemVer] `vMAJOR.MINOR.PATCH` **tag** is pushed, publishing
a “Stable” GitHub Release with a production Android APK buit on this `tag`.

### Weekly Deep Checks

[LogOut] ensures high quality code with additional ressource intensive checks
that run every Sunday at midnight on the `main` branch.

- Run Android **end-to-end tests** in emulator `maestro test maestro/android`
- Analyze dependencies for vulnerabilities or deprecations with `cargo deny`
  - Automatically open PRs to update dependencies and flake with `renovate`
- Test the tests’ comprehensiveness by introducing bugs they should catch
  - **Mutation testing** with `cargo-mutants`
- Publish report(s) of the above checks accessible via the forge

## Licensing and Credits

[LogOut] is licensed under the GPL-3.0, see [LICENSE](LICENSE) for details.

The [exercise database] is under the Unlicense (public domain).

[LogOut]: https://gfauredev.github.io/LogOut
[800+ exercises]: https://gfauredev.github.io/free-exercise-db
[old 800+ exercises]: https://github.com/yuhonas/free-exercise-db
[Cargo]: https://doc.rust-lang.org/cargo
[cargo test]: https://doc.rust-lang.org/cargo/commands/cargo-test.html
[cargo-llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov
[Clippy]: https://github.com/rust-lang/rust-clippy
[Conventional Commits]: https://www.conventionalcommits.org
[Conventional Branch]: https://conventional-branch.github.io
[Dioxus]: https://dioxuslabs.com
[dx]: https://dioxuslabs.com
[direnv]: https://direnv.net
[`direnv`]: https://direnv.net
[exercise database]: https://gfauredev.github.io/free-exercise-db
[exercise db]: https://gfauredev.github.io/free-exercise-db
[free-exercise-db]: https://gfauredev.github.io/free-exercise-db
[old free-exercise-db]: https://github.com/yuhonas/free-exercise-db
[Guilhem Fauré]: https://www.guilhemfau.re
[Git]: https://git-scm.com
[GitHub Flow]: https://githubflow.github.io
[Helix]: https://helix-editor.com
[lcov]: https://github.com/linux-test-project/lcov
[lldb]: https://lldb.llvm.org
[llvm-cov]: https://llvm.org/docs/CommandGuide/llvm-cov.html
[Maestro]: https://maestro.dev
[Nix]: https://nixos.org
[pwa]: https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps
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
