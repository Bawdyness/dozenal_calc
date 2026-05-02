# EXPRESSION_GRAMMAR.md — Eingabesprache und Auswertungsmodell

Dieses Dokument beschreibt die Eingabesprache des dozenal_calc formal, sodass eine Re-Implementierung in einer anderen Sprache möglich ist, ohne den Rust-Code (`src/eval.rs`, `src/logic.rs`) lesen zu müssen.

Quelle der Wahrheit:
- Token-Definitionen: `src/tokens.rs:7–68`
- Rational-Track-Parser: `src/logic.rs:317–460`
- f64-Track + Custom-Operator-Auflösung: `src/eval.rs:179–217`, `src/eval.rs:512–556`
- Implicit-Multiplication-Regeln: `src/eval.rs:225–262`

## Überblick: Zwei-Spuren-Auswertung

Bei jedem Druck auf `=` wird der `input_buffer` **zweimal** ausgewertet:

1. **Rational-Track** (exakt): Der Token-Stream wird in einen Pratt-ähnlichen rekursiven Abstiegsparser eingespeist, der mit `i128`-Brüchen rechnet. Liefert entweder ein exaktes `Rational` oder `None` (Track kollabiert).
2. **f64-Track** (Näherung): Derselbe Token-Stream wird in einen Infix-String übersetzt und an `meval` (oder einen funktional äquivalenten Float-Expression-Evaluator) übergeben. Liefert immer ein f64-Ergebnis (oder einen Syntaxfehler).

Wenn der Rational-Track ein Ergebnis liefert, wird es für die Anzeige bevorzugt — exakt mit Periodendetektion. Sonst wird das f64-Ergebnis mit fester Anzahl Bruchziffern angezeigt.

**Collapse-Bedingungen für den Rational-Track:**
- Ein nicht-rationaler Token im Stream (siehe Token-Tabelle, Spalte "rational?"): trigonometrische Funktionen, Hyperboliken, Logarithmus, irrationale Konstanten, `√` (außer wenn Ergebnis ganzzahlig ist — wird aber im aktuellen Code nicht als Sonderfall behandelt; jedes `√` collapsed)
- `i128`-Überlauf in einer Zwischenrechnung (`checked_*`-Arithmetik returnt `None`)
- Division durch Null
- Nicht-ganzzahliger Exponent in `^` (z.B. `2 ^ 0.5`)
- Syntaktisch kaputter Stream (unbalancierte Klammern, dangling Operator)

## Token-Tabelle

Die App-Zustands-Tokens (`Expand`, `AC`, `Del`, `=`, `STO`, `RCL`, `MC`, `Drg`, `DozDec`, `Info`, `Close`) sind keine Ausdrucks-Tokens — sie werden in `input.rs` gesondert behandelt und kommen niemals im `input_buffer` vor. Folgende Tabelle listet nur die Tokens, die im Ausdrucks-Stream auftauchen können:

### Werte-Tokens

| Token | Display | Bedeutung | rational? |
|---|---|---|---|
| `Digit(D0..D11)` | Custom-Glyph | Dozenale Ziffer 0–11 | ja (Teil eines Zahlen-Literals) |
| `Decimal` | `.` | Dozenaler Bruchpunkt | ja |
| `RatLit(r)` | `Ans` | Eingebetteter exakter Rational-Wert (von Ans/RCL) | ja |
| `ConstPi` | `π` | π ≈ 3.14159265… | nein (kollabiert Track) |
| `ConstE` | `e` | e ≈ 2.71828… | nein |
| `ConstPhi` | `φ` | φ = 1.618033988749895 | nein |
| `ConstSqrt2` | `√2` | √2 ≈ 1.41421… | nein |

**Zahlen-Literale** entstehen aus einer Folge von `Digit`-Tokens, optional unterbrochen von einem einzelnen `Decimal`. Beispiele:
- `[D5]` → `5` dezimal = `5` dozenal
- `[D1, D0]` → `12` dezimal = `10` dozenal
- `[D0, Decimal, D6]` → `0 + 6/12 = 0.5` dezimal = `0.6` dozenal
- `[Decimal, D6]` (führender Dezimalpunkt) → `0.5` dezimal = `0.6` dozenal

