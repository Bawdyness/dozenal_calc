# Was eine Rust-Crate zu einer macht, die Profis gerne benutzen

Recherche-Notiz als Inspiration und Maßstab für `dozenal_core`. Quellen am Ende.

---

## TL;DR — Die zehn wichtigsten Hebel

1. **Vollständige `Cargo.toml`-Metadaten** (`description`, `keywords`, `categories`, `license`, `repository`, `documentation`, `readme`, `rust-version`) — ohne sie ist die Crate auf crates.io faktisch unsichtbar.
2. **Stabile, schmale öffentliche API** mit `pub use`-Fassade in `lib.rs`; tiefe Modulpfade bleiben Implementierungsdetail.
3. **`#[non_exhaustive]`** auf öffentlichen Error-Enums und ggf. Konfig-Structs — eröffnet additive Versionierungs-Freiheit für 1.x.
4. **Bibliotheks-Fehler mit `thiserror`** (oder hand-rolled), niemals `anyhow` in der öffentlichen API.
5. **Crate-Level-Dokumentation in `//!`** auf der ersten docs.rs-Seite — Pitch + Beispiel + Feature-Tabelle + MSRV-Hinweis.
6. **`#![deny(missing_docs)]`** und `#![forbid(unsafe_code)]` als Compiler-Gates; Doc-Tests sind gleichzeitig Beispiele und Korrektheits-Tests.
7. **`no_std`-Kompatibilität als Default**, `std` als opt-in-Feature — entscheidend für WASM-Bundle-Größe und Embedded-Adoption.
8. **CI-Pipeline mit drei Quality-Gates**: `fmt + clippy -D warnings + test` für jeden PR, dazu `cargo-semver-checks` vor jedem Release.
9. **Semver-Disziplin via Automation**: `cargo-semver-checks` und `release-plz` ersetzen menschliche Diligence — laut Predrag Gruevski verletzen 1 von 6 Top-1000-Crates Semver mindestens einmal.
10. **Changelog im Keep-a-Changelog-Format**, conventional commits, kein "see git log" — Release-Notes sind die Stelle, an der Profis Vertrauen aufbauen.

---

## 1 — `Cargo.toml`-Hygiene

**Was Profis erwarten.** Eine Crate-Detailseite auf crates.io ist die erste Begegnung. Fehlen `description` oder `keywords`, vermutet der Leser ein Hobby-Projekt. Verbindlich aus den Rust API Guidelines (Necessities, Documentation):

