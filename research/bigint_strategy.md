# BigInt-Strategie für `dozenal_core` (Rust-WASM)

## TL;DR

**Empfehlung: `num-bigint` 0.4 mit `default-features = false, features = ["std"]` ausschliesslich auf der Rational-Schiene.** Pures Rust, `no_std`-fähig, dual-lizenziert MIT/Apache-2.0, kanonische Wahl mit ~25 Mio. Downloads pro Monat und letztem Release Juni 2024. Das löst den Overflow-Kollaps für alle realistischen Calculator-Eingaben (Periodenerkennung mit Nennern bis zu hunderten von Stellen, `frac_den = 12.pow(n)` ohne 38-Stellen-Cap, oplus mit grossen Brüchen). Der Bundle-Mehraufwand bleibt unter ~30 kB nach `wasm-opt -Oz` + Brotli und ist für eine wiederverwendbare Crate akzeptabel.

## Vergleichstabelle

| Kriterium | `i128` (heute) | `num-bigint` 0.4.6 | `ibig` 0.3.6 | `dashu` 0.4.2 | `malachite-nz` 0.9.1 | `rug` | Hand-rolled |
|---|---|---|---|---|---|---|---|
| Letztes Release | – | 2024-06 | 2022-09 | 2024-01 | 2026-02 | 2024 | – |
| Pure Rust | ja | ja | ja | ja | ja | **nein (GMP-FFI)** | ja |
| WASM (`wasm32-unknown-unknown`) | ja | ja | ja | ja | ja | **nein** | ja |
| `no_std` | ja | ja (feat-flag) | ja | ja (voll) | ja | nein | ja |
| Lizenz | – | MIT/Apache-2.0 | MIT/Apache-2.0 | MIT/Apache-2.0 | **LGPL-3.0-only** | LGPL-3+ | – |
| MSRV | – | 1.60 | 1.49 | 1.61 | aktuell | – | – |
| Verbreitung (Downloads/Monat) | – | ~25 Mio | ~29 k | ~94 k | gering | – | – |
| Helfer-Methoden (`gcd`, `pow`, `checked_*`) | manuell | `num-integer`-Trait, `pow` | reichhaltig | reichhaltig | reichhaltig | reichhaltig | minimal |
| Performance kleine Operanden (bigint-benchmark-rs) | Baseline (am schnellsten) | langsamster der Pure-Rust-Sets (~3× ibig auf `e 100k`) | schnell | schnell (~`e 100k`: 0.019s) | sehr schnell (~`e 100k`: 0.012s) | am schnellsten | unbekannt |
| Aktive Wartung | – | hoch (rust-num Org) | **stagniert seit 2022** | aktiv (cmpute) | aktiv (mhogrefe) | aktiv | – |
| Implementierungs-Aufwand für uns | null | minimal | minimal | minimal | minimal | n/a | **hoch (~500 LOC + Tests)** |

## Bundle-Daten (geschätzt)

Direkte Messungen für `wasm32-unknown-unknown` mit `wasm-opt -Oz` + Brotli publizieren die Maintainer nicht. Die folgenden Werte sind Schätzungen aus Source-Grösse, lib.rs-Metadaten und Community-Erfahrungswerten:

| Crate | Rohgrösse Quelle | Geschätzt `.wasm` (Oz) | Geschätzt nach Brotli | Quelle/Begründung |
|---|---|---|---|---|
| `num-bigint` 0.4 | ~295 kB / 7'500 LOC | ~80–110 kB | **~25–35 kB** | Source via lib.rs; LLVM-DCE entfernt ungenutzte Pfade aggressiv, weil wir nur `BigInt`, `gcd`, `pow`, `Rem`, `Div` brauchen. |
| `ibig` 0.3 | ähnliche LOC-Grössenordnung | ~90–120 kB | ~30–40 kB | Vergleichbare Funktions-Surface. |
| `dashu` 0.4 (full) | grösser (5 Sub-Crates) | ~150–220 kB | ~50–70 kB | Volles Set inkl. Float/Rational; nur `dashu-int` zu importieren ist möglich aber mehr Konfig-Aufwand. |
| `malachite-nz` 0.9 | ~12 MB / 258k LOC | ~250–400 kB | ~80–120 kB | Sehr grosse Code-Base inkl. abgeleiteter GMP/FLINT-Algorithmen. |
| Hand-rolled | ~500 LOC | ~15–25 kB | **~5–10 kB** | Nur das, was wir brauchen. |

Der derzeitige WASM-Bundle-Stand des dozenal_calc liegt nach den existierenden Build-Logs grob bei 1.5–2.5 MB komprimiert (egui dominiert). `+25–35 kB` für `num-bigint` ist im einstelligen Prozentbereich.

## Überlauf-Analyse: Wann kollabiert `i128` heute?

`i128` packt 39 Dezimalstellen (max ≈ 1.7e38). Empirische Schwellen (Python-Validierung, siehe Notiz unten):

