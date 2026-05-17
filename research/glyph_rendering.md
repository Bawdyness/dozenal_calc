# Glyph-Rendering-Recherche — DOM-Port des Dozenal-Rechners

Recherche-Stand: Mai 2026. Quelle der Glyph-Geometrie: `GLYPHS.md` und `src/painting.rs::paint_dozenal_digit`.

## TL;DR und Empfehlung

**Empfehlung: SVG-Sprite mit `<symbol>` + `<use>`** als einzige Glyph-Quelle für die gesamte App. Die zwölf Glyphen werden einmal als `<symbol id="d0">`…`<symbol id="d11">` in den DOM-Tree injiziert (oder als externes File für Cache), und jeder Auftritt einer Ziffer ist ein `<svg><use href="#dN"/></svg>` mit `role="img"` und `aria-label`. Damit fallen alle anderen Optionen aus: Bundle ist klein (~2 kB für alle zwölf Glyphen zusammen, einmalig), Render-Performance ist für ~25 Instanzen unkritisch, Accessibility ist sauber lösbar, CSS-Styling (`stroke`, `color`, `currentColor`) funktioniert nativ, und die Geometrie aus `paint_dozenal_digit` lässt sich 1:1 in `<path>`-Definitionen übersetzen.

## Vergleichstabelle

| Kriterium | Inline-SVG pro Auftritt | **SVG-Sprite `<symbol>`+`<use>`** | WOFF2 PUA-Font | Per-Glyph `<canvas>` | Single Canvas Sprite-Sheet | Hybrid (SVG+Canvas) |
|---|---|---|---|---|---|---|
| Bundle (kB) | ~25 × 200 B = 5 kB im HTML | **~2 kB einmalig + 25× 30 B `<use>` ≈ 2.8 kB** | 4–8 kB WOFF2 + Subset-Build-Tooling | 0 kB Markup, 4 kB JS-Painter | 0 kB Markup, 4 kB JS-Painter + 4 kB Sprite-PNG | je nach Mix 3–6 kB |
| Render-Performance | gut bis ~1000 Elemente, hier ~25 → unproblematisch | **gut, Browser teilen die Symbol-Definition; in Chrome/Edge sogar schneller als externe Sprites laut Cloud-Four-Test** | sehr gut (Browser cacht Glyph-Tabelle); aber Hinting/Rendering bei kleinen px-Grössen unscharf | unnötiger Overhead — 25 Canvas-Kontexte = 25 Compositing-Layer | sehr gut, aber Skalierung verlustbehaftet | mittelmässig komplex |
| Accessibility | sauber: `aria-label` pro Auftritt | **sauber: `aria-label` pro `<use>`-Container** | gut, wenn `aria-label` parallel ausgegeben wird; das PUA-Zeichen selbst wird nicht ausgesprochen | nur über `aria-label` am Container; Canvas-Inhalt ist Black Box | gleiches Problem wie Per-Glyph-Canvas | abhängig vom Display-Track |
| Implementations-Aufwand | gering — direkter Port aus `paint_dozenal_digit` | **mittel — einmal pro Glyph `<path>`/`<circle>` definieren, danach trivial referenzieren** | hoch — Glyph-Editor (FontForge / Glyphs), WOFF2-Build, Subsetting, CSS-`@font-face` | mittel — JS-Painter aus Rust portieren | hoch — Sprite-Generierung, Cache-Invalidierung beim Theming | hoch — zwei parallele Systeme |
| Wartbarkeit (Glyph-Änderung) | 12 Stellen ändern, falls naiv | **eine Stelle: das `<symbol>` editieren, alle Auftritte ändern sich** | Font neu bauen und ausliefern | JS-Code in einer Stelle | Sprite-Sheet neu rendern | doppelter Aufwand |
| Skalierungs-Verhalten | subpixel-sauber bei jeder Grösse | **subpixel-sauber bei jeder Grösse** | gut bis ~12 px, darunter Hinting-abhängig unscharf | rasterbasiert pro `devicePixelRatio` | rasterbasiert, sichtbare Artefakte bei Up-Scale | abhängig |
| CSS-Stylability | voll: `stroke`, `currentColor`, `fill` | **voll, sofern `<symbol>` `stroke="currentColor"` nutzt** | nur Schriftfarbe und -grösse; Strichstärke nicht steuerbar | nur per JS-Repaint | nur per JS-Repaint | mittel |
| Copy/Paste & Selektion | Glyph nicht textuell vorhanden → Custom-Lösung nötig | **selbe Lage; lösbar mit unsichtbarem Text-Sibling** | natively möglich, da PUA-Zeichen Unicode-Codepoints haben | unmöglich ohne Custom-Layer | unmöglich ohne Custom-Layer | je nach Display-Track |

