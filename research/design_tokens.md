# Design Tokens — Flutter → technologie-neutral

Extrahiert aus `/home/eric/dozenal_calc_flutter/lib/` als Quelle für den Rust-Web-Port. Die Tokens sind in Flutter konstanten-basiert, aber semantisch unabhängig von Material und lassen sich 1:1 nach CSS Custom Properties, SCSS-Variablen oder Rust-Konstanten exportieren. Pixel-Werte in Flutter sind **dp** (density-independent); für CSS gelten sie als logische Pixel.

## TL;DR — die fünf prägendsten Tokens

1. `--color-bg-app` = `#1F1F1F` (Material `scaffoldBackgroundColor` in `main.dart:55`) — der gesamte dunkle Grundton.
2. `--color-op-normal` = `#98C8FF` (`_kOpNormal`, "egui LIGHT_BLUE") — der hellblaue Akzent, der allen Operator-Tasten ihre Identität gibt.
3. `--color-equals` = `#8CDC8C` (`_kEquals`, "egui LIGHT_GREEN") — die grüne `=`-Bar als visueller Anker am Boden.
4. `--color-digit-pressed` = `#FFD700` ("egui GOLD") — Press-Feedback für Ziffern, gleichzeitig die Farbe des "armed"-Indikators und des `M`-Memory-Indikators.
5. `--radius-button` = `4 dp` (in `_PressableShell`, `keypad.dart:793`) — die zurückhaltend abgerundeten Ecken aller Tasten; `--radius-display` = `8 dp` ist 2× so weich und macht das Display visuell zur "Glasplatte".

Daraus folgt das Gesamtbild: dunkelgrauer Hintergrund, hellblaue Operatoren, weisse Ziffern, grüner Equals-Balken, goldfarbenes Pressed-Highlight — alles unaufgeregt zurückhaltend, mit weichen 4 dp-Tastenecken.

## Farb-Palette

| Token-Name | Hex | Verwendung | Flutter-Konstante / Stelle |
|---|---|---|---|
| `--color-bg-app` | `#1F1F1F` | Globaler Scaffold-Hintergrund, Intro-Hintergrund | `main.dart:55`, `intro_pages.dart:224,226` |
| `--color-bg-display` | `#101010` | Hintergrund des Two-Line-Displays (Kasten) | `display.dart:74` |
| `--color-bg-appbar` | `#1A1A1A` | AppBar in Info/License/Privacy/Feedback | `info_pages.dart:21`, `markdown_page.dart:44`, `feedback_dialog.dart:113` |
| `--color-bg-input` | `#2A2A2A` | Textfeld-Hintergrund (Feedback) und Inline-`code` | `feedback_dialog.dart:98`, `markdown_page.dart:97` |
| `--color-bg-roundbtn` | `#2A2A2A` | Hintergrund der runden `(i)` / `(?)` Knöpfe neben `=` | `keypad.dart:1100` |
| `--color-border-display` | `#333333` | Rahmen des Displays, Divider (Hoch-Mode), Divider zwischen Sets 1-5 / 6-10 im Breit-Mode | `display.dart:76`, `keypad.dart:235,716` |
| `--color-border-button` | `#505050` | Normal-Border der Tasten-`PressableShell` | `_kBorder`, `keypad.dart:30` |
| `--color-border-button-disabled` | `#303030` | Border einer disabled Taste (DEZ-Modus für 10/11) | `keypad.dart:797` |
| `--color-border-roundbtn` | `#555555` | Border der runden `(i)`/`(?)` Knöpfe | `keypad.dart:1102` |
| `--color-border-input` | `#404040` | Border der Feedback-Textfelder | `feedback_dialog.dart:100` |
| `--color-divider-list` | `#2C2C2C` | Trenner zwischen Listenzeilen im Info-Index | `info_pages.dart:33,65,88,111` |
| `--color-digit-normal` | `#FFFFFF` | Ziffern-Glyphe im Normalzustand | `_kDigitNormal`, `keypad.dart:24` |
| `--color-digit-pressed` | `#FFD700` | Ziffern-Glyphe gedrückt, "armed"-Dot, `M`-Memory-Indikator | `_kDigitPressed`, `keypad.dart:25,1008`, `display.dart:178` |
| `--color-digit-disabled` | `#606060` | Ziffer im DEZ-Modus für Werte 10/11 | `_kDigitDisabled`, `keypad.dart:26` |
| `--color-op-normal` | `#98C8FF` | Operator-Symbole und Border einer aktiven Mode-Taste | `_kOpNormal`, `keypad.dart:27,796` |
| `--color-op-pressed` | `#FF9090` | Operator-Symbol im Press-Zustand | `_kOpPressed`, `keypad.dart:28` |
| `--color-equals` | `#8CDC8C` | `=`-Bar Normal-Symbol | `_kEquals`, `keypad.dart:29` |
| `--color-ac` | `#FF4040` | AC-Taste Normal-Symbol | `_kAc`, `keypad.dart:31` |
| `--color-ac-pressed` | `#FF8080` | AC-Taste Press-Zustand | `_kAcPressed`, `keypad.dart:32` |
| `--color-accent-blue` | `#64C8FF` | Icon-Farbe der runden Knöpfe, DOZ/DEZ-Label, Markdown-Links, aktiver Page-Dot | `keypad.dart:1111`, `display.dart:208`, `markdown_page.dart:99`, `intro_pages.dart:393` |
| `--color-highlight-intro` | `#FF3030` | Rote Highlight-Markierungen im Bedienungs-Intro | `_kHighlight`, `intro_pages.dart:9` |
| `--color-cursor` | `redAccent` (`#FF5252`) | Vertikaler Cursor im Display | `display.dart:305` |
| `--color-error` | `redAccent.shade100` (`#FF8A80`) | Fehlertext im Result-Line | `display.dart:157` |
| `--color-text-primary` | `#FFFFFF` | Heading-Text in Info-Detail, Body in dark dialogs | `info_content.dart:46` |
| `--color-text-body` | `#E0E0E0` | Body-Text Info-Detail, Markdown-Paragraphen, Pre-Blöcke | `info_content.dart:65,84` |
| `--color-text-muted` | `#D0D0D0` | Sekundärer Body-Text, Lizenz-/Privacy-Listentitel | `info_pages.dart:77`, `info_content.dart:269` |
| `--color-text-soft` | `#D8D8D8` | Diagonalen-Formeln (Kapitel 5) | `info_content.dart:345` |
| `--color-text-tertiary` | `#C8C8C8` | Beschriftungs-Sekundärtext (Digit-Legende, Markdown `code`) | `info_content.dart:120,205`, `markdown_page.dart:94` |
| `--color-text-dim` | `#B4B4B4` | DEG/RAD/GRAD-Label im Display | `display.dart:193` |
| `--color-text-dim-2` | `#A0A0A0` | Index-Nummern (1./2./…) im Info-Index | `info_pages.dart:40` |
| `--color-text-meta` | `#9E9E9E` | Approximations-Text rechts neben Formeln | `info_content.dart:354` |
| `--color-text-faint` | `#8C8C8C` | Dezente Eck-Punkte im Kapitel-4-Diagramm | `info_content.dart:283` |
| `--color-text-disabled` | `#707070` | `chevron_right`-Icons in den Info-Listen, Version-Footer | `info_pages.dart:53,82,103,127,172` |
| `--color-diagram-stroke` | `#D0D0D0` | Umriss des Zwölfeck-Schaubilds | `info_content.dart:269` |
| `--color-diagram-stroke-soft` | `#6E6E6E` | Subtiler Zwölfeck-Outline in Kapitel 5 | `info_content.dart:383` |
| `--color-diagram-dot` | `#646464` | Eckpunkte im Kapitel-5-Diagramm | `info_content.dart:390` |
| `--color-diagram-teal-fill` | `#509FE1CB`* | Inscribed Triangle (Kapitel 4) Fill | `info_content.dart:166,256` |
| `--color-diagram-teal-stroke` | `#0F6E56` | Inscribed Triangle Stroke / `d₂`-Diagonale | `info_content.dart:167,302` |
| `--color-diagram-blue-fill` | `#5085B7EB`* | Inscribed Square (Kapitel 4) Fill | `info_content.dart:171,250` |
| `--color-diagram-blue-stroke` | `#185FA5` | Inscribed Square Stroke / `d₃`-Diagonale | `info_content.dart:172,303` |
| `--color-diagram-purple-fill` | `#50AFA9EC`* | Inscribed Hexagon (Kapitel 4) Fill | `info_content.dart:176,244` |
| `--color-diagram-purple-stroke` | `#534AB7` | Inscribed Hexagon Stroke / `d₄`-Diagonale | `info_content.dart:177,304` |
| `--color-diagram-gray-stroke` | `#5F5E5A` | `s = 1` (Kanten-Diagonale) Kapitel 5 | `info_content.dart:301` |
| `--color-diagram-orange-stroke` | `#993C1D` | `d₅`-Diagonale | `info_content.dart:305` |
| `--color-diagram-red-stroke` | `#A32D2D` | `d₆`-Diagonale | `info_content.dart:306` |

