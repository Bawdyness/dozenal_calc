// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville
// Copyright (c) 2026 Eric Naville

//! Hauptkeypad: Digit-Grid (4×3), Op-Grid (4×4), System-Reihe (1×4)
//! und volle `=`-Bar. Sets 6–10 (Erweiterungs-Overlay) folgen in Phase D.
//!
//! Layout-Logik orientiert sich an Flutter `_HochKeypad` und an der
//! Mobile-Variante von `crates/dozenal_calc_app/src/layout.rs::draw_mobile_layout`:
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

#[component]
pub fn MainKeypad() -> impl IntoView {
    view! {
        <div class="keypad">
            <div class="keypad-digits">
                {DIGIT_GRID.iter().flatten().map(|d| {
                    view! { <TokenButton token=CalcToken::Digit(*d) variant="digit"/> }
                }).collect_view()}
            </div>

            <div class="keypad-separator"/>

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

fn button_variant(t: &CalcToken) -> &'static str {
    match t {
        CalcToken::AC => "ac",
        CalcToken::Equals => "equals",
        CalcToken::Expand => "sys",
        CalcToken::Del | CalcToken::Decimal => "sys",
        CalcToken::Digit(_) => "digit",
        CalcToken::TriangleLeft | CalcToken::TriangleRight => "nav",
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
