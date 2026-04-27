# INFO_MODAL_CONTENT.md

Referenzdatei für den Inhalt des Info-Bereichs (Position 10.3). Diese Datei ist nicht Teil des Builds — sie dient Claude Code als Vorlage für die Texte, die im Info-Bereich dargestellt werden.

## UI-Verhalten

- **Navigation**: Liste → Detail → Zurück. Beim Klick auf „Info" (10.3) erscheint eine **Liste mit 12 Kapitelüberschriften**, vertikal übereinander. Ein Klick auf eine Überschrift öffnet die **Detailansicht** des Kapitels (scrollbar, volle Fläche). Ein Zurück-Knopf oben links kehrt zur Kapitelliste zurück.
- **Visueller Charakter**: Der Info-Bereich ist bewusst **nicht minimalistisch** — er darf Text, Tabellen und ggf. einfache Illustrationen (SVG-Grafiken, geometrische Zeichnungen) enthalten. Er ist vom Rechner klar getrennt: kein Taschenrechner-Key ist sichtbar, solange ein Kapitel offen ist.
- **Sprache**: Deutsch.
- **Kapitelanzahl**: Genau 12. Kein Wachstum darüber hinaus.

---

## Kapitelgliederung

---

### Kapitel 1 — Bedienung

**Titel in der Liste**: Bedienung des Rechners

**Inhalt**:

**Die Ziffern**

Dieser Rechner verwendet eigene Symbole für alle zwölf Ziffern. Die folgende Legende zeigt jedes Symbol neben seiner dezimalen Entsprechung:

> **[VISUELLE LEGENDE]**
> Zwölf Zeilen, jede bestehend aus: links das gezeichnete Dozenalsymbol (via `paint_dozenal_digit`), ein Gleichzeichen in der Mitte, rechts die vertraute Dezimalziffer (0–11). Das Rendering verwendet denselben Code wie die Tastatur — kein Bild, kein SVG, keine externe Grafik.
> Implementierungshinweis für CC: Für jedes `DozenalDigit::D0` bis `DozenalDigit::D11` ein kleines Rect zeichnen mit `paint_dozenal_digit`, daneben `=` als Text, daneben die Dezimalziffer als Text. Reihenfolge: D0=0, D1=1, D2=2, …, D11=11.

Das Symbolsystem hat eine innere Logik: die Ziffern 1, 4, 7 und 10 sind *Ankerziffern* — Pfeilspitzen, die in die vier Himmelsrichtungen zeigen (oben, links, rechts, unten). Sie teilen den Zahlenkreis in vier Dreiergruppen, wie die Stunden 12, 3, 6 und 9 auf einem Zifferblatt. Alle Ziffern dazwischen bestehen aus Halbkreisen und Vollkreisen. Die Null ist ein einfacher Kreis.

**Grundbedienung**

Tippe Zahlen und Operatoren wie auf einem gewöhnlichen Taschenrechner. Drücke die breite Taste am unteren Rand, um das Ergebnis zu berechnen. `AC` löscht die gesamte Eingabe und das Ergebnis, `Del` entfernt das Zeichen links vom Cursor.

**Cursor und Navigation**

Der rote Strich im Eingabefeld ist dein Cursor. Mit den Pfeiltasten `◀` und `▶` bewegst du ihn, um mitten in einer Formel Zeichen einzufügen oder zu löschen. Nach einer Berechnung wandert der Cursor ins Ergebnisfeld — die Pfeile bewegen dann den Ergebnis-Cursor. Sobald du eine neue Eingabe machst, springt der Cursor zurück ins Eingabefeld.

**Weiterrechnen**

Nach einer Berechnung kannst du direkt mit einem Operator weitermachen. Tippst du zum Beispiel `+ 5 =`, verwendet der Rechner automatisch das letzte Ergebnis als ersten Operanden. Wenn du stattdessen eine ganz neue Rechnung beginnen willst, drücke zuerst `AC`.

**Doppelklick für Umkehrfunktionen**

Ein zweiter Klick auf eine Funktionstaste wandelt sie in ihre Umkehrfunktion um: `sin` wird zu `sin⁻¹`, `cos` zu `cos⁻¹`, und so weiter. Das gilt auch für die hyperbolischen Funktionen im Erweiterungsfeld. Ein kleiner Marker auf der Taste zeigt dir, dass die Umkehrfunktion bereit ist.

**Spezialoperatoren**

- `x²` quadriert die vorangehende Zahl.
- `√` berechnet die Quadratwurzel. Steht links davon eine Zahl, wird diese als Wurzelgrad verwendet: `3√27` ergibt die dritte Wurzel von 27.
- `log` berechnet den Logarithmus zur Basis der vorangehenden Zahl.
- `⊕` berechnet die Paralleladdition: das Ergebnis von `a ⊕ b` ist `(a·b)/(a+b)`. Nützlich unter anderem für Parallelschaltungen von Widerständen.

**Erweiterungsfeld**

Die Taste `…` rechts unten auf der Haupttastatur öffnet das Erweiterungsfeld mit weiteren Funktionen: Speicher, Konstanten (π, e, φ, √2), hyperbolische Funktionen, erweiterte Operatoren und Einstellungen. Das Erweiterungsfeld schliesst sich über die Taste rechts unten im Erweiterungsfeld selbst.

**Speicher**

Im Erweiterungsfeld findest du die Speichertasten: `STO` speichert das aktuelle Ergebnis, `RCL` fügt den gespeicherten Wert in die Eingabe ein, `MC` löscht den Speicher. Ein kleines `M` im Display zeigt an, dass etwas gespeichert ist. `Ans` fügt das Ergebnis der letzten Berechnung ein.

**Periodenstrich**

Wenn das Ergebnis ein periodischer Bruch ist, zeigt der Rechner die sich wiederholenden Ziffern mit einem Strich darüber an. Beispiel: `1/5` ergibt `0.2497` mit Strich über allen vier Ziffern, weil sich die Folge `2497` unendlich wiederholt. Bei Perioden mit mehr als fünf Stellen werden nur die ersten fünf gezeigt, gefolgt von drei Punkten.

**Anzeige und Winkelmodus**

`Doz↔Dec` im Erweiterungsfeld schaltet die Anzeige zwischen dozenal (Basis 12) und dezimal (Basis 10) um — praktisch, um ein Ergebnis in vertrauter Schreibweise zu überprüfen. `DRG` wechselt den Winkelmodus für trigonometrische Funktionen: Grad, Bogenmaß oder Gon.

**Schwierigkeit**: Gering. ✅ Fliesstext fertig.

---

### Kapitel 2 — Das Dozenalsystem

**Titel in der Liste**: Was ist das Dozenalsystem?

**Inhalt**:

**Das Prinzip**

Im Dezimalsystem hat jede Stelle den zehnfachen Wert der Stelle rechts davon: Einer, Zehner, Hunderter. Im Dozenalsystem (auch Duodezimalsystem) ist die Basis nicht zehn, sondern zwölf. Die Stellenwerte sind Potenzen von 12: Einer, Zwölfer, Hundertvierundvierziger. Die Zahl „100" bedeutet hier nicht zehn mal zehn, sondern zwölf mal zwölf — also 144 im Dezimalen.

Dafür braucht man zwölf Ziffern statt zehn. Die Ziffern 0 bis 9 sind bekannt, für die Werte zehn und elf kommen zwei neue dazu, die dieser Rechner mit eigenen Symbolen darstellt (siehe Kapitel 1).

**Warum gerade zwölf?**

Der Grund ist Teilbarkeit. Zwölf hat sechs Teiler: 1, 2, 3, 4, 6 und 12. Zehn hat nur vier: 1, 2, 5 und 10. Das klingt nach einem kleinen Unterschied, aber die Auswirkung auf den Alltag ist erheblich — vor allem beim Bruchrechnen.

Vergleich der Stammbrüche:

| Bruch | Basis 10 | Basis 12 |
|---|---|---|
| 1/2 | 0.5 | 0.6 |
| 1/3 | 0.333… | 0.4 |
| 1/4 | 0.25 | 0.3 |
| 1/5 | 0.2 | 0.2497… |
| 1/6 | 0.166… | 0.2 |
| 1/8 | 0.125 | 0.16 |
| 1/9 | 0.111… | 0.14 |
| 1/10 | 0.1 | 0.1249… |
| 1/12 | 0.0833… | 0.1 |

In Basis 10 sind die Drittel und Sechstel unendliche Dezimalbrüche. In Basis 12 sind sie kurz und exakt. Dafür werden Fünftel und Zehntel periodisch — ein fairer Tausch, wenn man bedenkt, wie viel häufiger man im Alltag durch drei und vier teilt als durch fünf.

**Die Regel dahinter**

Welche Brüche endlich sind und welche periodisch werden, folgt einem einfachen Gesetz: ein Bruch 1/n hat in einer Basis b genau dann eine endliche Darstellung, wenn alle Primfaktoren von n auch Primfaktoren von b sind. Die Primfaktoren von 12 sind 2 und 3. Also ist jeder Bruch endlich, dessen Nenner nur aus Zweien und Dreien zusammengesetzt ist (2, 3, 4, 6, 8, 9, 12, 16, 18, 24, …). Alles andere — Nenner mit einer 5, 7 oder 11 — wird periodisch. Der Rechner zeigt diese Periodizität mit einem Strich über den sich wiederholenden Ziffern an.

**Spuren in der Geschichte**

Die Zwölf als Ordnungsgrösse ist älter als jedes Zahlensystem. Die Babylonier rechneten in Basis 60, aber organisierten ihre Ziffern in Gruppen von 12. Im Handel zählte man in Dutzenden (12) und Gros (144 = 12²). Die Angelsachsen teilten den Schilling in 12 Pence. Und bis heute hat der Tag 2×12 Stunden, das Jahr 12 Monate, der Vollkreis 360 = 30×12 Grad.

Die Dozenal Society of America (gegründet 1944, heute Dozenal Society of Great Britain und weitere Ableger) setzt sich seit Jahrzehnten dafür ein, die Vorzüge der Basis 12 bekannter zu machen. Dieser Rechner steht in dieser Tradition — nicht als Forderung nach einer Systemumstellung, sondern als Werkzeug zum Erforschen und Staunen.

**Schwierigkeit**: Gering. ✅ Fliesstext fertig.

---

### Kapitel 3 — Die Zwölf in der Zahlentheorie

**Titel in der Liste**: Fibonacci, Quadratzahlen und andere Kuriositäten

**Inhalt**:

**144 — wo sich zwei Welten treffen**

Die Fibonacci-Folge beginnt mit 1, 1, und jede weitere Zahl ist die Summe der beiden vorangehenden: 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, … Sie wächst exponentiell. Die Quadratzahlen — 1, 4, 9, 16, 25, 36, … — wachsen dagegen nur quadratisch. Zwei so unterschiedliche Folgen haben fast keinen Grund, sich jemals zu treffen. Und doch tun sie es: die zwölfte Fibonacci-Zahl ist 144, und 144 = 12². Die zwölfte Fibonacci-Zahl ist das Quadrat der Zwölf.

J. H. E. Cohn bewies 1964, dass dies kein Zufall ist, sondern ein Unikat: abgesehen von den trivialen Fällen F(1) = F(2) = 1 gibt es keine weitere Fibonacci-Zahl, die zugleich eine perfekte Quadratzahl ist. Die Zwölf steht also an einer einmaligen Kreuzung zweier fundamentaler Zahlenfolgen.

**Der Goldene Schnitt**

Die Fibonacci-Folge ist eng verknüpft mit dem Goldenen Schnitt φ = (1+√5)/2 ≈ 1.618 (dezimal). Das Verhältnis zweier aufeinanderfolgender Fibonacci-Zahlen nähert sich φ immer weiter an: 8/5 = 1.6, 13/8 = 1.625, 21/13 ≈ 1.615, 144/89 ≈ 1.618 (alles dezimal). In Basis 12 ist φ ≈ 1.74BB6772 — der Rechner hat φ als Konstante im Erweiterungsfeld. Wer `φ² =` tippt, wird sehen, dass das Ergebnis genau `φ + 1` ist — die definierende Eigenschaft des Goldenen Schnitts, live verifizierbar.

**12 = 2²×3 — eine Primfaktorzerlegung mit Folgen**

Dass zwölf genau die Primfaktoren 2 und 3 enthält (und zwar als 2²×3), erklärt nicht nur die schönen Brüche aus Kapitel 2, sondern auch eine Reihe zahlentheoretischer Auszeichnungen:

Zwölf ist eine *hochzusammengesetzte Zahl* (highly composite number): sie hat mehr Teiler als jede kleinere natürliche Zahl. Die Teiler von 12 sind 1, 2, 3, 4, 6, 12 — das sind sechs Stück. Keine Zahl unter 12 kommt auf sechs Teiler. Srinivasa Ramanujan definierte und untersuchte diese Klasse von Zahlen in einer berühmten Arbeit von 1915 in den Proceedings of the London Mathematical Society.

Zwölf ist auch die kleinste *abundante Zahl*: die Summe ihrer echten Teiler (1+2+3+4+6 = 16) übertrifft die Zahl selbst. Bei den meisten kleinen Zahlen ist es umgekehrt — bei 10 zum Beispiel ergibt 1+2+5 = 8, was kleiner ist als 10. Zwölf ist die erste Zahl, bei der die Teiler „überquellen".

**Platons ideale Stadt**

In seinen „Gesetzen" (Buch V) stellt Platon die Frage, wie viele Bürger eine ideale Stadt haben sollte. Seine Antwort: 5040. Das Argument ist nicht mystisch, sondern praktisch: eine Stadt muss ihre Bürger ständig in gleich grosse Gruppen einteilen — für Stämme, Gerichtssprengel, Militäreinheiten, Festversammlungen. Je mehr Teiler die Bürgerzahl hat, desto flexibler lässt sie sich aufteilen. Platon bemerkt ausdrücklich, dass 5040 durch jede Zahl von 1 bis 12 teilbar ist (mit der einzigen Ausnahme von 11) — und genau das macht sie nützlich.

Was Platon also intuitiv beschreibt, ist dieselbe Einsicht, die dem Dozenalsystem zugrunde liegt: im Alltag sind die *kleinen* Teiler die wichtigen. Wir halbieren, dritteln, vierteln und sechsteln ständig. Eine Zahl, die das alles glatt ermöglicht, ist praktischer als eine, die es nicht tut. Zwölf ist die kleinste Zahl mit dieser Eigenschaft, und 5040 = 7! ist ihre grosse Schwester — eine Zahl, die dieselbe Teilbarkeitsphilosophie auf eine ganze Stadtbevölkerung anwendet.

**Schwierigkeit**: Gering bis mittel. ✅ Fliesstext fertig.

---

### Kapitel 4 — Das regelmässige Zwölfeck (Einführung)

**Titel in der Liste**: Das Zwölfeck — Grundlagen

**Inhalt**:

**Was ist ein regelmässiges Zwölfeck?**

Ein regelmässiges Zwölfeck (Dodekagon) ist ein Vieleck mit zwölf gleich langen Seiten und zwölf gleich grossen Innenwinkeln. Jeder dieser Innenwinkel beträgt 150° — oder, im Dozenalsystem ausgedrückt, 106°. Es ist eine der ältesten und am häufigsten verwendeten geometrischen Formen: man findet sie in Zifferblättern, Münzen, Bauornamentik und Pflastermustern.

**Das Schweizer Taschenmesser der Vielecke**

