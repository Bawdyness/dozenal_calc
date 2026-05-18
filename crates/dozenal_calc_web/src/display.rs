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
use crate::state::{CalcState, NumeralSystem};
use dozenal_core::{CalcToken, MAX_PERIOD_DISPLAY, format_f64_as_decimal};
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
                let cursor_pos = state.cursor_pos.get();
                let active = !state.result_field_active.get();
                state.input_buffer.with(|buf| {
                    let mut items: Vec<AnyView> = Vec::new();
                    for (i, t) in buf.iter().enumerate() {
                        if i == cursor_pos && active {
                            items.push(view! { <span class="cursor"/> }.into_any());
                        }
                        items.push(render_token_node(t));
                    }
                    if cursor_pos >= buf.len() && active {
                        items.push(view! { <span class="cursor"/> }.into_any());
                    }
                    items.into_view()
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
            {move || match state.numeral_system.get() {
                NumeralSystem::Doz => view! { <DozResult/> }.into_any(),
                NumeralSystem::Dez => view! { <DecResult/> }.into_any(),
            }}
        </span>
    }
}

/// Dozenal-Resultat: rendert `result_buffer` mit den Dozenal-Glyphen und der
/// vom Rational-Track ermittelten Periode (Doz-Schiene, wie bisher).
#[component]
fn DozResult() -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context");
    view! {
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
                    for tok in &buf[..start] {
                        out.push(render_token_node(tok));
                    }
                    let end = (start + period_len).min(buf.len());
                    let mut period_inner: Vec<AnyView> = Vec::new();
                    for tok in &buf[start..end] {
                        period_inner.push(render_token_node(tok));
                    }
                    out.push(view! {
                        <span class="periodic">{period_inner.into_view()}</span>
                    }.into_any());
                    if capped {
                        out.push(view! { <span class="ellipsis-raised">"…"</span> }.into_any());
                    }
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
    }
}

/// Dezimal-Resultat: berechnet aus `last_ans` (Rational) die Periode in
/// Basis 10 und rendert sie als Plain-Text-Ziffern mit Überstrich. Ist die
/// Rational-Schiene kollabiert, fällt es auf `last_result_f64` zurück und
/// zeigt den `≈`-Prefix wie der Doz-Pfad.
///
/// Die Eingabe bleibt immer dozenal — dies ist eine reine View-Schicht,
/// kein zweiter Eingabemodus.
#[component]
fn DecResult() -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context");
    view! {
        {move || state.last_ans.get().map_or_else(
            || render_dec_approx(state.last_result_f64.get()),
            |r| render_dec_exact(&r),
        )}
    }
}

/// Exaktes Dezimal-Resultat aus dem Rational: Vorzeichen, Integer- und
/// Vorperiode-Ziffern als Plain-Text, Periode mit Überstrich (gekappt bei
/// `MAX_PERIOD_DISPLAY` und durch `…` markiert).
fn render_dec_exact(r: &dozenal_core::Rational) -> AnyView {
    let (int_d, pre_d, period_d) = r.to_periodic(10);
    let mut out: Vec<AnyView> = Vec::new();
    if r.is_negative() {
        out.push(view! { <span class="punct neg">"−"</span> }.into_any());
    }
    for d in &int_d {
        out.push(dec_digit(*d));
    }
    if !pre_d.is_empty() || !period_d.is_empty() {
        out.push(view! { <span class="punct">"."</span> }.into_any());
    }
    for d in &pre_d {
        out.push(dec_digit(*d));
    }
    if !period_d.is_empty() {
        let capped = period_d.len() > MAX_PERIOD_DISPLAY;
        let shown = period_d.iter().take(MAX_PERIOD_DISPLAY).copied();
        let period_inner: Vec<AnyView> = shown.map(dec_digit).collect();
        out.push(
            view! {
                <span class="periodic">{period_inner.into_view()}</span>
            }
            .into_any(),
        );
        if capped {
            out.push(view! { <span class="ellipsis-raised">"…"</span> }.into_any());
        }
    }
    out.into_view().into_any()
}

/// Rational-Schiene kollabiert (transzendent / irrational): f64 als
/// Dezimal-String, mit `≈`-Prefix wie im Doz-Pfad.
fn render_dec_approx(val: f64) -> AnyView {
    let s = format_f64_as_decimal(val);
    view! {
        <span class="approx-prefix">"≈"</span>
        <span class="dec-number">{s}</span>
    }
    .into_any()
}

/// Eine einzelne Dezimal-Ziffer als Plain-Text-Span — bewusst *nicht* als
/// `<Glyph>`, damit Doz-Modus und Dez-Modus visuell sofort unterscheidbar
/// sind. Das ist Modell-C-Identität: Dezimal ist ein Vergleichsfenster,
/// nicht ein zweiter Eingabemodus.
fn dec_digit(value: u32) -> AnyView {
    view! { <span class="dec-digit">{value.to_string()}</span> }.into_any()
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