*Bei den Fill-Farben sind die ersten zwei Hex-Stellen Alpha (0x50 ≈ 31 %, 0x20 ≈ 12 %). Im CSS-Skizzenblock sind die `rgba(...)`-Varianten angegeben.

## Geometrie und Abstände

| Token-Name | Wert | Verwendung | Flutter-Konstante / Stelle |
|---|---|---|---|
| `--touch-target` | `44 dp` | Minimum-Höhe jeder Taste (Material 48 / iOS 44, niedrigerer Wert gewählt) | `minTouchTarget`, `app_layout.dart:39` |
| `--display-min-h` | `60 dp` | Untergrenze Display-Höhe (sonst Scroll-Fallback) | `displayMinHeight`, `app_layout.dart:22` |
| `--display-max-h` | `170 dp` | Obergrenze Display-Höhe auf grossen Geräten | `displayMaxHeightCap`, `app_layout.dart:18` |
| `--display-ratio` | `0.20` | Display = 20 % der App-Body-Höhe | `displayHeightFor`, `app_layout.dart:33` |
| `--display-line-gap-ratio` | `0.06`, geclampt `[2, 10]` dp | Adaptive Lücke zwischen Input- und Result-Zeile | `display.dart:139` |
| `--display-padding-x` | `12 dp` | Innenabstand Display horizontal | `display.dart:78` |
| `--display-padding-y` | `6 dp` | Innenabstand Display vertikal | `display.dart:78` |
| `--radius-button` | `4 dp` | Border-Radius der Tasten | `keypad.dart:793` |
| `--radius-display` | `8 dp` | Border-Radius des Display-Containers | `display.dart:75` |
| `--radius-legend-swatch` | `2 dp` | Legenden-Quadrate in Kapitel 4 | `info_content.dart:196` |
| `--gap-row-hoch-loose` | `10 dp` | Vertikale Lücke zwischen Ziffern-Reihen (Hoch, normal) | `keypad.dart:220` |
| `--gap-row-hoch-tight` | `6 dp` | Dito, tight-regime (<560 dp Keypad-Höhe) | `keypad.dart:220` |
| `--gap-section-hoch-loose` | `14 dp` | Vertikale Lücke zwischen Ziffernblock-Sektion und Op-Section | `keypad.dart:221` |
| `--gap-section-hoch-tight` | `8 dp` | Dito, tight-regime | `keypad.dart:221` |
| `--gap-equals-hoch-loose` | `12 dp` | Lücke vor Equals-Reihe | `keypad.dart:222` |
| `--gap-equals-hoch-tight` | `8 dp` | Dito, tight-regime | `keypad.dart:222` |
| `--gap-oprow-hoch-loose` | `8 dp` | Lücke zwischen Op-Reihen | `keypad.dart:451` |
| `--gap-oprow-hoch-tight` | `6 dp` | Dito, tight-regime | `keypad.dart:451` |
| `--gap-opbottom-hoch-loose` | `12 dp` | Lücke vor System-/Mode-Reihe (5. bzw. 10.) | `keypad.dart:452` |
| `--gap-opbottom-hoch-tight` | `8 dp` | Dito, tight-regime | `keypad.dart:452` |
| `--gap-cell-h` | `8 dp` | Horizontale Lücke zwischen Tasten innerhalb einer Reihe (Hoch) | `keypad.dart:297,477` |
| `--gap-tablet-digit` | `8 dp` | Inner-Block-Gap im Breit-Layout (Ziffern-Grid h+v, Op-Spalten v) | `tabletDigitGap`, `app_layout.dart:57` |
| `--gap-tablet-col` | `8 dp` | Alias zu Digit-Gap, an Op-Column-Aufrufstellen | `tabletColGap`, `app_layout.dart:61` |
| `--gap-tablet-set` | `18 dp` | Referenz-Gap zwischen Sets im Breit-Layout (nicht direkt verwendet, dient als Proportion) | `tabletSetGap`, `app_layout.dart:52` |
| `--gap-group-base` | `18 dp` | Basis-Gap zwischen Gruppen (Ziffern-Pad / Sets 1-5 / Sets 6-10) im Breit | `groupGapBase`, `keypad.dart:541` |
| `--gap-group-max` | `100 dp` | Cap für Gruppen-Gap, damit Tablet nicht "auseinanderdriftet" | `maxGroupGap`, `keypad.dart:542` |
| `--gap-vertical-content` | `18 dp` | Lücke zwischen Hauptinhalt und Equals-Reihe im Breit-Modus | `verticalContentGap`, `keypad.dart:535` |
| `--ref-button-size` | `70 dp` | Referenz-Tastenkante (Tablet/Desktop), Clamp-Obergrenze | `tabletButtonSize`, `app_layout.dart:49` |
| `--equals-flex-ratio` | `10:8 = 1.25` | `=`-Reihe ist 25 % höher als eine normale Op-Reihe | `keypad.dart:252,267,277` |
| `--equals-fixed-multiplier` | `1.2` | Fixed-Heights Fallback: `=`-Reihe = `44 dp × 1.2` | `keypad.dart:267` |
| `--scaffold-padding` | `12 dp` | Aussen-Padding um den gesamten Calc-Inhalt | `main.dart:242` |
| `--display-keypad-gap` | `14 dp` | Vertikale Lücke zwischen Display und Keypad | `main.dart:274` |
| `--armed-dot-size` | `6 dp` | Durchmesser des goldenen "armed"-Punkts | `keypad.dart:1005,1006` |
| `--armed-dot-inset` | `4 dp` | Offset des Armed-Dots von rechts/oben | `keypad.dart:910,911` |
| `--cursor-width` | `1.5 dp` | Cursor-Strichbreite | `display.dart:304` |
| `--cursor-inset-y` | `6 dp` | Vertikaler Cursor-Inset (oben & unten je) | `display.dart:304` |
| `--info-icon-size` | `16 dp` (Legal-Liste) / `18 dp` (chevron) | Icons in Info-Listen | `info_pages.dart:53,72,93,114` |
| `--info-page-padding-x` | `16 dp` | Horizontales Padding der Info-Detailseiten | `info_pages.dart:201`, `markdown_page.dart:67` |
| `--info-page-padding-top` | `8 dp` | Top-Padding der Info-Detailseiten | `info_pages.dart:201` |
| `--info-page-padding-bottom` | `24 dp` | Bottom-Padding der Info-Detailseiten | `info_pages.dart:201` |
| `--heading-padding-top` | `14 dp` | Vertikaler Innenabstand `_H`-Heading oben | `info_content.dart:40` |
| `--heading-padding-bottom` | `4 dp` | Dito unten | `info_content.dart:40` |
| `--paragraph-padding-bottom` | `6 dp` | Innenabstand `_P`-Paragraph unten | `info_content.dart:58` |
| `--pre-padding-y` | `6 dp` | Vertikales Padding `_Pre`-Block | `info_content.dart:77` |
| `--legend-row-padding` | `4 dp` (`bottom`) | Padding zwischen Legenden-Zeilen | `info_content.dart:187,329` |
| `--swatch-w` | `14 dp` (Quadrat) / `24 dp` (Linie) | Kapitel-4-Quadrat / Kapitel-5-Linie | `info_content.dart:191,333` |
| `--swatch-h` | `14 dp` / `3 dp` | Dito | `info_content.dart:192,334` |
| `--stroke-glyph-keypad` | `2.5 dp` | Glyphen-Strichbreite auf Tasten | `keypad.dart:862` |
| `--stroke-glyph-display` | `1.6 dp` | Glyphen-Strichbreite im Display | `display.dart:362` |
| `--stroke-glyph-legend` | `1.4 dp` | Glyphen-Strichbreite in der Info-Legende | `info_content.dart:110` |
| `--stroke-token` | `2.0 dp` | Standard-Strichbreite Token-Painter (`+`, `−`, `√x`, …) | `keypad.dart:990` |
| `--stroke-oplus-inner` | `1.0 dp` | Kleines `+` im ⊕-Quadrat | `token_painter.dart:71` |
| `--stroke-period-bar` | `1.2 dp` | Periodenstrich über Wiederholungs-Ziffern | `display.dart:262` |
| `--stroke-diagonal` | `2.5 dp` | Diagonale-Linien Kapitel 5 | `info_content.dart:400` |
| `--stroke-polygon-fill` | `1.5 dp` | Inscribed Polygons Kapitel 4 | `info_content.dart:237` |
| `--stroke-outline-thick` | `2.0 dp` | Zwölfeck-Umriss Kapitel 4 | `info_content.dart:271` |
| `--stroke-outline-thin` | `1.0 dp` | Zwölfeck-Umriss Kapitel 5 | `info_content.dart:385` |
| `--stroke-highlight` | `3.0 dp` | Rote Intro-Highlight-Boxen/Kreise | `intro_pages.dart:318` |
| `--digit-q-ratio` | `0.18` | Halb-Quart-Faktor der Ziffer im Display (q = lineH × 0.18) | `_digitQRatio`, `display.dart:339` |
| `--digit-cell-padding` | `6 dp` | Zellbreite im Display = `2q + 6` | `display.dart:355` |
| `--overline-gap` | `4 dp` | Lücke zwischen Glyphen-Oberkante und Periodenstrich | `_overlineGap`, `display.dart:343` |
| `--font-size-glyph-keypad-factor` | `0.45` (von minEdge) | Token-Painter `x`-Text in `x^`/`√x`/`log`/`⊕` | `token_painter.dart:126` |
| `--font-size-glyph-corner-factor` | `0.18` (von minEdge) | Eck-Quadrat-Grösse für `x^`/`√x`/`log`/`⊕` | `token_painter.dart:113` |
| `--font-size-token-text-factor` | `0.35` (von minEdge) | Text-Fallback-Tasten (`sin`, `cos`, `STO`, …) | `token_painter.dart:100` |
| `--font-size-display-text-factor` | `0.42` (von lineH) | Operator- und Ans-Text im Display | `display.dart:370,386` |

