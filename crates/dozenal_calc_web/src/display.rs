// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Zwei-Zeilen-Display: obere Zeile Input, untere Zeile Resultat.
//!
//! Resultat-Zeile rechtsbündig (Casio-Konvention), mit Überstrich über
//! periodische Stellen. Drei Anzeige-Zustände:
//!
//! - **State A — Exakt**: kein Suffix
//! - **State B — Gerundet** (rationale Schiene kollabiert): `≈`-Prefix
//! - **State C — Periodisch gekappt**: `…` auf Überstrich-Höhe nach der
//!   abgeschnittenen Periode
//!
//! Kleine Indikatoren in den Ecken: `M` (Memory belegt), Winkelmodus
//! (DEG/RAD/GRD), Zahlsystem (DOZ/DEZ), Fehlermeldung in Rot.

use crate::glyph::Glyph;
use crate::state::CalcState;
use dozenal_core::CalcToken;
use leptos::prelude::*;

#[component]
pub fn TwoLineDisplay() -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context bereitstellen");

    view! {
        <div class="display">
            <div class="display-indicators">
                <span class="indicator-memory">
                    {move || if state.memory.with(|m| !m.is_empty()) { "M" } else { "" }}
                </span>
                <span class="indicator-mode-right">
                    <span class="indicator-numeral">
                        {move || state.numeral_system.get().label()}
                    </span>
                    <span class="indicator-angle">
                        {move || state.angle_mode.get().label()}
                    </span>
                </span>
            </div>

            <div class="display-line display-input">
                {move || {
                    state.error_msg.with(|err| err.as_ref().map_or_else(
                        || view! { <InputTokens/> }.into_any(),
                        |msg| view! { <span class="error-text">{msg.clone()}</span> }.into_any(),
                    ))
                }}
            </div>

            <div class="display-separator"/>

            <div class="display-line display-result">
                {move || if state.error_msg.with(Option::is_none) {
                    view! { <ResultTokens/> }.into_any()
                } else {
                    view! { <span/> }.into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn InputTokens() -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context");
    view! {
        <span class="token-row token-row-input">
            {move || {
                state.input_buffer.with(|buf| {
                    buf.iter().map(render_token_node).collect_view()
                })
            }}
        </span>
    }
}

#[component]
fn ResultTokens() -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context");

    view! {
        <span class="token-row token-row-result">
            {move || {
                let approx = state.is_f64_fallback();
                let period_start = state.result_period_start.get();
                let period_len = state.result_period_len.get();
                let capped = state.result_period_capped.get();

                state.result_buffer.with(|buf| {
                    let mut out: Vec<AnyView> = Vec::new();
                    if approx {
                        out.push(view! { <span class="approx-prefix">"≈"</span> }.into_any());
                    }
                    if let Some(start) = period_start {
                        // Stellen vor der Periode normal rendern
                        for tok in &buf[..start] {
                            out.push(render_token_node(tok));
                        }
                        // Stellen ab `start` mit Überstrich
                        let end = (start + period_len).min(buf.len());
                        let mut period_inner: Vec<AnyView> = Vec::new();
                        for tok in &buf[start..end] {
                            period_inner.push(render_token_node(tok));
                        }
                        out.push(view! {
                            <span class="periodic">
                                {period_inner.into_view()}
                            </span>
                        }.into_any());
                        if capped {
                            // State C: Auslassung auf Überstrich-Höhe
                            out.push(view! { <span class="ellipsis-raised">"…"</span> }.into_any());
                        }
                        // Falls noch Tokens nach der gekappten Periode kommen (selten):
                        if end < buf.len() {
                            for tok in &buf[end..] {
                                out.push(render_token_node(tok));
                            }
                        }
                    } else {
                        for tok in buf {
                            out.push(render_token_node(tok));
                        }
                    }
                    out.into_view()
                })
            }}
        </span>
    }
}

/// Rendert einen einzelnen CalcToken in seine sichtbare Form.
/// Reduziert für Phase B auf die im Display tatsächlich vorkommenden Tokens —
/// Operatoren und Funktionen im Input-Buffer folgen in Phase C (`TokenGlyph`).
fn render_token_node(tok: &CalcToken) -> AnyView {
    match tok {
        CalcToken::Digit(d) => view! { <Glyph digit={*d}/> }.into_any(),
        CalcToken::Decimal => view! { <span class="punct">"."</span> }.into_any(),
        CalcToken::Negate => view! { <span class="punct neg">"−"</span> }.into_any(),
        CalcToken::Add => view! { <span class="punct op">"+"</span> }.into_any(),
        CalcToken::Sub => view! { <span class="punct op">"−"</span> }.into_any(),
        CalcToken::Mul => view! { <span class="punct op">"×"</span> }.into_any(),
        CalcToken::Div => view! { <span class="punct op">"÷"</span> }.into_any(),
        CalcToken::ParenOpen => view! { <span class="punct">"("</span> }.into_any(),
        CalcToken::ParenClose => view! { <span class="punct">")"</span> }.into_any(),
        // Phase B: alle anderen Tokens als Text-Placeholder; richtige
        // Token-Glyphen kommen in Phase C zusammen mit der Tastatur.
        other => view! { <span class="punct placeholder">{format!("{other:?}")}</span> }.into_any(),
    }
}
