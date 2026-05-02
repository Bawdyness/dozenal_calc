# PORTING.md — Leitfaden zum Flutter-Rewrite

Dieses Dokument ist der Einstiegspunkt für eine Sitzung, in der der dozenal_calc von Rust+egui nach Flutter portiert wird. Es bündelt, **was zu tun ist**, **welche Quellen zu lesen sind**, und **wie die Zielarchitektur aussieht**.

## Empfohlene Lese-Reihenfolge

1. **`CLAUDE.md`** — vollständige Spezifikation der App: Layout-Invarianten, Tasten-Sets, Math-Konventionen, Periodic-Decimals, Two-Line-Display, Display-States, Memory-Modell, Interaction-Conventions. Das ist die längste, aber auch die zentralste Quelle.
2. **`GLYPHS.md`** — geometrische Spezifikation der zwölf Custom-Ziffer-Glyphen.
3. **`EXPRESSION_GRAMMAR.md`** — formale Eingabesprache, Operator-Präzedenzen, Custom-Operator-Auflösung, Implicit-Multiplication, Trig-Konventionen, Konstanten.
4. **`INFO_MODAL_CONTENT.md`** — vollständiger Text aller 12 Info-Kapitel auf Deutsch. Wird 1:1 ins Flutter-Asset übernommen.
5. **`DESIGN_NOTES.md`** — offene visuelle/UX-Fragen und Begründungen, die noch nicht zu Invarianten gehärtet sind.

`README.md` ist öffentlich orientiert und für den Port irrelevant. `CODE_QUALITY_SESSION.md` ist historisch und wird nicht portiert.

## Was unverändert übernommen wird

Aus `CLAUDE.md`, ohne Diskussion:
- **Layout-Invarianten** — strikt 4er-Tasten-Sets, vollbreite `=`-Taste, einfaches Overlay (keine Verschachtelung), Close in Position 10.4.
- **Sets 1–10** — Inhalt und Reihenfolge der Tasten in beiden Keypads (siehe Tabelle in `CLAUDE.md` "Layout Architecture").
- **Math-Konventionen** — `acot` Convention A, Hyperbolische Inverse als `arsinh`/`arcosh`/`artanh`/`arcoth`, exact-rational-Track für die Grundoperationen.
- **Display-States A/B/C** — Exact, Rounded mit `…` auf Grundlinie, Periodic-Truncated mit `…` auf Überstrichhöhe.
- **Two-Line-Display-Lifecycle** — obere Zeile aktiv vor `=`, untere Zeile aktiv nach `=`, neue Eingabe schaltet zurück nach oben.
- **Ans-Auto-Insertion** — nach `=` löst ein Operator (`+`, `−`, `*`, `/`, `⊕`, `^`, `√`, `log`) automatisches Voranstellen von `Ans` aus. Liste in `EXPRESSION_GRAMMAR.md`.
- **Doppelklick-Inverse** — Trig (Set 3) und Hyperbolisch (Set 8) toggeln zwischen Forward und Inverse.
- **Memory-Modell** — ein Slot, kein M+, exakter `Rational` falls verfügbar (sonst f64), `M`-Indikator oben links.
- **Error-States** — `DOMAIN ERROR` / `DIV BY ZERO` / `SYNTAX ERROR` als kurze, großgeschriebene Meldungen; nur `AC` oder ein neuer Token clearen.
- **Info-Modal-Struktur** — 12 Kapitel, List → Detail → Back, Text in Deutsch, alphanumerische Notation (A, B) statt Custom-Glyphen im Lesetext.

## Was neu konzipiert wird (Plattform-spezifisch)

- **Tastatur-Eingabe-Mapping** — auf Mobile gibt es keine physische Tastatur. Aktuell unterstützt der Rust-Code Tastatur-Shortcuts (`handle_keyboard` in `input.rs`); auf Mobile entfällt das ersatzlos. Auf einem Tablet mit angeschlossener Tastatur könnte Flutter `RawKeyboardListener` verwendet werden — Detail für später.
- **Touch vs. Click** — Flutter behandelt Touch nativ über `GestureDetector` / `InkWell`. Die Doppelklick-Erkennung für Inverse (Set 3, Set 8) sollte mit `onDoubleTap` umgesetzt werden, nicht über manuelle Zeitmessung.
- **Plattform-spezifische Anpassungen** — Status-Bar-Höhe (iOS Notch / Android Statusbar), Safe-Area-Insets, Gestik-Navigation (iOS Home-Indicator). Flutter `SafeArea`-Widget abdeckend einsetzen.
- **App-Icon und Splash-Screen** — neu zu gestalten. Vorschlag: ein zentriertes D5 (das geometrisch ausgewogenste Composite-Glyph) als Icon.