## Typographie

| Token-Name | Wert | Verwendung |
|---|---|---|
| `--font-family-mono` | `'monospace'` | Display-Text, Indikatoren (`M`, `DEG`, `DOZ`), Info-Pre-Blöcke, Markdown-`code`, Index-Nummern |
| `--font-family-sans` | (Default Material) | Alle anderen Texte |
| `--font-weight-light` | `300` | Body-Paragraphen `_P`, Legende-Labels |
| `--font-weight-normal` | `400` | List-Tile-Titles, Subtext |
| `--font-weight-bold` | `700` | Headings (`_H`), `M`-Indikator, DOZ/DEZ-Label, Highlight-Labels im Intro, Markdown-`strong`/`h1`/`h2` |
| `--font-size-heading-info` | `18 sp` | `_H`-Heading im Info-Detail |
| `--font-size-heading-md-h1` | `22 sp` | Markdown `h1` |
| `--font-size-heading-md-h2` | `16 sp` | Markdown `h2`, Headlines im Feedback-Dialog |
| `--font-size-body` | `16 sp` | `_P`-Paragraph |
| `--font-size-body-md` | `13.5 sp` | Markdown `p`, `li` |
| `--font-size-intro` | `15 sp` | Intro-Erklärtext |
| `--font-size-pre` | `14 sp` | `_Pre`-Monospace-Block |
| `--font-size-input` | `13 sp` | Feedback-TextField-Body |
| `--font-size-list-title` | `14 sp` | Info-/Legal-List-Tile-Titles |
| `--font-size-legend-mono` | `13 sp` | Digit-Legende, Diagonalen-Formeln |
| `--font-size-meta` | `12.5 sp` | Approximations-Text, Markdown-`code`, Feedback-Body |
| `--font-size-indicator-mono` | `12 sp` | `M`-Indikator |
| `--font-size-version` | `11 sp` | Version-Footer |
| `--font-size-mode-label` | `10 sp` | DEG/DOZ-Indikatoren im Display |
| `--line-height-heading-md` | `1.25` (h1), `1.35` (h2) | Markdown |
| `--line-height-body` | `1.45` | `_P`-Paragraph |
| `--line-height-md` | `1.5` | Markdown-Paragraph, `_Pre` |
| `--line-height-intro` | `1.4` | Intro-Slide-Text |

