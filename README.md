# LogOut
Turn off your computer, Log your workOut

A cross-platform workout logging application built with Dioxus 0.7. The app includes an embedded exercise database with 800+ exercises from [free-exercise-db](https://github.com/yuhonas/free-exercise-db).

## Features

- 🏋️ Browse 873+ exercises with search functionality
- 💪 Log workouts with sets, reps, and weights
- 📱 Mobile-first responsive design
- 🌐 Cross-platform (Web, with Android support planned)
- 💾 Exercise database embedded at build time for optimal performance
- 🎨 Modern, gradient-based UI

## Building

### Prerequisites

- Rust (latest stable)
- wasm-bindgen-cli: `cargo install wasm-bindgen-cli --version 0.2.108`
- wasm32 target: `rustup target add wasm32-unknown-unknown`

### Build for Web

```bash
# Build the wasm binary
cargo build --target wasm32-unknown-unknown --release

# Generate wasm bindings
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/release/logout.wasm

# Serve the app (requires a local web server)
python3 -m http.server 8000
# Then open http://localhost:8000 in your browser
```

## Project Structure

- `src/models/` - Data models for exercises and workouts
- `src/services/` - Business logic (exercise database, storage)
- `src/components/` - UI components (home, exercise list, workout log)
- `assets/` - Exercise database JSON (embedded at compile time)

## Exercise Database

The app uses the excellent [free-exercise-db](https://github.com/yuhonas/free-exercise-db) which provides:
- 873+ exercises with detailed instructions
- Exercise categories (strength, stretching, cardio, etc.)
- Primary and secondary muscle groups
- Equipment requirements
- Difficulty levels

The database is embedded into the application binary at build time for optimal performance and offline functionality.

