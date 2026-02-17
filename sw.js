// Service Worker for caching exercise images
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
