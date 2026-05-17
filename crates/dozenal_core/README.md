# dozenal_core

Exakte Rational-Arithmetik, Periodenerkennung und Ausdrucks-Auswertung
für das **Dozenal-Zahlensystem** (Basis 12) — und beliebige andere
Zahlbasen.

UI-agnostisch: keine Abhängigkeit auf egui, Flutter, Web-Frameworks oder
Plattform-Bibliotheken. Verwendbar in Desktop-Apps, WASM-Web-Apps und
(zukünftig) Embedded-Kontexten.

Diese Crate ist die Logik-Schicht des
[Dozenal-Taschenrechners](https://github.com/Bawdyness/dozenal_calc) und
wird unabhängig von der App-Schicht entwickelt.

## Was die Crate kann

- **`Rational`** — exakter Bruch (`i128` numerator/denominator, immer
  reduziert). Checked-Arithmetik mit `Option`-Rückgabe bei Overflow.
  Operationen: `add`, `sub`, `mul`, `div`, `pow` (mit ganzzahligem
  Exponenten), `oplus` (Parallel-Widerstand `(a·b)/(a+b)`).
- **Periodenerkennung** in beliebiger Basis: zerlegt einen Bruch in
  Ganzzahl-, Vorperiode- und Perioden-Anteil. Klassischer
  Restwert-Algorithmus, Cap bei 100 Perioden-Stellen.
- **`DozenalDigit` + `DozenalConverter`** — Ziffer-Typ mit
  `to_decimal_exact` (i128), `to_decimal` (f64), `from_decimal`,
  `frac_to_digits`.
- **`CalcToken` Enum** — kanonische Tasten-Sprache (Ziffern, Operatoren,
  Funktionen, Custom-Operatoren `⊕` `√` `log`, Memory-Tasten, Modi).
  Bewusst UI-agnostisch, sodass beliebige Frontends sie konsumieren
  können.
- **Pipeline-Helfer** für die Doppelschienen-Auswertung:
  - `build_rat_expr` — `CalcToken`-Folge → `RatExpr`-Atome für den
    exakten Pfad
  - `eval_rational` — Recursive-Descent-Auswertung der Atome
  - `with_implicit_muls` — fügt implizite Multiplikationen ein
    (`2π` → `2 * π`, `)(`  → `)*(`)
  - `resolve_custom_operators` — schreibt `⊕`, `√`, `log` in reine
    Infix-Ausdrücke um
  - `build_meval_string` — finaler String für eine externe
    Float-Auswertung
  - `format_rational_result` / `format_f64_result` — `CalcToken`-Folge
    aus einem Ergebnis-Wert

## Beispiel

```rust
use dozenal_core::Rational;

// 1/7 in Basis 12 hat eine Periode der Länge 6: 0.186A35̄
let r = Rational::new(1, 7).unwrap();
let (int_part, pre_period, period) = r.to_dozenal_periodic();
assert!(int_part.iter().all(|d| matches!(d, dozenal_core::DozenalDigit::D0)));
assert!(pre_period.is_empty());
assert_eq!(period.len(), 6);
```

```rust
use dozenal_core::{Rational, RatExpr, eval_rational};

// 1/2 + 1/3 = 5/6 (exakt, ohne Float-Drift)
let exprs = [
    RatExpr::Num(Rational::new(1, 2).unwrap()),
    RatExpr::Add,
    RatExpr::Num(Rational::new(1, 3).unwrap()),
];
let result = eval_rational(&exprs).unwrap();
assert_eq!(result, Rational::new(5, 6).unwrap());
```

## Anwendungsfälle

- Eigener Dozenal- oder Basis-N-Taschenrechner (Desktop, Web, Mobile)
- Lehrmittel zur Veranschaulichung, wie sich Brüche in verschiedenen
  Basen unterscheiden (`1/3` ist in Basis 10 periodisch, in Basis 12
  finit — die Crate macht das sichtbar)
- Exakte Bruchrechnung mit Periodenanzeige in beliebiger Basis
- Pipeline-Vorlage für ein Token-basiertes Calculator-Frontend, das
  ohne externe Math-Library auskommen will

## Roadmap

Die Crate ist in aktiver Entwicklung. Geplante Schritte:

- `i128` → `num-bigint` Migration (eliminiert die
  Overflow-Kollaps-Schiene)
- Eigener Recursive-Descent-Float-Evaluator (ablöst die externe
  `meval`-Abhängigkeit der App-Schicht)
- `no_std + alloc`-Tauglichkeit (für Embedded-Verwendung)
- `proptest`-Property-Tests für die Rational-Invarianten
- `criterion`-Benchmarks der Hotpaths
- API-Stabilisierung (`#[must_use]`-Annotierungen,
  `checked_*`-Naming-Konvention)

## Installation

```toml
[dependencies]
dozenal_core = "0.1"
```

## Lizenz

MIT. Siehe [`LICENSE`](LICENSE).
