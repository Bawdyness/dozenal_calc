// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Leptos-Web-Frontend für den Dozenal-Taschenrechner.
//!
//! Phase B: SVG-Sprite + Two-Line-Display. Tastatur folgt in Phase C —
//! bis dahin injiziert die App `1 ÷ 7` als Demonstrations-Input.

mod display;
mod glyph;
mod state;

use crate::display::TwoLineDisplay;
use crate::glyph::GlyphSprite;
use crate::state::CalcState;
use dozenal_core::{CalcToken, DozenalDigit};
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let state = CalcState::default();
    provide_context(state);

    // Phase-B-Demo: lade `1 ÷ 7` als Input und führe sofort calculate_result aus.
    // Verschwindet in Phase C, wenn die Tastatur die Eingabe übernimmt.
    Effect::new(move |_| {
        state.input_buffer.set(vec![
            CalcToken::Digit(DozenalDigit::D1),
            CalcToken::Div,
            CalcToken::Digit(DozenalDigit::D7),
        ]);
        state.calculate_result();

        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(splash) = doc.get_element_by_id("splash") {
                    let _ = splash.set_attribute("data-mounted", "");
                }
            }
        }
    });

    view! {
        <GlyphSprite/>
        <main class="app">
            <header class="app-header">
                <h1 class="app-title">"Dozenal Calc"</h1>
                <p class="app-subtitle">"Leptos-Vorschau · Phase B"</p>
            </header>
            <TwoLineDisplay/>
            <p class="phase-note">
                "Demonstration: " <code>"1 ÷ 7"</code>
                " — die Tastatur folgt in Phase C, bis dahin ist die Eingabe vorgeladen."
            </p>
        </main>
    }
}
