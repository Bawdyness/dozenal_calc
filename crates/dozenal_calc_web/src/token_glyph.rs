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

use crate::glyph::{Glyph, TokenSvgGlyph, token_svg_id};
use dozenal_core::{CalcToken, DozenalDigit};
use leptos::prelude::*;

/// Rendert das Label für eine Taste oder einen Buffer-Eintrag.
/// Digits → `Glyph`. Tokens mit SVG-Symbol (`+ − × ÷`, `x^`, `√`, `log`,
/// `⊕`, `◀`, `▶`) → `TokenSvgGlyph`. Alles andere → Text-Label.
pub fn token_label(t: &CalcToken) -> AnyView {
    if let CalcToken::Digit(d) = t {
        return view! { <Glyph digit={*d}/> }.into_any();
    }
    if let Some(id) = token_svg_id(t) {
        let aria = token_aria(t);
        return view! { <TokenSvgGlyph symbol_id=id aria=aria/> }.into_any();
    }
    match t {
        CalcToken::Sin => text_label("sin", "fn"),
        CalcToken::Cos => text_label("cos", "fn"),
        CalcToken::Tan => text_label("tan", "fn"),
        CalcToken::Cot => text_label("cot", "fn"),
        CalcToken::ArcSin => text_label("sin⁻¹", "fn"),
        CalcToken::ArcCos => text_label("cos⁻¹", "fn"),
        CalcToken::ArcTan => text_label("tan⁻¹", "fn"),
        CalcToken::ArcCot => text_label("cot⁻¹", "fn"),
        CalcToken::Sinh => text_label("sinh", "fn"),
        CalcToken::Cosh => text_label("cosh", "fn"),
        CalcToken::Tanh => text_label("tanh", "fn"),
        CalcToken::Coth => text_label("coth", "fn"),
        CalcToken::ArSinh => text_label("sinh⁻¹", "fn"),
        CalcToken::ArCosh => text_label("cosh⁻¹", "fn"),
        CalcToken::ArTanh => text_label("tanh⁻¹", "fn"),
        CalcToken::ArCoth => text_label("coth⁻¹", "fn"),
        CalcToken::Factorial => text_label("n!", "fn"),
        CalcToken::AbsVal => text_label("|x|", "fn"),
        CalcToken::Reciprocal => text_label("1∕x", "fn"),
        CalcToken::Mod => text_label("mod", "fn"),
        CalcToken::ParenOpen => text_label("(", "paren"),
        CalcToken::ParenClose => text_label(")", "paren"),
        CalcToken::AC => text_label("AC", "ac"),
        CalcToken::Del => text_label("⌫", "sys"),
        CalcToken::Decimal => text_label(".", "punct"),
        CalcToken::Equals => text_label("=", "equals"),
        CalcToken::Expand => text_label("▾▾", "sys"),
        CalcToken::Sto => text_label("STO", "mem"),
        CalcToken::Rcl => text_label("RCL", "mem"),
        CalcToken::Mc => text_label("MC", "mem"),
        CalcToken::Ans => text_label("Ans", "mem"),
        CalcToken::ConstPi => text_label("π", "const"),
        CalcToken::ConstE => text_label("e", "const"),
        CalcToken::ConstPhi => text_label("φ", "const"),
        CalcToken::ConstSqrt2 => text_label("√2", "const"),
        CalcToken::DozDec => text_label("Doz↔Dez", "mode"),
        CalcToken::Drg => text_label("DRG", "mode"),
        CalcToken::Info => text_label("Info", "mode"),
        CalcToken::Close => text_label("Schliessen", "mode"),
        CalcToken::RatLit(_) => text_label("Ans", "mem"),
        // Folgende Cases werden bereits oben via early-return abgefangen
        // (Digit + SVG-Operatoren), bleiben hier nur als Pflichtcatch-all.
        _ => text_label("", "punct"),
    }
}

fn text_label(text: &'static str, kind: &'static str) -> AnyView {
    view! { <span class={format!("tok tok-{kind}")}>{text}</span> }.into_any()
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
