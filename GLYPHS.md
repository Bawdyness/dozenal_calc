# GLYPHS.md — Spezifikation der zwölf dozenalen Ziffer-Glyphen

Dieses Dokument beschreibt die zwölf Custom-Ziffer-Symbole sprachneutral, sodass sie ohne Lektüre des Rust-Quellcodes (`src/painting.rs`) in einer anderen Render-Umgebung (Flutter `CustomPainter`, SVG, Canvas API) reproduziert werden können.

Quelle der Wahrheit ist die Funktion `paint_dozenal_digit` in `src/painting.rs:195–264`. Bei Diskrepanz zwischen diesem Dokument und dem Rust-Code gilt der Code.

## Designprinzip

Die zwölf Glyphen zerfallen in zwei Klassen:

**Anker-Ziffern (4 Stück): D1, D4, D7, D10.** Sie sind stilisierte Pfeilspitzen, die in die vier Himmelsrichtungen zeigen — D1 (oben), D4 (links), D7 (rechts), D10 (unten). Diese vier teilen den Zahlenkreis in vier Dreiergruppen analog zu den Stunden 12, 3, 6, 9 auf einem Zifferblatt.

**Komposit-Ziffern (8 Stück): D0, D2, D3, D5, D6, D8, D9, D11.** Sie sind Kompositionen aus oberen und unteren Halbkreisen oder Vollkreisen. Jede dieser Ziffern besteht aus genau zwei Elementen, die über- bzw. nebeneinander platziert sind.

## Koordinaten- und Pfad-Konventionen

Alle Glyphen werden in eine **rechteckige Bounding-Box** gezeichnet. Sei
- `min_edge = min(box.width, box.height)`
- `r = min_edge / 2`
- `q = min_edge / 4`
- `c = (box.center.x, box.center.y)` der Mittelpunkt der Box

**Y-Achse:** positiv nach unten (Bildschirm-Konvention). `c + (0, -q)` ist also **oben** vom Mittelpunkt.

**Strichstärke:** Aufrufer-Parameter `width`. In der App typischerweise `2.0` für Display-Glyphen, `1.5` für Legenden-Glyphen, `2.0` für Buttons. Die Strichstärke skaliert nicht mit `min_edge` — sie ist absolut in Pixeln.

**Linien-Endkappen und Joins:** Standard-Verhalten der Render-Engine (egui nutzt Default; Flutter empfiehlt `StrokeCap.round` für visuell konsistentes Ergebnis).

**Bögen werden als Polylinien aus 21 Stützpunkten gezeichnet** (`draw_arc` in `painting.rs:266–281`). Bei genauer Reproduktion sollte ein nativer Arc-Befehl (`Canvas.drawArc` in Flutter) gleichwertig sein; die 21-Punkt-Approximation ist ein Implementations-Detail, kein Designmerkmal.

**Winkel-Konvention für Bögen:** wie in egui und Flutter Canvas — `0° = positive x-Richtung (rechts)`, `90° = positive y-Richtung (unten)`, im Uhrzeigersinn positiv. Ein Bogen `start=−90°, end=+90°` zeichnet die **rechte Hälfte** eines Kreises (vom obersten Punkt im Uhrzeigersinn zum untersten Punkt). Ein Bogen `start=+90°, end=+270°` zeichnet die **linke Hälfte** (vom untersten Punkt im Uhrzeigersinn zum obersten Punkt).

## Anker-Ziffern (Pfeile)

Jede Anker-Ziffer besteht aus **zwei Liniensegmenten**, die einen V-förmigen Pfeil bilden. Die Spitze zeigt in die jeweilige Himmelsrichtung; die beiden "Federn" bilden den hinteren Teil des V.

| Ziffer | Richtung | Spitze (relativ zu c) | Feder 1 | Feder 2 |
|---|---|---|---|---|
| **D1** | ↑ oben   | `(0, -q)`  | `(-q, +q)` | `(+q, +q)` |
| **D4** | ← links  | `(-q, 0)`  | `(+q, -q)` | `(+q, +q)` |
| **D7** | → rechts | `(+q, 0)`  | `(-q, -q)` | `(-q, +q)` |
| **D10** | ↓ unten | `(0, +q)`  | `(-q, -q)` | `(+q, -q)` |

