// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Leptos-Web-Frontend für den Dozenal-Taschenrechner.
//!
//! Phase A — Skelett: nur Mount, ein Test-Render gegen `dozenal_core`,
//! um zu verifizieren, dass die Logik-Crate im WASM-Kontext arbeitet.
//! Tatsächliche UI (Glyphen, Display, Tastatur, Info-Surface) folgt in
//! den Phasen B–F.

use dozenal_core::{DozenalDigit, Rational};
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let one_seventh = Rational::from_ints(1, 7).expect("1/7 ist gültig");
    let (int_d, pre_d, period_d) = one_seventh.to_dozenal_periodic();

    let display = format!(
        "{}.{}({})",
        digits_to_alpha(&int_d),
        digits_to_alpha(&pre_d),
        digits_to_alpha(&period_d),
    );

    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            if let Some(doc) = window.document() {
                if let Some(splash) = doc.get_element_by_id("splash") {
                    let _ = splash.set_attribute("data-mounted", "");
                }
            }
        }
    });

    view! {
        <main style="padding: 32px 24px; max-width: 720px; margin: 0 auto;">
            <h1 style="font-size: 28px; margin: 0 0 4px;">Dozenal Calc</h1>
            <p style="color: var(--accent); margin: 0 0 24px; font-size: 13px; letter-spacing: 0.04em;">
                "LEPTOS-VORSCHAU · PHASE A"
            </p>

            <section style="background: rgba(255,255,255,0.04); padding: 20px; border-radius: 8px; margin-bottom: 24px;">
                <p style="margin: 0 0 8px; color: var(--muted); font-size: 13px;">
                    "Test-Aufruf gegen dozenal_core::Rational::to_dozenal_periodic"
                </p>
                <p style="margin: 0; font-size: 18px; font-family: ui-monospace, SFMono-Regular, Menlo, monospace;">
                    "1/7 in Basis 12 = " <strong>{display}</strong>
                </p>
                <p style="margin: 12px 0 0; color: var(--dim); font-size: 13px;">
                    "(Klammern markieren periodische Stellen; eigene Glyphen folgen in Phase B.)"
                </p>
            </section>

            <p style="color: var(--muted); font-size: 14px; line-height: 1.6;">
                "Das hier ist die noch leere Hülle der zukünftigen Leptos-Webversion. \
                Die produktive egui-Version läuft weiterhin unter "
                <a href="../" style="color: var(--accent);">{"/dozenal_calc/"}</a>
                "."
            </p>

            <footer style="margin-top: 64px; color: var(--dim); font-size: 12px;">
                "© 2026 Eric Naville · PolyForm Noncommercial License 1.0.0"
            </footer>
        </main>
    }
}

fn digits_to_alpha(digits: &[DozenalDigit]) -> String {
    digits
        .iter()
        .map(|d| match d.to_value() {
            v @ 0..=9 => char::from_digit(v, 10).unwrap_or('?'),
            10 => 'A',
            11 => 'B',
            _ => '?',
        })
        .collect()
}