## Animationen

| Token-Name | Dauer / Kurve | Verwendung |
|---|---|---|
| `--anim-overlay-swap` | `150 ms`, In `easeOut`, Out `easeIn` | AnimatedSwitcher zwischen Main-Ops und Overlay-Sets (`_MiddleSection`) |
| `--anim-page` | `250 ms`, `easeOut` | PageView-Übergang im Bedienungs-Intro |
| `--haptic-tap` | `HapticFeedback.selectionClick` | Bei jedem Taste-Tap im `_PressableShell` |
| `--press-feedback` | Synchron (kein Tween) | Setzt `_pressed` über `onTapDown/Up/Cancel` und löst `setState` aus — der Repaint wird vom GestureDetector getrieben, nicht von einer Curve |
| `--repaint-trigger-display` | `shouldRepaint` per Feld-Diff | `_TwoLineDisplayPainter` repaintet nur bei Buffer-/Cursor-/Period-Änderungen (`display.dart:309–323`) |

Keine Spring-Physik, keine `AnimationController`-getriebenen Tweens. Die Anwendung lebt fast vollständig von snappy synchronen Repaints; die einzigen echten Animationen sind der Overlay-Crossfade und der Intro-Page-Slide.

## Breakpoints

| Token-Name | Schwelle | Verhalten |
|---|---|---|
| `--bp-portrait` | `maxHeight > maxWidth` | Hoch-Layout (vertikaler Stack + AnimatedSwitcher); sonst Breit-Layout (alle 10 Sets nebeneinander, kein Overlay) | `isPortraitConstraints`, `app_layout.dart:72` |
| `--bp-keypad-tight` | Keypad-Höhe `< 560 dp` | Tight-Gap-Regime: Reihen-Gaps schrumpfen 10 → 6, Section-Gap 14 → 8, Equals-Gap 12 → 8 | `_kTightThreshold`, `keypad.dart:46` |
| `--bp-keypad-scroll` | Keypad-Höhe `< 480 dp` | Fallback: `SingleChildScrollView` mit fixen Höhen, damit keine Reihe unter den 44-dp-Floor rutscht | `_kScrollThreshold`, `keypad.dart:51` |
| `--bp-display-clamp` | `[60, 170] dp` | Display-Höhe = 20 % der Body-Höhe, dann geclampt | `displayHeightFor`, `app_layout.dart:33` |

