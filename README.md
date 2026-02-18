# LogOut

Turn off your computer, Log your workOut

A cross-platform workout logging application built with Dioxus 0.7.

The app initially populates its exercise database with 873+ exercises from
[free-exercise-db](https://github.com/yuhonas/free-exercise-db).

## Features

- 🏋️ Browse 873+ exercises with search functionality
- 💪 Log workouts with sets, reps, and weights
- 📊 **Analytics panel** with line charts to track progress over time
- 📱 Mobile-first responsive design
- 🌐 Cross-platform (Web, with Blitz support planned)
- 💾 Exercise database downloaded and embedded at build time for optimal performance
- 🖼️ Exercise images loaded from remote CDN with lazy loading
- 🎨 Modern, gradient-based UI
- 🔌 **Blitz-Ready**: Can run without JavaScript for native platforms

## Structure

- `src/models/` Data models for exercises and workouts
- `src/services/` Business logic (exercise database, storage)
- `src/components/` UI components (home, exercise list, workout log)

## Exercise Data

The app initially populates its exercise database with 873+ exercises from
[free-exercise-db](https://github.com/yuhonas/free-exercise-db), which provides:

- 873+ exercises with detailed instructions
- Exercise images for visual reference
- Exercise categories (strength, stretching, cardio, etc.)
- Primary and secondary muscle groups
- Equipment requirements
- Difficulty levels

Images are downloaded lazily as exercises are viewed.

## Build for Web (PWA with Service Worker)

`dx build --platform web --release`

…

### GitHub Pages Deployment

The PWA version of LogOut is deployed on GitHub Pages automatically at each commit on `main`.

## Build for Blitz "Dioxus Native" (without JavaScript)

…

## Testing

Unit tests cover the pure-Rust model functions (formatting, parsing, data structures) and the
non-wasm service stubs. They run on the native target and require no browser.

### Run tests

```sh
cargo test --bin log-workout
```

**All tests must pass before merging to `main`.**
The CI workflow (`.github/workflows/tests.yml`) enforces this on every push and pull request.

### Run tests with coverage

Install `cargo-llvm-cov` once:

```sh
cargo install cargo-llvm-cov
```

Then generate an LCOV report:

```sh
cargo llvm-cov --bin log-workout --lcov --output-path lcov.info
```

Or view an inline summary in the terminal:

```sh
cargo llvm-cov --bin log-workout
```