Was das regelmässige Zwölfeck einzigartig macht, ist nicht seine Form an sich, sondern was alles in ihm steckt. Verbindet man jede vierte Ecke miteinander, entsteht ein gleichseitiges Dreieck — exakt, nicht angenähert. Verbindet man jede dritte Ecke, entsteht ein Quadrat. Verbindet man jede zweite Ecke, entsteht ein regelmässiges Sechseck. Alle drei Figuren liegen perfekt im selben Kreis, der auch das Zwölfeck umschliesst.

Das bedeutet: das Zwölfeck enthält die drei fundamentalen regulären Vielecke der Geometrie als exakte Teilfiguren. Kein anderes Vieleck mit so wenigen Ecken kann das von sich behaupten. Ein Zehneck enthält zwar ein Fünfeck, aber kein Dreieck und kein Quadrat. Ein Achteck enthält ein Quadrat, aber kein Dreieck und kein Sechseck. Das Zwölfeck vereint alles — eine direkte Folge der Teilbarkeit von 12 durch 2, 3, 4 und 6.

**Konstruierbar mit Zirkel und Lineal**

Nicht jedes regelmässige Vieleck lässt sich mit Zirkel und Lineal exakt konstruieren. Ein Siebeneck zum Beispiel ist unmöglich. Das Zwölfeck dagegen ist konstruierbar, und die Konstruktion ist verblüffend einfach: man beginnt mit einem Kreis, teilt ihn in sechs gleiche Teile (das kann man, weil das Sechseck konstruierbar ist), halbiert dann jeden dieser Teile (das kann man immer), und hat zwölf gleichmässig verteilte Punkte auf dem Kreis. Verbinden — fertig.

Die mathematische Grundlage: ein regelmässiges n-Eck ist genau dann mit Zirkel und Lineal konstruierbar, wenn n ein Produkt aus einer Zweierpotenz und verschiedenen Fermat-Primzahlen ist (Gauss, 1796). Für 12 = 2² × 3 ist das erfüllt, weil 3 eine Fermat-Primzahl ist.

**Symmetrie**

Das regelmässige Zwölfeck hat 24 Symmetrien: 12 Drehungen (um 0°, 30°, 60°, …, 330°) und 12 Spiegelungen (6 durch gegenüberliegende Ecken, 6 durch gegenüberliegende Seitenmitten). In der Sprache der Algebra bilden diese 24 Symmetrien die Diedergruppe D₁₂. Die Zahl 24 = 2×12 ist dabei kein Zufall — jedes regelmässige n-Eck hat genau 2n Symmetrien.

**Schwierigkeit**: Gering. ✅ Fliesstext fertig.

**Illustration**: Das folgende SVG zeigt das Zwölfeck mit eingeschriebenem Dreieck (teal), Quadrat (blau) und Sechseck (violett). CC soll es als egui-Zeichnung oder eingebettetes SVG in der Kapitelansicht rendern.

```svg
<svg width="100%" viewBox="0 0 680 520" role="img">
<title>Regelmässiges Zwölfeck mit eingeschriebenem Dreieck, Quadrat und Sechseck</title>
<desc>Ein regelmässiges Zwölfeck mit drei farbig markierten eingeschriebenen Vielecken: einem grünen gleichseitigen Dreieck, einem blauen Quadrat und einem violetten regelmässigen Sechseck. Alle teilen denselben Umkreis.</desc>
<g transform="translate(340, 248)">
  <!-- Umkreis (subtil) -->
  <circle cx="0" cy="0" r="200" fill="none" stroke="var(--s)" stroke-width="0.5" stroke-dasharray="4 4" opacity="0.4"/>
  <!-- Eingeschriebenes Sechseck (jede 2. Ecke: 0,2,4,6,8,10) — Purple -->
  <polygon points="0,-200 173.21,-100 173.21,100 0,200 -173.21,100 -173.21,-100"
    fill="#AFA9EC" fill-opacity="0.12" stroke="#534AB7" stroke-width="1.5" stroke-linejoin="round"/>
  <!-- Eingeschriebenes Quadrat (jede 3. Ecke: 0,3,6,9) — Blue -->
  <polygon points="0,-200 200,0 0,200 -200,0"
    fill="#85B7EB" fill-opacity="0.12" stroke="#185FA5" stroke-width="1.5" stroke-linejoin="round"/>
  <!-- Eingeschriebenes Dreieck (jede 4. Ecke: 0,4,8) — Teal -->
  <polygon points="0,-200 173.21,100 -173.21,100"
    fill="#9FE1CB" fill-opacity="0.12" stroke="#0F6E56" stroke-width="1.5" stroke-linejoin="round"/>
  <!-- Zwölfeck selbst -->
  <polygon points="0,-200 103.53,-173.21 173.21,-103.53 200,0 173.21,103.53 103.53,173.21 0,200 -103.53,173.21 -173.21,103.53 -200,0 -173.21,-103.53 -103.53,-173.21"
    fill="none" stroke="var(--p)" stroke-width="2" stroke-linejoin="round"/>
  <!-- Eckpunkte des Zwölfecks -->
  <circle cx="0" cy="-200" r="3.5" fill="var(--p)"/>
  <circle cx="103.53" cy="-173.21" r="3" fill="var(--s)"/>
  <circle cx="173.21" cy="-103.53" r="3" fill="var(--s)"/>
  <circle cx="200" cy="0" r="3.5" fill="var(--p)"/>
  <circle cx="173.21" cy="103.53" r="3" fill="var(--s)"/>
  <circle cx="103.53" cy="173.21" r="3" fill="var(--s)"/>
  <circle cx="0" cy="200" r="3.5" fill="var(--p)"/>
  <circle cx="-103.53" cy="173.21" r="3" fill="var(--s)"/>
  <circle cx="-173.21" cy="103.53" r="3" fill="var(--s)"/>
  <circle cx="-200" cy="0" r="3.5" fill="var(--p)"/>
  <circle cx="-173.21" cy="-103.53" r="3" fill="var(--s)"/>
  <circle cx="-103.53" cy="-173.21" r="3" fill="var(--s)"/>
</g>
<!-- Legende -->
<rect x="40" y="476" width="14" height="14" rx="2" fill="#9FE1CB" stroke="#0F6E56" stroke-width="1"/>
<text class="ts" x="62" y="488" dominant-baseline="central">Dreieck (jede 4. Ecke)</text>
<rect x="240" y="476" width="14" height="14" rx="2" fill="#85B7EB" stroke="#185FA5" stroke-width="1"/>
<text class="ts" x="262" y="488" dominant-baseline="central">Quadrat (jede 3. Ecke)</text>
<rect x="450" y="476" width="14" height="14" rx="2" fill="#AFA9EC" stroke="#534AB7" stroke-width="1"/>
<text class="ts" x="472" y="488" dominant-baseline="central">Sechseck (jede 2. Ecke)</text>
</svg>
```

---

### Kapitel 5 — Winkel und Diagonalen im Zwölfeck

**Titel in der Liste**: Das Zwölfeck — Winkel und Diagonalen

**Inhalt**:

**54 Diagonalen**

Eine Diagonale verbindet zwei nicht benachbarte Ecken eines Vielecks. Die Formel n(n−3)/2 liefert für das Zwölfeck 12×9/2 = 54 Diagonalen. Das klingt nach einem unübersichtlichen Netz — aber die Struktur ist bemerkenswert geordnet.

**Sechs verschiedene Längen**

Jede Diagonale überspringt eine bestimmte Anzahl von Ecken. Da das Zwölfeck symmetrisch ist, haben alle Diagonalen, die gleich viele Ecken überspringen, dieselbe Länge. Es gibt fünf mögliche Sprungweiten (1 bis 5 Ecken überspringen, plus den Durchmesser mit 6), also insgesamt sechs verschiedene Längen — die Seitenlänge s mitgezählt.

Bei Seitenlänge s = 1 lauten die exakten Werte:

| Bezeichnung | Überspringt | Exakte Länge | Dezimalwert |
|---|---|---|---|
| Seite s | 0 Ecken | 1 | 1.000 |
| d₂ | 1 Ecke | √(2+√3) | 1.932 |
| d₃ | 2 Ecken | 1+√3 | 2.732 |
| d₄ | 3 Ecken | (3√2+√6)/2 | 3.346 |
| d₅ | 4 Ecken | 2+√3 | 3.732 |
| d₆ (Durchmesser) | 5 Ecken | √6+√2 | 3.864 |

Diese Werte lassen sich alle mit dem Rechner verifizieren — man braucht nur die Wurzeltaste und die Grundrechenarten. Die Spalte „Dezimalwert" zeigt den Wert in Basis 10; der Rechner zeigt stattdessen den dozenalen Wert an.

**Verborgene Muster**

Die Tabelle enthält mehr Ordnung, als auf den ersten Blick sichtbar ist. Zwei Beobachtungen:

Die dritte und die fünfte Diagonale unterscheiden sich um genau 1: d₃ = 1+√3 und d₅ = 2+√3. Die Differenz ist die Seitenlänge selbst. Das ist kein rechnerischer Zufall, sondern eine geometrische Tatsache: im Zwölfeck bilden bestimmte Diagonalen Dreiecke mit der Seitenlänge als einer Seite, und d₃ und d₅ liegen an zwei Seiten eines solchen Dreiecks.

Der Durchmesser d₆ ist exakt doppelt so lang wie die kürzeste Diagonale d₂: √6+√2 = 2·√(2+√3). Der Durchmesser und die kürzeste Diagonale stehen also im Verhältnis 2:1 — dieselbe Proportion wie die Oktave in der Musik.

**Das 15-Grad-Raster**

Alle Winkel, die im Zwölfeck auftreten — zwischen Seiten, zwischen Diagonalen, zwischen Seiten und Diagonalen — sind Vielfache von 15°. Das liegt daran, dass die zwölf Ecken den Vollkreis in zwölf gleiche Sektoren zu je 30° teilen, und jedes Dreieck, das man aus Ecken des Zwölfecks bildet, Winkel erzeugt, die Vielfache der halben Sektorgrösse sind.

15° ist 1/24 des Vollkreises. Im Dozenalsystem: 15° = 13° (dozenal), und 30° = 26° (dozenal). Das Zwölfeck erzeugt also ein feines, gleichmässiges Winkelraster, das mit dem Dozenalsystem harmoniert: alle auftretenden Winkel lassen sich dozenal als ganzzahlige Vielfache von 13° (dozenal) schreiben.

**Schwierigkeit**: Mittel. ✅ Fliesstext fertig.

**Illustration**: Das folgende SVG zeigt je eine Diagonale jedes der sechs Typen, farbig unterschieden, mit exakten Längenwerten in der Legende rechts.

```svg
<svg width="100%" viewBox="0 0 680 560" role="img">
<title>Die sechs Diagonalentypen des regelmässigen Zwölfecks</title>
<desc>Ein regelmässiges Zwölfeck, in dem je eine Diagonale jedes der sechs Typen farbig eingezeichnet ist: die Seite selbst und die fünf Diagonalen unterschiedlicher Länge, von der kürzesten bis zum Durchmesser.</desc>
<g transform="translate(300, 248)">
  <!-- Zwölfeck (dezent) -->
  <polygon points="0,-200 100,-173.21 173.21,-100 200,0 173.21,100 100,173.21 0,200 -100,173.21 -173.21,100 -200,0 -173.21,-100 -100,-173.21"
    fill="none" stroke="var(--s)" stroke-width="1" stroke-linejoin="round" opacity="0.5"/>
  <!-- Eckpunkte (dezent) -->
  <circle cx="0" cy="-200" r="3" fill="var(--s)"/>
  <circle cx="100" cy="-173.21" r="3" fill="var(--s)"/>
  <circle cx="173.21" cy="-100" r="3" fill="var(--s)"/>
  <circle cx="200" cy="0" r="3" fill="var(--s)"/>
  <circle cx="173.21" cy="100" r="3" fill="var(--s)"/>
  <circle cx="100" cy="173.21" r="3" fill="var(--s)"/>
  <circle cx="0" cy="200" r="3" fill="var(--s)"/>
  <circle cx="-100" cy="173.21" r="3" fill="var(--s)"/>
  <circle cx="-173.21" cy="100" r="3" fill="var(--s)"/>
  <circle cx="-200" cy="0" r="3" fill="var(--s)"/>
  <circle cx="-173.21" cy="-100" r="3" fill="var(--s)"/>
  <circle cx="-100" cy="-173.21" r="3" fill="var(--s)"/>
  <!-- d₁ Seite: Ecke 0 → Ecke 1 (grau) -->
  <line x1="0" y1="-200" x2="100" y2="-173.21" stroke="#5F5E5A" stroke-width="2.5" stroke-linecap="round"/>
  <!-- d₂ kürzeste Diagonale: Ecke 1 → Ecke 3 (teal) -->
  <line x1="100" y1="-173.21" x2="200" y2="0" stroke="#0F6E56" stroke-width="2.5" stroke-linecap="round"/>
  <!-- d₃: Ecke 0 → Ecke 3 (blue) -->
  <line x1="0" y1="-200" x2="200" y2="0" stroke="#185FA5" stroke-width="2.5" stroke-linecap="round"/>
  <!-- d₄: Ecke 1 → Ecke 5 (purple) -->
  <line x1="100" y1="-173.21" x2="100" y2="173.21" stroke="#534AB7" stroke-width="2.5" stroke-linecap="round"/>
  <!-- d₅: Ecke 0 → Ecke 5 (coral) -->
  <line x1="0" y1="-200" x2="100" y2="173.21" stroke="#993C1D" stroke-width="2.5" stroke-linecap="round"/>
  <!-- d₆ Durchmesser: Ecke 0 → Ecke 6 (red) -->
  <line x1="0" y1="-200" x2="0" y2="200" stroke="#A32D2D" stroke-width="2.5" stroke-linecap="round"/>
  <!-- Endpunkte hervorheben -->
  <circle cx="0" cy="-200" r="4.5" fill="var(--p)"/>
  <circle cx="100" cy="-173.21" r="4.5" fill="var(--p)"/>
  <circle cx="200" cy="0" r="4.5" fill="var(--p)"/>
  <circle cx="100" cy="173.21" r="4.5" fill="var(--p)"/>
  <circle cx="0" cy="200" r="4.5" fill="var(--p)"/>
</g>
<!-- Legende rechts -->
<line x1="524" y1="80" x2="554" y2="80" stroke="#5F5E5A" stroke-width="2.5" stroke-linecap="round"/>
<text class="ts" x="564" y="80" dominant-baseline="central">s = 1</text>
<text class="ts" x="564" y="96" dominant-baseline="central" opacity="0.6">≈ 1.000</text>
<line x1="524" y1="130" x2="554" y2="130" stroke="#0F6E56" stroke-width="2.5" stroke-linecap="round"/>
<text class="ts" x="564" y="130" dominant-baseline="central">d₂ = √(2+√3)</text>
<text class="ts" x="564" y="146" dominant-baseline="central" opacity="0.6">≈ 1.932</text>
<line x1="524" y1="180" x2="554" y2="180" stroke="#185FA5" stroke-width="2.5" stroke-linecap="round"/>
<text class="ts" x="564" y="180" dominant-baseline="central">d₃ = 1+√3</text>
<text class="ts" x="564" y="196" dominant-baseline="central" opacity="0.6">≈ 2.732</text>
<line x1="524" y1="230" x2="554" y2="230" stroke="#534AB7" stroke-width="2.5" stroke-linecap="round"/>
<text class="ts" x="564" y="230" dominant-baseline="central">d₄ = (3√2+√6)/2</text>
<text class="ts" x="564" y="246" dominant-baseline="central" opacity="0.6">≈ 3.346</text>
<line x1="524" y1="280" x2="554" y2="280" stroke="#993C1D" stroke-width="2.5" stroke-linecap="round"/>
<text class="ts" x="564" y="280" dominant-baseline="central">d₅ = 2+√3</text>
<text class="ts" x="564" y="296" dominant-baseline="central" opacity="0.6">≈ 3.732</text>
<line x1="524" y1="330" x2="554" y2="330" stroke="#A32D2D" stroke-width="2.5" stroke-linecap="round"/>
<text class="ts" x="564" y="330" dominant-baseline="central">d₆ = √6+√2</text>
<text class="ts" x="564" y="346" dominant-baseline="central" opacity="0.6">≈ 3.864</text>
</svg>
```

