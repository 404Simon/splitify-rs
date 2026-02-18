/**
 * Service Worker Registration Script
 * Registers the service worker with update detection and notification
 */

// Check if service workers are supported
if ('serviceWorker' in navigator) {
  // Register on page load
  window.addEventListener('load', () => {
    registerServiceWorker();
  });
}

async function registerServiceWorker() {
  try {
    const registration = await navigator.serviceWorker.register('/sw.js', {
      scope: '/',
    });

    console.log('[PWA] Service worker registered successfully:', registration.scope);

    // Check for updates periodically
    setInterval(() => {
      registration.update();
    }, 60000); // Check every minute

    // Handle service worker updates
    registration.addEventListener('updatefound', () => {
      const newWorker = registration.installing;
      
      if (!newWorker) return;

      newWorker.addEventListener('statechange', () => {
        if (newWorker.state === 'installed' && navigator.serviceWorker.controller) {
          // New service worker is available
          showUpdateNotification(newWorker);
        }
      });
    });

    // Handle controller change (new SW activated)
    navigator.serviceWorker.addEventListener('controllerchange', () => {
      console.log('[PWA] New service worker activated, reloading page...');
      window.location.reload();
    });

  } catch (error) {
    console.error('[PWA] Service worker registration failed:', error);
  }
}

function showUpdateNotification(newWorker) {
  // Check if user wants to see update notifications
  const hideNotifications = localStorage.getItem('pwa-hide-update-notifications');
  if (hideNotifications === 'true') return;

  // Create update notification
  const notification = document.createElement('div');
  notification.id = 'pwa-update-notification';
  notification.className = 'fixed bottom-4 right-4 bg-indigo-600 text-white rounded-lg shadow-2xl p-4 max-w-sm z-50 transform transition-all duration-300 ease-in-out';
  notification.innerHTML = `
    <div class="flex items-start">
      <div class="flex-shrink-0">
        <svg class="h-6 w-6 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
        </svg>
      </div>
      <div class="ml-3 flex-1">
        <p class="text-sm font-medium">New version available!</p>
        <p class="mt-1 text-sm text-indigo-200">Click update to get the latest features.</p>
        <div class="mt-3 flex space-x-2">
          <button id="pwa-update-btn" class="bg-white text-indigo-600 px-3 py-1 rounded text-sm font-semibold hover:bg-indigo-50 transition">
            Update Now
          </button>
          <button id="pwa-dismiss-btn" class="bg-indigo-700 text-white px-3 py-1 rounded text-sm hover:bg-indigo-800 transition">
            Later
          </button>
        </div>
      </div>
      <button id="pwa-close-btn" class="ml-4 flex-shrink-0 text-indigo-200 hover:text-white">
        <svg class="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
          <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
        </svg>
      </button>
    </div>
  `;

  document.body.appendChild(notification);

  // Update button handler
  document.getElementById('pwa-update-btn').addEventListener('click', () => {
    newWorker.postMessage({ type: 'SKIP_WAITING' });
    notification.remove();
  });

  // Dismiss button handler
  document.getElementById('pwa-dismiss-btn').addEventListener('click', () => {
    notification.remove();
  });

  // Close button handler
  document.getElementById('pwa-close-btn').addEventListener('click', () => {
    notification.remove();
  });

  // Animate in
  setTimeout(() => {
    notification.style.transform = 'translateY(0)';
  }, 100);
}

// Handle offline/online events
window.addEventListener('online', () => {
  console.log('[PWA] Back online');
  showToast('You are back online!', 'success');
});

window.addEventListener('offline', () => {
  console.log('[PWA] Gone offline');
  showToast('You are offline. Some features may be limited.', 'warning');
});

function showToast(message, type = 'info') {
  const toast = document.createElement('div');
  toast.className = `fixed top-4 right-4 px-4 py-3 rounded-lg shadow-lg z-50 transform transition-all duration-300 ${
    type === 'success' ? 'bg-green-500' : 
    type === 'warning' ? 'bg-yellow-500' : 
    type === 'error' ? 'bg-red-500' : 
    'bg-blue-500'
  } text-white`;
  toast.textContent = message;
  toast.style.transform = 'translateX(400px)';

  document.body.appendChild(toast);

  setTimeout(() => {
    toast.style.transform = 'translateX(0)';
  }, 100);

  setTimeout(() => {
    toast.style.transform = 'translateX(400px)';
    setTimeout(() => toast.remove(), 300);
  }, 3000);
}