Die Rust-Web-Version verwendet derzeit eine andere Breakpoint-Achse (`MOBILE_BREAKPOINT_PX = 500.0`, viewportbreitenbasiert, `layout.rs:11`). Beim Port sollte die Flutter-Logik übernommen werden: Portrait-Switch auf Constraints-Verhältnis statt feste Breite.

## CSS-Skizze (`:root`-Block)

```css
:root {
  /* === Farben — Hintergrund === */
  --color-bg-app:           #1f1f1f;
  --color-bg-display:       #101010;
  --color-bg-appbar:        #1a1a1a;
  --color-bg-input:         #2a2a2a;
  --color-bg-roundbtn:      #2a2a2a;

  /* === Farben — Border und Divider === */
  --color-border-display:          #333333;
  --color-border-button:           #505050;
  --color-border-button-disabled:  #303030;
  --color-border-roundbtn:         #555555;
  --color-border-input:            #404040;
  --color-divider-list:            #2c2c2c;

  /* === Farben — Symbole === */
  --color-digit-normal:    #ffffff;
  --color-digit-pressed:   #ffd700;
  --color-digit-disabled:  #606060;
  --color-op-normal:       #98c8ff;
  --color-op-pressed:      #ff9090;
  --color-equals:          #8cdc8c;
  --color-ac:              #ff4040;
  --color-ac-pressed:      #ff8080;
  --color-accent-blue:     #64c8ff;
  --color-highlight-intro: #ff3030;
  --color-cursor:          #ff5252;
  --color-error:           #ff8a80;

  /* === Farben — Text-Hierarchie === */
  --color-text-primary:   #ffffff;
  --color-text-body:      #e0e0e0;
  --color-text-muted:     #d0d0d0;
  --color-text-soft:      #d8d8d8;
  --color-text-tertiary:  #c8c8c8;
  --color-text-dim:       #b4b4b4;
  --color-text-dim-2:     #a0a0a0;
  --color-text-meta:      #9e9e9e;
  --color-text-faint:     #8c8c8c;
  --color-text-disabled:  #707070;

  /* === Farben — Diagramme === */
  --color-diagram-stroke:        #d0d0d0;
  --color-diagram-stroke-soft:   #6e6e6e;
  --color-diagram-dot:           #646464;
  --color-diagram-teal-fill:     rgba(159, 225, 203, 0.31);
  --color-diagram-teal-stroke:   #0f6e56;
  --color-diagram-blue-fill:     rgba(133, 183, 235, 0.31);
  --color-diagram-blue-stroke:   #185fa5;
  --color-diagram-purple-fill:   rgba(175, 169, 236, 0.31);
  --color-diagram-purple-stroke: #534ab7;
  --color-diagram-gray-stroke:   #5f5e5a;
  --color-diagram-orange-stroke: #993c1d;
  --color-diagram-red-stroke:    #a32d2d;

  /* === Geometrie — Tasten und Touch === */
  --touch-target:        44px;
  --radius-button:       4px;
  --radius-display:      8px;
  --ref-button-size:     70px;
  --armed-dot-size:      6px;
  --armed-dot-inset:     4px;

  /* === Geometrie — Display === */
  --display-min-h:       60px;
  --display-max-h:       170px;
  --display-ratio:       0.20;
  --display-padding-x:   12px;
  --display-padding-y:   6px;
  --cursor-width:        1.5px;
  --cursor-inset-y:      6px;

  /* === Geometrie — Gaps Hoch-Mode (loose / tight regime) === */
  --gap-row-loose:        10px;
  --gap-row-tight:        6px;
  --gap-section-loose:    14px;
  --gap-section-tight:    8px;
  --gap-equals-loose:     12px;
  --gap-equals-tight:     8px;
  --gap-oprow-loose:      8px;
  --gap-oprow-tight:      6px;
  --gap-opbottom-loose:   12px;
  --gap-opbottom-tight:   8px;
  --gap-cell-h:           8px;

  /* === Geometrie — Gaps Breit-Mode === */
  --gap-tablet-inner:     8px;
  --gap-tablet-set:       18px;
  --gap-group-base:       18px;
  --gap-group-max:        100px;
  --gap-vertical-content: 18px;

  /* === Geometrie — Scaffold === */
  --scaffold-padding:    12px;
  --display-keypad-gap:  14px;

  /* === Stroke-Widths === */
  --stroke-glyph-keypad:   2.5px;
  --stroke-glyph-display:  1.6px;
  --stroke-glyph-legend:   1.4px;
  --stroke-token:          2.0px;
  --stroke-oplus-inner:    1.0px;
  --stroke-period-bar:     1.2px;
  --stroke-diagonal:       2.5px;
  --stroke-polygon-fill:   1.5px;
  --stroke-outline-thick:  2.0px;
  --stroke-outline-thin:   1.0px;
  --stroke-highlight:      3.0px;

  /* === Typographie === */
  --font-family-mono:     monospace;
  --font-family-sans:     system-ui, "Helvetica Neue", Arial, sans-serif;
  --font-weight-light:    300;
  --font-weight-normal:   400;
  --font-weight-bold:     700;

  --font-size-heading-info:  18px;
  --font-size-heading-h1:    22px;
  --font-size-heading-h2:    16px;
  --font-size-body:          16px;
  --font-size-body-md:       13.5px;
  --font-size-intro:         15px;
  --font-size-pre:           14px;
  --font-size-input:         13px;
  --font-size-list-title:    14px;
  --font-size-legend-mono:   13px;
  --font-size-meta:          12.5px;
  --font-size-indicator-mono: 12px;
  --font-size-version:       11px;
  --font-size-mode-label:    10px;

  --line-height-heading-h1:  1.25;
  --line-height-heading-h2:  1.35;
  --line-height-body:        1.45;
  --line-height-md:          1.5;
  --line-height-intro:       1.4;

  /* === Animation === */
  --anim-overlay-swap-ms:    150ms;
  --anim-overlay-swap-curve-in:  cubic-bezier(0, 0, 0.58, 1);    /* easeOut */
  --anim-overlay-swap-curve-out: cubic-bezier(0.42, 0, 1, 1);    /* easeIn */
  --anim-page-ms:            250ms;
  --anim-page-curve:         cubic-bezier(0, 0, 0.58, 1);        /* easeOut */

  /* === Breakpoints (verwendet via @container / @media) === */
  --bp-keypad-tight:    560px;
  --bp-keypad-scroll:   480px;
}
```