---

### Kapitel 6 — Flächenverhältnisse im Zwölfeck

**Titel in der Liste**: Das Zwölfeck — Flächen und Verhältnisse

**Inhalt**:

**Die Fläche des Zwölfecks**

Ein regelmässiges Zwölfeck mit Seitenlänge s hat die Fläche A = 3s²(2+√3). Die Herleitung ist anschaulich: man zerlegt das Zwölfeck vom Mittelpunkt aus in 12 gleichschenklige Dreiecke, berechnet die Fläche eines einzelnen Dreiecks und multipliziert mit 12.

Bei s = 1 ergibt das A ≈ 11.196 (dezimal). Zum Vergleich: der Umkreis hat die Fläche πR² ≈ 11.725 (dezimal). Das Zwölfeck füllt seinen Umkreis also zu etwas mehr als 95% — deutlich besser als ein Sechseck (83%) und weit besser als ein Quadrat (64%) oder ein Dreieck (41%).

**3/π — ein elegantes Verhältnis**

Das Verhältnis der Zwölfeck-Fläche zur Umkreis-Fläche vereinfacht sich zu einem überraschend einfachen Ausdruck: A₁₂/A_Kreis = 3/π. Die Herleitung nutzt eine Identität für sin²(15°): weil sin²(15°) = (2−√3)/4 ist, kürzt sich im Flächenverhältnis das Produkt (2+√3)(2−√3) zu 1, und es bleibt genau 3/π übrig.

3/π ≈ 0.9549 (dezimal) — das Zwölfeck erfasst also 95.5% der Kreisfläche. Diese Zahl lässt sich im Rechner sofort verifizieren: tippe `3 / π =` und vergleiche das Ergebnis mit der Tabelle unten.

**Vier Vielecke im Vergleich**

Alle folgenden Figuren teilen denselben Umkreis. Die allgemeine Formel für die Fläche eines regulären n-Ecks im Umkreis mit Radius R ist A = (n/2)·R²·sin(2π/n).

| Figur | Fläche / Kreisfläche | Anteil |
|---|---|---|
| Gleichseitiges Dreieck | 3√3/(4π) | 41.3% |
| Quadrat | 2/π | 63.7% |
| Regelmässiges Sechseck | 3√3/(2π) | 82.7% |
| Regelmässiges Zwölfeck | 3/π | 95.5% |

Zwei Beobachtungen fallen auf: das Sechseck hat exakt die doppelte Fläche des Dreiecks (beide enthalten den Faktor 3√3, beim Sechseck mit 2 im Zähler statt 4 im Nenner). Und jeder Schritt — Dreieck, Quadrat, Sechseck, Zwölfeck — bringt einen grösseren Flächenzuwachs relativ zum vorherigen, weil die zusätzlichen Ecken den Kreis immer enger umschliessen.

**Archimedes und die Kreiszahl**

Historisch war die Vieleck-Approximation der erste Weg zur Berechnung von π. Archimedes von Syrakus (ca. 287–212 v. Chr.) verwendete ein 96-Eck — das ist kein willkürlicher Wert, sondern 96 = 12 × 8 = 12 × 2³. Archimedes begann mit dem Sechseck (das sich trivial konstruieren lässt), verdoppelte die Eckenzahl dreimal (6 → 12 → 24 → 48 → 96) und berechnete so, dass 3 + 10/71 < π < 3 + 1/7. Der Ausgangspunkt seiner Methode war also das Zwölfeck — nach nur einer Verdoppelung des Sechsecks.

Ein 96-Eck füllt den Umkreis zu 99.93%. Von den 95.5% des Zwölfecks zu den 99.93% des 96-Ecks sind es nur drei Verdoppelungsschritte — ein bemerkenswertes Tempo der Konvergenz.

**Schwierigkeit**: Mittel. ✅ Fliesstext fertig.

---

### Kapitel 7 — Der Pentagondodekaeder (Einführung)

**Titel in der Liste**: Der Dodekaeder — zwölf Fünfecke im Raum

**Inhalt**:

**Zwölf Flächen**

Der Dodekaeder ist ein Körper aus zwölf regelmässigen Fünfecken. Jede Fläche ist identisch, jede Kante gleich lang, und an jeder Ecke treffen genau drei Fünfecke zusammen. Insgesamt hat er 12 Flächen, 30 Kanten und 20 Ecken. Der Name kommt vom griechischen δώδεκα (dōdeka, zwölf) und ἕδρα (hedra, Sitzfläche) — wörtlich: Zwölfflächner.

Er ist einer der fünf *platonischen Körper* — die einzigen konvexen Körper, deren Flächen ausschliesslich aus identischen regelmässigen Vielecken bestehen und bei denen an jeder Ecke dieselbe Anzahl Flächen zusammentrifft. Es gibt genau fünf solche Körper, und es kann keine weiteren geben:

| Körper | Flächen | Kanten | Ecken | Flächenform |
|---|---|---|---|---|
| Tetraeder | 4 | 6 | 4 | Dreiecke |
| Hexaeder (Würfel) | 6 | 12 | 8 | Quadrate |
| Oktaeder | 8 | 12 | 6 | Dreiecke |
| Dodekaeder | 12 | 30 | 20 | Fünfecke |
| Ikosaeder | 20 | 30 | 12 | Dreiecke |

Der Dodekaeder ist der einzige platonische Körper mit fünfeckigen Flächen — und derjenige mit der höchsten Flächenzahl unter denen, die keine Dreiecke verwenden.

**Wie sieht er aus?**

Stell dir einen Ball vor, der aus zwölf Lederstücken zusammengenäht ist — aber anders als ein Fussball. Ein Fussball besteht aus Fünfecken *und* Sechsecken (genauer: 12 Fünfecke und 20 Sechsecke, ein sogenanntes abgestumpftes Ikosaeder). Der Dodekaeder besteht dagegen ausschliesslich aus Fünfecken, und er ist nicht ganz rund — er hat spürbare Kanten und Ecken, fast wie ein geschliffener Edelstein.

Wer Rollenspiele spielt, kennt ihn als den D12 — den zwölfseitigen Würfel. Er liegt angenehm in der Hand, rollt gut und kommt zuverlässig auf einer Fläche zum Liegen, weil sein Schwerpunkt bei jeder Ruhelage stabil ist.

**Begegnungen mit dem Dodekaeder**

Neben dem Rollenspielwürfel gibt es eine historische Kuriosität: die *gallorömischen Pentagondodekaeder*. Über hundert dieser kleinen Bronzeobjekte wurden in Nordeuropa gefunden, datiert auf das 2. bis 4. Jahrhundert n. Chr. Sie haben zwölf fünfeckige Flächen mit unterschiedlich grossen runden Löchern darin, und niemand weiss mit Sicherheit, wofür sie verwendet wurden. Hypothesen reichen von Kerzenhaltern über Vermessungsinstrumenten bis zu religiösen Gegenständen. Das Rätsel ist bis heute ungelöst.

**Eulers Polyedersatz**