## Test-Korpus

Die Tests in `src/logic.rs` und `src/eval.rs` definieren das erwartete Verhalten. Sie sind die ausführbare Spezifikation der Auswertungs-Schicht. Beim Port:

1. Jeden Test 1:1 als Dart-Test übersetzen. Die Test-Namen sind sprachneutral lesbar (z.B. `period_one_seventh`, `oplus_with_paren_right_operand`).
2. Wenn alle Tests in der Dart-Implementation grün sind, ist die Auswertungs-Schicht zu hoher Wahrscheinlichkeit korrekt portiert.
3. Bei jedem neu entdeckten Edge-Case während des Ports: zuerst einen Dart-Test schreiben, dann erst den Code anpassen. So bleibt der Korpus monoton wachsend.

Die wichtigsten Test-Module:

- **`logic::tests`** — Rational-Arithmetik (`rational_add`, `rational_div_by_zero`, `rational_pow_negative`, …), Periodendetektion (`period_one_seventh`, `period_one_eleventh`, `period_finite_half`), Rational-Track-Parser (`eval_oplus`, `eval_pow_fraction_collapses`, `eval_unary_minus`).
- **`eval::tests`** — Custom-Operator-Auflösung (`oplus_with_paren_right_operand`, `sqrt_with_paren_arg`, `log_with_paren_base`), `acot` Convention A, n-te Wurzel (`nth_root_with_paren_arg`), End-to-End-Verifikation (`rational_oplus_with_paren`).

## Dependency-Mapping

| Rust-Crate | Flutter-Äquivalent | Hinweis |
|---|---|---|
| `eframe` | Top-level `MaterialApp` oder `CupertinoApp` | Flutter abstrahiert Plattform-Setup von selbst. |
| `egui` Widgets (Button, Label) | `Widget`-Tree (`ElevatedButton`, `Text`, `Container`) | Konvention: jede Taste ist ein `GestureDetector` um einen `CustomPaint` herum, weil die Tasten ein eigenes Glyph haben. |
| `egui::Painter` | `Canvas` via `CustomPainter` | Zeichnen der 12 Ziffer-Glyphen, aller Token-Glyphen (sin, log, etc.), Display-Inhalts (Cursor, Überstrich, `…`-Suffixe). |
| `egui::Stroke` | `Paint()..style = PaintingStyle.stroke..strokeWidth = w` | |
| `egui::Color32::WHITE` etc. | `Colors.white`, `Colors.lightBlue`, `Colors.lightGreen` | Flutters Material-Farben sind ähnlich; ggf. exakt matchen mit `Color.fromARGB(...)`. |
| `meval` (f64-Expression-Parser) | Eigene Implementierung auf Basis von `EXPRESSION_GRAMMAR.md` ODER pub.dev `math_expressions` ODER `petitparser`-basiert | Eigenbau ist machbar (~400 Zeilen Dart) und vermeidet Dependency. |
| `Trunk` (Web-Build) | Flutter Web (out-of-the-box mit `flutter build web`) | Kein zusätzlicher Build-Step. |
| `cargo test` | `flutter test` | Standard-Test-Runner. |

**Eigene `Rational`-Implementierung:** im Rust-Code von Hand geschrieben (`logic.rs:Rational`, ~250 Zeilen). In Dart genauso machen — `BigInt` von Dart unterstützt beliebige Präzision (kein 128-Bit-Limit nötig wie bei Rust `i128`), das macht den Port sogar **einfacher**: alle `checked_*`-Aufrufe entfallen, weil `BigInt` nicht überläuft. Der Collapse-Bedingung "i128-Überlauf" wird damit obsolet — der Rational-Track collabiert in Dart nur noch durch nicht-rationale Tokens und Division durch Null.

## Layout-Konstanten (numerisch)

Diese Werte sind aktuell in `src/layout.rs` verstreut. Beim Port als zentrale `app_layout.dart`-Konstanten ablegen:

