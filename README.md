# LogOut
Turn off your computer, Log your workOut

A cross-platform workout logging application built with Dioxus 0.7. The app includes an exercise database with 873+ exercises from [free-exercise-db](https://github.com/yuhonas/free-exercise-db), processed at build time for optimal performance.

## Features

- 🏋️ Browse 873+ exercises with search functionality
- 💪 Log workouts with sets, reps, and weights
- 📱 Mobile-first responsive design
- 🌐 Cross-platform (Web, with Android support planned)
- 💾 Exercise database downloaded and embedded at build time for optimal performance
- 🖼️ Exercise images loaded from remote CDN with lazy loading
- 🎨 Modern, gradient-based UI

## Building

### Prerequisites

- Rust (latest stable)
- wasm-bindgen-cli: `cargo install wasm-bindgen-cli --version 0.2.108`
- wasm32 target: `rustup target add wasm32-unknown-unknown`
- curl or wget (for downloading exercise database during build)

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
- `build.rs` - Build script that downloads and embeds the exercise database

## Build Process

The exercise database is automatically downloaded and processed at build time using a Cargo build script (`build.rs`):

1. The build script downloads the latest `exercises.json` from [free-exercise-db](https://github.com/yuhonas/free-exercise-db/blob/main/dist/exercises.json)
2. Validates the JSON format
3. Generates Rust code with the JSON embedded as a static string constant
4. The generated code is included in `src/services/exercise_db.rs` at compile time

This approach provides:
- Always up-to-date exercise database (downloaded from source at build time)
- Build-time validation of the exercise database
- No local asset storage required
- Same performance as direct embedding (data is still compiled into the binary)
- Reduced repository size

### Exercise Images

Exercise images are not stored locally. Instead, they are loaded on-demand from the free-exercise-db GitHub repository:
- Base URL: `https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/`
- Images use lazy loading for optimal performance
- Browser caching provides offline functionality for previously viewed images

## Exercise Database

The app uses the excellent [free-exercise-db](https://github.com/yuhonas/free-exercise-db) which provides:
- 873+ exercises with detailed instructions
- Exercise images for visual reference
- Exercise categories (strength, stretching, cardio, etc.)
- Primary and secondary muscle groups
- Equipment requirements
- Difficulty levels

The database is downloaded at build time and embedded into the application binary for optimal performance. Images are loaded on-demand from the GitHub CDN with browser caching for offline functionality.

