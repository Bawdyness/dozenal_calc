// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville
//
// Service Worker für Dozenal Calc — minimale Offline-Strategie.
//
// - Versionierte Assets (`*-{hash}.wasm`, `*-{hash}.js`): cache-first,
//   nie revalidieren. Hash im Dateinamen → Inhalt unveränderlich.
// - Alles andere (index.html, manifest, icon): network-first mit
//   Cache-Fallback. Damit kommt eine neue index.html, die auf die
//   neuen Hash-Assets verweist, sofort durch — der alte Cache-Eintrag
//   wird nur bei Offline genutzt.
// - `skipWaiting` + `clients.claim` für sauberen Update-Flow.

const CACHE_NAME = 'dozenal-calc-v1';

self.addEventListener('install', () => {
    self.skipWaiting();
});

self.addEventListener('activate', (event) => {
    event.waitUntil(
        caches
            .keys()
            .then((keys) =>
                Promise.all(
                    keys.filter((k) => k !== CACHE_NAME).map((k) => caches.delete(k))
                )
            )
            .then(() => self.clients.claim())
    );
});

self.addEventListener('fetch', (event) => {
    if (event.request.method !== 'GET') return;
    const url = new URL(event.request.url);
    if (url.origin !== location.origin) return;

    const isImmutable = /-[a-f0-9]+(_bg)?\.(wasm|js)$/i.test(url.pathname);

    if (isImmutable) {
        event.respondWith(
            caches.match(event.request).then(
                (cached) =>
                    cached ||
                    fetch(event.request).then((res) => {
                        if (res.ok) {
                            const clone = res.clone();
                            caches.open(CACHE_NAME).then((c) => c.put(event.request, clone));
                        }
                        return res;
                    })
            )
        );
        return;
    }

    event.respondWith(
        fetch(event.request)
            .then((res) => {
                if (res.ok) {
                    const clone = res.clone();
                    caches.open(CACHE_NAME).then((c) => c.put(event.request, clone));
                }
                return res;
            })
            .catch(() =>
                caches
                    .match(event.request)
                    .then((cached) => cached || new Response('Offline', { status: 503 }))
            )
    );
});