### Binäre Operatoren

| Token | Display | Operation | Präzedenz | Assoziativität | rational? |
|---|---|---|---|---|---|
| `Add` | `+` | Addition | 1 | links | ja |
| `Sub` | `−` | Subtraktion / unäres Minus | 1 (binär), 3 (unär) | links / rechts | ja |
| `Mul` | `×` | Multiplikation | 2 | links | ja |
| `Div` | `÷` | Division | 2 | links | ja |
| `OplusBotLeft` | `⊕` | Paralleladdition `(a*b)/(a+b)` | 2 | links | ja |
| `ExpTopRight` | `^` | Potenz | 3 | rechts | ja, falls Exponent ganzzahlig |
| `RootTopLeft` | `√` | Quadratwurzel oder n-te Wurzel | siehe unten | siehe unten | **nein** (immer Collapse) |
| `LogBotRight` | `log` | Logarithmus zur Basis | siehe unten | siehe unten | **nein** |

### Unäre Operatoren

| Token | Display | Operation | Position |
|---|---|---|---|
| `Negate` | `−` | unäres Minus (entsteht im Result-Buffer für negative Zahlen; im Input-Buffer wird `Sub` verwendet, der Parser unterscheidet kontextuell) | Präfix |
| `Factorial` | `n!` | Fakultät | wird als unäre Funktion `fact(...)` realisiert; technisch Funktions-Aufruf, kein Postfix-Operator |
| `AbsVal` | `\|x\|` | Absolutbetrag | als `abs(...)` |
| `Reciprocal` | `1/x` | Kehrwert | als `recip(...)` |
| `Mod` | `mod` | Rest der Division | binär, im aktuellen Code nicht in Tabellen-Präzedenz integriert (meval-only) |

### Funktionen (alle kollabieren den Rational-Track)

Trigonometrisch: `Sin`, `Cos`, `Tan`, `Cot`, `ArcSin`, `ArcCos`, `ArcTan`, `ArcCot`
Hyperbolisch: `Sinh`, `Cosh`, `Tanh`, `Coth`, `ArSinh`, `ArCosh`, `ArTanh`, `ArCoth`

### Klammern

| Token | Display | rational? |
|---|---|---|
| `ParenOpen` | `(` | ja |
| `ParenClose` | `)` | ja |

## Rational-Track-Grammatik (EBNF)

Direkt aus `RatParser` in `src/logic.rs`:

```
expression  = additive
additive    = multiplicative { ("+" | "-") multiplicative }
multiplicative = unary { ("*" | "/" | "⊕") unary }
unary       = ("+" | "-") unary | power
power       = primary [ "^" unary ]            (rechtsassoziativ über unary)
primary     = number | "(" expression ")"
number      = (Rational-Literal aus Digit/Decimal-Folge oder eingebettetes RatLit)
```

Anmerkungen:
- `power` ist rechtsassoziativ: `2^3^2 = 2^(3^2) = 2^9 = 512` (nicht `(2^3)^2 = 64`).
- `^` mit nicht-ganzzahligem Exponent collapsed den Track sofort.
- `⊕` hat dieselbe Präzedenz wie `*`/`/` und ist linksassoziativ.

## f64-Track-Custom-Operator-Auflösung

Der f64-Track baut keinen eigenen Parser, sondern eine Infix-String-Repräsentation für `meval`. Dafür werden die drei Custom-Operatoren `⊕`, `√`, `log` als String-Rewrites aufgelöst, **bevor** der String an `meval` geht. Die Auflösung erkennt Klammerausdrücke als ganze Operanden (nicht nur einzelne Tokens).

### `⊕` (Paralleladdition)

Mit Operanden `a` und `b` (beide können Zahlen, Variablen oder geklammerte Ausdrücke sein):
```
a ⊕ b   →   ((a * b) / (a + b))
```
Beispiel: `5 ⊕ (3+2)` → `((5 * (3+2)) / (5 + (3+2)))` = 5/2.

### `√` (Wurzel)

