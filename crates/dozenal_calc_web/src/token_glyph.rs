// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Sichtbare Darstellung der Nicht-Ziffer-Tokens für Tasten und Display.
//!
//! Im Gegensatz zu den Dozenal-Ziffer-Glyphen (`glyph.rs`) verwenden die
//! Operator- und Funktions-Tokens hier eine Mischung aus Unicode-Text und
//! kleinen Inline-SVG-Kompositionen (z. B. `x^` mit hochgestelltem y,
//! `√` mit Radikal-Linie). Das bleibt schlank, accessible (Screenreader
//! sehen Unicode-Operatoren nativ) und entspricht visuell der
//! egui-Variante.

use crate::glyph::Glyph;
use dozenal_core::{CalcToken, DozenalDigit};
use leptos::prelude::*;

/// Rendert das Label für eine Taste oder einen Buffer-Eintrag.
/// Für `Digit` fällt der Aufruf an `Glyph` zurück.
pub fn token_label(t: &CalcToken) -> AnyView {
    match t {
        CalcToken::Digit(d) => view! { <Glyph digit={*d}/> }.into_any(),
        CalcToken::Add => text_label("+", "op").into_any(),
        CalcToken::Sub => text_label("−", "op").into_any(),
        CalcToken::Mul => text_label("×", "op").into_any(),
        CalcToken::Div => text_label("÷", "op").into_any(),
        CalcToken::Negate => text_label("−", "neg").into_any(),
        CalcToken::OplusBotLeft => text_label("⊕", "op").into_any(),
        CalcToken::ExpTopRight => view! {
            <span class="tok composite">
                <span class="tok-base">"x"</span>
                <span class="tok-super">"y"</span>
            </span>
        }
        .into_any(),
        CalcToken::RootTopLeft => text_label("√", "op").into_any(),
        CalcToken::LogBotRight => view! {
            <span class="tok composite">
                <span class="tok-base">"log"</span>
                <span class="tok-sub">"n"</span>
            </span>
        }
        .into_any(),
        CalcToken::Sin => text_label("sin", "fn").into_any(),
        CalcToken::Cos => text_label("cos", "fn").into_any(),
        CalcToken::Tan => text_label("tan", "fn").into_any(),
        CalcToken::Cot => text_label("cot", "fn").into_any(),
        CalcToken::ArcSin => text_label("sin⁻¹", "fn").into_any(),
        CalcToken::ArcCos => text_label("cos⁻¹", "fn").into_any(),
        CalcToken::ArcTan => text_label("tan⁻¹", "fn").into_any(),
        CalcToken::ArcCot => text_label("cot⁻¹", "fn").into_any(),
        CalcToken::Sinh => text_label("sinh", "fn").into_any(),
        CalcToken::Cosh => text_label("cosh", "fn").into_any(),
        CalcToken::Tanh => text_label("tanh", "fn").into_any(),
        CalcToken::Coth => text_label("coth", "fn").into_any(),
        CalcToken::ArSinh => text_label("arsinh", "fn").into_any(),
        CalcToken::ArCosh => text_label("arcosh", "fn").into_any(),
        CalcToken::ArTanh => text_label("artanh", "fn").into_any(),
        CalcToken::ArCoth => text_label("arcoth", "fn").into_any(),
        CalcToken::Factorial => text_label("n!", "fn").into_any(),
        CalcToken::AbsVal => text_label("|x|", "fn").into_any(),
        CalcToken::Reciprocal => text_label("1∕x", "fn").into_any(),
        CalcToken::Mod => text_label("mod", "fn").into_any(),
        CalcToken::ParenOpen => text_label("(", "paren").into_any(),
        CalcToken::ParenClose => text_label(")", "paren").into_any(),
        CalcToken::TriangleLeft => text_label("◀", "nav").into_any(),
        CalcToken::TriangleRight => text_label("▶", "nav").into_any(),
        CalcToken::AC => text_label("AC", "ac").into_any(),
        CalcToken::Del => text_label("⌫", "sys").into_any(),
        CalcToken::Decimal => text_label(".", "punct").into_any(),
        CalcToken::Equals => text_label("=", "equals").into_any(),
        CalcToken::Expand => text_label("▾▾", "sys").into_any(),
        CalcToken::Sto => text_label("STO", "mem").into_any(),
        CalcToken::Rcl => text_label("RCL", "mem").into_any(),
        CalcToken::Mc => text_label("MC", "mem").into_any(),
        CalcToken::Ans => text_label("Ans", "mem").into_any(),
        CalcToken::ConstPi => text_label("π", "const").into_any(),
        CalcToken::ConstE => text_label("e", "const").into_any(),
        CalcToken::ConstPhi => text_label("φ", "const").into_any(),
        CalcToken::ConstSqrt2 => text_label("√2", "const").into_any(),
        CalcToken::DozDec => text_label("Doz↔Dez", "mode").into_any(),
        CalcToken::Drg => text_label("DRG", "mode").into_any(),
        CalcToken::Info => text_label("Info", "mode").into_any(),
        CalcToken::Close => text_label("Schliessen", "mode").into_any(),
        CalcToken::RatLit(_) => text_label("Ans", "mem").into_any(),
    }
}