## Rust-Skizze (`dozenal_core::tokens`)

```rust
// dozenal_core/src/tokens.rs
//
// Technologie-neutrale Design-Tokens. Konsumiert von Rust-Web-Frontends
// (egui, Leptos, Dioxus, …) und ggf. exportiert nach CSS/JSON für andere
// Frontends.

pub mod color {
    /// 24-Bit-Hex-Wert, kompatibel mit den meisten Crates über `from_u32`.
    pub type Hex = u32;

    // Hintergrund
    pub const BG_APP:                Hex = 0x1F1F1F;
    pub const BG_DISPLAY:            Hex = 0x101010;
    pub const BG_APPBAR:             Hex = 0x1A1A1A;
    pub const BG_INPUT:              Hex = 0x2A2A2A;
    pub const BG_ROUNDBTN:           Hex = 0x2A2A2A;

    // Border + Divider
    pub const BORDER_DISPLAY:        Hex = 0x333333;
    pub const BORDER_BUTTON:         Hex = 0x505050;
    pub const BORDER_BUTTON_DISABLED:Hex = 0x303030;
    pub const BORDER_ROUNDBTN:       Hex = 0x555555;
    pub const BORDER_INPUT:          Hex = 0x404040;
    pub const DIVIDER_LIST:          Hex = 0x2C2C2C;

    // Symbole / Akzente
    pub const DIGIT_NORMAL:          Hex = 0xFFFFFF;
    pub const DIGIT_PRESSED:         Hex = 0xFFD700;
    pub const DIGIT_DISABLED:        Hex = 0x606060;
    pub const OP_NORMAL:             Hex = 0x98C8FF;
    pub const OP_PRESSED:            Hex = 0xFF9090;
    pub const EQUALS:                Hex = 0x8CDC8C;
    pub const AC:                    Hex = 0xFF4040;
    pub const AC_PRESSED:            Hex = 0xFF8080;
    pub const ACCENT_BLUE:           Hex = 0x64C8FF;
    pub const HIGHLIGHT_INTRO:       Hex = 0xFF3030;
    pub const CURSOR:                Hex = 0xFF5252;
    pub const ERROR:                 Hex = 0xFF8A80;

    // Text-Hierarchie
    pub const TEXT_PRIMARY:          Hex = 0xFFFFFF;
    pub const TEXT_BODY:             Hex = 0xE0E0E0;
    pub const TEXT_MUTED:            Hex = 0xD0D0D0;
    pub const TEXT_SOFT:             Hex = 0xD8D8D8;
    pub const TEXT_TERTIARY:         Hex = 0xC8C8C8;
    pub const TEXT_DIM:              Hex = 0xB4B4B4;
    pub const TEXT_DIM_2:            Hex = 0xA0A0A0;
    pub const TEXT_META:             Hex = 0x9E9E9E;
    pub const TEXT_FAINT:            Hex = 0x8C8C8C;
    pub const TEXT_DISABLED:         Hex = 0x707070;

    // Diagramme (Fill-Farben mit Alpha als (rgb, alpha) Paar)
    pub const DIAGRAM_STROKE:        Hex = 0xD0D0D0;
    pub const DIAGRAM_STROKE_SOFT:   Hex = 0x6E6E6E;
    pub const DIAGRAM_DOT:           Hex = 0x646464;
    pub const DIAGRAM_TEAL_STROKE:   Hex = 0x0F6E56;
    pub const DIAGRAM_TEAL_FILL:     (Hex, u8) = (0x9FE1CB, 0x50);
    pub const DIAGRAM_BLUE_STROKE:   Hex = 0x185FA5;
    pub const DIAGRAM_BLUE_FILL:     (Hex, u8) = (0x85B7EB, 0x50);
    pub const DIAGRAM_PURPLE_STROKE: Hex = 0x534AB7;
    pub const DIAGRAM_PURPLE_FILL:   (Hex, u8) = (0xAFA9EC, 0x50);
    pub const DIAGRAM_GRAY_STROKE:   Hex = 0x5F5E5A;
    pub const DIAGRAM_ORANGE_STROKE: Hex = 0x993C1D;
    pub const DIAGRAM_RED_STROKE:    Hex = 0xA32D2D;
}

pub mod geom {
    /// Pixelwerte als logical-pixel f32 (auf Web: CSS-px; auf Desktop: egui-pt).
    pub const TOUCH_TARGET:        f32 = 44.0;
    pub const RADIUS_BUTTON:       f32 = 4.0;
    pub const RADIUS_DISPLAY:      f32 = 8.0;
    pub const REF_BUTTON_SIZE:     f32 = 70.0;
    pub const ARMED_DOT_SIZE:      f32 = 6.0;
    pub const ARMED_DOT_INSET:     f32 = 4.0;

    pub const DISPLAY_MIN_H:       f32 = 60.0;
    pub const DISPLAY_MAX_H:       f32 = 170.0;
    pub const DISPLAY_RATIO:       f32 = 0.20;
    pub const DISPLAY_PADDING_X:   f32 = 12.0;
    pub const DISPLAY_PADDING_Y:   f32 = 6.0;
    pub const CURSOR_WIDTH:        f32 = 1.5;
    pub const CURSOR_INSET_Y:      f32 = 6.0;

    pub const GAP_ROW_LOOSE:       f32 = 10.0;
    pub const GAP_ROW_TIGHT:       f32 = 6.0;
    pub const GAP_SECTION_LOOSE:   f32 = 14.0;
    pub const GAP_SECTION_TIGHT:   f32 = 8.0;
    pub const GAP_EQUALS_LOOSE:    f32 = 12.0;
    pub const GAP_EQUALS_TIGHT:    f32 = 8.0;
    pub const GAP_OPROW_LOOSE:     f32 = 8.0;
    pub const GAP_OPROW_TIGHT:     f32 = 6.0;
    pub const GAP_OPBOTTOM_LOOSE:  f32 = 12.0;
    pub const GAP_OPBOTTOM_TIGHT:  f32 = 8.0;
    pub const GAP_CELL_H:          f32 = 8.0;

    pub const GAP_TABLET_INNER:    f32 = 8.0;
    pub const GAP_TABLET_SET:      f32 = 18.0;
    pub const GAP_GROUP_BASE:      f32 = 18.0;
    pub const GAP_GROUP_MAX:       f32 = 100.0;
    pub const GAP_VERTICAL_CONTENT:f32 = 18.0;

    pub const SCAFFOLD_PADDING:    f32 = 12.0;
    pub const DISPLAY_KEYPAD_GAP:  f32 = 14.0;

    pub const STROKE_GLYPH_KEYPAD:  f32 = 2.5;
    pub const STROKE_GLYPH_DISPLAY: f32 = 1.6;
    pub const STROKE_GLYPH_LEGEND:  f32 = 1.4;
    pub const STROKE_TOKEN:         f32 = 2.0;
    pub const STROKE_OPLUS_INNER:   f32 = 1.0;
    pub const STROKE_PERIOD_BAR:    f32 = 1.2;
    pub const STROKE_DIAGONAL:      f32 = 2.5;
    pub const STROKE_POLYGON_FILL:  f32 = 1.5;
    pub const STROKE_OUTLINE_THICK: f32 = 2.0;
    pub const STROKE_OUTLINE_THIN:  f32 = 1.0;
    pub const STROKE_HIGHLIGHT:     f32 = 3.0;
}

pub mod typo {
    pub const FAMILY_MONO:  &str = "monospace";

    pub const WEIGHT_LIGHT:   u16 = 300;
    pub const WEIGHT_NORMAL:  u16 = 400;
    pub const WEIGHT_BOLD:    u16 = 700;

    pub const SIZE_HEADING_INFO:     f32 = 18.0;
    pub const SIZE_HEADING_H1:       f32 = 22.0;
    pub const SIZE_HEADING_H2:       f32 = 16.0;
    pub const SIZE_BODY:             f32 = 16.0;
    pub const SIZE_BODY_MD:          f32 = 13.5;
    pub const SIZE_INTRO:            f32 = 15.0;
    pub const SIZE_PRE:              f32 = 14.0;
    pub const SIZE_INPUT:            f32 = 13.0;
    pub const SIZE_LIST_TITLE:       f32 = 14.0;
    pub const SIZE_LEGEND_MONO:      f32 = 13.0;
    pub const SIZE_META:             f32 = 12.5;
    pub const SIZE_INDICATOR_MONO:   f32 = 12.0;
    pub const SIZE_VERSION:          f32 = 11.0;
    pub const SIZE_MODE_LABEL:       f32 = 10.0;

    pub const LH_HEADING_H1: f32 = 1.25;
    pub const LH_HEADING_H2: f32 = 1.35;
    pub const LH_BODY:       f32 = 1.45;
    pub const LH_MD:         f32 = 1.5;
    pub const LH_INTRO:      f32 = 1.4;
}

pub mod anim {
    use std::time::Duration;

    pub const OVERLAY_SWAP:  Duration = Duration::from_millis(150);
    pub const PAGE_SLIDE:    Duration = Duration::from_millis(250);
    // Curve-Konstanten in Bezier-Form: ease-in / ease-out / ease-in-out
    pub const EASE_OUT:      (f32, f32, f32, f32) = (0.0,  0.0, 0.58, 1.0);
    pub const EASE_IN:       (f32, f32, f32, f32) = (0.42, 0.0, 1.0,  1.0);
}

pub mod bp {
    /// Keypad-Höhe in logical-px, unterhalb derer das Tight-Regime gilt.
    pub const KEYPAD_TIGHT:  f32 = 560.0;
    /// Keypad-Höhe in logical-px, unterhalb derer auf einen Scroll-Fallback
    /// umgeschaltet wird.
    pub const KEYPAD_SCROLL: f32 = 480.0;
}
```

