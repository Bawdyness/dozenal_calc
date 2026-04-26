# Dozenaltaschenrechner

Ein kleiner Taschenrechner, der nativ im **Dozenal-System (Basis 12)** rechnet. Dieses Projekt kombiniert die Präzision von Rust mit einer speziellen und sicher gewöhnungsbedürftigen Numbersymbolik für Anhänger des Duodezimalsystems.

## Warum Dozenal?
Das Dozenal-System nutzt die Basis 12, welche durch ihre hohe Teilbarkeit (2, 3, 4, 6) gegenüber der Basis 10 erhebliche Vorteile bei Bruchrechnungen und im täglichen Messwesen bietet. 

## Hauptfunktionen
- **Natives Base-12 Rechnen:** Alle Ein- und Ausgaben erfolgen in duodezimaler Symbolik.
- **Wissenschaftlicher Funktionsumfang:**
  - Trigonometrie: `sin`, `cos`, `tan`, `cot` (inklusive Invers-Funktionen per Doppelklick-Logik).
  - Potenzrechnung, n-te Wurzel und Logarithmus zur Basis n.
- **Spezial-Operatoren:**
  - **$\oplus$ (Parallele Addition):** Berechnet die harmonische Summe nach der Formel $\frac{a \cdot b}{a + b}$ (ideal für Parallelschaltungen von Widerständen).
- **Intelligente UI:**
  - Interaktiver Cursor für präzise Formelbearbeitung.
  - Kontextsensitive `M+` Speichertaste.
  - Große "Spacebar-Style" Gleichheits-Taste.

## Technik
Das Projekt ist mithilfe von Gemini 3 in **Rust** geschrieben und nutzt:
- [eframe/egui](https://github.com/emilk/egui) für das Immediate-Mode GUI.
- Eine benutzerdefinierte Zeichen-Engine für die Darstellung der Dozenal-Ziffern.
- [meval](https://crates.io/crates/meval) für die mathematische Evaluation der geparsten Ausdrücke.

## Installation & Start
Stelle sicher, dass du Rust und Cargo installiert hast.

1. Repository klonen:
```bash
   git clone [https://github.com/Bawdyness/dozenal_calc.git](https://github.com/Bawdyness/dozenal_calc.git)
   cd dozenal_calc
```
   
2. Projekt starten:
    
```bash
  cargo run
```

## Bedienungshinweise

  Doppelklick: Ein zweiter Klick auf eine trigonometrische Funktion (z.B. sin) schaltet diese automatisch auf die Invers-Funktion (sin⁻¹) um.

  Navigation: Nutze die Pfeiltasten auf dem Display, um den roten Cursor innerhalb deiner Formel zu bewegen.

  Dezimalpunkt: Der Rechner behandelt Nachkommastellen nativ als Potenzen von 12−n.