| Eingabe | Effekt | Kollaps? |
|---|---|---|
| `0.123456789AB` (dozenale Eingabe mit 11 Nachkommastellen) | `frac_den = 12^11 ≈ 7.4e11` (12 Stellen) | nein |
| `12^35` als Zwischenergebnis (z. B. `12^35`-Eingabe) | 38 Stellen, passt knapp | nein |
| **`12^36` als Zwischenergebnis** | **39 Stellen, überschreitet i128 (1.7e38 < 7.1e38)** | **ja** |
| **`11^40` (`B^40`) als Zwischenergebnis** | **42 Stellen** | **ja** |
| **`(1/7 + 1/13) * (1/17 + 1/19) * (1/23 + 1/29)`** | LCM-Nenner explodieren, `checked_mul` in `Rational::add` schlägt fehl, sobald `self.num * rhs.den` > i128 | **ja, bei ~10–12 nicht-trivialen Brüchen verkettet** |
| **`oplus(a, b)` mit `a, b > 1.3e19`** | `a*b` quadratisch → i128-Overflow (Quadratwurzel-Schwelle ≈ 1.3e19) | **ja, ab ~20-stelligen Operanden** |
| **Periodendetektions-Loop bei `den` > `i128_max / 12 ≈ 1.4e37`** | `rem *= 12` (Zeile 285 in `logic.rs`) ist **kein `checked_mul`** — im Release silent wrap, falscher Period | **ja, latent** |

**Realistische Frequenz heute:** Im Standard-Einsatz (kurze Eingaben, einzelne Brüche) tritt der Kollaps selten auf. Aber er ist unsystematisch: Sobald ein Nutzer chains baut (`Ans * 1/7`, dann `* 1/11`, dann `* 1/13`...), wechselt die Periode bei zufälliger Tiefe spontan in den f64-Fallback und der Überstrich verschwindet. Genau dieses Verhalten ist es, was der Flutter-Port mit `BigInt` eliminiert hat.

**Zudem:** Der Periodenerkennungs-Loop (`to_dozenal_periodic`, Zeilen 285-287 von `src/logic.rs`) verwendet `rem *= 12` **ohne** `checked_mul`. Das ist im Debug-Mode ein Panic, im Release ein Wrap — also eine bestehende latente Korrektheits-Lücke jenseits des reinen "Track kollabiert sauber"-Verhaltens.

## Empfehlung mit Begründung (~200 Wörter)

`num-bigint` 0.4 ist die richtige Wahl. Ausschlaggebend sind drei Punkte:

**1. Korrektheit ist die Motivation, nicht Performance.** Der Wechsel löst das `Option<Rational>`-Threading durch jeden Operator, eliminiert den Periodizitäts-Verlust in verketteten Brüchen und schliesst die latente Wrap-Lücke in `to_dozenal_periodic`. Performance ist nebensächlich — ein Taschenrechner berechnet eine Operation pro Tastendruck, nicht Millionen.

**2. Konservative Wahl für eine wiederverwendbare Crate.** `num-bigint` ist Teil der `rust-num`-Organisation mit ~25 Mio. monatlichen Downloads, kontinuierlicher Wartung und der breitesten Trait-Integration im Ökosystem (`num-integer`, `num-traits`). Nachnutzer einer `dozenal_core`-Crate werden eher Dependencies mit `num-bigint` haben als mit `ibig` oder `dashu`. `ibig` hat seit September 2022 kein Release mehr — für eine v1-Crate ist das ein Risiko.

**3. Lizenz und Bundle sind kein Hindernis.** Dual MIT/Apache-2.0 passt zur Projekt-Lizenz-Strategie. Der Bundle-Mehraufwand von ~25–35 kB nach Brotli ist gegenüber dem aktuellen ~2-MB-egui-Bundle nicht messbar.

**Verworfen:** `rug` (GMP-FFI, kein WASM), `malachite-nz` (LGPL inkompatibel mit MIT/Apache-Dual-License-Plan), `ibig` (Stagnation), `dashu` (jung, weniger etabliert), Hand-rolled (Wartungs-Last für Gewinn von ~20 kB), Fortführung von `i128` (Korrektheits-Lücken bleiben).

## Migrations-Skizze

```toml
# Cargo.toml
[dependencies]
num-bigint = { version = "0.4", default-features = false, features = ["std"] }
num-integer = { version = "0.1", default-features = false, features = ["std"] }
num-traits  = { version = "0.2", default-features = false, features = ["std"] }
```

