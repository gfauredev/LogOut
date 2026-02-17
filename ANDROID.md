# Android Build Instructions

## Current Status

The app is currently configured for **Web** deployment, which works on mobile browsers. For native Android deployment, additional setup is required.

## Web Deployment (Mobile-Friendly)

The current build works on mobile browsers with responsive design. To deploy:

1. Build the project: `./build.sh`
2. Upload the `dist/` folder and `index.html` to any web host
3. Access from any mobile browser

## Future: Native Android Build

To build as a native Android app using Dioxus 0.7, you would need:

### Prerequisites
- Android SDK and NDK installed
- Java Development Kit (JDK) 11 or higher
- Dioxus CLI: `cargo install dioxus-cli`

### Configuration
Update `Dioxus.toml`:
```toml
[application]
default_platform = "android"

[bundle.android]
min_sdk_version = 24
target_sdk_version = 35
```

### Build Command
```bash
dx build --platform android --release
```

This will generate an APK file in `target/android/release/`.

## Progressive Web App (PWA)

For a better mobile experience without native builds, consider adding:
- Service Worker for offline support
- Web App Manifest for "Add to Home Screen" functionality

## Notes

- The exercise database is embedded at compile time, ensuring offline functionality
- Current web build is ~2MB (including all 873 exercises)
- For production, consider optimizing WASM size with `wasm-opt`