Kontext-abhängig:
- **Wenn der vorangehende Token ein Operator (`+`, `−`, `*`, `/`) oder `(` ist, oder `√` am Anfang steht**: Quadratwurzel.
  ```
  √ x   →   (x ^ (1/2))
  ```
- **Sonst (vorangehende Zahl/Klammerausdruck):** n-te Wurzel.
  ```
  n √ x   →   (x ^ (1/n))
  ```

Beispiele:
- `√16` → `(16 ^ (1/2))` = 4
- `3√27` → `(27 ^ (1/3))` = 3
- `5 + √9` → `5 + (9 ^ (1/2))` = 8

Im Rational-Track wird `√` derzeit nicht aufgelöst — er erscheint als Token, der nicht zu einem `RatExpr` mappt, und collapsed den Track. Der f64-Track liefert das Ergebnis.

### `log` (Logarithmus zur Basis)

Mit Argument `x` links und Basis `n` rechts:
```
x log n   →   (ln(x) / ln(n))
```
Beispiele:
- `64 log 2` → `(ln(64) / ln(2))` = 6
- `64 log (2+2)` → `(ln(64) / ln((2+2)))` = 3

`log` collapsed den Rational-Track (kein `RatExpr`-Mapping).

## Implicit-Multiplication

Zwischen zwei Tokens wird automatisch `*` eingefügt, wenn (a) der erste Token einen Wert produziert und (b) der zweite Token eine neue Sub-Expression beginnt. Aus `src/eval.rs:225–262`.

**Linker Token produziert einen Wert**, wenn er einer der folgenden ist (und der nächste Token nicht zur selben Zahl gehört):
- letzte `Digit` einer Zahl
- `ParenClose` (`)`)
- `RatLit` (Ans-Literal)
- Konstante: `ConstPi`, `ConstE`, `ConstPhi`, `ConstSqrt2`

**Rechter Token beginnt eine neue Sub-Expression**, wenn er einer der folgenden ist:
- `Digit` (= neue Zahl)
- `ParenOpen` (`(`)
- `RatLit`
- Konstante (`ConstPi`, `ConstE`, `ConstPhi`, `ConstSqrt2`)
- Funktion: alle Trig/Inverse-Trig/Hyperbol/`AbsVal`/`Reciprocal`

**Nicht eingefügt** wird ein impliziter `*` zwischen zwei `Digit`/`Decimal`-Tokens (das ist ja Teil derselben Zahl), und vor/nach Operatoren oder Klammer-Schließung.

Beispiele:
- `π π` → `π * π`
- `2 (` → `2 * (`
- `) (` → `) * (`
- `2 sin` → `2 * sin`
- `Ans 5` → `Ans * 5`

## Trigonometrische Konventionen

**Winkelmodus** (Set 10.2 `DRG`-Taste): zyklisch zwischen `RAD` (Default), `DEG`, `GRAD`. Steuert sowohl Vorwärts- als auch Rückwärts-Trig.

**Forward-Trig** (`sin`, `cos`, `tan`, `cot`): Eingang in der aktuellen Modus-Einheit, intern in Radianten konvertiert, dann an die f64-Trig-Funktion übergeben.

**Inverse-Trig** (`asin`, `acos`, `atan`, `acot`): Liefert Wert in Radianten, dann in den aktuellen Modus konvertiert.

**`acot` Convention A** (Range `(0, π)`, Formel `π/2 − atan(x)`):
- `acot(0) = π/2`
- `acot(1) = π/4`
- `acot(−1) = 3π/4`

Diese Konvention liefert positive Werte für alle reellen `x`. Convention B (`atan(1/x)`, Range `(−π/2, π/2]`) wird **nicht** verwendet.

**`cot(x) = 1 / tan(x)`** — ohne Sonderfall-Behandlung; gibt `±∞` für `x = 0` und das wird als `DIV BY ZERO`-Fehler angezeigt.

## Hyperbolische Konventionen

**Forward-Hyperboliken** (`sinh`, `cosh`, `tanh`, `coth`): Standard-Definitionen, kein Winkelmodus-Einfluss (Hyperboliken nehmen reelle Argumente, keine Winkel).