## Übersetzungs-Hinweise

- **dp → CSS-px**: Flutter rechnet in density-independent Pixels. Auf Web mit `devicePixelRatio` ≥ 1 bedeutet Flutter-`44 dp` → CSS-`44 px`. Wer einen Touch-First-Web-Port macht, sollte zusätzlich `min(44px, 2.75rem)` verwenden, damit eine erhöhte Browser-Schriftgröße die Tastengröße nicht unterschreitet.
- **sp → CSS-px**: Schriftgrößen sind in `sp` (skalierbar). Für CSS empfehle ich die Umrechnung in `rem` (`16 sp → 1 rem`). Im obigen Block stehen `px`, weil das im Skizzen-Modus klarer ist; produktiv lieber `rem` für Accessibility.
- **Material `redAccent`**: Konkrete Werte sind `#FF5252` (`redAccent`) und `#FF8A80` (`redAccent.shade100`). Beim Export aus Flutter werden sie als benannte Konstanten verwendet — im CSS/Rust müssen sie explizit ausgeschrieben sein.
- **`HapticFeedback.selectionClick`**: Im Web nicht 1:1 verfügbar. Substitut: kurzes `navigator.vibrate(10)` (wo erlaubt) oder ganz weglassen. CSS:hover/active reicht visuell für Desktop.
- **`Curves.easeOut`/`easeIn`**: Im Skizzenblock sind die korrekten Bezier-Parameter eingetragen. `Curves.easeInOut` = `(0.42, 0, 0.58, 1)` falls später nötig.
- **`AnimatedSwitcher`**: Ein Crossfade zwischen zwei Layouts (Main-Panel ↔ Overlay-Panel) lässt sich in CSS mit `opacity`-Transition über zwei absolut positionierte Container nachbauen; alternativ Framework-spezifisch (z. B. `<Show>` in Leptos mit `transition`-Klasse).
- **`SafeArea`**: Auf Web nicht direkt nötig, aber `env(safe-area-inset-bottom)` als Padding-Bottom-Variable einsetzen, falls die App als PWA installiert wird.
- **Flutter-`Color` mit Alpha-Prefix (`0x50...`)**: Die beiden ersten Hex-Stellen sind Alpha (0..0xFF). Das CSS-Skizzenblock-`rgba(...)` rechnet dies bereits aus.
- **Inkonsistenz in der Rust-Codebase**: `MOBILE_BREAKPOINT_PX = 500.0` (Viewportbreite) ist nicht äquivalent zu Flutter's `isPortraitConstraints` (Höhe > Breite). Beim Port die Flutter-Semantik übernehmen — sie funktioniert in Split-Screen, Foldables und resizebaren Browser-Fenstern korrekt; die feste 500-px-Schwelle bricht im Tablet-Portrait.

