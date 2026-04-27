# Dozenal Calculator

Ein wissenschaftlicher Taschenrechner, der nativ im Duodezimalsystem (Basis 12) rechnet — mit eigens entworfenen Ziffernsymbolen für die Stellen zehn und elf.

## Warum Basis 12?

Zwölf ist durch 2, 3, 4 und 6 teilbar — Basis 10 nur durch 2 und 5. Das macht viele Alltagsbrüche kürzer: 1/3 = 0.4, 1/4 = 0.3, 1/6 = 0.2. Der Rechner macht diesen Unterschied sichtbar, indem er periodische Dezimalbrüche mit einem Periodenstrich über der Periode anzeigt.

## Funktionsumfang

**Haupttastenfeld (Sets 1–5)**

- Grundrechenarten: +, −, ×, ÷
- Spezialoperatoren: Paralleladdition (a·b)/(a+b), x², Wurzel, log
- Trigonometrie: sin, cos, tan, cot — Doppelklick schaltet auf die Umkehrfunktion
- Klammerrechnung und Cursorbewegung
- AC, Del, Dezimalpunkt, Overlay-Öffner

**Erweiterungstastenfeld / Overlay (Sets 6–10)**

- Speicher: STO, RCL, MC, Ans
- Konstanten: π, e, φ, √2
- Hyperbelfunktionen: sinh, cosh, tanh, coth — Doppelklick für Umkehrfunktionen
- Erweiterte Funktionen: n!, |x|, 1/x, mod
- Einstellungen: Doz/Dec-Umschaltung, Winkelmodus (DEG/RAD/GRD), Info, Schliessen

**Periodische Dezimalbrüche**

Liefert die exakte rationale Auswertung einen periodischen Bruch, wird die Periode mit einem Überstrich dargestellt. Perioden über fünf Stellen werden auf fünf gekürzt und mit … markiert. Endliche Brüche werden ohne Überstrich angezeigt. Diese Auswertung überlebt STO/RCL-Zyklen.

**Info-Bereich**

12 deutsche Kapitel über Dozenalmathematik, Geometrie und Geschichte — zugänglich über Taste 10.3.

## Tastatureingabe

| Taste | Funktion |
|---|---|
| 0–9, a, b | Ziffern (a = zehn, b = elf) |
| +  −  *  / | Grundrechenarten |
| ^ | Potenz |
| . | Dezimalpunkt |
| = oder Enter | Ergebnis berechnen |
| Backspace | Zeichen löschen |
| Escape | Alles löschen (AC) |
| Pfeiltasten | Cursor bewegen |

## Bauen und Starten

Voraussetzung: [Rust](https://rustup.rs) mit Cargo.

```bash
git clone https://github.com/Bawdyness/dozenal_calc.git
cd dozenal_calc

# Desktop
cargo run

# Tests
cargo test

# Web (erfordert trunk: cargo install trunk)
trunk serve
```

Der Produktions-Webbuild wird automatisch über GitHub Actions auf GitHub Pages deployt.

## Technik

- **Sprache:** Rust
- **GUI:** [eframe / egui](https://github.com/emilk/egui) 0.27 — Immediate-Mode, läuft nativ und als WebAssembly
- **Auswertung:** [meval](https://crates.io/crates/meval) für den f64-Pfad; eigene rationale Arithmetik (`Rational`) für den Periodenstrich-Pfad
- **Ziffernsymbole:** Eigene Vektor-Zeichenroutine, keine Schriftart

## Lizenz

PolyForm NonCommercial License 1.0.0 — freie Nutzung für nicht-kommerzielle Zwecke.  
Copyright (c) 2026 Eric Naville  
Lizenztext: [LICENSE](LICENSE)