Pseudocode für D1:
```
line(from = c + (0, -q), to = c + (-q, +q))
line(from = c + (0, -q), to = c + (+q, +q))
```

## Komposit-Ziffern

Alle Komposit-Ziffern haben **zwei Elemente**: ein **oberes** und ein **unteres**. Jedes Element ist entweder ein Vollkreis, ein linker Halbkreis oder ein rechter Halbkreis, mit Radius `q` und Mittelpunkt bei `c + (0, -q)` (oben) bzw. `c + (0, +q)` (unten).

| Element-Typ | Beschreibung | Definition |
|---|---|---|
| **Voll** | Vollkreis | `circle_stroke(center, radius=q)` |
| **R** | rechter Halbkreis (rechte Seite des Kreises sichtbar) | `arc(center, radius=q, start=−90°, end=+90°)` |
| **L** | linker Halbkreis | `arc(center, radius=q, start=+90°, end=+270°)` |

Die zwölf-Ziffer-Tabelle verteilt sich dann so:

| Ziffer | Oberes Element | Unteres Element | Bedeutung |
|---|---|---|---|
| **D0** | (nichts) | Vollkreis bei `c` (zentriert), Radius `q` | Sonderfall: ein einzelner Kreis am Mittelpunkt |
| **D2** | R bei `c+(0,-q)` | L bei `c+(0,+q)` | Z-förmige S-Kurve |
| **D3** | R | R | beide Halbkreise rechts → C-Form gespiegelt |
| **D5** | L | R | mittige X-artige Form |
| **D6** | L | Voll | linker Halbkreis oben, Vollkreis unten |
| **D8** | Voll | Voll | zwei Vollkreise übereinander (Achterform) |
| **D9** | Voll | R | Vollkreis oben, rechter Halbkreis unten |
| **D11** | R | Voll | rechter Halbkreis oben, Vollkreis unten |

### D0 — Sonderfall

D0 ist der einzige Glyph mit nur einem Element. Er wird als zentrierter Vollkreis gezeichnet:
```
circle_stroke(center = c, radius = q)
```
Nicht zu verwechseln mit D8, das aus zwei separaten Vollkreisen besteht.

### Beispiel: D5 als Pseudocode

```
arc(center = c + (0, -q), radius = q, start_deg = +90, end_deg = +270)   # linker Halbkreis oben
arc(center = c + (0, +q), radius = q, start_deg = -90, end_deg = +90)    # rechter Halbkreis unten
```

### Beispiel: D8 als Pseudocode

```
circle_stroke(center = c + (0, -q), radius = q)
circle_stroke(center = c + (0, +q), radius = q)
```

## Geometrie-Konsistenz

Bei allen Komposit-Ziffern berühren oder überlappen sich die beiden Elemente am Mittelpunkt `c`:
- Die **Oberkante des unteren Elements** liegt bei `y = +q − q = 0` (Mittelpunkt).
- Die **Unterkante des oberen Elements** liegt bei `y = −q + q = 0` (Mittelpunkt).

Bei Halbkreisen kommen die beiden offenen Enden der Bögen exakt am Mittelpunkt zusammen. Bei Vollkreisen schneiden sich die Kreise im Mittelpunkt tangential (sie berühren sich an einem Punkt).

Diese Eigenschaft sorgt für die optische Geschlossenheit der Komposit-Glyphen.

## Verifikation

Beim Port: D5 in der Zielumgebung rendern und visuell mit der Rust-Web-Version (`trunk serve` oder GitHub Pages: `https://bawdyness.github.io/dozenal_calc/`) vergleichen. Wenn D5 stimmt, ist es sehr wahrscheinlich, dass alle anderen Komposit-Ziffern ebenfalls stimmen, weil sie dieselben Primitives verwenden. Die Anker-Ziffern (D1, D4, D7, D10) müssen separat verifiziert werden, weil sie eine andere Konstruktionslogik haben.
