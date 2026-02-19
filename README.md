---
lang: en
---

<!--toc:start-->

- [LogOut](#logout)
  - [Structure](#structure)
  - [Development Server (with hot-reload)](#development-server-with-hot-reload)
  - [Web Build (PWA)](#web-build-pwa)
    - [GitHub Pages deployment](#github-pages-deployment)
  - [Unit Testing](#unit-testing)
  - [End-to-End Testing](#end-to-end-testing)
  - [Code Quality Standards](#code-quality-standards)
    - [Run quality checks locally](#run-quality-checks-locally)

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

## Structure

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

## Development Server (with hot-reload)

```sh
dx serve # Serves at http://localhost:8080
```

## Web Build (PWA)

```sh
dx build --platform web --release
```

Output is written to `target/dx/log-workout/release/web/public/`.

### GitHub Pages deployment

The PWA is deployed automatically on every push to `main` via
`.github/workflows/deploy.yml` on `https://gfauredev.github.io/LogOut`.

## Unit Testing

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

## End-to-End Testing

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

## Code Quality Standards

Every pull request is validated by `.github/workflows/ci.yml`, ensuring that all
five jobs pass before a PR can be merged.

| Job            | Command                       | Requirement                                                 |
| -------------- | ----------------------------- | ----------------------------------------------------------- |
| **Formatting** | `cargo fmt --check`           | Code must match `rustfmt` style exactly                     |
| **Linting**    | `cargo clippy -- -D warnings` | Zero Clippy warnings                                        |
| **Unit tests** | `cargo llvm-cov …`            | All unit tests pass, covering more than 90% of the codebase |
| **E2E tests**  | `npx playwright test`         | All Playwright tests pass                                   |
| **PageSpeed**  | Lighthouse CLI                | Performance scores posted as PR comment                     |

### Run quality checks locally

```sh
cargo fmt --check # Formatting
cargo clippy -- -D warnings # Linting
cargo test # Unit tests
npx playwright test # E2E tests (starts dev server)
```

[Guilhem Fauré]: https://www.guilhemfau.re
[free-exercise-db]: https://github.com/yuhonas/free-exercise-db
[800+ exercises]: https://github.com/yuhonas/free-exercise-db
[Rust]: https://rust-lang.org
[rust]: https://rust-lang.org
[Dioxus]: https://dioxuslabs.com
[dioxuslabs]: https://dioxuslabs.com
[Node.js]: https://nodejs.org
[Playwright]: https://playwright.dev