**Inverse-Hyperboliken**:
- `arsinh(x) = ln(x + √(x²+1))`, definiert für alle reellen `x`
- `arcosh(x) = ln(x + √(x²−1))`, definiert für `x ≥ 1`
- `artanh(x) = (1/2) · ln((1+x)/(1−x))`, definiert für `|x| < 1`
- `arcoth(x) = (1/2) · ln((x+1)/(x−1))`, definiert für `|x| > 1`

Für Eingaben außerhalb des Definitionsbereichs liefern die f64-Funktionen `NaN`, was als `DOMAIN ERROR` angezeigt wird.

## Konstanten

Alle vier Konstanten kollabieren den Rational-Track sofort:

| Konstante | Wert (f64) |
|---|---|
| `π` | `std::f64::consts::PI` ≈ 3.141592653589793 |
| `e` | `std::f64::consts::E` ≈ 2.718281828459045 |
| `φ` | exakt `1.618033988749895` (hartcodiert) |
| `√2` | `std::f64::consts::SQRT_2` ≈ 1.4142135623730951 |

## Fehler-Klassifikation

Beim Druck auf `=` kann einer der folgenden Zustände entstehen:

| Bedingung | Fehlermeldung |
|---|---|
| f64-Ergebnis ist NaN (Domain-Error: `arcosh(0)`, `artanh(1)`, …) | `DOMAIN ERROR` |
| f64-Ergebnis ist `±∞` (Division durch Null, ggf. `tan(π/2)` mit Roundoff) | `DIV BY ZERO` |
| meval-Parser scheitert (unbalancierte Klammern, dangling Operator, leerer Ausdruck) | `SYNTAX ERROR` |

Im Error-Zustand bleibt das Display leer (rote Meldung), nur `AC` oder ein neuer Eingabe-Token (Digit/Operator/Funktion) clearen den Fehler. Mode-Tasten (`DRG`, `Doz↔Dec`) und Navigation (`Expand`, Cursor) bleiben blockiert bis `AC`.

## Display-States für Ergebnisse

Drei mögliche Zustände, abhängig von Rational-Track und Periode:

**State A — Exakt finit:** Rational-Track erfolgreich, Periode hat Länge 0. Display zeigt alle Ziffern, kein Suffix.
Beispiel: `1/4` → `0.3` (dozenal exakt).

**State B — Gerundet (irrational oder Track-Collapse):** Rational-Track gibt `None`. f64-Ergebnis wird mit `F64_FRAC_DIGITS = 4` Bruchziffern angezeigt, gefolgt von `…` auf der Grundlinie.
Beispiel: `π` → `3.1848…`.

**State C — Periodisch, gekürzt:** Rational-Track erfolgreich, Periode > `MAX_PERIOD_DISPLAY = 5` Stellen. Es werden 5 Periodenziffern unter Überstrich angezeigt, gefolgt von `…` **auf Höhe des Überstrichs** (nicht auf der Grundlinie). Wenn die Periode genau 1–5 Stellen hat, wird sie vollständig unter Überstrich angezeigt — kein Suffix nötig.
Beispiel: `1/7` (in dozenal Periode 6 Stellen `186A35`) → State C; `1/5` (in dozenal Periode 4 Stellen `2497`) → vollständige Periode unter Überstrich, kein Suffix.

## Verifikation

Bestehende Tests in `src/logic.rs` und `src/eval.rs` decken die meisten dieser Regeln ab. Der Port muss diese Tests 1:1 als Tests in der Zielsprache übersetzen — wenn alle 38 Tests grün sind und visuell die Custom-Glyphen stimmen (siehe `GLYPHS.md`), ist die Auswertungs-Schicht korrekt portiert.

Konkrete Hauptregressionen, die bei einem Port immer mitgetestet werden sollten:
- `1/7` → periodisch mit 6-stelliger Periode
- `1/5` → periodisch mit 4-stelliger Periode `2497`
- `5⊕(3+2)` → exakt `5/2`, also dozenal `2.6` ohne Periode
- `3√27` → `3` (n-te Wurzel-Syntax)
- `2^3^2` → `512` (Rechtsassoziativität von `^`)
- `acot(−1)` → `3π/4` im RAD-Modus (Convention A)
