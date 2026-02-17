// Service Worker for caching exercise images
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

const CACHE_NAME = 'workout-images-v1';
const IMAGE_BASE_URL = 'https://raw.githubusercontent.com/yuhonas/free-exercise-db/main/exercises/';

// Install event - set up the cache
self.addEventListener('install', (event) => {
  console.log('Service Worker: Installing...');
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('Service Worker: Activating...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames.map((cacheName) => {
          if (cacheName !== CACHE_NAME) {
            console.log('Service Worker: Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    })
  );
  return self.clients.claim();
});

// Fetch event - cache images from the exercise database
self.addEventListener('fetch', (event) => {
  const url = event.request.url;
  
  // Only cache exercise images
  if (url.startsWith(IMAGE_BASE_URL)) {
    event.respondWith(
      caches.open(CACHE_NAME).then((cache) => {
        return cache.match(event.request).then((cachedResponse) => {
          if (cachedResponse) {
            // Return cached version
            return cachedResponse;
          }
          
          // Fetch from network and cache
          return fetch(event.request).then((response) => {
            // Only cache successful responses
            if (response && response.status === 200) {
              // Clone the response before caching
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
  }
});
