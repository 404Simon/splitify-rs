/**
 * Splitify ↔ MapLibre GL JS bridge.
 *
 * This is the only place in the codebase that knows about `maplibregl`.
 * It exposes a small, imperative API on `window.SplitifyMap` that the Rust
 * WASM hydration layer drives. Keeping every MapLibre call here means the
 * Rust side stays clean and the library can be swapped without touching
 * application code.
 *
 * The worker URL is resolved automatically: MapLibre derives it from
 * `import.meta.url` (see `defaultWorkerUrl`), so the worker must be emitted
 * next to this bundle under the name `maplibre-gl-worker.mjs`.
 */

import {
  LngLatBounds,
  Map as MapLibreMap,
  Marker,
} from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import emojiData from './emoji.json';

/** Styles scoped to the glue's custom DOM (kept here so the app stays clean). */
const glueStyles = document.createElement('style');
glueStyles.textContent = `
  .splitify-temp-marker {
    width: 30px;
    height: 42px;
    cursor: grab;
    background-repeat: no-repeat;
    background-size: contain;
    filter: drop-shadow(0 3px 4px rgba(0, 0, 0, 0.35));
  }
  .splitify-temp-marker:active {
    cursor: grabbing;
  }
  /* The marker element itself is positioned by MapLibre via an inline
     transform; it must never carry a transition or scale, otherwise the
     marker slides/lags behind the camera while the map pans. */
  .splitify-marker-emoji {
    font-size: 26px;
    line-height: 1;
    cursor: pointer;
    filter: drop-shadow(0 2px 3px rgba(0, 0, 0, 0.4));
  }
  .splitify-marker-icon {
    display: inline-block;
    transition: transform 0.2s ease;
  }
  .splitify-marker-icon.splitify-marker-active {
    transform: scale(1.5);
  }
`;
document.head.appendChild(glueStyles);

/** @typedef {{ id: number, lng: number, lat: number, name: string, emoji: string, description: string, creator: string }} MarkerDto */

const DEFAULT_ZOOM = 2;

/** Follows the app's dark mode: an explicit `.dark`/`.light` class on
 *  `<html>` (set by the navbar toggle) wins over the OS preference. */
const darkQuery = window.matchMedia('(prefers-color-scheme: dark)');
const themeWatchers = new Set();

function isDarkTheme() {
  const root = document.documentElement;
  if (root.classList.contains('dark')) return true;
  if (root.classList.contains('light')) return false;
  return darkQuery.matches;
}

// Re-apply the effective style to every live map when the theme changes.
const applyThemeToAll = () => {
  for (const watcher of themeWatchers) watcher();
};
darkQuery.addEventListener('change', applyThemeToAll);
const themeObserver = new MutationObserver(applyThemeToAll);
themeObserver.observe(document.documentElement, {
  attributes: true,
  attributeFilter: ['class'],
});

/** Red pin used for the temporary "pending" marker while adding a location. */
const TEMP_PIN_URL =
  "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='30' height='42' viewBox='0 0 24 41'><path fill='%23e11d48' d='M12 0C5.4 0 0 5.4 0 12c0 9 12 29 12 29s12-20 12-29c0-6.6-5.4-12-12-12z'/><circle fill='white' cx='12' cy='12' r='5'/></svg>\")";

/** @type {Map<string, { map: maplibregl.Map, markers: Map<number, maplibregl.Marker>, tempMarker: maplibregl.Marker|null, callbacks: { onMapLoad: Function|null, onMapClick: Function|null, onMarkerClick: Function|null, onTempMarkerMoved: Function|null }, addMode: boolean }>} */
const runtimes = new Map();

function runtimeFor(containerId) {
  return runtimes.get(containerId) ?? null;
}

/**
 * Build the marker element showing the location's emoji icon.
 *
 * The outer element is what MapLibre positions (inline `transform`), so the
 * active-marker scale is applied to the inner icon instead — putting a
 * `transform` transition on the positioned element makes markers lag behind
 * the camera while it pans.
 */
function buildMarkerElement(emoji) {
  const element = document.createElement('div');
  element.className = 'splitify-marker-emoji';
  const icon = document.createElement('span');
  icon.className = 'splitify-marker-icon';
  icon.textContent = emoji;
  element.appendChild(icon);
  return element;
}

function applyAddMode(state) {
  if (!state.map) return;
  state.map.getCanvas().style.cursor = state.addMode ? 'crosshair' : '';
}

export function create(containerId, options) {
  const existing = runtimeFor(containerId);
  if (existing) return containerId;

  const center = options.center ?? [0, 0];
  const zoom = options.zoom ?? DEFAULT_ZOOM;

  const map = new MapLibreMap({
    container: containerId,
    style: isDarkTheme() ? options.darkStyleUrl : options.styleUrl,
    center,
    zoom,
    attributionControl: false,
  });

  // Attribution ("© OpenFreeMap") is rendered by the page as a subtle text link
  // instead of MapLibre's default control, which adds a clunky logo button.

  const state = {
    map,
    style: { light: options.styleUrl, dark: options.darkStyleUrl },
    markers: new Map(),
    tempMarker: null,
    callbacks: {
      onMapLoad: null,
      onMapClick: null,
      onMarkerClick: null,
      onTempMarkerMoved: null,
    },
    addMode: false,
    activeMarkerId: null,
    lastStyle: null,
    applyTheme: null,
  };
  runtimes.set(containerId, state);

  // Swap the style when the theme changes (markers are unaffected).
  state.applyTheme = () => {
    const target = isDarkTheme() ? state.style.dark : state.style.light;
    if (state.lastStyle === target) return;
    state.lastStyle = target;
    state.map.setStyle(target);
  };
  state.lastStyle = isDarkTheme() ? options.darkStyleUrl : options.styleUrl;
  themeWatchers.add(state.applyTheme);

  map.on('load', () => {
    state.callbacks.onMapLoad?.();
  });

  map.on('click', (event) => {
    const { lng, lat } = event.lngLat;
    state.callbacks.onMapClick?.([lng, lat]);
  });

  return containerId;
}

