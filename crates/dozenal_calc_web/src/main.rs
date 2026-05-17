// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Leptos-Web-Frontend für den Dozenal-Taschenrechner.
//!
//! Phase C: Hauptkeypad (Sets 1–5 + `=`-Bar) ist live, plus
//! Hardware-Tastatur-Mapping. Overlay (Sets 6–10), Info-Surface
//! und PWA-Polish folgen in den Phasen D, E und F.

mod display;
mod glyph;
mod keypad;
mod state;
mod token_glyph;

use crate::display::TwoLineDisplay;
use crate::glyph::GlyphSprite;
use crate::keypad::MainKeypad;
use crate::state::CalcState;
use dozenal_core::{CalcToken, DozenalDigit};
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let state = CalcState::default();
    provide_context(state);

    Effect::new(move |_| {
        attach_keyboard_handler(&state);
        hide_splash();
    });

    view! {
        <GlyphSprite/>
        <main class="app">
            <header class="app-header">
                <h1 class="app-title">"Dozenal Calc"</h1>
                <p class="app-subtitle">"Leptos-Vorschau · Phase C"</p>
            </header>
            <TwoLineDisplay/>
            <MainKeypad/>
        </main>
    }
}

fn hide_splash() {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(splash) = doc.get_element_by_id("splash") {
                let _ = splash.set_attribute("data-mounted", "");
            }
        }
    }
}

/// Hardware-Tastatur → CalcToken-Mapping, parallel zum Flutter-`_charKeyMap`
/// und `_logicalKeyMap` aus `lib/main.dart`. Setup einmalig via Closure;
/// das `forget()` lässt den Closure für die Lebensdauer der Seite leben.
fn attach_keyboard_handler(state: &CalcState) {
    let state = *state;
    let Some(window) = web_sys::window() else {
        return;
    };

    let cb = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        let token = token_for_key(&key);
        if let Some(t) = token {
            event.prevent_default();
            state.handle_click(t);
        }
    }) as Box<dyn FnMut(_)>);

    let _ = window.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref());
    cb.forget();
}

fn token_for_key(key: &str) -> Option<CalcToken> {
    match key {
        "0" => Some(CalcToken::Digit(DozenalDigit::D0)),
        "1" => Some(CalcToken::Digit(DozenalDigit::D1)),
        "2" => Some(CalcToken::Digit(DozenalDigit::D2)),
        "3" => Some(CalcToken::Digit(DozenalDigit::D3)),
        "4" => Some(CalcToken::Digit(DozenalDigit::D4)),
        "5" => Some(CalcToken::Digit(DozenalDigit::D5)),
        "6" => Some(CalcToken::Digit(DozenalDigit::D6)),
        "7" => Some(CalcToken::Digit(DozenalDigit::D7)),
        "8" => Some(CalcToken::Digit(DozenalDigit::D8)),
        "9" => Some(CalcToken::Digit(DozenalDigit::D9)),
        "a" | "A" => Some(CalcToken::Digit(DozenalDigit::D10)),
        "b" | "B" => Some(CalcToken::Digit(DozenalDigit::D11)),
        "+" => Some(CalcToken::Add),
        "-" => Some(CalcToken::Sub),
        "*" => Some(CalcToken::Mul),
        "/" => Some(CalcToken::Div),
        "^" => Some(CalcToken::ExpTopRight),
        "." | "," => Some(CalcToken::Decimal),
        "=" | "Enter" => Some(CalcToken::Equals),
        "(" => Some(CalcToken::ParenOpen),
        ")" => Some(CalcToken::ParenClose),
        "Backspace" => Some(CalcToken::Del),
        "Escape" => Some(CalcToken::AC),
        "ArrowLeft" => Some(CalcToken::TriangleLeft),
        "ArrowRight" => Some(CalcToken::TriangleRight),
        _ => None,
    }
}