## Empfehlung mit Begründung

Der SVG-Sprite-Ansatz (`<symbol>` + `<use>`) gewinnt klar, weil er drei voneinander unabhängige Anforderungen gleichzeitig erfüllt: die **didaktische Aussage** der Glyphen bleibt geometrisch exakt erhalten (subpixel-saubere Vektoren statt schriftspezifischer Hinting-Tricks), die **Wartung** ist trivial (ein `<symbol>` ändern reicht für alle 25 Auftritte), und die **Bundle-Kosten** sind quasi null (~2 kB für die Symbol-Definitionen, plus eine handvoll Bytes pro `<use>`-Verweis). Die Konkurrenz scheitert jeweils an genau einem dieser drei Punkte: Inline-SVG dupliziert Markup unnötig; ein PUA-Font wäre die elegantere Lösung in einer Welt mit kleinen Pixel-Dichten, scheitert aber an Hinting-Unschärfe bei den ~16-px-Tastatur-Beschriftungen und am hohen Build-Aufwand für zwölf Glyphen; Canvas-basierte Ansätze rauben Accessibility und CSS-Stylability. Performance ist hier ohnehin kein Engpass: 25 SVG-Symbol-Referenzen sind drei Grössenordnungen unter der Schwelle, an der SVG einbricht (~1000 DOM-Knoten laut Cloud-Four-Stresstest). Der **Hybrid-Ansatz** (Option 6) ist nicht nötig — der CLAUDE.md-Geist des Minimalismus spricht für **einen Glyph-Pfad für alle Kontexte**.

## Drei SVG-Snippets (D1, D5, D8)

Koordinatensystem: `viewBox="0 0 100 100"`, Zentrum c = (50, 50), q = 25. Y wächst nach unten (SVG-Konvention, identisch zur Rust-Implementierung). `stroke="currentColor"` macht den Glyph CSS-färbbar; `stroke-width` und `stroke-linecap` sind über CSS-Variablen oder Inline überschreibbar.

### D1 — Anker-Ziffer, Pfeil nach oben

```html
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"
     role="img" aria-label="eins"
     fill="none" stroke="currentColor" stroke-width="6" stroke-linecap="round">
  <!-- Spitze c+(0,-q)=(50,25), Federn (25,75) und (75,75) -->
  <path d="M50 25 L25 75 M50 25 L75 75"/>
</svg>
```

### D5 — Komposit-Ziffer, linker Halbkreis oben + rechter Halbkreis unten

```html
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"
     role="img" aria-label="fünf"
     fill="none" stroke="currentColor" stroke-width="6" stroke-linecap="round">
  <!-- Oberer Halbkreis: Mittelpunkt (50,25), r=25, LINKE Hälfte.
       Bogen von (50,50) nach (50,0), 180°, gegen den Uhrzeigersinn (sweep=0). -->
  <path d="M50 50 A25 25 0 0 0 50 0"/>
  <!-- Unterer Halbkreis: Mittelpunkt (50,75), r=25, RECHTE Hälfte.
       Bogen von (50,50) nach (50,100), 180°, im Uhrzeigersinn (sweep=1). -->
  <path d="M50 50 A25 25 0 0 1 50 100"/>
</svg>
```

### D8 — Komposit-Ziffer, zwei Vollkreise übereinander

```html
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"
     role="img" aria-label="acht"
     fill="none" stroke="currentColor" stroke-width="6" stroke-linecap="round">
  <!-- Oberer Vollkreis: Mittelpunkt (50,25), r=25 -->
  <circle cx="50" cy="25" r="25"/>
  <!-- Unterer Vollkreis: Mittelpunkt (50,75), r=25 -->
  <circle cx="50" cy="75" r="25"/>
</svg>
```

