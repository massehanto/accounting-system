// frontend/public/sw.js
const CACHE_NAME = 'accsys-v1.0.2';
const STATIC_CACHE = 'accsys-static-v1';
const DYNAMIC_CACHE = 'accsys-dynamic-v1';
const API_CACHE = 'accsys-api-v1';

const urlsToCache = [
  '/',
  '/pkg/frontend.wasm',
  '/pkg/frontend.js',
  '/assets/styles/global.css',
  'https://cdn.tailwindcss.com',
  'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap',
  '/manifest.json'
];

// Network-first strategy for API calls
const apiPaths = ['/api/'];

// Cache-first strategy for static assets
const staticPaths = ['/pkg/', '/assets/', '/icons/'];

self.addEventListener('install', event => {
  event.waitUntil(
    Promise.all([
      caches.open(STATIC_CACHE).then(cache => cache.addAll(urlsToCache)),
      self.skipWaiting()
    ])
  );
});

self.addEventListener('activate', event => {
  event.waitUntil(
    Promise.all([
      caches.keys().then(cacheNames => {
        return Promise.all(
          cacheNames.map(cacheName => {
            if (![CACHE_NAME, DYNAMIC_CACHE, API_CACHE, STATIC_CACHE].includes(cacheName)) {
              return caches.delete(cacheName);
            }
          })
        );
      }),
      self.clients.claim()
    ])
  );
});

self.addEventListener('fetch', event => {
  const { request } = event;
  
  // Handle API calls with network-first strategy
  if (apiPaths.some(path => request.url.includes(path))) {
    event.respondWith(networkFirstStrategy(request, API_CACHE));
    return;
  }
  
  // Handle static assets with cache-first strategy
  if (staticPaths.some(path => request.url.includes(path))) {
    event.respondWith(cacheFirstStrategy(request, STATIC_CACHE));
    return;
  }
  
  // Default strategy for other requests
  event.respondWith(staleWhileRevalidateStrategy(request, DYNAMIC_CACHE));
});

async function networkFirstStrategy(request, cacheName) {
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(cacheName);
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    const cachedResponse = await caches.match(request);
    return cachedResponse || new Response('Offline', { status: 503 });
  }
}

async function cacheFirstStrategy(request, cacheName) {
  const cachedResponse = await caches.match(request);
  if (cachedResponse) {
    return cachedResponse;
  }
  
  try {
    const response = await fetch(request);
    if (response.ok) {
      const cache = await caches.open(cacheName);
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    return new Response('Offline', { status: 503 });
  }
}

async function staleWhileRevalidateStrategy(request, cacheName) {
  const cachedResponse = await caches.match(request);
  
  const fetchPromise = fetch(request).then(response => {
    if (response.ok) {
      const cache = caches.open(cacheName);
      cache.then(c => c.put(request, response.clone()));
    }
    return response;
  }).catch(() => cachedResponse);
  
  return cachedResponse || fetchPromise;
}

// Background sync for offline actions
self.addEventListener('sync', event => {
  if (event.tag === 'background-sync') {
    event.waitUntil(syncOfflineActions());
  }
});

async function syncOfflineActions() {
  // This would integrate with your offline action queue
  console.log('Background sync triggered');
}