/**
 * Splitify Service Worker
 * 
 * Modern PWA service worker with:
 * - Stale-while-revalidate strategy for assets
 * - Network-first strategy for API calls
 * - Cache-first strategy for static assets
 * - Offline fallback support
 */

const CACHE_VERSION = 'v1';
const CACHE_NAME = `splitify-${CACHE_VERSION}`;
const OFFLINE_PAGE = '/offline';

// Assets to precache on install
const PRECACHE_ASSETS = [
  '/',
  '/offline',
  '/pkg/rustify-app.css',
  '/pkg/rustify-app.js',
  '/pkg/rustify-app_bg.wasm',
  '/favicon.svg',
  '/favicon-192x192.png',
  '/favicon-384x384.png',
  '/favicon-512x512.png',
  'https://fonts.bunny.net/css?family=figtree:400,500,600&display=swap'
];

// URLs that should always be fetched from network
const NETWORK_ONLY_PATTERNS = [
  /\/api\//,
  /\/auth\//,
  /__reload/
];

// URLs that should use cache-first strategy
const CACHE_FIRST_PATTERNS = [
  /\.wasm$/,
  /\.css$/,
  /\.js$/,
  /\.png$/,
  /\.jpg$/,
  /\.svg$/,
  /fonts\./
];

/**
 * Install event - precache essential assets
 */
self.addEventListener('install', (event) => {
  console.log('[SW] Installing service worker...');
  
  event.waitUntil(
    caches.open(CACHE_NAME)
      .then((cache) => {
        console.log('[SW] Precaching assets');
        // Don't fail installation if some assets fail to cache
        return Promise.allSettled(
          PRECACHE_ASSETS.map(url => 
            cache.add(url).catch(err => {
              console.warn(`[SW] Failed to cache ${url}:`, err);
            })
          )
        );
      })
      .then(() => {
        console.log('[SW] Installation complete');
        // Force the waiting service worker to become active
        return self.skipWaiting();
      })
  );
});

/**
 * Activate event - cleanup old caches
 */
self.addEventListener('activate', (event) => {
  console.log('[SW] Activating service worker...');
  
  event.waitUntil(
    caches.keys()
      .then((cacheNames) => {
        return Promise.all(
          cacheNames
            .filter(cacheName => cacheName.startsWith('splitify-') && cacheName !== CACHE_NAME)
            .map(cacheName => {
              console.log(`[SW] Deleting old cache: ${cacheName}`);
              return caches.delete(cacheName);
            })
        );
      })
      .then(() => {
        console.log('[SW] Activation complete');
        // Take control of all clients immediately
        return self.clients.claim();
      })
  );
});

/**
 * Fetch event - handle network requests with different strategies
 */
self.addEventListener('fetch', (event) => {
  const { request } = event;
  const url = new URL(request.url);

  // Skip cross-origin requests except fonts
  if (url.origin !== location.origin && !url.hostname.includes('fonts')) {
    return;
  }

  // Network-only for specific patterns
  if (NETWORK_ONLY_PATTERNS.some(pattern => pattern.test(url.pathname))) {
    event.respondWith(fetch(request));
    return;
  }

  // Cache-first for static assets
  if (CACHE_FIRST_PATTERNS.some(pattern => pattern.test(url.pathname))) {
    event.respondWith(cacheFirst(request));
    return;
  }

  // Network-first with offline fallback for HTML pages
  if (request.mode === 'navigate') {
    event.respondWith(networkFirstWithOffline(request));
    return;
  }

  // Stale-while-revalidate for everything else
  event.respondWith(staleWhileRevalidate(request));
});

/**
 * Cache-first strategy
 * Try cache first, fallback to network
 */
async function cacheFirst(request) {
  const cache = await caches.open(CACHE_NAME);
  const cached = await cache.match(request);
  
  if (cached) {
    console.log(`[SW] Serving from cache: ${request.url}`);
    return cached;
  }
  
  try {
    const response = await fetch(request);
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    console.error(`[SW] Cache-first fetch failed for ${request.url}:`, error);
    throw error;
  }
}

/**
 * Network-first strategy with offline fallback
 * Try network first, fallback to cache, then offline page
 */
async function networkFirstWithOffline(request) {
  const cache = await caches.open(CACHE_NAME);
  
  try {
    const response = await fetch(request);
    if (response.ok) {
      cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    console.log(`[SW] Network failed, trying cache: ${request.url}`);
    
    const cached = await cache.match(request);
    if (cached) {
      return cached;
    }
    
    // Return offline page for navigation requests
    if (request.mode === 'navigate') {
      const offlinePage = await cache.match(OFFLINE_PAGE);
      if (offlinePage) {
        return offlinePage;
      }
    }
    
    throw error;
  }
}

/**
 * Stale-while-revalidate strategy
 * Return cached response immediately, update cache in background
 */
async function staleWhileRevalidate(request) {
  const cache = await caches.open(CACHE_NAME);
  const cached = await cache.match(request);
  
  // Fetch and update cache in background
  const fetchPromise = fetch(request)
    .then(response => {
      if (response.ok) {
        cache.put(request, response.clone());
      }
      return response;
    })
    .catch(error => {
      console.warn(`[SW] Background fetch failed for ${request.url}:`, error);
    });
  
  // Return cached response immediately if available
  if (cached) {
    console.log(`[SW] Serving stale cache: ${request.url}`);
    return cached;
  }
  
  // Otherwise wait for network
  return fetchPromise;
}

/**
 * Message event - handle messages from clients
 */
self.addEventListener('message', (event) => {
  if (event.data && event.data.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
  
  if (event.data && event.data.type === 'CACHE_URLS') {
    const urls = event.data.urls || [];
    event.waitUntil(
      caches.open(CACHE_NAME)
        .then(cache => cache.addAll(urls))
    );
  }
});