Für jeden konvexen Polyeder gilt eine verblüffend einfache Beziehung: Ecken minus Kanten plus Flächen ist immer gleich zwei. Leonhard Euler formulierte dieses Gesetz 1758. Für den Dodekaeder: 20 − 30 + 12 = 2. Diese Formel verbindet die drei Grundgrössen eines jeden Körpers in einer einzigen Gleichung — und sie gilt für alle fünf platonischen Körper, für jedes Prisma, für jede Pyramide, für jeden konvexen Körper überhaupt.

**Schwierigkeit**: Gering. ✅ Fliesstext fertig.

---

### Kapitel 8 — Mathematik des Dodekaeders

**Titel in der Liste**: Der Dodekaeder — φ, Dualität und Symmetrie

**Inhalt**:

**Der Goldene Schnitt im Dodekaeder**

Jede Fläche des Dodekaeders ist ein regelmässiges Fünfeck — und das regelmässige Fünfeck ist die Heimat des Goldenen Schnitts. Die Diagonale eines solchen Fünfecks verhält sich zu seiner Seite exakt wie φ = (1+√5)/2 ≈ 1.618 (dezimal) zu 1. Diese Proportion durchdringt den gesamten Körper: der Umkugelradius ist R = a·√3·φ/2 (bei Kantenlänge a), und sogar der Diederwinkel — der Winkel, in dem zwei benachbarte Fünfecke aufeinandertreffen — hat den exakten Wert 2·arctan(φ) ≈ 116.57° (dezimal).

Wer im Rechner `φ² =` tippt, erhält `φ + 1`. Das ist die definierende Eigenschaft des Goldenen Schnitts: φ ist die einzige positive Zahl, deren Quadrat genau um 1 grösser ist als sie selbst. Diese Identität macht φ zu einer algebraisch einzigartigen Konstante, und sie ist der Grund, warum φ in so vielen Formeln des Dodekaeders erscheint — die Geometrie des regelmässigen Fünfecks erzwingt es.

**Volumen und Oberfläche**

Bei Kantenlänge a = 1:

| Grösse | Formel | Dezimalwert |
|---|---|---|
| Volumen | (15+7√5)/4 | ≈ 7.663 |
| Oberfläche | 3√(25+10√5) | ≈ 20.646 |
| Umkugelradius | √3·φ/2 | ≈ 1.401 |
| Inkugelradius | √(25+11√5)/(2√10) | ≈ 1.114 |

Beide Radien und das Volumen enthalten √5 — nicht direkt als φ geschrieben, aber algebraisch äquivalent, weil φ = (1+√5)/2 ist. Man kann jede dieser Formeln in eine reine φ-Darstellung umschreiben, und viele Lehrbücher tun das. Der Rechner kann alle Werte verifizieren, da sowohl φ als auch √2 als Konstanten verfügbar sind.

**Dualität — der Spiegel des Ikosaeders**

Zu jedem platonischen Körper gibt es einen *dualen* Körper: man ersetzt jede Fläche durch eine Ecke (im Mittelpunkt der Fläche) und verbindet benachbarte neue Ecken mit Kanten. Beim Dodekaeder entsteht so das Ikosaeder — und umgekehrt. Die Zahlen spiegeln sich:

| | Dodekaeder | Ikosaeder |
|---|---|---|
| Flächen | 12 | 20 |
| Kanten | 30 | 30 |
| Ecken | 20 | 12 |

Flächen und Ecken tauschen die Plätze, die Kantenzahl bleibt gleich. Die 12 erscheint in beiden Körpern — einmal als Flächenzahl, einmal als Eckenzahl. Und beide Körper teilen dieselbe Symmetriegruppe, weil die Dualität die Symmetrien erhält.

**120 Symmetrien**

Der Dodekaeder (und sein Dual, das Ikosaeder) besitzt die reichste Symmetrie unter allen platonischen Körpern: die Ikosaedergruppe I_h mit 120 Elementen — 60 Drehungen und 60 Dreh-Spiegelungen. Zum Vergleich: der Würfel hat nur 48 Symmetrien, das Tetraeder 24.

Die Zahl 120 = 5! hat selbst eine elegante Faktorisierung: 120 = 2³×3×5. Die drei Primfaktoren sind 2, 3 und 5 — exakt die drei Zahlen, die in den Flächen des Dodekaeders (Fünfecke) und in der Teilbarkeit von 12 (= 2²×3) zusammenkommen. In der Ecke, wo sich drei Fünfecke treffen, begegnen sich die Zwei, die Drei und die Fünf — und ihre Symmetriegruppe spiegelt genau das wider.

**Schwierigkeit**: Mittel. ✅ Fliesstext fertig.

---

### Kapitel 9 — Die Zwölf am Himmel

**Titel in der Liste**: Zwölf Tierkreiszeichen und der Himmel

**Inhalt**:

**360 Grad und die Babylonier**

Dass ein Vollkreis 360 Grad hat, ist keine Naturkonstante — es ist eine menschliche Festlegung, und sie geht auf die Babylonier zurück. Die babylonische Mathematik verwendete die Basis 60 (das Sexagesimalsystem), und 360 = 6 × 60. Aber 360 lässt sich auch als 12 × 30 schreiben, und genau so teilten die Babylonier den Himmel auf: die scheinbare Sonnenbahn (die Ekliptik) wurde in 12 gleiche Abschnitte zu je 30° zerlegt. Jedem Abschnitt wurde ein Sternbild zugeordnet — die zwölf Tierkreiszeichen.

Die Wahl von 12 war kein Zufall. Die Babylonier organisierten ihre 60er-Basis intern in Gruppen von 12, weil 60 = 12 × 5. Die Zwölf war für sie eine natürliche Untereinheit, die sich ständig wiederholte: in der Zeitmessung, im Kalender, in der Astronomie.

**Der Mond und die Zwölf**

Warum gerade zwölf Abschnitte am Himmel? Weil die Natur selbst eine Zwölfteilung nahelegt: ein Sonnenjahr enthält fast genau 12 Mondzyklen. Ein synodischer Monat (von Neumond zu Neumond) dauert etwa 29.53 Tage (dezimal). 12 Mondzyklen ergeben 354.4 Tage — nur 11 Tage weniger als ein Sonnenjahr von 365.24 Tagen. Diese Beinahe-Übereinstimmung machte die Zwölf zur offensichtlichen Einteilung des Jahres.

Die Abweichung von 11 Tagen war den Babyloniern durchaus bekannt. Ihr Kalender verwendete Schaltmonate, um das Mondjahr mit dem Sonnenjahr zu synchronisieren — ein Problem, das Kalender bis heute beschäftigt (der islamische Kalender verzichtet auf den Ausgleich und wandert deshalb durch die Jahreszeiten).

**Ordnung am Himmel**

Die Zwölfteilung des Himmels war für die alten Kulturen weit mehr als ein Koordinatensystem. In der regelmässigen Wiederkehr der Sternbilder, im Rhythmus der Planeten, in der stillen Ordnung des Nachthimmels sahen sie einen Ausdruck kosmischer Weisheit — ein Muster, das grösser war als der Mensch und das dennoch lesbar schien. Die Zwölf als Ordnungszahl des Himmels verband das Beobachtbare mit dem Bedeutsamen.