## Kandidaten — noch nicht extrahiert / ggf. eigenes Token wert

- `0.42` und `0.35` als minEdge/lineH-Faktoren für Text-Größe in `token_painter.dart`/`display.dart` — wären als `--font-scale-token` etc. eigenständige Tokens; aktuell nur Inline-Werte.
- `0.18` als Eck-Quadrat-Faktor in `token_painter.dart` — analog.
- `0.5` als Icon-Skalierung im `_RoundIconButton` (`icon size = size * 0.5`, `keypad.dart:1110`) — Kandidat für `--icon-scale-roundbtn`.
- `3.6` als Dot-Spacing-Multiplikator im State-C-`…` (`display.dart:280`) — magisch, sollte ggf. `--ellipsis-dot-spacing-factor` werden.
- `0.025` als Dot-Radius-Faktor (`r = rect.height * 0.025`, `display.dart:278`) — analog.
- `1.5` Inset (Periodenstrich-Endenkürzung in `display.dart:255-256`) — Kandidat für `--period-bar-inset`.
- `0.085` Decoder-Label-Offset in `intro_pages.dart:88-103` — Kandidat für `--decoder-label-offset`, hängt aber vom Screenshot-Format ab.

## Berichts-Trailer

(a) **~135 Tokens** extrahiert (43 Farben + 56 Geometrie/Stroke + 19 Typo + 5 Animation/Curve + 4 Breakpoints + 7 Diagramm-Kandidaten + 7 explizite Kandidaten am Ende). Genauer: 43 Farb-Konstanten, 56 Geometrie-Konstanten, 19 typografische Konstanten, 5 Animations-/Haptik-Hinweise, 4 Breakpoint-Schwellwerte.

(b) **Datei:** `/home/eric/dozenal_calc/research/design_tokens.md`

(c) **Überraschendster Befund:** Der Flutter-Code kommentiert mehrere Farben explizit mit ihren egui-Erblasten (`// egui GOLD`, `// egui LIGHT_BLUE`, `// egui LIGHT_GREEN`, `// egui LIGHT_RED`). Die Flutter-Werte (`#FFD700`, `#98C8FF`, `#8CDC8C`, `#FF9090`) stimmen jedoch nicht mit den tatsächlichen egui-Konstanten überein: egui's `LIGHT_BLUE` ist `#ADD8E6`, `LIGHT_RED` `#FFB4B4`, `LIGHT_GREEN` `#90EE90`, `GOLD` `#FFD700` (nur dieser stimmt). Die Flutter-Werte sind also bewusst nachgesättigte Eigenversionen — die Kommentare suggerieren aber 1:1-Übernahme. Außerdem: `tabletSetGap` (`18 dp`) in `app_layout.dart` ist als "Referenz-Gap zwischen Sets im Breit-Layout" deklariert, wird aber in `keypad.dart` nicht direkt verwendet; stattdessen baut der Breit-Modus seine eigenen Gaps (`interBlockGap = tabletColGap = 8 dp` plus `groupGapBase = 18 dp`). Die `tabletSetGap`-Konstante ist tot — Kandidat zum Aufräumen.