| Konstante | Wert | Bedeutung |
|---|---|---|
| `MOBILE_BREAKPOINT_PX` | `500.0` | Schwelle Width < `500.0` → Mobile-Layout. |
| `DISPLAY_LINE_H` | `80.0` | Höhe einer Display-Zeile (oben Input, unten Result). |
| `DISPLAY_GAP` | `10.0` | Vertikaler Abstand zwischen Input- und Result-Zeile. |
| Desktop button size | `Vec2::splat(50.0)` | Quadratische Tasten 50×50 in Desktop-Layout. |
| Desktop set gap | `15.0` | Horizontaler Abstand zwischen Sets in Desktop-Layout. |
| Mobile spacing | `8.0` | Horizontaler/vertikaler Standard-Abstand in Mobile-Layout. |
| Mobile num spacing y | `10.0` | Vertikaler Abstand der Zahlentasten in Mobile-Layout. |
| Mobile button height | (berechnet aus verfügbarer Breite) | num_btn_width = (avail_w − 2 · 8.0) / 3, ops_btn_width = (avail_w − 3 · 8.0) / 4. |
| Mobile equals height | `50.0` | Höhe der vollbreiten `=`-Taste. |
| Default window size | `[400.0, 600.0]` | Initiale Native-Fenstergröße (`main.rs`); in Flutter Mobile irrelevant, in Flutter Desktop als `windowManager.setSize` verwendbar. |

Diese Werte sind **nicht heilig** — sie wurden visuell justiert. Beim Port leichte Abweichungen erlaubt, solange das Layout-Gefühl erhalten bleibt.

## Schritt-für-Schritt-Vorschlag für den Port

Diese Reihenfolge minimiert Frustration durch frühe Sichtbarkeit:

1. **Pilot-Glyphen-Port (bereits gemacht)** — bestätigt Stack und Spec.
2. **Alle 12 Glyphen** in `lib/glyphs.dart` als ein einzelner `CustomPainter` mit `enum DozenalDigit`. Visuell mit Rust-Web-Version vergleichen.
3. **Token-Enum** in `lib/tokens.dart` 1:1 von `src/tokens.rs::CalcToken` portieren.
4. **`Rational` und Periodendetektion** in `lib/logic/rational.dart` portieren — `BigInt` statt `i128` macht das einfacher. Tests aus `logic::tests` als Dart-Tests übersetzen.
5. **Token-Stream-Parser für Rational-Track** in `lib/logic/rat_parser.dart` aus `RatParser` (`logic.rs:321–460`) portieren. Tests aus `logic::tests::eval_*` übersetzen.
6. **f64-Track + Custom-Operator-Auflösung** in `lib/logic/expression.dart`. Eigenbau-Parser anhand `EXPRESSION_GRAMMAR.md`. Tests aus `eval::tests` übersetzen.
7. **Static Display-Mock** — `CustomPaint`, das einen festen `Vec<CalcToken>` als Two-Line-Display rendert. Cursor, Überstrich, `…`-Suffixe testen.
8. **Keypad** — die fünf Sets 1–5 als Widget-Tree, mit Tap-Handlern, die in `handle_click` (entsprechend portiert) feeden.
9. **State-Management** — eine zentrale `DozenalCalcState extends ChangeNotifier`-Klasse, analog zu `DozenalCalcApp` in `tokens.rs`. Pro Tap/Doppeltap eine State-Mutation, dann `notifyListeners()`.
10. **Overlay (Sets 6–10)** — mit `Stack` über dem Hauptlayout, semitransparenter Hintergrund.
11. **Info-Modal** — `Navigator.push` auf eine eigene Route. Inhalte aus `INFO_MODAL_CONTENT.md` (oder aus dem extrahierten `lib/info_content.dart`).
12. **Mobile/Desktop-Layout-Switch** anhand `MediaQuery.of(context).size.width < MOBILE_BREAKPOINT_PX`.
13. **Polishing** — Touch-Feedback, Vibration auf Tasten-Tap (haptic feedback), App-Icon, Splash, AppBar-Title.
14. **Builds**: `flutter build apk` (Android), `flutter build ios` (iOS, macOS-Maschine nötig), `flutter build web`.

Schritte 2–6 sind das Fundament. Wenn Schritte 2 und 4 zusammen grün sind (Glyphen sehen richtig aus, Periodendetektion stimmt), ist 80 % des Risikos eliminiert — der Rest ist Fleißarbeit auf bekanntem Boden.

## Was im aktuellen Repo bleibt

Der Rust-Code bleibt erhalten und live (GitHub Pages-Web-Version). Bei einem stabilen Flutter-Build kann später entschieden werden, ob der Rust-Code archiviert oder weiter gepflegt wird. Eine Doppelpflege beider Codebasen ist nicht das Ziel — die Flutter-Version soll mittelfristig die Rust-Version ablösen.

Backup-Tag `pre-flutter-rewrite` markiert den Zustand vor diesem Übergang.
