const CACHE_NAME = 'accsys-v1.0.1';
const STATIC_CACHE = 'accsys-static-v1';
const DYNAMIC_CACHE = 'accsys-dynamic-v1';

const urlsToCache = [
  '/',
  '/pkg/frontend.wasm', // Updated to match actual output
  '/pkg/frontend.js',   // Updated to match actual output
  '/assets/styles/global.css', // Fixed path
  'https://cdn.tailwindcss.com',
  'https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap',
  // PWA essentials
  '/manifest.json', // Fixed path
  '/icons/icon-192x192.png', // Fixed path
  '/icons/icon-512x512.png'  // Fixed path
];

// Rest of the service worker code remains the same...