// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! SVG-Sprite und Glyph-Komponente für die zwölf Dozenal-Ziffer-Symbole.
//!
//! Geometrie aus `/research/glyph_rendering.md` — entspricht 1:1 der
//! Spezifikation in `GLYPHS.md`. Anker-Ziffern (D1/D4/D7/D10) sind Pfeile;
//! Komposit-Ziffern (D0/D2/D3/D5/D6/D8/D9/D11) bestehen aus Voll- oder
//! Halbkreisen.
//!
//! `GLYPH_SPRITE` wird einmal in den DOM gemountet (versteckt, nur als
//! `<defs>`). Jede Glyph-Instanz ist ein `<svg><use href="#dN"/></svg>` —
//! der Browser kopiert die geometrische Form aus dem Sprite, Stroke und
//! Farbe kommen via CSS-`currentColor`.

use dozenal_core::DozenalDigit;
use leptos::prelude::*;

/// SVG-Sprite-Quelltext mit den 12 Symbol-Definitionen.
///
/// Wird per `Html`-Komponente einmal an den Body angehängt; danach kann
/// jede Glyph-Instanz die Symbole per `<use href="#dN"/>` referenzieren.
pub const GLYPH_SPRITE: &str = r#"
<svg xmlns="http://www.w3.org/2000/svg" style="position:absolute;width:0;height:0;overflow:hidden" aria-hidden="true">
  <defs>
    <symbol id="d0" viewBox="0 0 100 100">
      <circle cx="50" cy="50" r="25"/>
    </symbol>
    <symbol id="d1" viewBox="0 0 100 100">
      <path d="M 50 25 L 25 75 M 50 25 L 75 75"/>
    </symbol>
    <symbol id="d2" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 1 50 50 A 25 25 0 0 0 50 100"/>
    </symbol>
    <symbol id="d3" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 1 50 50 A 25 25 0 0 1 50 100"/>
    </symbol>
    <symbol id="d4" viewBox="0 0 100 100">
      <path d="M 25 50 L 75 25 M 25 50 L 75 75"/>
    </symbol>
    <symbol id="d5" viewBox="0 0 100 100">
      <path d="M 50 0 A 25 25 0 0 0 50 50 A 25 25 0 0 1 50 100"/>
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
  </defs>
</svg>
"#;

/// Mountet das SVG-Sprite einmalig in den DOM. Ein Aufruf reicht für
/// die ganze Anwendung.
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
/// Stroke-Farbe folgt `currentColor`, Strichstärke und Größe per CSS.
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