Ein faszinierendes astronomisches Detail: der Frühlingspunkt — der Ort am Himmel, an dem die Sonne zu Frühlingsbeginn steht — wandert langsam durch die Sternbilder, weil die Erdachse wie ein Kreisel taumelt (die sogenannte Präzession, mit einer Periode von etwa 25'800 Jahren). Die Sternbilder, durch die der Frühlingspunkt im Laufe der Jahrtausende wandert, heissen deshalb auch *Zeitalter* — das „Zeitalter der Fische", das „Zeitalter des Wassermanns". Auch hier strukturiert die Zwölf die Zeit: zwölf Sternbilder, zwölf Zeitalter, ein grosser Kreis.

**Die Zwölf anderswo am Himmel**

Die Babylonier waren nicht die einzigen, die den Himmel in Zwölfer teilten. Die alten Ägypter teilten Tag und Nacht in je 12 Stunden — daher unsere 24-Stunden-Einteilung. Die Stundenlänge variierte allerdings mit den Jahreszeiten (eine Sommerstunde am Tag war länger als eine Winterstunde), weil die Ägypter die lichte und die dunkle Zeit jeweils in genau 12 gleiche Teile zerlegten.

Auch die chinesische Tradition kennt zwölf Tierkreiszeichen — Ratte, Büffel, Tiger, Hase, Drache, Schlange, Pferd, Ziege, Affe, Hahn, Hund, Schwein. Die Systeme sind unabhängig voneinander entstanden und beziehen sich auf verschiedene Zyklen (das chinesische System ist ein 12-Jahres-Zyklus, nicht ein 12-Monats-Zyklus), aber die Konvergenz auf die Zahl 12 ist bemerkenswert.

**Schwierigkeit**: Mittel. ✅ Fliesstext fertig.

---

### Kapitel 10 — Der Dodekaeder in der Natur

**Titel in der Liste**: Zwölf Flächen in Kristallen und Lebewesen

**Inhalt**:

**Pyrit — das Narren-Dodekaeder**

Pyrit (FeS₂), wegen seines goldenen Glanzes auch als „Narrengold" bekannt, kristallisiert häufig in einer Form, die dem platonischen Dodekaeder zum Verwechseln ähnlich sieht: der *Pyritoeder*. Er hat zwölf fünfeckige Flächen, 20 Ecken und 30 Kanten — dieselbe Topologie wie der reguläre Dodekaeder aus Kapitel 7. Aber bei genauem Hinsehen zeigt sich ein entscheidender Unterschied: die Fünfecke des Pyritoeders sind nicht regelmässig. Ihre Winkel und Seitenlängen variieren leicht, und die scheinbare fünfzählige Symmetrie ist eine optische Täuschung. In der Kristallographie ist echte fünfzählige Drehsymmetrie bei periodischen Kristallen unmöglich — nur die Symmetrien der Ordnung 1, 2, 3, 4 und 6 sind erlaubt. Der Pyritoeder schummelt sich mit unregelmässigen Fünfecken an dieser Regel vorbei.

Trotz dieser Unregelmässigkeit bleibt die Zwölf: zwölf Flächen, auf den ersten Blick fünfeckig, auf einem Kristall, der seit Jahrtausenden fasziniert. Römische Pentagondodekaeder aus Bronze, deren Funktion bis heute unbekannt ist (siehe Kapitel 7), könnten von Pyritkristallen inspiriert worden sein — Belege dafür gibt es allerdings nicht.

**Granat — ein anderer Zwölfflächner**

Die Minerale der Granat-Gruppe kristallisieren bevorzugt als *Rhombendodekaeder* — ebenfalls ein Körper mit zwölf Flächen, aber ganz anderer Natur. Die Flächen sind keine Fünfecke, sondern Rauten (Rhomben). Der Rhombendodekaeder hat 14 Ecken und 24 Kanten und gehört zum kubischen Kristallsystem mit oktaedrischer Symmetrie. Die Kristallographen der Universität Waterloo bezeichnen ihn als das „Markenzeichen" der Granate — keine andere Mineralgruppe ist so eng mit einer einzelnen Kristallform verbunden.

Hier lohnt ein Moment des Staunens: die Natur verwendet die Zahl 12 als Flächenzahl für zwei völlig verschiedene Kristallformen — Fünfecke beim Pyrit, Rauten beim Granat. Die Zwölf ist nicht an eine bestimmte Geometrie gebunden, sondern taucht in unterschiedlichsten Arrangements auf.

**Radiolarien — Skelette aus Glas**

Radiolarien sind einzellige Meeresorganismen, kaum grösser als ein Zehntel Millimeter, die filigrane Skelette aus Siliziumdioxid (Kieselsäure) bilden. Einige Arten formen Skelette mit ikosaedrischer Symmetrie — also der Symmetrie des Ikosaeders, des Duals zum Dodekaeder. Zwölf Ecken, zwölf Fünfeck-Anordnungen, die in der 3D-Struktur wiederkehren.

Der deutsche Biologe Ernst Haeckel zeichnete diese Organismen 1904 in seinem Werk „Kunstformen der Natur" mit einer Detailtreue und einem ästhetischen Anspruch, die bis heute beeindrucken. Seine Tafeln zeigen Radiolarien, deren geometrische Perfektion kaum zu glauben ist — als hätte die Evolution selbst Mathematik studiert.

**Quasikristalle — die Ausnahme, die die Regel bestätigt**

Wir haben oben erwähnt, dass echte fünfzählige Symmetrie in periodischen Kristallen unmöglich ist. 1982 entdeckte der israelische Materialwissenschaftler Dan Shechtman in einer Aluminium-Mangan-Legierung ein Muster, das genau diese verbotene Symmetrie zeigte — aber nicht periodisch war. Die Fachwelt reagierte zunächst mit Ablehnung; Shechtman wurde für seine Behauptung scharf kritisiert. Doch die Beobachtung hielt stand, und 2011 erhielt er den Nobelpreis für Chemie.

Die Symmetrie dieser sogenannten Quasikristalle ist ikosaedrisch — sie enthält die Zwölf sowohl als Flächen- als auch als Eckenzahl (über die Dualität von Dodekaeder und Ikosaeder). Es gibt sogar natürlich vorkommende Quasikristalle: das Mineral Icosahedrit wurde 2009 in einem Meteoriten im Korjakischen Gebirge in Kamtschatka entdeckt. Die Zwölf, die in der klassischen Kristallographie als unmöglich galt, fand auf einem Umweg doch ihren Platz in der Natur.

**Schwierigkeit**: Hoch. ✅ Fliesstext fertig.

---

### Kapitel 11 — Die Zwölf in der Biologie

**Titel in der Liste**: Zwölf Glieder an der Hand

**Inhalt**:

**Zwölf an einer Hand**

Halte eine Hand vor dich, den Daumen abgespreizt, und betrachte die vier Finger. Jeder Finger hat drei Glieder (Phalangen), getrennt durch sichtbare Gelenke. Vier Finger mal drei Glieder — das sind zwölf. Der Daumen kann als Zeiger dienen: er berührt nacheinander jedes Glied der vier Finger und zählt so von eins bis zwölf.

Diese Methode ist keine moderne Erfindung. In Teilen Südostasiens, Indiens und des Nahen Ostens wird sie seit Jahrhunderten verwendet, und es gibt Hinweise, dass sie sehr viel älter ist. Sie ist schnell, benötigt keine Hilfsmittel, und hat einen entscheidenden Vorteil gegenüber dem westlichen Fingerzählen (bei dem jeder Finger eine Einheit darstellt und man nur bis fünf oder zehn kommt): sie nutzt eine Hand für zwölf Einheiten statt für fünf.

**Von zwölf zu sechzig**

Die zweite Hand zählt die vollen Durchgänge. Jedes Mal, wenn die erste Hand eine Runde von zwölf vollendet hat, streckt die zweite Hand einen Finger aus. Fünf Finger mal zwölf — das ergibt sechzig. Mit zwei Händen kann man also bis 60 zählen, und das ohne jedes Hilfsmittel.

Diese Verbindung von 12 und 60 ist vermutlich kein Zufall: das babylonische Sexagesimalsystem (Basis 60) könnte seinen Ursprung in genau dieser Zählmethode haben. 60 = 12 × 5 = 5 × 4 × 3 — es vereint die Zwölf der Fingerglieder mit der Fünf der Finger. Eine elegante Verschmelzung von Anatomie und Arithmetik.

**Weitere Zwölfer im menschlichen Körper**

Die Hand ist nicht der einzige Ort, an dem die Zwölf in der Anatomie auftaucht:

Der Mensch hat in der Regel 12 Rippenpaare — sieben „echte" Rippen, die direkt am Brustbein ansetzen, drei „falsche", die indirekt verbunden sind, und zwei „freie", die frei enden. (Anatomische Variationen kommen vor: manche Menschen haben 11 oder 13 Paare.)

Die klassische Anatomie kennt 12 Hirnnervenpaare, nummeriert mit römischen Ziffern von I (Nervus olfactorius, der Riechnerv) bis XII (Nervus hypoglossus, der Zungenmuskelnerv). Diese Einteilung geht auf den griechischen Arzt Galen (2. Jahrhundert n. Chr.) zurück und wurde von Thomas Willis im 17. Jahrhundert in die heute gebräuchliche Form gebracht.

**Hat die Hand das Zahlensystem geformt?**

Die Hypothese liegt nahe: wenn die Menschen im Nahen Osten seit Jahrtausenden an den Fingergliedern bis zwölf zählen, hat diese Praxis möglicherweise die Entstehung von Zwölfer-Systemen beeinflusst — den 12-Stunden-Tag, die 12 Monate, das Dutzend. Der Kausalzusammenhang ist plausibel, aber nicht bewiesen. Es könnte auch umgekehrt sein: die Menschen begannen an den Fingergliedern zu zählen, *weil* die Zwölf in ihrer Kultur bereits eine wichtige Zahl war (durch die Mondzahl am Himmel, durch die Teilbarkeit). Wahrscheinlich haben sich beide Richtungen gegenseitig verstärkt.

Was sicher ist: die menschliche Hand bietet eine natürliche physische Grundlage für die Zwölf, und das ist bemerkenswert. Die Zehn, die unsere heutige Zahlenwelt dominiert, stammt ebenfalls von den Händen — aber von einer weniger raffinierten Zähltechnik, die nur die Finger zählt und nicht die Glieder.

**Schwierigkeit**: Gering bis mittel. ✅ Fliesstext fertig.

---

### Kapitel 12 — Die Zwölf im Messwesen

**Titel in der Liste**: Zoll, Fuss, Pfund — und warum sie dozenal Sinn ergeben

**Inhalt**:

**Zwölfer im Alltag**

Bevor das metrische System sich durchsetzte, war die Zwölf eine Selbstverständlichkeit im Messwesen. Ein Fuss hat 12 Zoll. Ein Troy-Pfund (für Edelmetalle) hat 12 Unzen. Ein Schilling hatte 12 Pence (im britischen Münzsystem vor 1971). Ein Dutzend sind 12 Stück, ein Gros 144 = 12² Stück. Diese Einteilungen sind keine historischen Zufälle — sie wurden gewählt, weil sie das Teilen erleichtern.

Ein Fuss lässt sich in zwei gleiche Teile teilen (je 6 Zoll), in drei (je 4 Zoll), in vier (je 3 Zoll) und in sechs (je 2 Zoll). Jede dieser Teilungen geht exakt auf. Ein Meter dagegen lässt sich in zwei gleiche Teile teilen (je 50 cm) und in fünf (je 20 cm), aber ein Drittel Meter ist 33.333… cm (dezimal) — ein unendlicher Bruch, sobald man es exakt nehmen will. Im Handwerk, wo ständig gedrittelt und geviertelt wird, ist die Zwölf praktischer als die Zehn.

**Das metrische System — und sein blinder Fleck**

Das metrische System hat grosse Stärken: es ist kohärent (alle Einheiten passen zusammen), es skaliert dezimal (Kilo, Mega, Milli, Mikro), und es ist weltweit standardisiert. Diese Vorzüge sind real und gewichtig. Kein vernünftiger Mensch würde vorschlagen, SI abzuschaffen.

Aber das metrische System erbt die Schwäche seiner Basis. In Basis 10 ist ein Drittel ein unendlicher Bruch: 0.333… Und ein Sechstel auch: 0.166… Das sind keine seltenen Operationen — im Ingenieurwesen, in der Küche, auf der Baustelle wird ständig durch drei und sechs geteilt. Jedes Mal entsteht eine Rundung, und Rundungen akkumulieren sich.

In einem dozenalen metrischen System wäre das kein Problem. Ein Drittel wäre 0.4 — exakt, kurz, ohne Restfehler. Ein Sechstel wäre 0.2. Ein Viertel wäre 0.3. Die Eleganz des metrischen Prinzips (kohärente Einheiten, systematische Präfixe) bliebe erhalten — nur die Basis wäre besser.

**Tom Pendleburys TGM**

Genau diesen Gedanken hat der Engländer Tom Pendlebury, Mitglied der Dozenal Society of Great Britain, konsequent zu Ende gedacht. Sein System heisst TGM — benannt nach seinen drei Grundeinheiten Tim, Grafut und Maz (Zeit, Länge, Masse).

Pendleburys Ansatz war ungewöhnlich: er begann nicht mit der Länge (wie das metrische System mit dem Meter), sondern mit der Zeit. Die Stunde ist bereits dozenal in unser Leben eingebaut — ein Zifferblatt hat 12 Markierungen, eine Viertelstunde steht bei der 3, eine halbe bei der 6. Pendlebury teilte die Stunde in 12⁴ gleiche Teile und nannte die kleinste Einheit einen Tim (etwa 0.174 Sekunden, dezimal). Aus dem Tim leitete er über die Erdbeschleunigung die Längeneinheit ab: ein Grafut (etwa 29.6 cm, dezimal, also knapp ein Fuss) ist die Strecke, die ein frei fallender Gegenstand in einem Tim-Quadrat zurücklegt. Die Masseeinheit Maz ergab sich dann aus dem Volumen eines Kubik-Grafut Wasser.

Das Ergebnis ist ein vollständig kohärentes Einheitensystem, in dem alle Umrechnungen in Potenzen von 12 erfolgen — genau wie im metrischen System alles in Potenzen von 10 skaliert, nur mit besserer Bruchfreundlichkeit. TGM wurde nie über Enthusiastenkreise hinaus angenommen, aber es demonstriert, dass ein dozenales Metriksystem nicht nur möglich, sondern in mancher Hinsicht dem dezimalen überlegen wäre.

**Was dieser Rechner zeigt**

Die Frage „Dozenal oder Dezimal?" wird in der Praxis vermutlich nie entschieden werden — dafür ist das Dezimalsystem zu tief in unserer Zivilisation verwurzelt. Aber die mathematischen Vorteile der Basis 12 sind objektiv und messbar, und dieser Rechner macht sie erlebbar. Wer `1 / 3 =` tippt und `0.4` sieht — kurz, exakt, ohne Periodenstrich — versteht in einer Sekunde, was Seiten voller Argumente nicht vermitteln können.

**Schwierigkeit**: Mittel. ✅ Fliesstext fertig.

---

## Anmerkungen für die Implementierung

- Die Kapiteltexte werden einzeln ausgearbeitet und als finaler Fliesstext in diese Datei eingefügt. Bis dahin dienen die Stichpunkte als Platzhalter.
- Illustrationen (SVG-Zeichnungen des 12-Ecks, des Dodekaeders, der Fingerglieder-Zählmethode) sind wünschenswert, aber optional für v1. Die Texte müssen auch ohne Illustrationen funktionieren.
- Mathematische Formeln werden als Unicode-Text dargestellt (z.B. `A = 3s²(2+√3)`), nicht als LaTeX. Der Rechner hat keinen Formelrenderer.
- Dozenale Zahldarstellungen verwenden im Info-Text die alphanumerische Konvention (A = zehn, B = elf), nicht die Rechnersymbole. Das Info ist ein Lesekontext.
- Jedes Kapitel sollte in 2–3 Bildschirmhöhen auf einem Mobilgerät passen (ca. 400–600 Wörter). Längere Kapitel werden gekürzt, nicht umgebrochen.
