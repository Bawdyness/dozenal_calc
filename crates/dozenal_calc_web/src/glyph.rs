// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! SVG-Sprite mit allen Calc-Glyphen — Ziffern UND Operator-Symbole.
//!
//! Ziffer-Geometrie aus `/research/glyph_rendering.md` und `GLYPHS.md`.
//! Operator-Geometrie aus `crates/dozenal_calc_app/src/painting.rs::paint_token`
//! 1:1 portiert (q=25 in viewBox 100, c=(50,50)): `+ −` einfache Linien,
//! `× ÷` Diagonalen, Dreiecke gefüllte Polygone, Composite-Operatoren
//! (`x^`, `√`, `log`, `⊕`) als zentriertes `x` plus stroke-Square an einer
//! Ecke — die Ecke kodiert die Operation.
//!
//! Halbkreis-Glyphen (D2/D3/D5) werden bewusst als zwei separate `<path>`-
//! Elemente gerendert, damit jeder Halbkreis seine eigene `stroke-linecap:
//! round` bekommt. Ein zusammenhängender Pfad würde am Treffpunkt (50,50)
//! eine 180°-Ecke erzeugen, die `stroke-linejoin: round` als kleine
//! halbkreisförmige Beule darstellt — sichtbar als „Naht" zwischen den
//! beiden Hälften. Mit getrennten Pfaden überlappen sich zwei runde
//! Stroke-Enden im selben Punkt und das Ergebnis liest sich nahtlos.

use dozenal_core::{CalcToken, DozenalDigit};
use leptos::prelude::*;

pub const GLYPH_SPRITE: &str = r#"
<svg xmlns="http://www.w3.org/2000/svg" style="position:absolute;width:0;height:0;overflow:hidden" aria-hidden="true">
  <defs>
    <!-- Ziffer-Glyphen -->
    <symbol id="d0" viewBox="0 0 100 100">
      <circle cx="50" cy="50" r="25"/>
    </symbol>
    <symbol id="d1" viewBox="0 0 100 100">
      <path d="M 50 25 L 25 75 M 50 25 L 75 75"/>
    </symbol>
    <symbol id="d2" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 1 50 50"/>
      <path d="M 50 50 A 25 25 0 0 0 50 100"/>
    </symbol>
    <symbol id="d3" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 1 50 50"/>
      <path d="M 50 50 A 25 25 0 0 1 50 100"/>
    </symbol>
    <symbol id="d4" viewBox="0 0 100 100">
      <path d="M 25 50 L 75 25 M 25 50 L 75 75"/>
    </symbol>
    <symbol id="d5" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 0 50 50"/>
      <path d="M 50 50 A 25 25 0 0 1 50 100"/>
    </symbol>
    <symbol id="d6" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 0 50 50"/>
      <circle cx="50" cy="75" r="25"/>
    </symbol>
    <symbol id="d7" viewBox="0 0 100 100">
      <path d="M 75 50 L 25 25 M 75 50 L 25 75"/>
    </symbol>
    <symbol id="d8" viewBox="0 0 100 100">
      <circle cx="50" cy="25" r="25"/>
      <circle cx="50" cy="75" r="25"/>
    </symbol>
    <symbol id="d9" viewBox="0 0 100 100">
      <circle cx="50" cy="25" r="25"/>
      <path d="M 50 50 A 25 25 0 0 1 50 100"/>
    </symbol>
    <symbol id="d10" viewBox="0 0 100 100">
      <path d="M 50 75 L 25 25 M 50 75 L 75 25"/>
    </symbol>
    <symbol id="d11" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 1 50 50"/>
      <circle cx="50" cy="75" r="25"/>
    </symbol>

    <!-- Operator-Symbole (q=25, c=(50,50)) -->
    <symbol id="op-add" viewBox="0 0 100 100">
      <line x1="25" y1="50" x2="75" y2="50"/>
      <line x1="50" y1="25" x2="50" y2="75"/>
    </symbol>
    <symbol id="op-sub" viewBox="0 0 100 100">
      <line x1="25" y1="50" x2="75" y2="50"/>
    </symbol>
    <symbol id="op-mul" viewBox="0 0 100 100">
      <line x1="25" y1="25" x2="75" y2="75"/>
      <line x1="25" y1="75" x2="75" y2="25"/>
    </symbol>
    <symbol id="op-div" viewBox="0 0 100 100">
      <line x1="25" y1="75" x2="75" y2="25"/>
    </symbol>

    <!-- Composite-Operatoren: zentriertes 'x' + Marker-Quadrat an einer Ecke.
         Quadrat-Grösse 26×26 und Stroke 5 (waren 18 / 3), damit der Marker
         auf typischen Button-Grössen klar als Quadrat lesbar bleibt und nicht
         als „nur ein x" durchgeht. Style-Attribute zwingen Fill/Stroke
         deterministisch — vermeidet, dass die CSS-Regel `.glyph { fill: none }`
         über Shadow-DOM-Inheritance via <use> die Werte schluckt. -->
    <symbol id="op-pow" viewBox="0 0 100 100">
      <text x="50" y="55" text-anchor="middle" dominant-baseline="central"
            font-family="ui-monospace,SFMono-Regular,Menlo,monospace"
            font-size="56" font-weight="700"
            style="fill: currentColor; stroke: none;">x</text>
      <rect x="68" y="8" width="26" height="26"
            style="fill: none; stroke: currentColor; stroke-width: 5;"
            vector-effect="non-scaling-stroke"/>
    </symbol>
    <symbol id="op-root" viewBox="0 0 100 100">
      <text x="50" y="55" text-anchor="middle" dominant-baseline="central"
            font-family="ui-monospace,SFMono-Regular,Menlo,monospace"
            font-size="56" font-weight="700"
            style="fill: currentColor; stroke: none;">x</text>
      <rect x="6" y="8" width="26" height="26"
            style="fill: none; stroke: currentColor; stroke-width: 5;"
            vector-effect="non-scaling-stroke"/>
    </symbol>
    <symbol id="op-log" viewBox="0 0 100 100">
      <text x="50" y="55" text-anchor="middle" dominant-baseline="central"
            font-family="ui-monospace,SFMono-Regular,Menlo,monospace"
            font-size="56" font-weight="700"
            style="fill: currentColor; stroke: none;">x</text>
      <rect x="68" y="66" width="26" height="26"
            style="fill: none; stroke: currentColor; stroke-width: 5;"
            vector-effect="non-scaling-stroke"/>
    </symbol>
    <symbol id="op-oplus" viewBox="0 0 100 100">
      <text x="50" y="55" text-anchor="middle" dominant-baseline="central"
            font-family="ui-monospace,SFMono-Regular,Menlo,monospace"
            font-size="56" font-weight="700"
            style="fill: currentColor; stroke: none;">x</text>
      <rect x="6" y="66" width="26" height="26"
            style="fill: none; stroke: currentColor; stroke-width: 5;"
            vector-effect="non-scaling-stroke"/>
      <line x1="19" y1="73" x2="19" y2="85"
            style="stroke: currentColor; stroke-width: 4;" vector-effect="non-scaling-stroke"/>
      <line x1="13" y1="79" x2="25" y2="79"
            style="stroke: currentColor; stroke-width: 4;" vector-effect="non-scaling-stroke"/>
    </symbol>

    <!-- Cursor-Pfeile als gefüllte Dreiecke — style-Attribut zwingt das Fill,
         damit es nicht von der globalen `.glyph { fill: none }`-Regel
         überschrieben wird (Shadow-DOM-Inheritance via <use>). -->
    <symbol id="op-tri-left" viewBox="0 0 100 100">
      <path d="M 25 50 L 70 25 L 70 75 Z"
            style="fill: currentColor; stroke: none;"/>
    </symbol>
    <symbol id="op-tri-right" viewBox="0 0 100 100">
      <path d="M 75 50 L 30 25 L 30 75 Z"
            style="fill: currentColor; stroke: none;"/>
    </symbol>
  </defs>
