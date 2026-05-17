// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Hash-Routing (GitHub-Pages-tauglich, kein Server-Rewrite nötig).
//!
//! Leptos bringt keinen Hash-Mode mit (Issue #2184), aber für die paar
//! Routen, die diese App braucht, reicht eine ~60-Zeilen-Eigenimplementierung:
//! Listener auf `hashchange`, ReadSignal mit dem aktuell parsed `Route`,
//! eine `navigate`-Funktion, die nur die `location.hash` setzt — den Rest
//! erledigt der Browser-Event-Loop.

use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Calc,
    Info { anchor: Option<String> },
}

impl Route {
    /// Parst eine Hash-Komponente (mit oder ohne führendem `#`) in eine Route.
    /// Grammatik:
    /// - leer / `#` / `#/` / `#/calc` → `Calc`
    /// - `#/info` / `#/info/` → `Info { anchor: None }`
    /// - `#/info/<slug>` → `Info { anchor: Some(<slug>) }`
    pub fn from_hash(hash: &str) -> Self {
        let h = hash.strip_prefix('#').unwrap_or(hash);
        let trimmed = h.trim_start_matches('/');
        let mut parts = trimmed.split('/').filter(|s| !s.is_empty());
        match parts.next() {
            None | Some("calc") => Self::Calc,
            Some("info") => {
                let anchor = parts.next().map(str::to_string);
                Self::Info { anchor }
            }
            _ => Self::Calc,
        }
    }

    pub fn to_hash(&self) -> String {
        match self {
            Self::Calc => String::from("#/calc"),
            Self::Info { anchor: None } => String::from("#/info"),
            Self::Info { anchor: Some(a) } => format!("#/info/{a}"),
        }
    }
}

fn current_route() -> Route {
    web_sys::window()
        .and_then(|w| w.location().hash().ok())
        .map_or(Route::Calc, |h| Route::from_hash(&h))
}

/// Liefert ein Signal, das auf Hash-Änderungen reagiert. Beim ersten Aufruf
/// wird der `hashchange`-Listener am `window` installiert; das vom Closure
/// referenzierte `RwSignal` lebt für die Lebensdauer der Seite.
pub fn use_route() -> ReadSignal<Route> {
    let (read, write) = signal(current_route());

    if let Some(window) = web_sys::window() {
        let cb = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            write.set(current_route());
        }) as Box<dyn FnMut(_)>);
        let _ = window.add_event_listener_with_callback("hashchange", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    read
}

pub fn navigate(route: &Route) {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_hash(&route.to_hash());
    }
}
