# Recherche — Ausdrucksauswerter für `dozenal_core`

Stand: 2026-05-17 · Kontext: Migration von `meval = "0.2"` (von 2020) im Zuge der Extraktion einer wiederverwendbaren Crate `dozenal_core` auf crates.io unter MIT OR Apache-2.0.

## TL;DR und Empfehlung

`meval` ist seit ~6 Jahren ohne Release, hängt auf `nom = 1.2.4` (2016) und hat einen offenen Bug zur künftigen Rust-Unrustability dieser Transitive (Issue #30 von 2023). Externe Evaluator-Crates sind entweder lizenz-inkompatibel (`evalexpr` ist AGPL-3.0), tot (`fasteval` letzter Commit 2020), überdimensioniert (`rhai` ist eine ganze Skriptsprache) oder bringen wieder die gleiche Wartungs-Wette mit. **Empfehlung: hand-geschriebener Recursive-Descent-Parser nach Vorbild des Flutter-Ports, als drei Submodule (`lexer`, `parser`, `interpret`) innerhalb von `dozenal_core::eval`.** Das ist ~350 LoC, eliminiert eine wackelige Dependency komplett und erschließt direkten Zugang zu numerischen Härtungen, die der Flutter-Port bereits etabliert hat.

## Vergleichstabelle

Bewertung gegen die acht Kriterien aus dem Briefing. Für Lizenz steht "MIT|A" für "MIT OR Apache-2.0", das Zielprofil von `dozenal_core`.

| Option | Letztes Release | Lizenz | Compat MIT\|A | WASM | no\_std | Edge-Case-Kontrolle | Custom-Ops (⊕, √, log) | Bundle-Adder¹ |
|---|---|---|---|---|---|---|---|---|
| `meval 0.2` | Sep 2020 | Unlicense/MIT | ja | ja | nein | gering (closed) | nur via Pre-Rewrite | ~50 kB + `nom 1.2` |
| `evalexpr 13.1` | aktiv (2025+) | **AGPL-3.0-only** | **nein** | ja | nein | mittel | ja (Context) | ~80 kB |
| `fasteval 0.2.4` | Sep 2020 | MIT | ja | unklar | nein | gering | begrenzt | ~70 kB |
| `fasteval2 2.1.1` | Yandex-Fork | MIT | ja | unklar | nein | gering | begrenzt | ~70 kB |
| `mexprp 0.3.1` | 2019 | MPL-2.0 | dual nicht möglich | ja | nein | mittel | umständlich | ~60 kB |
| `rhai 1.24` | aktiv 2025+ | MIT|A | ja | ja | optional | ungeeignet (skript) | Overkill | ~250 kB+ |
| `chumsky 0.10` | März 2024 (1.0-α.8 erschienen 2025) | MIT | ja, fehlt Apache | ja | ja | sehr hoch (selbst geschrieben) | ja | ~100 kB Combinator-Tree |
| `winnow 1.0` | März 2026 stabil | MIT | ja, fehlt Apache | ja | ja | sehr hoch (selbst geschrieben) | ja | ~80 kB |
| `pest 2.8.6` | Feb 2025 | **MIT OR Apache-2.0** | ja | ja | optional | hoch (PEG-Grammatik) | ja | ~120 kB + proc-macro |
| **Hand-rolled** | — | beliebig | ja | ja | trivial | **maximal** | trivial | ~0 kB extra |

¹ Bundle-Adder = grobe Schätzung des release-WASM-Beitrags nach `wasm-opt`, basiert auf Erfahrungswerten ähnlicher Crates; nicht gemessen.

## `meval`-Risiko-Analyse

**Repository-Status** (github.com/rekka/meval-rs):
- 110 Commits, 4 offene Issues, 5 offene PRs.
- "No releases published" auf GitHub — die einzige veröffentlichte Version ist `0.2.0` auf crates.io vom **September 2020**.
- `rust-version: unknown` im Manifest; keine Edition-Deklaration.

**Offene Issues (Auszug):**
- **#30 (März 2023, ungeschlossen):** *"Code that will be rejected by a future version of Rust: nom v1.2.4"*. Das ist die einzige Transitive Dependency neben `fnv`, und sie ist seit 2016 nicht aktualisiert. Sobald ein nicht-rückwärtskompatibler `rustc`-Release diese Pattern verbannt, ist `meval` ohne neuen Maintainer-Push tot. `cargo tree` im Projekt bestätigt: `meval → nom v1.2.4 + fnv v1.0.7`.
- **#28 (März 2022):** Implicit-Multiplication fehlt — die Anwendung dieses Pakets hat sie ohnehin auf Pre-Token-Ebene umgesetzt (`with_implicit_muls` in `src/eval.rs`).
- **#32 (Feb 2025):** Pattern wie `2 --- 1` parsen ohne Fehler — das deutet darauf hin, dass auch der vorhandene Parser Edge-Cases hat, die das Projekt aktuell durch eigene Pre-Validation kompensiert.

**Rust-Edition-Kompatibilität:** Das Crate selbst hat keine Edition-Direktive (vor 2018). Im Workspace funktioniert es noch, aber die Abhängigkeitsbasis (`nom 1.x` mit `closure!`-Makro) erbt das ursprüngliche Bug-Surface. Es gibt **keinen Maintainer-Pfad** für einen Sprung auf `nom 7+` oder das Edition-Upgrade.

**Funktionales Risiko jetzt:** keines — `meval 0.2.0` funktioniert in der App. Das Risiko ist mittelfristig (Pages-Build wird einmal mit einem rustc-Upgrade unverlinkbar) und langfristig sicher (das Crate wird nicht mehr aktualisiert).

## Empfehlung mit Begründung

Hand-geschriebener Recursive-Descent-Parser, identisch strukturiert zum Flutter-Port (`/home/eric/dozenal_calc_flutter/lib/logic/expression.dart`, Zeilen 611-924).

**Warum nicht eine andere Crate?**
- **Lizenzfilter:** `evalexpr` (AGPL) und `mexprp` (MPL-2.0) sind raus, wenn `dozenal_core` als MIT OR Apache-2.0 publiziert werden soll. AGPL ist viral, MPL-2.0 ist file-level copyleft und nicht dual-MIT/Apache-kompatibel.
- **Wartungsfilter:** `meval`, `fasteval`, `fasteval2` sind alle seit 2020 unmaintained. Eine Migration auf eine zweite tote Crate löst das Problem nicht.
- **Scope-Filter:** `rhai` bringt eine vollständige Skriptsprache (Variablen, Closures, Module) für einen Anwendungsfall, der sieben Infix-Operatoren und ~25 Built-in-Funktionen umfasst. Das ist eine Bundle-Größe und API-Komplexität, die didaktisch zur Minimalismus-Identität des Projekts in Konflikt steht.
- **Parser-Generator-Filter:** `chumsky`, `winnow` und `pest` sind technisch alle solide Optionen. Aber: die Grammatik aus `EXPRESSION_GRAMMAR.md` ist klein (~30 Zeilen EBNF), und der Flutter-Port hat empirisch gezeigt, dass ein hand-geschriebener Parser für **diese** Grammatik ~310 Zeilen Dart und damit ~350 Zeilen Rust umfasst. Eine Parser-Combinator-Library bringt für so wenig Grammatik mehr Konzept-Overhead als Nutzen, und kein Combinator macht numerische Härtungen wie `arsinh`-Cancellation-Vermeidung oder `tanh`-Saturierung lesbarer.

**Warum hand-geschrieben gewinnt:**
1. **Numerische Härtung:** Die Flutter-Implementation enthält symmetrischen `arsinh`, `tanh`-Saturierung ab `|x|>20`, BigInt-`fact`. Das sind genau die Stellen, an denen das Vorbild-Profil von `dozenal_core` glänzen soll — und sie sind nur möglich, wenn der Evaluator-Code direkt im Projekt liegt.
2. **Custom-Operatoren werden trivial:** `⊕`, `√`, `log` sind im aktuellen `meval`-basierten Code via String-Rewrite (`resolve_custom_operators`) gelöst. In einem eigenen Parser werden sie native Grammatik-Knoten und brauchen keine Pre-Lex-Stufe mehr.
3. **Zero zusätzliche Dependencies:** Der WASM-Bundle wird leichter, nicht schwerer. Die `Cargo.toml` schrumpft.
4. **Lesbarkeit als Vorbild:** Das Briefing nennt explizit, dass `dozenal_core`-Code für Profis lesbar sein soll. Ein 350-Zeilen-Recursive-Descent-Parser, der eine formalisierte Grammatik 1:1 spiegelt, ist deutlich lesbarer als eine Crate-Abhängigkeit mit eigener API-Konvention.

## Skizze der Datei-Aufteilung in `dozenal_core`

```
dozenal_core/
├── src/
│   ├── lib.rs                  # Re-exports
│   ├── rational.rs             # Aus aktuellem logic.rs
│   ├── digit.rs                # DozenalDigit, DozenalConverter
│   └── eval/
│       ├── mod.rs              # öffentliche eval(tokens, ctx) -> Result<Value>
│       ├── lexer.rs            # Tokenisierung des input_buffer; ~80 LoC
│       ├── parser.rs           # Recursive descent, baut AST; ~150 LoC
│       ├── ast.rs              # AST-Enum: BinOp, UnaryOp, Call, Num, Const; ~40 LoC
│       ├── interpret.rs        # AST → f64, dual Rational; ~80 LoC
│       └── builtins.rs         # sin/cos/tan/cot/asin/... mit Härtungen; ~120 LoC
```

**Schnittstelle:**
- `pub fn eval(tokens: &[Token], ctx: &EvalContext) -> Result<EvalResult, EvalError>` ist die einzige öffentliche Funktion des Submoduls.
- `EvalContext` hält Winkel-Modus und optional eine Erweiterungsfunktion-Map (`HashMap<&'static str, fn(f64) -> f64>`), damit Konsumenten der Crate eigene Funktionen registrieren können, ohne den Parser zu fragen.
- `EvalResult` ist `{ f64_value: f64, rational: Option<Rational> }` — die zweispurige Auswertung bleibt im Parser, nicht im Anwendungs-Code.

Diese Aufteilung folgt dem `clippy.toml`-Mindset des Projekts: pure Datenschicht, keine UI-Abhängigkeiten, jede Datei eine Verantwortung.

## Migrations-Aufwand

**Geschätzte LoC:** ~350 LoC neuer Rust-Code für `eval/`. Entfernt werden im Gegenzug:
- `make_meval_context` (25 LoC)
- `build_meval_string` (50 LoC)
- `resolve_custom_operators` und Helfer (90 LoC)
- `meval = "0.2"` aus `Cargo.toml`

Netto ändert sich der LoC-Stand des Projekts also nur um **+185 LoC**, weil ~165 LoC bereits dem aktuellen meval-Workaround-Code gewidmet sind.

**Risiken:**
1. **Test-Regression:** Die 38 Tests aus `src/eval.rs` und `src/logic.rs` sind die Spec. Solange alle grün bleiben, ist die Migration korrekt. Niedriges Risiko, weil die Grammatik in `EXPRESSION_GRAMMAR.md` formalisiert ist.
2. **Performance:** meval ist auf String-Parsen optimiert. Ein hand-geschriebener Parser auf vorbereiteten Tokens ist mindestens gleich schnell und vermutlich schneller (eine Lex-Stufe entfällt). Keine Bench-Sorge bei den Eingabengrößen einer Taschenrechner-Session.
3. **Subtile Float-Differenzen:** Wenn `meval` für eine bestimmte Eingabe einen leicht anderen f64-Bitwert liefert als die direkte Berechnung, ändert sich das angezeigte gerundete Ergebnis um eine letzte Dozenal-Ziffer. Mitigation: bei den Tests Toleranzen auf `1e-10` halten und einmalig manuelle Cross-Validation gegen die aktuelle App durchführen.
4. **Custom-Operator-Edge-Cases:** Die Tests `oplus_with_paren_left_operand`, `nth_root_with_paren_arg`, `log_with_paren_base` decken die kritischen Fälle ab. Migration heißt: dieselben Tests bleiben, der Parser kennt die Operatoren direkt.

**Aufwandschätzung:** für einen erfahrenen Rust-Entwickler ein 1-2-Tage-Block, einschließlich Test-Migration. Der Flutter-Port als Referenz reduziert die Design-Arbeit drastisch.

## Quellen

- [crates.io: meval 0.2.0](https://crates.io/crates/meval) — Last release Sep 2020, `Unlicense/MIT`.
- [github.com/rekka/meval-rs/issues](https://github.com/rekka/meval-rs/issues) — Issue #30 (nom 1.2.4 Rust-Future), #28 (implicit mul), #32 (parse bug Feb 2025).
- [crates.io: evalexpr](https://crates.io/crates/evalexpr) und Manifest `license: AGPL-3.0-only`.
- [crates.io: fasteval](https://crates.io/crates/fasteval) — `MIT`, letzter Commit Sep 2020.
- [crates.io: fasteval2](https://crates.io/crates/fasteval2) — Yandex-Fork, 4 Stars, kaum Aktivität.
- [crates.io: mexprp 0.3.1](https://crates.io/crates/mexprp) — `MPL-2.0`, kein Release seit 2019.
- [crates.io: chumsky](https://crates.io/crates/chumsky) — `MIT`, 0.10.0 von 2024, `1.0.0-alpha.8` 2025, no\_std-fähig.
- [crates.io: winnow](https://crates.io/crates/winnow) — `MIT`, 1.0.0 im März 2026 erschienen, no\_std-fähig.
- [crates.io: pest 2.8.6](https://crates.io/crates/pest) — `MIT OR Apache-2.0`, aktiv (Feb 2025).
- [crates.io: rhai 1.24](https://crates.io/crates/rhai) — `MIT OR Apache-2.0`, vollständige Skriptsprache, ~250 kB.
- Projekt-intern: `/home/eric/dozenal_calc_flutter/lib/logic/expression.dart` Zeilen 611-924, `/home/eric/dozenal_calc/EXPRESSION_GRAMMAR.md`, `/home/eric/dozenal_calc/src/eval.rs`.