export function destroy(containerId) {
  const state = runtimeFor(containerId);
  if (!state) return;
  if (state.applyTheme) {
    themeWatchers.delete(state.applyTheme);
  }
  for (const marker of state.markers.values()) {
    marker.remove();
  }
  state.markers.clear();
  if (state.tempMarker) {
    state.tempMarker.remove();
    state.tempMarker = null;
  }
  state.map.remove();
  runtimes.delete(containerId);
}

export function setCallbacks(containerId, callbacks) {
  const state = runtimeFor(containerId);
  if (!state) return;
  state.callbacks.onMapLoad = callbacks.onMapLoad ?? null;
  state.callbacks.onMapClick = callbacks.onMapClick ?? null;
  state.callbacks.onMarkerClick = callbacks.onMarkerClick ?? null;
  state.callbacks.onTempMarkerMoved = callbacks.onTempMarkerMoved ?? null;
}

export function setMarkers(containerId, markers) {
  const state = runtimeFor(containerId);
  if (!state) return;

  for (const marker of state.markers.values()) {
    marker.remove();
  }
  state.markers.clear();

  for (const marker of markers) {
    const markerInstance = new Marker({ element: buildMarkerElement(marker.emoji) })
      .setLngLat([marker.lng, marker.lat])
      .addTo(state.map);

    markerInstance.getElement().addEventListener('click', (event) => {
      event.stopPropagation();
      state.callbacks.onMarkerClick?.(marker.id);
    });

    state.markers.set(marker.id, markerInstance);
  }

  applyActiveMarker(state);
}

/** Highlight the active marker by scaling it up. */
export function setActiveMarker(containerId, markerId) {
  const state = runtimeFor(containerId);
  if (!state) return;
  state.activeMarkerId = markerId;
  applyActiveMarker(state);
}

function applyActiveMarker(state) {
  for (const [id, marker] of state.markers) {
    const icon = marker.getElement().querySelector('.splitify-marker-icon');
    icon?.classList.toggle(
      'splitify-marker-active',
      state.activeMarkerId != null && id === state.activeMarkerId,
    );
  }
}

/** The categorized emoji dataset used by the picker. */
export function getEmojiData() {
  return emojiData;
}

export function fitMarkers(containerId) {
  const state = runtimeFor(containerId);
  if (!state || state.markers.size === 0) return;

  const bounds = new LngLatBounds();
  for (const marker of state.markers.values()) {
    bounds.extend(marker.getLngLat());
  }

  state.map.fitBounds(bounds, {
    padding: { top: 60, bottom: 60, left: 60, right: 60 },
    maxZoom: 16,
    duration: 0,
  });
}

export function flyTo(containerId, lng, lat, zoom = null) {
  const state = runtimeFor(containerId);
  if (!state) return;
  const options = { center: [lng, lat], essential: true };
  if (zoom !== null) options.zoom = zoom;
  state.map.flyTo(options);
}

/** Gently pan to a coordinate (no zoom animation). */
export function centerOn(containerId, lng, lat) {
  const state = runtimeFor(containerId);
  if (!state) return;
  state.map.easeTo({ center: [lng, lat], duration: 300 });
}

export function getCenter(containerId) {
  const state = runtimeFor(containerId);
  if (!state) return null;
  const center = state.map.getCenter();
  return { lng: center.lng, lat: center.lat };
}

export function setAddMode(containerId, enabled) {
  const state = runtimeFor(containerId);
  if (!state) return;
  state.addMode = Boolean(enabled);
  applyAddMode(state);
}

/**
 * Create (or move) the temporary draggable marker used while adding a
 * location. Reports drags back through `onTempMarkerMoved`.
 */
export function setTempMarker(containerId, lng, lat) {
  const state = runtimeFor(containerId);
  if (!state) return;
  if (state.tempMarker) {
    state.tempMarker.setLngLat([lng, lat]);
    return;
  }

  const element = document.createElement('div');
  element.className = 'splitify-temp-marker';
  element.style.backgroundImage = TEMP_PIN_URL;

  const marker = new Marker({ element, draggable: true })
    .setLngLat([lng, lat])
    .addTo(state.map);

  marker.on('dragend', () => {
    const lngLat = marker.getLngLat();
    state.callbacks.onTempMarkerMoved?.([lngLat.lng, lngLat.lat]);
  });

  state.tempMarker = marker;
}

export function removeTempMarker(containerId) {
  const state = runtimeFor(containerId);
  if (!state || !state.tempMarker) return;
  state.tempMarker.remove();
  state.tempMarker = null;
}

window.SplitifyMap = {
  create,
  destroy,
  setCallbacks,
  setMarkers,
  setActiveMarker,
  getEmojiData,
  fitMarkers,
  flyTo,
  centerOn,
  getCenter,
  setAddMode,
  setTempMarker,
  removeTempMarker,
};