Die `stroke-width="6"` ist ein Default für die viewBox-Skala (entspricht ~2 px Strichstärke bei 33-px-Rendering). In der finalen Sprite-Variante steht die Strichstärke nicht im `<symbol>`, sondern wird per CSS (`.glyph { stroke-width: 2; }` und Kontext-spezifische Overrides) gesetzt — exakt wie die heutigen `width: f32`-Parameter in `paint_dozenal_digit`.

### Sprite-Form (Produktion)

```html
<svg style="display:none" aria-hidden="true">
  <symbol id="d1" viewBox="0 0 100 100"><path d="M50 25 L25 75 M50 25 L75 75"/></symbol>
  <symbol id="d5" viewBox="0 0 100 100">
    <path d="M50 50 A25 25 0 0 0 50 0"/>
    <path d="M50 50 A25 25 0 0 1 50 100"/>
  </symbol>
  <symbol id="d8" viewBox="0 0 100 100">
    <circle cx="50" cy="25" r="25"/>
    <circle cx="50" cy="75" r="25"/>
  </symbol>
  <!-- d0, d2, d3, d4, d6, d7, d9, d10, d11 analog -->
</svg>

<!-- Auftritt: -->
<svg class="glyph" role="img" aria-label="acht"><use href="#d8"/></svg>
```

Im Stylesheet:
```css
.glyph { fill: none; stroke: currentColor; stroke-width: 2; stroke-linecap: round; width: 1em; height: 1em; }
.keypad .glyph { stroke-width: 2; }
.display .glyph { stroke-width: 2.5; }
.legend .glyph { stroke-width: 1.5; }
```

## Accessibility-Skizze für ein periodisches Display-Ergebnis

Beispiel-Display: `0.1̄8̄6̄Ā3̄…` (das Ergebnis von 1/7 in Dozenal, mit Überstrich über `186A3` und State-C-`…`-Suffix auf Überstrich-Höhe).

**Problem.** Ein Screenreader, der das DOM Glyph für Glyph abliest, würde 25 Mal `<svg aria-label="...">` plus die Punkt- und Überstrich-Elemente einzeln vorlesen — `null punkt eins acht sechs zehn drei Auslassung`. Das ist mathematisch korrekt, aber unbrauchbar, weil die didaktische Information „Periode beginnt nach dem Komma, läuft von 1 bis 3" verloren geht.

**Lösung in zwei Ebenen.** Der Result-Container bekommt `role="math"` und ein zusammenfassendes `aria-label`, das die Periode explizit benennt. Die einzelnen Glyph-`<svg>`s werden parallel mit `aria-hidden="true"` versteckt, damit sie nicht zusätzlich vorgelesen werden.

```html
<div class="result" role="math"
     aria-label="null Komma; Periode beginnt: eins, acht, sechs, zehn, drei; Periode wiederholt sich">
  <svg aria-hidden="true"><use href="#d0"/></svg>
  <span class="point" aria-hidden="true">.</span>
  <span class="overline" aria-hidden="true">
    <svg><use href="#d1"/></svg>
    <svg><use href="#d8"/></svg>
    <svg><use href="#d6"/></svg>
    <svg><use href="#d10"/></svg>
    <svg><use href="#d3"/></svg>
  </span>
  <span class="ellipsis-overline" aria-hidden="true">…</span>
</div>
```

Der `aria-label`-String wird im Result-Formatter (`eval.rs` bzw. dessen DOM-Pendant) zusammen mit dem visuellen `result_buffer` erzeugt — eine zusätzliche `aria_text(&Rational, pre_len, period_len)`-Funktion in der Logik-Schicht, die die Period-Detection-Information in deutschsprachigen Lesetext umwandelt. Die Funktion lebt zwingend in `logic.rs` / `eval.rs`, weil sie auf `period_len` zugreifen muss, **nicht** in `painting.rs` / der DOM-Layer — das wahrt die in CLAUDE.md festgeschriebene Layer-Trennung.

