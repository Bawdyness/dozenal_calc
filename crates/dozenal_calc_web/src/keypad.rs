// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Hauptkeypad: Digit-Grid (4×3), Op-Grid (4×4), System-Reihe (1×4)
//! und volle `=`-Bar. Erweiterungs-Overlay (Sets 6–10) wird per
//! Panel-Swap aktiviert: das Digit-Grid bleibt sichtbar, Op-Grid /
//! System-Reihe / Equals-Bar werden gegen Sets 6–9 / Set 10
//! ausgetauscht (kein Equals im Overlay, weil Close in 10.4 die
//! Trigger-Position einnimmt).
//!
//! Layout-Logik orientiert sich an Flutter `_HochKeypad` und an der
//! Mobile-Variante von `crates/dozenal_calc_app/src/layout.rs`:
//! vertikale Stapelung der Sektionen, 3 Spalten für Ziffern, 4 Spalten
//! für Operatoren, 44-dp-Touch-Floor pro Taste.

use crate::state::CalcState;
use crate::token_glyph::{token_aria, token_label};
use dozenal_core::{CalcToken, DozenalDigit};
use leptos::prelude::*;

const DIGIT_GRID: [[DozenalDigit; 3]; 4] = [
    [DozenalDigit::D10, DozenalDigit::D11, DozenalDigit::D0],
    [DozenalDigit::D7, DozenalDigit::D8, DozenalDigit::D9],
    [DozenalDigit::D4, DozenalDigit::D5, DozenalDigit::D6],
    [DozenalDigit::D1, DozenalDigit::D2, DozenalDigit::D3],
];

/// Op-Grid in der Form von Flutter `_hochOpRows` — Reihen sind die
/// horizontale Anordnung pro Set-Spalte.
fn op_rows() -> [[CalcToken; 4]; 4] {
    [
        [
            CalcToken::Add,
            CalcToken::OplusBotLeft,
            CalcToken::Sin,
            CalcToken::ParenOpen,
        ],
        [
            CalcToken::Sub,
            CalcToken::ExpTopRight,
            CalcToken::Cos,
            CalcToken::ParenClose,
        ],
        [
            CalcToken::Mul,
            CalcToken::RootTopLeft,
            CalcToken::Tan,
            CalcToken::TriangleLeft,
        ],
        [
            CalcToken::Div,
            CalcToken::LogBotRight,
            CalcToken::Cot,
            CalcToken::TriangleRight,
        ],
    ]
}

fn system_row() -> [CalcToken; 4] {
    [
        CalcToken::AC,
        CalcToken::Del,
        CalcToken::Decimal,
        CalcToken::Expand,
    ]
}

/// Overlay-Grid (Sets 6–9), Reihen-orientiert wie Flutter `_hochOverlayRows`.
/// Set 6 (Memory), Set 7 (Konstanten), Set 8 (Hyperbolik), Set 9 (Extended).
fn overlay_rows() -> [[CalcToken; 4]; 4] {
    [
        [
            CalcToken::Sto,
            CalcToken::ConstPi,
            CalcToken::Sinh,
            CalcToken::Factorial,
        ],
        [
            CalcToken::Rcl,
            CalcToken::ConstE,
            CalcToken::Cosh,
            CalcToken::AbsVal,
        ],
        [
            CalcToken::Mc,
            CalcToken::ConstPhi,
            CalcToken::Tanh,
            CalcToken::Reciprocal,
        ],
        [
            CalcToken::Ans,
            CalcToken::ConstSqrt2,
            CalcToken::Coth,
            CalcToken::Mod,
        ],
    ]
}

/// Set 10 — Modes & Meta. `Close` belegt 10.4 als Anker (mirror von Expand).
fn set10_row() -> [CalcToken; 4] {
    [
        CalcToken::DozDec,
        CalcToken::Drg,
        CalcToken::Info,
        CalcToken::Close,
    ]
}

#[component]
pub fn MainKeypad() -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context bereitstellen");

    view! {
        <div class="keypad">
            <div class="keypad-digits">
                {DIGIT_GRID.iter().flatten().map(|d| {
                    view! { <TokenButton token=CalcToken::Digit(*d) variant="digit"/> }
                }).collect_view()}
            </div>

            <div class="keypad-separator"/>

            {move || if state.overlay_open.get() {
                view! { <OverlayPanel/> }.into_any()
            } else {
                view! { <MainPanel/> }.into_any()
            }}
        </div>
    }
}

#[component]
fn MainPanel() -> impl IntoView {
    view! {
        <div class="panel panel-main">
            <div class="keypad-ops">
                {op_rows().into_iter().flatten().map(|t| {
                    let v = button_variant(&t);
                    view! { <TokenButton token=t variant=v/> }
                }).collect_view()}
            </div>

            <div class="keypad-system">
                {system_row().into_iter().map(|t| {
                    let v = button_variant(&t);
                    view! { <TokenButton token=t variant=v/> }
                }).collect_view()}
            </div>

            <TokenButton token=CalcToken::Equals variant="equals"/>
        </div>
    }
}

#[component]
fn OverlayPanel() -> impl IntoView {
    view! {
        <div class="panel panel-overlay">
            <div class="keypad-ops">
                {overlay_rows().into_iter().flatten().map(|t| {
                    let v = button_variant(&t);
                    view! { <TokenButton token=t variant=v/> }
                }).collect_view()}
            </div>

            <div class="keypad-system">
                {set10_row().into_iter().map(|t| {
                    let v = button_variant(&t);
                    view! { <TokenButton token=t variant=v/> }
                }).collect_view()}
            </div>
        </div>
    }
}

fn button_variant(t: &CalcToken) -> &'static str {
    match t {
        CalcToken::AC => "ac",
        CalcToken::Equals => "equals",
        CalcToken::Expand | CalcToken::Close => "sys",
        CalcToken::Del | CalcToken::Decimal => "sys",
        CalcToken::Digit(_) => "digit",
        CalcToken::TriangleLeft | CalcToken::TriangleRight => "nav",
        CalcToken::Sto | CalcToken::Rcl | CalcToken::Mc | CalcToken::Ans | CalcToken::RatLit(_) => {
            "mem"
        }
        CalcToken::ConstPi | CalcToken::ConstE | CalcToken::ConstPhi | CalcToken::ConstSqrt2 => {
            "const"
        }
        CalcToken::Sinh
        | CalcToken::Cosh
        | CalcToken::Tanh
        | CalcToken::Coth
        | CalcToken::ArSinh
        | CalcToken::ArCosh
        | CalcToken::ArTanh
        | CalcToken::ArCoth => "fn",
        CalcToken::Factorial | CalcToken::AbsVal | CalcToken::Reciprocal | CalcToken::Mod => "fn",
        CalcToken::DozDec | CalcToken::Drg | CalcToken::Info => "mode",
        _ => "op",
    }
}

#[component]
fn TokenButton(token: CalcToken, variant: &'static str) -> impl IntoView {
    let state = use_context::<CalcState>().expect("CalcState im Context bereitstellen");
    let aria = token_aria(&token);
    let captured = token.clone();
    let armed_token = token.clone();

    let armed = move || state.is_armed(&armed_token);

    let on_click = move |_| state.handle_click(captured.clone());

    view! {
        <button
            class={format!("token-btn token-btn-{variant}")}
            class:armed=armed
            aria-label=aria
            on:click=on_click
        >
            {token_label(&token)}
        </button>
    }
}