</svg>
"#;

/// Mountet das SVG-Sprite einmalig in den DOM.
#[component]
pub fn GlyphSprite() -> impl IntoView {
    view! { <div inner_html=GLYPH_SPRITE /> }
}

fn digit_id(d: DozenalDigit) -> &'static str {
    match d {
        DozenalDigit::D0 => "#d0",
        DozenalDigit::D1 => "#d1",
        DozenalDigit::D2 => "#d2",
        DozenalDigit::D3 => "#d3",
        DozenalDigit::D4 => "#d4",
        DozenalDigit::D5 => "#d5",
        DozenalDigit::D6 => "#d6",
        DozenalDigit::D7 => "#d7",
        DozenalDigit::D8 => "#d8",
        DozenalDigit::D9 => "#d9",
        DozenalDigit::D10 => "#d10",
        DozenalDigit::D11 => "#d11",
    }
}

fn digit_aria(d: DozenalDigit) -> &'static str {
    match d {
        DozenalDigit::D0 => "null",
        DozenalDigit::D1 => "eins",
        DozenalDigit::D2 => "zwei",
        DozenalDigit::D3 => "drei",
        DozenalDigit::D4 => "vier",
        DozenalDigit::D5 => "fünf",
        DozenalDigit::D6 => "sechs",
        DozenalDigit::D7 => "sieben",
        DozenalDigit::D8 => "acht",
        DozenalDigit::D9 => "neun",
        DozenalDigit::D10 => "zehn",
        DozenalDigit::D11 => "elf",
    }
}

/// Rendert eine einzelne Dozenal-Ziffer als skaliertes SVG.
#[component]
pub fn Glyph(digit: DozenalDigit) -> impl IntoView {
    let id = digit_id(digit);
    let label = digit_aria(digit);
    view! {
        <svg class="glyph" viewBox="0 0 100 100" role="img" aria-label=label>
            <use href=id/>
        </svg>
    }
}

/// Rendert ein Operator/Funktions-Symbol als SVG (nicht-Ziffer-Tokens
/// mit grafischer Darstellung).
#[component]
#[allow(clippy::needless_pass_by_value)] // Leptos-Idiom: Props sind owned.
pub fn TokenSvgGlyph(#[prop(into)] symbol_id: String, #[prop(into)] aria: String) -> impl IntoView {
    let href = format!("#{symbol_id}");
    view! {
        <svg class="glyph" viewBox="0 0 100 100" role="img" aria-label=aria>
            <use href=href/>
        </svg>
    }
}

/// Gibt den SVG-Symbol-Identifier für Tokens zurück, die eine SVG-Glyphe
/// statt eines Text-Labels nutzen. `None` für Text-Tokens (sin, π, AC, …).
pub fn token_svg_id(t: &CalcToken) -> Option<&'static str> {
    Some(match t {
        CalcToken::Add => "op-add",
        CalcToken::Sub | CalcToken::Negate => "op-sub",
        CalcToken::Mul => "op-mul",
        CalcToken::Div => "op-div",
        CalcToken::ExpTopRight => "op-pow",
        CalcToken::RootTopLeft => "op-root",
        CalcToken::LogBotRight => "op-log",
        CalcToken::OplusBotLeft => "op-oplus",
        CalcToken::TriangleLeft => "op-tri-left",
        CalcToken::TriangleRight => "op-tri-right",
        _ => return None,
    })
}