**Copy/Paste.** Eine separate, visuell versteckte Textnode (`<span class="sr-only">0.[186A3]…</span>` mit der alphanumerischen Notation, wie sie auch im Info-Modal verwendet wird) hängt im Result-Container und gewährleistet Auswahl und Kopieren in eine textbasierte Umgebung. Diese Notation passt zur Konvention der Info-Modal-Tabellen (siehe `INFO_MODAL_CONTENT.md`), in denen die Ziffern als `A`/`B` notiert sind.

## Migration-Pfad aus `paint_dozenal_digit`

- **Schritt 1 — Symbol-Generator.** Eine Build-Helper-Funktion `src/glyph_svg.rs::emit_symbols() -> &'static str` portiert die zwölf `match digit`-Arme aus `painting.rs` in `<symbol>`-Strings. Anker-Ziffern werden zu zwei `<path d="M…L…">`-Einträgen, Komposit-Ziffern zu Kombinationen aus `<circle>` und `<path d="M50 50 A25 25 0 …">`. Die Halbkreis-`sweep-flag`-Logik folgt der Winkel-Konvention aus `GLYPHS.md`: `start=-90°,end=+90°` (rechte Hälfte) → SVG-`A` von oberster Position nach unterster mit `sweep=1`; `start=+90°,end=+270°` (linke Hälfte) → `sweep=0`.
- **Schritt 2 — Statisches Asset.** Das Output ist eine einzelne `glyphs.svg` (oder ein Inline-Block am Start des `<body>`). Inline ist im egui-DOM-Port vorteilhaft, weil `<use href="#d5">` ohne Same-Origin- und Fetch-Latenz-Probleme funktioniert.
- **Schritt 3 — Glyph-Komponente.** In Leptos/Dioxus: `Glyph(digit: DozenalDigit) -> View` rendert `<svg class="glyph" role="img" aria-label={german_name(digit)}><use href={format!("#d{}", digit as u8)}/></svg>`. Die deutschen Namen (`german_name`) leben in `logic.rs`, weil sie sprach-/lokalisierungsabhängig sind — nicht im UI-Layer.
- **Schritt 4 — Result-Renderer.** Die heute in `painting.rs` lebende Overline-Logik wird zu CSS-`text-decoration: overline` über einen `<span class="overline">`-Wrapper. State-C-`…` wird ein zweites `<span class="ellipsis-overline">` mit `top` so positioniert, dass es auf Überstrich-Höhe sitzt. Die State-Bestimmung (A/B/C) bleibt unverändert in `eval.rs`.
- **Schritt 5 — Quality-Check.** Visuelle Diff-Validierung gegen die aktuelle Trunk-Version: D5 nebeneinander rendern (Rust-Web vs. DOM-Port), Pixel-für-Pixel vergleichen. Wenn D5 stimmt, gelten alle Komposit-Glyphen (selbe Primitive — siehe `GLYPHS.md` § Verifikation).
- **Schritt 6 — paint_dozenal_digit auslaufen lassen.** Erst nach erfolgreicher Diff-Validierung wird die alte Funktion aus `painting.rs` entfernt. Bis dahin existieren beide Pfade parallel, gesteuert über ein Feature-Flag.

## Quellen

- [ARIA: img role — MDN](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/img_role)
- [ARIA: aria-label — MDN](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Attributes/aria-label)
- [Accessible SVGs — CSS-Tricks](https://css-tricks.com/accessible-svgs/)
- [Using ARIA to enhance SVG accessibility — TPGi](https://www.tpgi.com/using-aria-enhance-svg-accessibility/)
- [Which SVG technique performs best for way too many icons? — Cloud Four](https://cloudfour.com/thinks/svg-icon-stress-test/)
- [SVG sprites: old-school, modern, unknown, and forgotten — Vadim Makeev](https://pepelsbey.dev/articles/svg-sprites/)
- [Optimizing SVGs for Web Performance & Scalability — DEV Community](https://dev.to/frontendtoolstech/optimizing-svgs-for-web-performance-scalability-in-2025-3df2)
- [SVG vs Canvas — JointJS Blog](https://www.jointjs.com/blog/svg-versus-canvas)
- [fontTools subset documentation](https://fonttools.readthedocs.io/en/stable/subset/)
- [Glyphhanger — Stefan Judis](https://www.stefanjudis.com/notes/glyphhanger-a-tool-subset-and-optimize-fonts/)
