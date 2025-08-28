const CACHE_NAME = 'punch-clock-v1';
const OFFLINE_URL = '/punch-clock/offline';

// Cache essential assets
const CACHE_URLS = [
  '/punch-clock/',
  '/punch-clock/static/manifest.json',
  OFFLINE_URL
];

// Install event - cache essential resources
self.addEventListener('install', event => {
  console.log('Service Worker installing...');
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then(cache => {
        console.log('Opened cache');
        return cache.addAll(CACHE_URLS);
      })
      .then(() => {
        // Force the waiting service worker to become the active service worker
        return self.skipWaiting();
      })
  );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
  console.log('Service Worker activating...');
  event.waitUntil(
    caches.keys().then(cacheNames => {
      return Promise.all(
        cacheNames.map(cacheName => {
          if (cacheName !== CACHE_NAME) {
            console.log('Deleting old cache:', cacheName);
            return caches.delete(cacheName);
          }
        })
      );
    }).then(() => {
      // Claim all clients immediately
      return self.clients.claim();
    })
  );
});

// Fetch event - serve from cache when offline
self.addEventListener('fetch', event => {
  // Skip non-GET requests
  if (event.request.method !== 'GET') {
    return;
  }

  // Skip cross-origin requests
  if (!event.request.url.startsWith(self.location.origin)) {
    return;
  }

  // Handle punch-clock app requests
  if (event.request.url.includes('/punch-clock/')) {
    event.respondWith(
      caches.match(event.request)
        .then(response => {
          // Return cached version if available
          if (response) {
            return response;
          }

          // Try to fetch from network
          return fetch(event.request)
            .then(response => {
              // Don't cache failed responses or non-OK responses
              if (!response || response.status !== 200 || response.type !== 'basic') {
                return response;
              }

              // Clone the response since it can only be consumed once
              const responseToCache = response.clone();

              // Cache successful responses for static assets and pages
              caches.open(CACHE_NAME)
                .then(cache => {
                  cache.put(event.request, responseToCache);
                });

              return response;
            })
            .catch(() => {
              // Network failed, try to serve cached offline page for navigation requests
              if (event.request.mode === 'navigate') {
                return caches.match(OFFLINE_URL);
              }
              
              // For other requests, just return a generic offline response
              return new Response('Offline content not available', {
                status: 503,
                statusText: 'Service Unavailable',
                headers: new Headers({
                  'Content-Type': 'text/plain'
                })
              });
            });
        })
    );
  }
});

// Background sync for punch-clock data
self.addEventListener('sync', event => {
  if (event.tag === 'punch-clock-sync') {
    event.waitUntil(syncPunchClockData());
  }
});

async function syncPunchClockData() {
  // This would handle syncing clock in/out data when back online
  console.log('Syncing punch clock data...');
  // Implementation would depend on your backend sync API
}

// Push notifications (for future enhancement)
self.addEventListener('push', event => {
  if (event.data) {
    const data = event.data.json();
    const options = {
      body: data.body,
      icon: '/punch-clock/static/icons/icon-192x192.png',
      badge: '/punch-clock/static/icons/icon-192x192.png',
      tag: 'punch-clock-notification'
    };
    
    event.waitUntil(
      self.registration.showNotification(data.title, options)
    );
  }
});