```rust
// src/logic.rs (Neufassung von Rational)
use num_bigint::BigInt;
use num_integer::Integer;       // gcd, div_rem
use num_traits::{One, Zero, Signed};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rational {
    pub num: BigInt,
    pub den: BigInt,  // Invariante: den > 0
}

impl Rational {
    /// Einziger verbleibender Fehlerfall: Division durch null.
    pub fn new(num: BigInt, den: BigInt) -> Option<Self> {
        if den.is_zero() { return None; }
        let g = num.gcd(&den);
        let (num, den) = (num / &g, den / &g);
        // den-Vorzeichen normalisieren
        Some(if den.is_negative() {
            Self { num: -num, den: -den }
        } else {
            Self { num, den }
        })
    }

    pub fn zero() -> Self { Self { num: BigInt::zero(), den: BigInt::one() } }
    pub fn one()  -> Self { Self { num: BigInt::one(),  den: BigInt::one() } }

    /// Addition ohne Overflow-Pfad — gibt immer `Self` zurück.
    pub fn add(&self, rhs: &Self) -> Self {
        let num = &self.num * &rhs.den + &rhs.num * &self.den;
        let den = &self.den * &rhs.den;
        // SAFETY: den = self.den * rhs.den, beide > 0 → den > 0, also reduziert new() niemals zu None
        Self::new(num, den).unwrap()
    }

    /// Division: einziger Option-Pfad bleibt erhalten (rhs.num == 0).
    pub fn div(&self, rhs: &Self) -> Option<Self> {
        if rhs.num.is_zero() { return None; }
        let num = &self.num * &rhs.den;
        let den = &self.den * &rhs.num;
        Self::new(num, den)
    }
    // ... mul, sub, pow, oplus analog: add/sub/mul/pow nicht mehr Option, nur div und oplus.
}
```

**Auswirkungen auf den Parser `eval_rational` und Aufrufer:**
- `parse_add_sub` / `parse_mul_div` / `parse_pow` geben weiter `Option<Rational>` zurück, aber das `?` an Overflow-Stellen verschwindet — übrig bleiben nur `?` für Div/0 und ungültige Exponenten (`exp.den != 1`).
- `eval.rs` Zeilen 56-57, 77-78: `12_i128.checked_pow(...)?` wird `BigInt::from(12).pow(...)` (kein `?` mehr).
- `to_dozenal_periodic`: `rem *= 12` wird `rem = &rem * BigInt::from(12)`. Die `HashMap<BigInt, usize>`-Variante braucht `Hash` (von `num-bigint` mitgeliefert).
- Tests in `logic.rs` und `eval.rs` müssen `Rational::new(p, q).unwrap()` auf `Rational::new(BigInt::from(p), BigInt::from(q)).unwrap()` umstellen — mechanisch.
- `Rational` ist nicht mehr `Copy`. Aufrufe wandeln sich von `r.add(s)` auf `r.add(&s)` (oder Ableitung `Clone` + `.clone()` an Hot-Spots).
- `Memory` (`tokens.rs`) hält jetzt `Rational` heap-allocated statt 32 Byte stack — irrelevant für die Datenmenge.

## Quellen

- [num-bigint auf crates.io](https://crates.io/crates/num-bigint) — 25 Mio Downloads/Monat, Release 0.4.6 (Juni 2024)
- [num-bigint auf lib.rs](https://lib.rs/crates/num-bigint) — Lizenz, no_std, Code-Grösse
- [num-bigint GitHub](https://github.com/rust-num/num-bigint) — Repo der rust-num Org
- [ibig auf lib.rs](https://lib.rs/crates/ibig) — Release 0.3.6 (Sep 2022), seitdem keine Release
- [ibig GitHub](https://github.com/tczajka/ibig-rs) — 106 Sterne, Stagnation
- [dashu auf lib.rs](https://lib.rs/crates/dashu) — Release 0.4.2 (Jan 2024), aktiv
- [dashu-int docs.rs](https://docs.rs/dashu-int/latest/dashu_int/) — IBig/UBig API
- [dashu GitHub (cmpute)](https://github.com/cmpute/dashu) — Pure Rust, no_std, 345 Commits
- [malachite-nz auf lib.rs](https://lib.rs/crates/malachite-nz) — LGPL-3.0-only, Release 0.9.1 (Feb 2026), 12 MB Quelle
- [malachite.rs](https://www.malachite.rs/) — Algorithmen aus GMP/FLINT abgeleitet
- [bigint-benchmark-rs (tczajka)](https://github.com/tczajka/bigint-benchmark-rs) — Vergleichsbenchmark: rug (0.009s) < malachite (0.012s) < dashu (0.019s) ≈ ibig (0.020s) < num-bigint (0.058s) auf `e 100k`
- [num-rational RELEASES](https://github.com/rust-num/num-rational/blob/master/RELEASES.md) — i128/u128-Ratio-Komponenten seit Rust 1.26
- [Rust+WASM Code-Size Book](https://rustwasm.github.io/book/reference/code-size.html) — LLVM-DCE und `wasm-opt -Oz` als Standard-Toolchain
- [Leptos Binary-Size-Guide](https://book.leptos.dev/deployment/binary_size.html) — Brotli-Empfehlungen
- Lokale Overflow-Analyse via Python — siehe Sektion "Überlauf-Analyse" oben (Reproduktion: `python3 -c "print(12**36, 2**127-1)"`)