fn text_label(text: &'static str, kind: &'static str) -> impl IntoView {
    view! { <span class={format!("tok tok-{kind}")}>{text}</span> }
}

/// Aria-Label für Screenreader (statt visueller Symbole).
pub fn token_aria(t: &CalcToken) -> String {
    match t {
        CalcToken::Digit(DozenalDigit::D0) => "null".into(),
        CalcToken::Digit(DozenalDigit::D1) => "eins".into(),
        CalcToken::Digit(DozenalDigit::D2) => "zwei".into(),
        CalcToken::Digit(DozenalDigit::D3) => "drei".into(),
        CalcToken::Digit(DozenalDigit::D4) => "vier".into(),
        CalcToken::Digit(DozenalDigit::D5) => "fünf".into(),
        CalcToken::Digit(DozenalDigit::D6) => "sechs".into(),
        CalcToken::Digit(DozenalDigit::D7) => "sieben".into(),
        CalcToken::Digit(DozenalDigit::D8) => "acht".into(),
        CalcToken::Digit(DozenalDigit::D9) => "neun".into(),
        CalcToken::Digit(DozenalDigit::D10) => "zehn".into(),
        CalcToken::Digit(DozenalDigit::D11) => "elf".into(),
        CalcToken::Add => "plus".into(),
        CalcToken::Sub | CalcToken::Negate => "minus".into(),
        CalcToken::Mul => "mal".into(),
        CalcToken::Div => "geteilt durch".into(),
        CalcToken::OplusBotLeft => "Parallelwiderstand".into(),
        CalcToken::ExpTopRight => "Potenz".into(),
        CalcToken::RootTopLeft => "Wurzel".into(),
        CalcToken::LogBotRight => "Logarithmus zur Basis".into(),
        CalcToken::Sin => "Sinus".into(),
        CalcToken::Cos => "Kosinus".into(),
        CalcToken::Tan => "Tangens".into(),
        CalcToken::Cot => "Kotangens".into(),
        CalcToken::ArcSin => "Arkussinus".into(),
        CalcToken::ArcCos => "Arkuskosinus".into(),
        CalcToken::ArcTan => "Arkustangens".into(),
        CalcToken::ArcCot => "Arkuskotangens".into(),
        CalcToken::Sinh => "Sinus hyperbolicus".into(),
        CalcToken::Cosh => "Kosinus hyperbolicus".into(),
        CalcToken::Tanh => "Tangens hyperbolicus".into(),
        CalcToken::Coth => "Kotangens hyperbolicus".into(),
        CalcToken::ArSinh => "Areasinus hyperbolicus".into(),
        CalcToken::ArCosh => "Areakosinus hyperbolicus".into(),
        CalcToken::ArTanh => "Areatangens hyperbolicus".into(),
        CalcToken::ArCoth => "Areakotangens hyperbolicus".into(),
        CalcToken::Factorial => "Fakultät".into(),
        CalcToken::AbsVal => "Absolutbetrag".into(),
        CalcToken::Reciprocal => "Kehrwert".into(),
        CalcToken::Mod => "Modulo".into(),
        CalcToken::ParenOpen => "Klammer auf".into(),
        CalcToken::ParenClose => "Klammer zu".into(),
        CalcToken::TriangleLeft => "Cursor links".into(),
        CalcToken::TriangleRight => "Cursor rechts".into(),
        CalcToken::AC => "Alles löschen".into(),
        CalcToken::Del => "Zeichen löschen".into(),
        CalcToken::Decimal => "Dezimalpunkt".into(),
        CalcToken::Equals => "ist gleich".into(),
        CalcToken::Expand => "Erweitern".into(),
        CalcToken::Sto => "Speichern".into(),
        CalcToken::Rcl => "Abrufen".into(),
        CalcToken::Mc => "Speicher löschen".into(),
        CalcToken::Ans => "Letztes Ergebnis".into(),
        CalcToken::ConstPi => "Pi".into(),
        CalcToken::ConstE => "Eulersche Zahl".into(),
        CalcToken::ConstPhi => "Goldener Schnitt".into(),
        CalcToken::ConstSqrt2 => "Wurzel zwei".into(),
        CalcToken::DozDec => "Dozenal Dezimal Umschalten".into(),
        CalcToken::Drg => "Winkelmodus umschalten".into(),
        CalcToken::Info => "Info öffnen".into(),
        CalcToken::Close => "Schliessen".into(),
        CalcToken::RatLit(_) => "Letztes Ergebnis".into(),
    }
}
