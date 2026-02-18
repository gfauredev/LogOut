// Service Worker for offline PWA support and exercise image caching
//
// IMPORTANT: This file must remain as JavaScript
//
// Service Workers run in a separate browser context and must be JavaScript files.
// While Dioxus recommends implementing logic in Rust, Service Workers are a special case:
// - They run in a separate worker context (not in the main WASM module)
// - They intercept network requests before they reach the WASM application
// - They use Service Worker-specific APIs only available in the worker context
// - They must be standalone JavaScript files that the browser can execute independently
//
// The registration of this Service Worker is handled in Rust (src/services/service_worker.rs)
// following Dioxus best practices - only the worker script itself must be JavaScript.
//
// BLITZ COMPATIBILITY NOTE:
// This file is NOT used when building for Dioxus Blitz or other non-web platforms.
// Blitz doesn't have a JavaScript engine, so Service Worker functionality is disabled
// via feature flags. The app works perfectly fine without offline caching.

const CACHE_VERSION = 'v2';
const APP_CACHE_NAME = 'logout-app-' + CACHE_VERSION;
const IMAGE_CACHE_NAME = 'workout-images-v1';
const IMAGE_BASE_URL = 'https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/';

// App shell assets to pre-cache for offline use
const APP_SHELL_URLS = [
  './',
  './manifest.json',
];

// Install event - pre-cache the app shell
self.addEventListener('install', (event) => {
  console.log('Service Worker: Installing...');
  event.waitUntil(
    caches.open(APP_CACHE_NAME).then((cache) => {
      return cache.addAll(APP_SHELL_URLS).catch((err) => {
        console.warn('Service Worker: Could not pre-cache some shell assets:', err);
      });
    })
  );
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('Service Worker: Activating...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== APP_CACHE_NAME && cacheName !== IMAGE_CACHE_NAME) {
            console.log('Service Worker: Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
  return self.clients.claim();
});

// Fetch event - serve from cache with appropriate strategy
self.addEventListener('fetch', (event) => {
  const url = event.request.url;

  // Exercise images: cache-first (immutable CDN assets)
  if (url.startsWith(IMAGE_BASE_URL)) {
    event.respondWith(
      caches.open(IMAGE_CACHE_NAME).then((cache) => {
        return cache.match(event.request).then((cachedResponse) => {
          if (cachedResponse) {
            return cachedResponse;
          }
          return fetch(event.request).then((response) => {
            if (response && response.status === 200) {
              cache.put(event.request, response.clone());
            }
            return response;
          }).catch((error) => {
            console.error('Service Worker: Fetch failed for', url, error);
            throw error;
          });
        });
      })
    );
    return;
  }

  // Same-origin app assets: network-first, fall back to cache
  if (url.startsWith(self.location.origin)) {
    event.respondWith(
      fetch(event.request).then((response) => {
        // Cache successful GET responses for offline fallback
        if (response && response.status === 200 && event.request.method === 'GET') {
          const responseClone = response.clone();
          caches.open(APP_CACHE_NAME).then((cache) => {
            cache.put(event.request, responseClone);
          });
        }
        return response;
      }).catch(() => {
        // Network failed – serve from cache
        return caches.match(event.request).then((cachedResponse) => {
          if (cachedResponse) {
            return cachedResponse;
          }
          // Fallback to app root for navigation requests (SPA offline support)
          if (event.request.mode === 'navigate') {
            return caches.match('./');
          }
        });
      })
    );
  }
});
