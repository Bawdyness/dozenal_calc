# Dozenal Calculator

Ein wissenschaftlicher Taschenrechner, der nativ im Duodezimalsystem (Basis 12) rechnet — mit eigens entworfenen Ziffernsymbolen für die Stellen zehn und elf.

## Warum Basis 12?

Zwölf ist durch 2, 3, 4 und 6 teilbar — Basis 10 nur durch 2 und 5. Das macht viele Alltagsbrüche kürzer: 1/3 = 0.4, 1/4 = 0.3, 1/6 = 0.2. Der Rechner macht diesen Unterschied sichtbar, indem er periodische Dezimalbrüche mit einem Periodenstrich über der Periode anzeigt — und Bruchwerte exakt durch die Rechen­kette trägt, bis ein transzendenter Operator die Genauigkeit zwangsläufig auf `f64` herunterbricht.

## Funktionsumfang

**Haupttastenfeld (Sets 1–5)**

- Grundrechenarten: +, −, ×, ÷
- Spezialoperatoren: Paralleladdition (a·b)/(a+b), x^y, n-te Wurzel, Logarithmus zur frei wählbaren Basis
- Trigonometrie: sin, cos, tan, cot — Doppelklick schaltet auf die Umkehrfunktion
- Klammerrechnung und Cursorbewegung
- AC, Del, Dezimalpunkt, Overlay-Öffner

**Erweiterungstastenfeld / Overlay (Sets 6–10)**

- Speicher: STO, RCL, MC, Ans (überträgt den exakten `Rational`, nicht nur die `f64`-Approximation)
- Konstanten: π, e, φ, √2
- Hyperbelfunktionen: sinh, cosh, tanh, coth — Doppelklick für Umkehrfunktionen
- Erweiterte Funktionen: n!, |x|, 1/x, mod
- Einstellungen: Doz↔Dez-Umschaltung, Winkelmodus (DEG/RAD/GRD), Info, Schliessen

**Periodische Dezimalbrüche**

Liefert die exakte rationale Auswertung einen periodischen Bruch, wird die Periode mit einem Überstrich dargestellt. Perioden über fünf Stellen werden auf fünf gekürzt und mit `…` auf Überstrich-Höhe markiert. Endliche Brüche werden ohne Überstrich angezeigt. Diese Auswertung überlebt STO/RCL- und Ans-Zyklen.

**Info-Bereich**

12 deutsche Kapitel über Dozenalmathematik, Geometrie und Geschichte — zugänglich über Taste 10.3.

## Tastatureingabe

| Taste | Funktion |
|---|---|
| 0–9, a, b | Ziffern (a = zehn, b = elf) |
| +  −  *  / | Grundrechenarten |
| ^ | Potenz |
| . oder , | Dezimalpunkt |
| = oder Enter | Ergebnis berechnen |
| Backspace | Zeichen löschen |
| Escape | Alles löschen (AC) |
| Pfeiltasten | Cursor bewegen |
| ( und ) | Klammern |

## Projektstruktur

Der Code lebt in einem Cargo-Workspace mit drei Crates:

```
dozenal_calc/
├── crates/
│   ├── dozenal_core/        ← MIT — Logik-Schicht, publizierbar auf crates.io
│   ├── dozenal_calc_app/    ← PolyForm — Desktop-Anwendung (egui)
│   └── dozenal_calc_web/    ← PolyForm — Web-Anwendung (Leptos, in Aufbau)
```

- **`dozenal_core`** enthält die reine Logik: `Rational`-Arithmetik mit Periodenerkennung in beliebiger Basis, eigenen `f64`-Evaluator (Lexer + Recursive-Descent-Parser + Funktions-Dispatch), `CalcToken`-Domäne und Pipeline-Helfer. Keine UI-Crate, keine Drittabhängigkeit jenseits `num-bigint`/`num-integer`/`num-traits`. Wiederverwendbar in Native-Apps, WASM-Web-Apps und (potenziell) Embedded.
- **`dozenal_calc_app`** ist der etablierte Build mit eframe/egui — läuft als Desktop-Anwendung und als WebAssembly (heute die produktive Web-Veröffentlichung).
- **`dozenal_calc_web`** ist der laufende Leptos-Port. Schlankeres WASM-Bundle, native Browser-Accessibility, SVG-Glyphen statt Canvas-Rendering, PWA-fähig (offline-installierbar). Wird parallel zur egui-Variante deployt unter `/dozenal_calc/preview/`.

## Bauen und Starten

Voraussetzung: [Rust](https://rustup.rs) mit Cargo. Für die Web-Builds zusätzlich [Trunk](https://trunkrs.dev): `cargo install trunk`.

```bash
git clone https://github.com/Bawdyness/dozenal_calc.git
cd dozenal_calc

# Desktop-Anwendung (egui)
cargo run

# Tests workspace-weit
cargo test --workspace

# Local Web-Dev-Server der egui-Variante
cd crates/dozenal_calc_app && trunk serve

# Local Web-Dev-Server der Leptos-Variante (Vorschau)
cd crates/dozenal_calc_web && trunk serve
```

CI baut beide Web-Varianten und deployt auf GitHub Pages: egui unter `/dozenal_calc/`, Leptos-Vorschau unter `/dozenal_calc/preview/`.

## Technik

- **Sprache:** Rust 2024-Edition
- **Logik-Crate:** Pure Rust mit `num-bigint` — `Rational` exakt (kein Overflow), eigene Periodenerkennung in beliebiger Basis, eigener `f64`-Evaluator (replaces `meval`)
- **Desktop / Legacy-Web:** [eframe / egui](https://github.com/emilk/egui) 0.27 — Immediate-Mode
- **Leptos-Web:** [Leptos](https://leptos.dev) 0.8 (CSR) — Fine-grained-Reactivity, SVG-Sprite für die zwölf Ziffer-Glyphen + Operator-Symbole, Hash-Routing für die Info-Sektion, Service Worker für Offline-Betrieb
- **Ziffernsymbole:** Eigene Vektor-Geometrie, sprachneutral spezifiziert in `GLYPHS.md`. Auf egui per `egui::Painter`, auf Leptos per `<svg>`-Sprite

## Lizenz

Zweigeteilt:

- **`dozenal_core`** unter [MIT](crates/dozenal_core/LICENSE) — freie Nutzung, auch kommerziell. Gedacht zur Wiederverwendung für eigene Basis-N-Projekte.
- **`dozenal_calc_app`** und **`dozenal_calc_web`** unter [PolyForm NonCommercial 1.0.0](LICENSE) — freie nicht-kommerzielle Nutzung der fertigen App.

Copyright (c) 2026 Eric Naville
