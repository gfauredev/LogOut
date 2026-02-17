# LogOut
Turn off your computer, Log your workOut

A cross-platform workout logging application built with Dioxus 0.7. The app includes an exercise database with 873+ exercises from [free-exercise-db](https://github.com/yuhonas/free-exercise-db), processed at build time for optimal performance.

## Features

- 🏋️ Browse 873+ exercises with search functionality
- 💪 Log workouts with sets, reps, and weights
- 📱 Mobile-first responsive design
- 🌐 Cross-platform (Web, with Blitz support planned)
- 💾 Exercise database downloaded and embedded at build time for optimal performance
- 🖼️ Exercise images loaded from remote CDN with lazy loading
- 🎨 Modern, gradient-based UI
- 🔌 **Blitz-Ready**: Can run without JavaScript for native platforms

## Building

### Prerequisites

- Rust (latest stable)
- wasm-bindgen-cli: `cargo install wasm-bindgen-cli --version 0.2.108`
- wasm32 target: `rustup target add wasm32-unknown-unknown`
- curl or wget (for downloading exercise database during build)

### Build for Web (with Service Worker)

```bash
# Build the wasm binary with default features (includes Service Worker)
cargo build --target wasm32-unknown-unknown --release

# Generate wasm bindings
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/release/logout.wasm

# Serve the app (requires a local web server)
python3 -m http.server 8000
# Then open http://localhost:8000 in your browser
```

### Build for Blitz/Native (without JavaScript)

When Blitz becomes production-ready, it will use native targets instead of WASM. For now, to test Blitz-compatible mode (without Service Worker):

```bash
# Build without Service Worker for Blitz compatibility testing
cargo build --target wasm32-unknown-unknown --release --no-default-features

# When Blitz is production-ready, the command will be similar to:
# cargo build --release --no-default-features
# (using native target instead of wasm32-unknown-unknown)
```

The `--no-default-features` flag disables the `web-platform` feature, which removes Service Worker registration. The app runs perfectly fine without it - images are simply fetched from the network without offline caching.

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
- Service Worker caching provides offline functionality for previously viewed images

**Note on Service Worker Implementation:**
- The Service Worker registration is implemented in Rust (`src/services/service_worker.rs`) following Dioxus best practices
- The Service Worker script itself (`sw.js`) must remain as JavaScript because Service Workers run in a separate browser context outside the WASM application
- This is a browser architecture limitation, not a framework choice

## Exercise Database

The app uses the excellent [free-exercise-db](https://github.com/yuhonas/free-exercise-db) which provides:
- 873+ exercises with detailed instructions
- Exercise images for visual reference
- Exercise categories (strength, stretching, cardio, etc.)
- Primary and secondary muscle groups
- Equipment requirements
- Difficulty levels

The database is downloaded at build time and embedded into the application binary for optimal performance. Images are loaded on-demand from the GitHub CDN.

### Service Worker Architecture

The Service Worker (`sw.js`) provides offline caching for exercise images:
- **Registration**: Implemented in Rust (`src/services/service_worker.rs`) following Dioxus best practices
- **Worker Script**: Must be JavaScript due to browser architecture - Service Workers run in a separate context outside the WASM application
- **Functionality**: Caches images from the GitHub CDN for offline access

This hybrid approach maximizes the use of Rust while respecting browser API limitations.

### Platform Compatibility

### Web Platform (Default)
- Full functionality including Service Worker for offline caching
- Build with: `cargo build --target wasm32-unknown-unknown --release`

### Blitz/Native Platforms (Future)
- **Blitz-Ready**: The app can run without JavaScript
- Service Worker is disabled via feature flags
- Images are fetched from network (no offline caching)
- Build for testing Blitz-compatible mode: `cargo build --target wasm32-unknown-unknown --release --no-default-features`
- When Blitz is production-ready: `cargo build --release --no-default-features` (using native target)

**Note**: Blitz is a new Dioxus renderer for native desktop/mobile that doesn't use JavaScript. This app is architecturally ready for Blitz - the Service Worker is the only JavaScript dependency and is already optional.

### Future Native Enhancements
When targeting Blitz or native platforms, offline caching could be implemented using:
- Platform-specific HTTP caching
- Local file system storage
- Native image caching libraries

These would replace the Service Worker functionality without requiring JavaScript.