- `description` — ein Satz, der die Crate für jemanden erklärt, der gerade gesucht hat.
- `keywords` — bis zu fünf, kleingeschrieben, ohne Bindestriche; bei Mathe-Crates z. B. `["rational","arithmetic","base","fraction","duodecimal"]`.
- `categories` — aus der [festen crates.io-Liste](https://crates.io/category_slugs). Für `dozenal_core` passen `mathematics`, `science`, `no-std`, `wasm`.
- `license` — SPDX-Ausdruck. Industrie-Standard für duale Lizenz: `"MIT OR Apache-2.0"` (die Großschreibung des `OR` ist normativ).
- `repository`, `homepage`, `documentation` — sonst zeigt docs.rs keine Quell-Links neben dem Header.
- `readme = "README.md"` — wird auf der Crate-Seite gerendert.
- `rust-version = "1.XX"` — MSRV pinnen. Empfehlung: ein konservativer Wert (z. B. aktuell − 6 Monate), in CI mit einem `msrv`-Job verifizieren (`cargo +1.XX check`). Ohne MSRV brechen Updates Downstream-Builds unsichtbar.

**Was schief geht.** Crates ohne `categories` tauchen in keiner Browse-Liste auf. Crates ohne `rust-version` werden in Firmen-Builds, die ältere Toolchains pinnen, abgewählt. Ein einzelnes `license = "MIT/Apache-2.0"` (alt) statt `"MIT OR Apache-2.0"` (SPDX) produziert Warnungen im Audit.

**Feature-Flags.** Sparsam und additiv. `default = ["std"]`, `std` aktiviert `std`-Abhängiges, `serde` aktiviert Serde-Impls hinter `#[cfg(feature = "serde")]`. **Niemals subtraktive Features** (kein `no-std`-Feature, das `std` deaktiviert — das verletzt Cargos "features are additive"-Vertrag). Etikette für Downstream: `default-features = false` muss eine sinnvolle minimale Crate ergeben.

**Empfehlung `dozenal_core`.** MSRV 1.74 oder konservativer; Features: `default = ["std"]`, optional `serde` für `Rational`-Serialisierung; `#[no_std]` ohne `alloc` würde Period-Detection unmöglich machen — also `extern crate alloc;` und `default-features = false` ⇒ noch immer mit `alloc`. Workspace-`package.*`-Inheritance benutzen.

## 2 — API-Design

**Was Profis erwarten.** Die Rust API Guidelines bündeln 11 Kategorien Mindeststandard. Konkret zu studieren bei `dozenal_core`:

- **`pub use`-Fassade.** `lib.rs` re-exportiert nur das, was der Konsument konstruieren soll — `Rational`, `RationalParseError`, `Base`, `PeriodicExpansion`, `evaluate`. Module wie `internal_parser`, `gcd`, `digit_buffer` bleiben `pub(crate)`. Sonst wachsen Importpfade ungebremst (`dozenal_core::periodic::detect::detector::DetectionState` ist ein Anti-Pattern).
- **`#[non_exhaustive]` auf öffentlichen Enums** — speziell auf `enum CalcError { DivByZero, Overflow, SyntaxError, … }`. Damit kann man später `Underflow` hinzufügen, ohne die Major-Version zu bumpen. Das ist Standard-Praxis bei `std::io::ErrorKind`, `thiserror`-generierten Errors und allen größeren Bibliotheken.
- **Builder vs. direkte Konstruktoren.** Builder lohnen sich erst ab vier optionalen Parametern oder mehreren Konstruktionswegen mit dem gleichen Ziel. `Rational::new(num, den) -> Result<Self, _>` reicht; ein `RationalBuilder` wäre Overkill.
- **`impl Trait` in Rückgaben** ist erlaubt, aber bei Bibliotheken mit Bedacht: konkrete Typen sind diagnose-freundlicher. Iterator-Rückgaben (z. B. `expansion_digits()`) gerne als `impl Iterator<Item = DozenalDigit> + '_`.
- **Error-Typen mit `thiserror`.** `anyhow` gehört in Anwendungen, nicht in `dozenal_core` — Konsumenten müssen auf `CalcError::DivByZero` matchen können, sonst verlieren sie Auswahl. Manuelle `impl std::error::Error` ist die zero-dependency-Alternative; für `no_std` empfiehlt sich `core::error::Error` (seit 1.81 stabil).
- **Newtypes** statt `f64`-Tupel überall: `struct Base(u32)`, `struct DozenalDigit(u8)` mit Wertebereich-Garantie. Boolesche Parameter werden zu Enums (`Direction::Left`/`Right`), nie zu `bool` — siehe Hertleif.
- **Sealed Traits** — sinnvoll, falls `trait Numeric` eingeführt wird, das nicht für externe Implementierung gedacht ist. Privates Super-Trait `mod sealed { pub trait Sealed {} }`. Verhindert Versions-Brüche durch fremdes `impl`.

**Empfehlung `dozenal_core`.** Konkret schmal: `Rational`, `Base`, `Expression`, `evaluate(expr: &str) -> Result<Value, EvalError>`, `Value::{Rational, Float}`, `Value::expand_in(base: Base) -> Expansion`, `Expansion::{Finite, Periodic}`. Alle Enums `#[non_exhaustive]`. Alle Konstruktoren `Result`-gibend bei Validierung. Keine `unwrap()` im öffentlichen Pfad.

## 3 — Dokumentation

**Was Profis erwarten.** Die docs.rs-Landeseite ist der zentrale Verkaufstext. Vorbild `regex`: 9 Hauptsektionen (Overview, Usage, Examples, Performance, Unicode, Syntax, Untrusted Input, Crate Features, Other Crates). Vorbild `serde`: ein Satz, der in 12 Worten erklärt, was die Crate ist, dann eine fett-formatierte Liste der unterstützten Formate.

Konkrete Bausteine:

- **Crate-Level-Doc** in `//!`-Kommentaren am Anfang von `lib.rs`. Erster Absatz = Pitch. Zweiter Absatz = Mini-Beispiel als Doc-Test. Danach Sektionen `# Features`, `# Examples`, `# MSRV`, `# Comparison with related crates`.
- **Alle öffentlichen Items dokumentieren**, mindestens ein Doc-Test pro nicht-trivialer Funktion. Doc-Tests laufen via `cargo test --doc` automatisch.
- **`#![deny(missing_docs)]`** als CI-Gate. `#![warn(missing_doc_code_examples)]` (nightly) als zusätzliche Anregung.
- **Hyperlinks** — Hertleif/Gomez: "wenn du über einen Typ sprichst, verlinke ihn, immer." Intra-doc-Links: `` [`Rational::new`] ``.
- **`examples/`-Ordner** im Repo, mit eigenständigen, compilierenden `*.rs`-Files. Sie tauchen *nicht* auf docs.rs auf, aber GitHub-Browser sehen sie.
- **README ≠ crate-doc.** README ist Marketing, crate-doc ist Referenz. Mit `cargo-readme` aus `lib.rs` extrahieren, falls sie sich überlappen sollen.

**Was schief geht.** Crates mit leerer Landeseite (nur Modul-Liste) wirken vernachlässigt. Crates ohne ein einziges runnable example zwingen Konsumenten in den Source-Tree.

**Empfehlung `dozenal_core`.** `//!`-Block in `lib.rs` mit: 1 Satz Pitch, 10-Zeilen-Beispiel (`evaluate("1/7")` → "0.186A3̄"), Feature-Tabelle, MSRV-Hinweis, Verweis auf `EXPRESSION_GRAMMAR.md`. Doc-Tests auf `Rational::new`, `evaluate`, `expand_in`. `examples/desk_calculator.rs` mit 30-Zeilen-REPL.

## 4 — Testing

**Was Profis erwarten.** Drei Layer:

- **Unit-Tests** in-Modul (`#[cfg(test)] mod tests`) für Implementierungs-Details — Zugang zu `pub(crate)`-Symbolen.
- **Integration-Tests** in `tests/`-Verzeichnis — kompilieren gegen die öffentliche API. Diese decken die "kann ich diese Crate so benutzen wie dokumentiert"-Frage ab.
- **Doc-Tests** als ausführbare Dokumentation.

Darüber hinaus:

- **Property-Tests mit `proptest`** (moderner als `quickcheck`: per-value strategies, bessere Shrinking). Für `Rational` lohnt sich z. B. die Eigenschaft `(a + b) * c == a * c + b * c` für zufällige `i128`-Paare ohne Überlauf.
- **Fuzzing mit `cargo-fuzz`** für den Parser. Yoshua Wuyts hat darüber geschrieben, dass Property-Tests in CI laufen, Fuzzing kontinuierlich daneben.
- **Mutation-Testing mit `cargo-mutants`** — punktuell, nicht als CI-Gate. Findet Tests, die zu nachsichtig sind.
- **Coverage** mit `cargo-llvm-cov`. Ein Schwellwert (z. B. 80 %) als weicher Indikator, nicht als Gate.

**Empfehlung `dozenal_core`.** Bestehende Tests aus `logic.rs` und `eval.rs` umziehen. Neu hinzu: `tests/integration.rs` (öffentliche API), `proptest`-Sektion für `Rational`-Invarianten (`den > 0`, gcd(num,den) = 1 nach jeder Op, keine Panics bei Overflow), `fuzz/fuzz_targets/parse_expr.rs` für den Parser. CI führt `cargo test`, `cargo test --no-default-features`, `cargo test --all-features` aus.

## 5 — Performance

**Was Profis erwarten.** Bei einer Mathe-Crate ist Performance Teil der Identität.

- **`criterion`-Benchmarks** in `benches/` — statistische Auswertung, HTML-Report. Standard für alles, was `rust-bencher` mal war.
- **Regressions-Schutz** via [CodSpeed](https://codspeed.io): instrumentierte Messung mit <1 % Varianz, auf jeden PR. Alternative: in CI Baseline-JSON checken-in und manuell vergleichen.
- **Benches sind keine Tests.** Sie laufen nicht bei `cargo test`, brechen kein PR — aber Regressionen ohne Erklärung sind ein Code-Review-Kriterium.

**Empfehlung `dozenal_core`.** `benches/rational_ops.rs` (add/mul/div/period_detect), `benches/evaluate.rs` (Parser + Eval auf Standard-Ausdrücken). CodSpeed in der GitHub-Workflow optional, sobald die Crate v0.2 erreicht.

## 6 — Maintenance-Signale

**Was Profis erwarten.** Vor dem Code-Import wird das Repo geprüft. Erwartete Marker:

- **README mit Badges**: CI-Status, crates.io-Version, docs.rs, license, MSRV, downloads. Vorbild `tokio`, `serde`, `regex` — die Top-Zeile ist immer ein Badge-Stripe.
- **`CHANGELOG.md`** im Keep-a-Changelog-Format. `## [Unreleased]` ganz oben, dann `## [0.2.0] - 2026-05-17` mit `### Added/Changed/Fixed/Removed`-Sektionen.
- **`CONTRIBUTING.md`** kurz: wie baut man, wie testet man, was muss vor einem PR grün sein.
- **`SECURITY.md`** mit einer E-Mail-Adresse für Disclosure (auch wenn Mathe-Crate kein hohes Sicherheitsprofil hat — Form vor Inhalt).
- **Issue-Templates** für Bug-Report und Feature-Request.
- **`CODEOWNERS`** (für größere Teams; bei Solo-Crates entbehrlich).
- **Conventional Commits** + `release-plz` als CI-Bot. Der Bot öffnet automatisch eine "Release v0.3.0"-PR mit aktualisierter `Cargo.toml`, `Cargo.lock` und `CHANGELOG.md`. Merge ⇒ Tag + Publish auf crates.io.
- **Semver-Disziplin**: `cargo-semver-checks` als CI-Job, der die öffentliche API gegen die letzte Published-Version diff't. Standard bei `tokio`, `cargo` selbst, PyO3.

**Empfehlung `dozenal_core`.** Alle obigen Files anlegen, ehe v0.1.0 publiziert wird. Badges in der README oben: CI, crates.io, docs.rs, MSRV, license. `release-plz` ab v0.2.0 einbinden — vor v0.1.0 manuell, weil Conventional Commits noch nicht eingespielt sind.

## 7 — WASM-Spezifisches

**Was Profis erwarten.** Der Dozenal-Rechner lebt auch im Browser. Konsumenten von `dozenal_core` werden ebenfalls oft WASM bauen.

- **`no_std` als Default**, `std` als opt-in. Auf `wasm32-unknown-unknown` ist `std` zwar verfügbar, aber `no_std`-Crates haben kleinere Bundles und sind in Embedded brauchbar.
- **`extern crate alloc;`** — `String`, `Vec`, `Box` ohne `std`. Period-Detection braucht `Vec<u8>` für Remainder-Tabellen; das ist `alloc`-only, kein Problem.
- **Niemals `getrandom` mit `js`-Feature in der Bibliothek aktivieren.** Die offizielle Doku warnt: "strongly recommended against enabling this feature in libraries (except for tests) since it is known to break non-Web WASM builds". Der Endkunde (Anwendung) entscheidet das.
- **Pure Rust, keine C-Bindings.** Keine `cc`/`bindgen`-Abhängigkeiten — sie brechen `wasm32-unknown-unknown` und `wasm32-wasi` häufig.
- **Bundle-Awareness.** `wasm-opt`-freundliche Symbolnamen (kurz wo möglich), keine `panic!`-Texte mit langen Format-Strings im Hot-Path.

**Empfehlung `dozenal_core`.** `#![no_std]` mit `extern crate alloc;`. Feature `std` aktiviert `f64`-Pfad (`f64::sin` etc.); ohne `std` ist die Crate rational-only. Keine `getrandom`-Abhängigkeit. CI-Job `cargo check --target wasm32-unknown-unknown --no-default-features`.

---

## Vorbild-Crate-Galerie

- **`serde`** — Goldstandard für Trait-zentrierte APIs. Crate-Doc beginnt mit einem Satz, der das Wertversprechen klärt; `#[derive(Serialize, Deserialize)]` ist ein Lehrstück für ergonomische Macros. Aber: monolithisch in der Ableitung, daher kein Vorbild für `no_std`-Schlankheit.
- **`regex`** — die docs.rs-Landeseite ist Lehrbuch-Qualität: neun klar abgegrenzte Sektionen, Performance offen besprochen, Untrusted-Input-Sektion explizit. Wer einen Tag in dieser Doku verbringt, weiß, wie man Doku schreibt.
- **`tokio`** — Maßstab für Maintenance-Signale: CI, semver-checks, release-plz, conventional commits, Triage-Bot, klar separierte Features (`rt`, `net`, `sync`, …) mit transparenter Doku.
- **`thiserror`** + **`anyhow`** (David Tolnay) — minimalistisch, schmal, eine Aufgabe, makellos. Vorbild für Crate-Größe, die `dozenal_core` anstreben sollte: lesbare `lib.rs`, fokussierter Scope.
- **`ibig`** — vergleichbar groß zu `dozenal_core` (pure-Rust BigInt). API-Design lohnt sich zu studieren: zwei separate Typen (`UBig`/`IBig`), `ubig!`-Macro für Literale, `modular`-Submodul. Doku-Coverage ist nur ~61 %, das ist ein Negativ-Beispiel.
- **`chumsky`** — Parser-Bibliothek mit ausführlichem Guide *außerhalb* der rustdoc, Beispiel-Projekte vom Brainfuck bis ML-Interpreter, explizites `no_std`. Vorbild für eine Crate, die Lern-Material wie ein Buch behandelt.
- **`time`** — saubere v0.3-Migration mit Semver-Disziplin; Beispiel dafür, dass Major-Bumps Ankündigung und Begründung verdienen.
- **`itertools`** — additive Erweiterungen via Extension-Trait, eindeutiges Single-Purpose. Lehrt, was eine "Mitnimm-Crate" ausmacht.

Die README, die sich am meisten lohnt zu lesen: **`regex`** — wegen der "Performance"- und "Untrusted Input"-Sektionen, die zeigen, wie man Trade-offs ehrlich kommuniziert.

## Anti-Patterns

- **`unwrap()`/`panic!()` im öffentlichen Pfad** statt `Result`-Rückgaben. Konsumenten können nicht erkennen, dass eine Funktion panicken kann.
- **Ungelinkte Modul-Hierarchie**: `crate::foo::bar::baz::qux::Thing`. Re-exports fehlen, Konsument muss raten, wo etwas lebt.
- **Subtraktive Features** (`no-std` als Feature, das `std` ausschaltet). Bricht Cargos additive-feature-Vertrag in Workspaces.
- **`anyhow::Error` in der öffentlichen API**. Konsument kann nicht selektiv handhaben.
- **Versteckte Breaking Changes** ohne Major-Bump: Trait-Bounds verschärfen, Felder zu öffentlichen Structs hinzufügen ohne `#[non_exhaustive]`, MSRV-Erhöhung in einer Patch-Version.
- **`README.md` und `lib.rs`-Doc divergieren.** Konsument liest README, kommt zu docs.rs, findet anderes — Vertrauensverlust.
- **Doc-Tests deaktiviert** (`no_run` überall). Beispiele compilieren nicht mehr, niemand bemerkt es.
- **C-Bindings für Convenience.** Killt WASM und macht Audit teurer.
- **`default-features = true` mit schwergewichtigen Defaults.** Konsument zieht ohne Vorwarnung 14 Transitive-Abhängigkeiten.
- **Leeres CHANGELOG** oder "see git log". Profis lesen Release-Notes vor dem Update.

## CI-Skizze für `dozenal_core`

Ein GitHub-Actions-Workflow `.github/workflows/ci.yml` mit den folgenden Jobs:

| Job | Trigger | Befehle | Gate? |
|---|---|---|---|
| `fmt` | jeder PR | `cargo fmt --all --check` | ja |
| `clippy` | jeder PR | `cargo clippy --all-targets --all-features -- -D warnings` | ja |
| `test-stable` | jeder PR | `cargo test --all-features` | ja |
| `test-no-default` | jeder PR | `cargo test --no-default-features` | ja |
| `test-msrv` | jeder PR | `cargo +1.74 check --all-features` | ja |
| `wasm-check` | jeder PR | `cargo check --target wasm32-unknown-unknown --no-default-features` | ja |
| `doc` | jeder PR | `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features` | ja |
| `semver-checks` | bei Tag/Release | `cargo semver-checks check-release` | ja |
| `coverage` | Push auf `main` | `cargo llvm-cov --all-features --lcov --output-path lcov.info` + Codecov-Upload | nein |
| `bench` | manuell oder wöchentlich | `cargo bench --no-fail-fast` | nein |
| `release-plz` | Push auf `main` | `release-plz/action@v0.5` | (öffnet PR) |

Badges in der README: `[CI]` (Actions-Status), `[crates.io]` (Version), `[docs.rs]`, `[MSRV 1.74+]`, `[License MIT or Apache-2.0]`, optional `[codecov]`.

## Quellen

- [Rust API Guidelines — Checkliste](https://rust-lang.github.io/api-guidelines/checklist.html)
- [The Cargo Book — Manifest Format](https://doc.rust-lang.org/cargo/reference/manifest.html)
- [The Cargo Book — SemVer Compatibility](https://doc.rust-lang.org/cargo/reference/semver.html)
- [The Cargo Book — Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [Rustdoc Book — Wie man Doku schreibt](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html)
- [Effective Rust — Item 27: Document public interfaces](https://effective-rust.com/documentation.html)
- [Effective Rust — Item 21: Understand SemVer](https://lurklurk.org/effective-rust/semver.html)
- [Pascal Hertleif — Elegant Library APIs in Rust](https://deterministic.space/elegant-apis-in-rust.html)
- [Guillaume Gomez — Guide on how to write documentation for a Rust crate](https://blog.guillaume-gomez.fr/articles/2020-03-12+Guide+on+how+to+write+documentation+for+a+Rust+crate)
- [Predrag Gruevski — SemVer in Rust: Tooling, Breakage, and Edge Cases (FOSDEM 2024)](https://predr.ag/blog/semver-in-rust-tooling-breakage-and-edge-cases/)
- [Yoshua Wuyts — Bridging fuzzing and property testing](https://blog.yoshuawuyts.com/bridging-fuzzing-and-property-testing/)
- [Luca Palmieri — Error Handling In Rust: A Deep Dive](https://www.lpalmieri.com/posts/error-handling-rust/)
- [Luca Palmieri — An Introduction to Property-Based Testing in Rust](https://lpalmieri.com/posts/an-introduction-to-property-based-testing-in-rust/)
- [cargo-semver-checks](https://github.com/obi1kenobi/cargo-semver-checks)
- [cargo-public-api](https://github.com/cargo-public-api/cargo-public-api)
- [release-plz](https://release-plz.dev/)
- [criterion.rs](https://github.com/bheisler/criterion.rs)
- [CodSpeed — Writing Benchmarks with criterion.rs](https://codspeed.io/docs/benchmarks/rust/criterion)
- [proptest](https://lib.rs/crates/proptest)
- [getrandom docs — WASM-Empfehlungen](https://docs.rs/getrandom)
- [RFC 2008 — `#[non_exhaustive]`](https://rust-lang.github.io/rfcs/2008-non-exhaustive.html)
- [Rust Patterns — Privacy For Extensibility](https://rust-unofficial.github.io/patterns/idioms/priv-extend.html)
- [Rust API Guidelines — Future Proofing (Sealed Traits)](https://rust-lang.github.io/api-guidelines/future-proofing.html)
- [docs.rs — `regex`](https://docs.rs/regex/latest/regex/)
- [docs.rs — `serde`](https://docs.rs/serde/latest/serde/)
- [docs.rs — `chumsky`](https://docs.rs/chumsky/latest/chumsky/)
- [docs.rs — `ibig`](https://docs.rs/ibig/latest/ibig/)
- [Keep a Changelog](https://keepachangelog.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
