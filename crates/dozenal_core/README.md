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

- **`Rational`** — exakter Bruch (`num-bigint::BigInt` numerator/
  denominator, immer reduziert, `den > 0`). Kein Overflow-Risiko;
  Add/Sub/Mul geben `Self` zurück, nur `div` (gegen Null) und
  `pow(0, neg)` geben `Option`. Operationen: `add`, `sub`, `mul`, `div`,
  `pow` (mit ganzzahligem Exponenten), `oplus` (Parallel-Widerstand
  `(a·b)/(a+b)`), `to_f64`, `is_negative`.
- **Periodenerkennung in beliebiger Basis** (≥ 2): `to_periodic(base)`
  zerlegt einen Bruch in `(int_digits, pre_period, period)` als
  `Vec<u32>`. Klassischer Restwert-Algorithmus, Cap bei 100
  Perioden-Stellen. Der Spezialfall `to_dozenal_periodic()` ist ein
  dünner Doz-Wrapper für Aufrufer, die direkt mit `DozenalDigit` arbeiten.
- **`DozenalDigit` + `DozenalConverter`** — Ziffer-Typ mit
  `to_decimal_exact` (i128), `to_decimal` (f64), `from_decimal`,
  `frac_to_digits`.
- **`CalcToken` Enum** — kanonische Tasten-Sprache (Ziffern, Operatoren,
  Funktionen, Custom-Operatoren `⊕` `√` `log`, Memory-Tasten, Modi).
  Bewusst UI-agnostisch, sodass beliebige Frontends sie konsumieren
  können.
- **Eigener f64-Evaluator** (`eval/`-Submodul) — Lexer, Recursive-Descent-
  Parser und Interpret-Schicht für die Float-Schiene. Keine externe
  Math-Library-Abhängigkeit; deckt `sin`/`cos`/`tan`/`cot` und ihre
  Inversen, Hyperbolische und ihre Inversen, `log`/`ln`, `exp`, `√`,
  `n!`, `mod`, `abs` ab.
- **Pipeline-Helfer** für die Doppelschienen-Auswertung:
  - `build_rat_expr` — `CalcToken`-Folge → `RatExpr`-Atome für den
    exakten Pfad
  - `eval_rational` — Recursive-Descent-Auswertung der Atome
  - `with_implicit_muls` — fügt implizite Multiplikationen ein
    (`2π` → `2 * π`, `)(`  → `)*(`)
  - `resolve_custom_operators` — schreibt `⊕`, `√`, `log` in reine
    Infix-Ausdrücke um
  - `build_meval_string` — baut den finalen Ausdrucks-String für
    `eval_f64` (Name historisch)
  - `eval_f64(expr, angle_mode)` — Float-Auswertung des Strings
  - `format_rational_result` / `format_f64_result` — `CalcToken`-Folge
    aus einem Ergebnis-Wert (Dozenal-Tokens)
  - `format_f64_as_decimal` — Plain-Text-Dezimal-String mit Strip von
    trailing Nullen, NaN/∞-tolerant

## Beispiel

```rust
use dozenal_core::Rational;

// Derselbe Bruch — Perioden in zwei Basen.
let one_third = Rational::from_ints(1, 3).unwrap();

// 1/3 in Basis 12 ist *endlich*: 0.4
let (_int, pre, period) = one_third.to_periodic(12);
assert_eq!(pre, vec![4]);
assert!(period.is_empty());

// 1/3 in Basis 10 ist *periodisch*: 0.3̄
let (_int, pre, period) = one_third.to_periodic(10);
assert!(pre.is_empty());
assert_eq!(period, vec![3]);

// 1/7 ist in beiden Basen periodisch, mit unterschiedlichen Stellen.
let (_int, _pre, period_b10) = Rational::from_ints(1, 7).unwrap().to_periodic(10);
assert_eq!(period_b10, vec![1, 4, 2, 8, 5, 7]); // 0.142857̄

let (_int, _pre, period_b12) = Rational::from_ints(1, 7).unwrap().to_periodic(12);
assert_eq!(period_b12.len(), 6); // 0.186A35̄
```

```rust
use dozenal_core::{Rational, RatExpr, eval_rational};

// 1/2 + 1/3 = 5/6 (exakt, ohne Float-Drift)
let exprs = [
    RatExpr::Num(Rational::from_ints(1, 2).unwrap()),
    RatExpr::Add,
    RatExpr::Num(Rational::from_ints(1, 3).unwrap()),
];
let result = eval_rational(&exprs).unwrap();
assert_eq!(result, Rational::from_ints(5, 6).unwrap());
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

- `no_std + alloc`-Tauglichkeit (für Embedded-Verwendung)
- `proptest`-Property-Tests für die Rational-Invarianten
- `criterion`-Benchmarks der Hotpaths
- API-Stabilisierung (`#[must_use]`-Annotierungen,
  `checked_*`-Naming-Konvention)
- crates.io-Publikation

Erledigt:

- `i128` → `num-bigint` Migration (Overflow-Pfad eliminiert).
- Eigener Recursive-Descent-Float-Evaluator (externe `meval`-
  Abhängigkeit entfernt).
- Periodenerkennung basis-generisch (`to_periodic(base)` für jede Basis ≥ 2).

## Installation

```toml
[dependencies]
dozenal_core = "0.1"
```

## Lizenz

MIT. Siehe [`LICENSE`](LICENSE).
