# LogOut

Turn off your computer, Log your workOut

A cross-platform workout logging application built with Dioxus 0.7 and compiled to WebAssembly.

The app initially populates its exercise database with 873+ exercises from
[free-exercise-db](https://github.com/yuhonas/free-exercise-db).

## Features

- 🏋️ Browse 873+ exercises with search functionality
- 💪 Log workout sessions with sets, reps, weights, distances, and durations
- 📊 **Analytics panel** with line charts to track progress over time
- 📱 Mobile-first responsive design with a bottom navigation bar
- 🔌 **Offline-ready PWA** with service worker caching
- 💾 Exercise database embedded at build time for instant search
- 🖼️ Exercise images lazy-loaded from a remote CDN

## Structure

```
src/
  main.rs              # Application entry point and routing
  models/              # Data models: exercises, sessions, sets, enums
  services/            # Business logic: exercise DB, storage (IndexedDB), service worker
  components/          # UI components: home, exercise list, session view, analytics
  utils.rs             # Pure utility functions (date formatting, etc.)
e2e/
  app.spec.ts          # Playwright end-to-end tests
assets/
  styles.css           # Application stylesheet
public/
  manifest.json        # PWA manifest
  sw.js                # Service worker (JavaScript — required by the browser SW spec)
```

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| [Rust stable](https://rustup.rs) | Compile the application | `rustup install stable` |
| `wasm32-unknown-unknown` target | Cross-compile to WebAssembly | `rustup target add wasm32-unknown-unknown` |
| [Dioxus CLI (`dx`)](https://dioxuslabs.com/learn/0.6/getting_started) | Build and serve the web app | `cargo install dioxus-cli` |
| [Node.js ≥ 20](https://nodejs.org) + npm | Run Playwright E2E tests | — |

## Build

### Development server (with hot-reload)

```sh
dx serve
```

The app is served at `http://localhost:8080/LogOut/`.

### Production build (PWA)

```sh
dx build --platform web --release
```

Output is written to `target/dx/log-workout/release/web/public/`.

### GitHub Pages deployment

The production PWA is deployed automatically on every push to `main` via
`.github/workflows/deploy.yml`.

## Testing

### Unit tests

Unit tests cover pure-Rust model functions (formatting, parsing, serialization),
service stubs, and utility helpers. They compile and run on the native target —
no browser or WASM toolchain required.

```sh
cargo test
```

All unit tests must pass.

#### Unit test coverage

Install `cargo-llvm-cov` once:

```sh
cargo install cargo-llvm-cov
```

Print a summary inline:

```sh
cargo llvm-cov --bin log-workout
```

Generate an LCOV report:

```sh
cargo llvm-cov --bin log-workout --lcov --output-path lcov.info
```

### E2E tests (Playwright)

End-to-end tests exercise the full application running in Chromium via
Playwright. They start `dx serve` automatically before the test run.

Install dependencies once:

```sh
npm install
npx playwright install --with-deps chromium
```

Run the tests:

```sh
npx playwright test
```

Run with a visible browser (useful for debugging):

```sh
npx playwright test --headed
```

All E2E tests must pass. When a test fails, Playwright captures a screenshot
automatically and saves it to `test-results/`.

## Code Quality Requirements

Every pull request is validated by `.github/workflows/ci.yml`.
All five jobs must pass before a PR can be merged.
GitHub automatically holds runs from first-time contributors for maintainer
approval before any code is executed.

| Job | Command | Requirement |
|-----|---------|-------------|
| **Formatting** | `cargo fmt --check` | Code must match `rustfmt` style exactly |
| **Linting** | `cargo clippy -- -D warnings` | Zero Clippy warnings |
| **Unit tests** | `cargo llvm-cov …` | All unit tests pass; coverage report posted as PR comment |
| **E2E tests** | `npx playwright test` | All Playwright tests pass; failure screenshots posted as PR comment |
| **PageSpeed** | Lighthouse CLI | Performance scores posted as PR comment |

### Run all quality checks locally

```sh
# Formatting
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Unit tests
cargo test

# E2E tests (starts the dev server automatically)
npx playwright test
```

Permissions used by the CI workflow: `contents: read`, `pull-requests: write`.

