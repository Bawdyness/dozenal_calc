// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Info-modal content. The 12 chapters of dozenal-system context that the
//! calculator surfaces under the `Info` overlay key. Pure content + structure;
//! visual primitives (`info_h`, `info_p`, SVGs, digit legend) live in `painting.rs`.

use crate::painting::{
    draw_chapter4_svg, draw_chapter5_svg, draw_digit_legend, info_h, info_p, info_pre,
};
use eframe::egui;

pub const INFO_TITLES: [&str; 12] = [
    "Bedienung des Rechners",
    "Was ist das Dozenalsystem?",
    "Fibonacci, Quadratzahlen und Kuriositäten",
    "Das Zwölfeck — Grundlagen",
    "Das Zwölfeck — Winkel und Diagonalen",
    "Das Zwölfeck — Flächen und Verhältnisse",
    "Der Dodekaeder — zwölf Fünfecke im Raum",
    "Der Dodekaeder — φ, Dualität und Symmetrie",
    "Zwölf Tierkreiszeichen und der Himmel",
    "Zwölf Flächen in Kristallen und Lebewesen",
    "Zwölf Glieder an der Hand",
    "Zoll, Fuss, Pfund — und warum sie dozenal Sinn ergeben",
];

pub fn draw_info_chapter(ui: &mut egui::Ui, chapter: usize) {
    match chapter {
        0 => {
            info_h(ui, "Die Ziffern");
            info_p(
                ui,
                "Dieser Rechner verwendet eigene Symbole für alle zwölf Ziffern. Vier Ankerziffern sind stilisierte Pfeilspitzen, die in die vier Himmelsrichtungen zeigen — 1 (oben), 4 (links), 7 (rechts), A (unten). Sie teilen den Zahlenkreis in vier Dreiergruppen, wie die Stunden 12, 3, 6 und 9 auf einem Zifferblatt.",
            );
            info_p(
                ui,
                "Alle Ziffern dazwischen bestehen aus Halbkreisen und Vollkreisen. Die Null ist ein einfacher Kreis, B (= elf) ein gefüllter Kreis.",
            );
            draw_digit_legend(ui);

            info_h(ui, "Grundbedienung");
            info_p(
                ui,
                "Tippe Zahlen und Operatoren wie auf einem gewöhnlichen Taschenrechner. Drücke die breite Taste am unteren Rand, um das Ergebnis zu berechnen. AC löscht die gesamte Eingabe und das Ergebnis, Del entfernt das Zeichen links vom Cursor.",
            );

            info_h(ui, "Cursor und Navigation");
            info_p(
                ui,
                "Der rote Strich im Eingabefeld ist der Cursor. Mit ◀ und ▶ bewegst du ihn, um mitten in einer Formel Zeichen einzufügen oder zu löschen. Nach einer Berechnung wandert der Cursor ins Ergebnisfeld — die Pfeile bewegen dann den Ergebnis-Cursor. Sobald du eine neue Eingabe machst, springt der Cursor zurück ins Eingabefeld.",
            );

            info_h(ui, "Weiterrechnen");
            info_p(
                ui,
                "Nach einer Berechnung kannst du direkt mit einem Operator weitermachen. Tippst du zum Beispiel + 5 =, verwendet der Rechner automatisch das letzte Ergebnis als ersten Operanden. Wenn du stattdessen eine ganz neue Rechnung beginnen willst, drücke zuerst AC.",
            );

            info_h(ui, "Doppelklick für Umkehrfunktionen");
            info_p(
                ui,
                "Ein zweiter Klick auf eine Funktionstaste wandelt sie in ihre Umkehrfunktion um: sin wird zu sin⁻¹, cos zu cos⁻¹, und so weiter. Das gilt auch für die hyperbolischen Funktionen im Erweiterungsfeld. Ein kleiner goldener Punkt auf der Taste zeigt an, dass der nächste Klick umkehrt.",
            );

            info_h(ui, "Spezialoperatoren");
            info_p(
                ui,
                "x² quadriert die vorangehende Zahl. √ berechnet die Quadratwurzel — steht links davon eine Zahl, wird diese als Wurzelgrad verwendet: 3√27 ergibt die dritte Wurzel von 27. log berechnet den Logarithmus zur Basis der vorangehenden Zahl. ⊕ berechnet die Paralleladdition: a ⊕ b = (a·b)/(a+b), nützlich für Parallelschaltungen von Widerständen.",
            );

            info_h(ui, "Erweiterungsfeld");
            info_p(
                ui,
                "Die Taste … rechts unten öffnet das Erweiterungsfeld mit weiteren Funktionen: Speicher, Konstanten (π, e, φ, √2), hyperbolische Funktionen, erweiterte Operatoren und Einstellungen. Es schliesst sich über die Taste rechts unten im Erweiterungsfeld selbst.",
            );
            info_pre(
                ui,
                "  6 — Speicher:    STO   RCL   MC    Ans\n  7 — Konstanten:  π     e     φ     √2\n  8 — Hyperbel:   sinh  cosh  tanh  coth\n  9 — Erweitert:  n!    |x|   1/x   mod\n  10 — Modi:      Doz↔  DRG   Info  ×",
            );

            info_h(ui, "Speicher");
            info_p(
                ui,
                "STO speichert das aktuelle Ergebnis, RCL fügt den gespeicherten Wert in die Eingabe ein, MC löscht den Speicher. Ein kleines M im Display zeigt an, dass etwas gespeichert ist. Ans fügt das Ergebnis der letzten Berechnung ein — exakte rationale Werte werden vollständig mitgespeichert, Periodizität bleibt erhalten.",
            );

            info_h(ui, "Periodenstrich");
            info_p(
                ui,
                "Wenn das Ergebnis ein periodischer Bruch ist, zeigt der Rechner die sich wiederholenden Ziffern mit einem Strich darüber an. Beispiel: 1/5 ergibt 0.2497 mit Strich über allen vier Ziffern. Bei Perioden mit mehr als fünf Stellen werden nur die ersten fünf gezeigt, gefolgt von …",
            );

            info_h(ui, "Anzeige und Winkelmodus");
            info_p(
                ui,
                "Doz↔Dec im Erweiterungsfeld schaltet die Anzeige zwischen dozenal (Basis 12) und dezimal (Basis 10) um — praktisch, um ein Ergebnis in vertrauter Schreibweise zu überprüfen. DRG wechselt den Winkelmodus für trigonometrische Funktionen: Rad → Grad → Gon → Rad, angezeigt oben rechts im Display.",
            );
        }
        1 => {
            info_h(ui, "Das Prinzip");
            info_p(
                ui,
                "Im Dezimalsystem hat jede Stelle den zehnfachen Wert der Stelle rechts davon: Einer, Zehner, Hunderter. Im Dozenalsystem ist die Basis nicht zehn, sondern zwölf. Die Stellenwerte sind Potenzen von 12: Einer, Zwölfer, Hundertvierundvierziger. Die Zahl »100« bedeutet hier nicht zehn mal zehn, sondern zwölf mal zwölf — also 144 im Dezimalen.",
            );
            info_p(
                ui,
                "Dafür braucht man zwölf Ziffern statt zehn. Für die Werte zehn und elf kommen zwei neue hinzu, die dieser Rechner mit eigenen Symbolen darstellt (A = zehn, B = elf). Beispiel: 2B (dozenal) = 2·12 + 11 = 35 (dezimal).",
            );

            info_h(ui, "Warum gerade zwölf?");
            info_p(
                ui,
                "Der Grund ist Teilbarkeit. Zwölf hat sechs Teiler: 1, 2, 3, 4, 6 und 12. Zehn hat nur vier: 1, 2, 5 und 10. Das klingt nach einem kleinen Unterschied, aber die Auswirkung auf den Alltag ist erheblich — vor allem beim Bruchrechnen.",
            );

            info_h(ui, "Stammbrüche im Vergleich");
            info_pre(
                ui,
                "  Bruch   Basis 10    Basis 12\n  1/2     0.5         0.6\n  1/3     0.333…      0.4\n  1/4     0.25        0.3\n  1/5     0.2         0.2497…\n  1/6     0.166…      0.2\n  1/8     0.125       0.16\n  1/9     0.111…      0.14\n  1/10    0.1         0.1249…\n  1/12    0.0833…     0.1",
            );
            info_p(
                ui,
                "In Basis 10 sind Drittel und Sechstel unendliche Dezimalbrüche. In Basis 12 sind sie kurz und exakt. Dafür werden Fünftel und Zehntel periodisch — ein fairer Tausch, wenn man bedenkt, wie viel häufiger man durch drei und vier teilt als durch fünf.",
            );

            info_h(ui, "Die Regel dahinter");
            info_p(
                ui,
                "Welche Brüche endlich sind und welche periodisch werden, folgt einem einfachen Gesetz: ein Bruch 1/n hat in einer Basis b genau dann eine endliche Darstellung, wenn alle Primfaktoren von n auch Primfaktoren von b sind. Die Primfaktoren von 12 sind 2 und 3. Also ist jeder Bruch endlich, dessen Nenner nur aus Zweien und Dreien zusammengesetzt ist. Alles andere — Nenner mit einer 5, 7 oder 11 — wird periodisch. Der Rechner zeigt diese Periodizität mit einem Strich über den sich wiederholenden Ziffern an.",
            );

            info_h(ui, "Spuren in der Geschichte");
            info_p(
                ui,
                "Die Zwölf als Ordnungsgrösse ist älter als jedes Zahlensystem. Die Babylonier rechneten in Basis 60, aber organisierten ihre Ziffern in Gruppen von 12. Im Handel zählte man in Dutzenden (12) und Gros (144 = 12²). Der Tag hat 2×12 Stunden, das Jahr 12 Monate, der Vollkreis 360 = 30×12 Grad.",
            );
            info_p(
                ui,
                "Die Dozenal Society of America (gegründet 1944, heute mit weiteren Ablegern) setzt sich dafür ein, die Vorzüge der Basis 12 bekannter zu machen. Dieser Rechner steht in dieser Tradition — nicht als Forderung nach einer Systemumstellung, sondern als Werkzeug zum Erforschen und Staunen.",
            );
        }
        2 => {
            info_h(ui, "144 — wo sich zwei Welten treffen");
            info_p(
                ui,
                "Die Fibonacci-Folge beginnt mit 1, 1, und jede weitere Zahl ist die Summe der beiden vorangehenden: 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, … Sie wächst exponentiell. Die Quadratzahlen — 1, 4, 9, 16, 25, 36, … — wachsen dagegen nur quadratisch. Zwei so unterschiedliche Folgen haben fast keinen Grund, sich jemals zu treffen. Und doch tun sie es: die zwölfte Fibonacci-Zahl ist 144, und 144 = 12².",
            );
            info_p(
                ui,
                "J. H. E. Cohn bewies 1964, dass dies kein Zufall ist, sondern ein Unikat: abgesehen von F(1) = F(2) = 1 gibt es keine weitere Fibonacci-Zahl, die zugleich eine perfekte Quadratzahl ist. Die Zwölf steht an einer einmaligen Kreuzung zweier fundamentaler Zahlenfolgen.",
            );

            info_h(ui, "Der Goldene Schnitt");
            info_p(
                ui,
                "φ = (1+√5)/2 ≈ 1.618 ist der Grenzwert des Verhältnisses aufeinanderfolgender Fibonacci-Zahlen: F(n+1)/F(n) → φ. In Basis 12: φ ≈ 1.74BB677… — der Rechner hat φ als Konstante im Erweiterungsfeld.",
            );
            info_p(
                ui,
                "Wer φ² = tippt, wird sehen, dass das Ergebnis genau φ+1 ist — die definierende Eigenschaft des Goldenen Schnitts. Diese Identität macht φ zu einer algebraisch einzigartigen Konstante.",
            );

            info_h(ui, "12 = 2² × 3 — eine Primfaktorzerlegung mit Folgen");
            info_p(
                ui,
                "Zwölf ist eine hochzusammengesetzte Zahl (highly composite number): sie hat mehr Teiler als jede kleinere natürliche Zahl. Die Teiler von 12 sind 1, 2, 3, 4, 6, 12 — das sind sechs Stück. Srinivasa Ramanujan definierte und untersuchte diese Klasse von Zahlen in einer berühmten Arbeit von 1915 in den Proceedings of the London Mathematical Society.",
            );
            info_p(
                ui,
                "Zwölf ist auch die kleinste abundante Zahl: die Summe ihrer echten Teiler (1+2+3+4+6 = 16) übertrifft die Zahl selbst. Bei den meisten kleinen Zahlen ist es umgekehrt — bei 10 ergibt 1+2+5 = 8, was kleiner ist als 10. Zwölf ist die erste Zahl, bei der die Teiler »überquellen«.",
            );

            info_h(ui, "Platons ideale Stadt");
            info_p(
                ui,
                "In seinen »Gesetzen« (Buch V) stellt Platon die Frage, wie viele Bürger eine ideale Stadt haben sollte. Seine Antwort: 5040. Das Argument ist nicht mystisch, sondern praktisch: eine Stadt muss ihre Bürger ständig in gleich grosse Gruppen einteilen. 5040 ist durch jede Zahl von 1 bis 12 teilbar (mit der einzigen Ausnahme von 11).",
            );
            info_p(
                ui,
                "Was Platon intuitiv beschreibt, ist dieselbe Einsicht, die dem Dozenalsystem zugrunde liegt: im Alltag sind die kleinen Teiler die wichtigen. 5040 = 7! ist die grosse Schwester der Zwölf — dieselbe Teilbarkeitsphilosophie auf eine ganze Stadtbevölkerung angewendet.",
            );
        }
        3 => {
            info_h(ui, "Was ist ein regelmässiges Zwölfeck?");
            info_p(
                ui,
                "Ein regelmässiges Zwölfeck (Dodekagon) ist ein Vieleck mit zwölf gleich langen Seiten und zwölf gleich grossen Innenwinkeln. Jeder dieser Innenwinkel beträgt 150° — oder, im Dozenalsystem ausgedrückt, 106°. Es ist eine der ältesten und am häufigsten verwendeten geometrischen Formen: man findet sie in Zifferblättern, Münzen, Bauornamentik und Pflastermustern.",
            );

            info_h(ui, "Das Schweizer Taschenmesser der Vielecke");
            info_p(
                ui,
                "Was das Zwölfeck einzigartig macht, ist nicht seine Form an sich, sondern was alles in ihm steckt. Verbindet man jede vierte Ecke, entsteht ein gleichseitiges Dreieck — exakt, nicht angenähert. Jede dritte Ecke ergibt ein Quadrat. Jede zweite Ecke ein regelmässiges Sechseck. Alle drei Figuren liegen perfekt im selben Kreis, der auch das Zwölfeck umschliesst.",
            );
            info_p(
                ui,
                "Das bedeutet: das Zwölfeck enthält die drei fundamentalen regulären Vielecke der Geometrie als exakte Teilfiguren. Kein anderes Vieleck mit so wenigen Ecken kann das von sich behaupten. Eine direkte Folge der Teilbarkeit von 12 durch 2, 3, 4 und 6.",
            );

            info_h(ui, "Konstruierbar mit Zirkel und Lineal");
            info_p(
                ui,
                "Nicht jedes regelmässige Vieleck lässt sich mit Zirkel und Lineal exakt konstruieren. Das Zwölfeck dagegen ist konstruierbar: man beginnt mit einem Kreis, teilt ihn in sechs gleiche Teile (das gelingt, weil das Sechseck konstruierbar ist), halbiert dann jeden dieser Bögen, und hat zwölf gleichmässig verteilte Punkte auf dem Kreis.",
            );
            info_p(
                ui,
                "Die mathematische Grundlage: ein reguläres n-Eck ist genau dann konstruierbar, wenn n ein Produkt einer Zweierpotenz und verschiedener Fermat-Primzahlen ist (Gauss, 1796). Für 12 = 2² × 3 ist das erfüllt, weil 3 eine Fermat-Primzahl ist.",
            );

            info_h(ui, "Symmetrie");
            info_p(
                ui,
                "Das regelmässige Zwölfeck hat 24 Symmetrien: 12 Drehungen (um 0°, 30°, 60°, …, 330°) und 12 Spiegelungen (6 durch gegenüberliegende Ecken, 6 durch gegenüberliegende Seitenmitten). In der Sprache der Algebra bilden diese 24 Symmetrien die Diedergruppe D₁₂. Jedes reguläre n-Eck hat genau 2n Symmetrien.",
            );

            draw_chapter4_svg(ui);
        }
        4 => {
            info_h(ui, "54 Diagonalen");
            info_p(
                ui,
                "Eine Diagonale verbindet zwei nicht benachbarte Ecken eines Vielecks. Die Formel n(n−3)/2 liefert für das Zwölfeck 12×9/2 = 54 Diagonalen. Das klingt nach einem unübersichtlichen Netz — aber die Struktur ist bemerkenswert geordnet.",
            );

            info_h(ui, "Sechs verschiedene Längen");
            info_p(
                ui,
                "Jede Diagonale überspringt eine bestimmte Anzahl von Ecken. Da das Zwölfeck symmetrisch ist, haben alle Diagonalen, die gleich viele Ecken überspringen, dieselbe Länge. Es gibt fünf mögliche Sprungweiten (1 bis 5 Ecken), plus den Durchmesser mit 6 — also sechs verschiedene Längentypen. Bei Seitenlänge s = 1:",
            );
            info_pre(
                ui,
                "  Typ        Überspringt  Länge (exakt)\n  s (Seite)  —            1\n                          \u{2248} 1.000\n  d₂         1 Ecke       \u{221a}(2+\u{221a}3)\n                          \u{2248} 1.932\n  d₃         2 Ecken      1+\u{221a}3\n                          \u{2248} 2.732\n  d₄         3 Ecken      (3\u{221a}2+\u{221a}6)/2\n                          \u{2248} 3.346\n  d₅         4 Ecken      2+\u{221a}3\n                          \u{2248} 3.732\n  d₆ (⌀)     5 Ecken      \u{221a}6+\u{221a}2\n                          \u{2248} 3.864",
            );

            info_h(ui, "Verborgene Muster");
            info_p(
                ui,
                "Die dritte und die fünfte Diagonale unterscheiden sich um genau 1: d₃ = 1+√3 und d₅ = 2+√3. Die Differenz ist die Seitenlänge selbst — eine geometrische Tatsache, keine rechnerische.",
            );
            info_p(
                ui,
                "Der Durchmesser d₆ ist exakt doppelt so lang wie die kürzeste Diagonale d₂: √6+√2 = 2·√(2+√3). Durchmesser und kürzeste Diagonale stehen im Verhältnis 2:1 — dieselbe Proportion wie die Oktave in der Musik.",
            );

            info_h(ui, "Das 15-Grad-Raster");
            info_p(
                ui,
                "Alle Winkel, die im Zwölfeck auftreten — zwischen Seiten, zwischen Diagonalen — sind Vielfache von 15°. Das liegt daran, dass die zwölf Ecken den Vollkreis in zwölf Sektoren zu je 30° teilen. 15° = 1/24 des Vollkreises. Dozenal: 15° = 13°doz, und 30° = 26°doz. Alle auftretenden Winkel lassen sich dozenal als ganzzahlige Vielfache von 13° schreiben.",
            );

            draw_chapter5_svg(ui);
        }
        5 => {
            info_h(ui, "Die Fläche des Zwölfecks");
            info_p(
                ui,
                "Ein regelmässiges Zwölfeck mit Seitenlänge s hat die Fläche A = 3s²(2+√3). Die Herleitung ist anschaulich: man zerlegt das Zwölfeck vom Mittelpunkt aus in 12 gleichschenklige Dreiecke, berechnet die Fläche eines einzelnen Dreiecks und multipliziert mit 12.",
            );
            info_p(
                ui,
                "Bei s = 1 ergibt das A ≈ 11.196 (dezimal). Zum Vergleich: der Umkreis hat die Fläche πR² ≈ 11.725. Das Zwölfeck füllt seinen Umkreis zu mehr als 95% — deutlich besser als ein Sechseck (83%) und weit besser als ein Quadrat (64%) oder ein Dreieck (41%).",
            );

            info_h(ui, "3/π — ein elegantes Verhältnis");
            info_p(
                ui,
                "Das Verhältnis der Zwölfeck-Fläche zur Umkreis-Fläche vereinfacht sich zu 3/π. Die Herleitung nutzt sin²(15°) = (2−√3)/4, wodurch sich im Flächenverhältnis der Faktor (2+√3)(2−√3) zu 1 kürzt, und es bleibt genau 3/π übrig.",
            );
            info_p(
                ui,
                "3/π ≈ 0.9549 (dezimal) — das Zwölfeck erfasst 95.5% der Kreisfläche. Tippe 3 / π = im Rechner, um es zu verifizieren.",
            );

            info_h(ui, "Vier Vielecke im Vergleich");
            info_p(
                ui,
                "Alle folgenden Figuren teilen denselben Umkreis. Formel: A = (n/2)·R²·sin(2π/n).",
            );
            info_pre(
                ui,
                "  Figur          Anteil    Formel\n  Dreieck        41.3%     3\u{221a}3/(4\u{03c0})\n  Quadrat        63.7%     2/\u{03c0}\n  Sechseck       82.7%     3\u{221a}3/(2\u{03c0})\n  Zwölfeck       95.5%     3/\u{03c0}",
            );
            info_p(
                ui,
                "Das Sechseck hat exakt die doppelte Fläche des Dreiecks (beide enthalten den Faktor 3√3). Und jeder Schritt bringt einen grösseren Flächenzuwachs, weil die Ecken den Kreis immer enger umschliessen.",
            );

            info_h(ui, "Archimedes und die Kreiszahl");
            info_p(
                ui,
                "Archimedes berechnete π über Vielecke. Er verwendete ein 96-Eck — 96 = 12×8 = 12×2³. Er begann mit dem Sechseck (das trivial konstruierbar ist) und verdoppelte die Eckenzahl dreimal: 6→12→24→48→96. Der Ausgangspunkt seiner Methode war also das Zwölfeck.",
            );
            info_p(
                ui,
                "Sein Ergebnis: 3 + 10/71 < π < 3 + 1/7. Ein 96-Eck füllt den Umkreis zu 99.93%. Von den 95.5% des Zwölfecks zu 99.93% sind es nur drei Verdoppelungsschritte — ein bemerkenswertes Tempo der Konvergenz.",
            );
        }
        6 => {
            info_h(ui, "Zwölf Flächen");
            info_p(
                ui,
                "Der Dodekaeder ist ein Körper aus zwölf regelmässigen Fünfecken. Jede Fläche ist identisch, jede Kante gleich lang, und an jeder Ecke treffen genau drei Fünfecke zusammen. Insgesamt hat er 12 Flächen, 30 Kanten und 20 Ecken. Er ist einer der fünf platonischen Körper — die einzigen konvexen Körper, deren Flächen ausschliesslich aus identischen regelmässigen Vielecken bestehen.",
            );

            info_h(ui, "Die fünf platonischen Körper");
            info_pre(
                ui,
                "  Körper           Flächen  Ecken  Kanten  Form\n  Tetraeder            4      4       6   Dreiecke\n  Hexaeder/Würfel      6      8      12   Quadrate\n  Oktaeder             8      6      12   Dreiecke\n  Dodekaeder          12     20      30   Fünfecke\n  Ikosaeder           20     12      30   Dreiecke",
            );
            info_p(
                ui,
                "Der Dodekaeder ist der einzige platonische Körper mit fünfeckigen Flächen. Platon ordnete in seiner Kosmologie die vier anderen Körper den Elementen zu — den Dodekaeder dem Kosmos selbst.",
            );

            info_h(ui, "Wie sieht er aus?");
            info_p(
                ui,
                "Wer Rollenspiele spielt, kennt ihn als D12 — den zwölfseitigen Würfel. Er liegt angenehm in der Hand und kommt zuverlässig auf einer Fläche zum Liegen. Der Fussball ist kein Dodekaeder: er ist ein abgestumpftes Ikosaeder aus 12 Fünfecken und 20 Sechsecken.",
            );

            info_h(ui, "Gallorömische Pentagondodekaeder");
            info_p(
                ui,
                "Über hundert kleine Bronzeobjekte in Form des Dodekaeders wurden in Nordeuropa gefunden, datiert auf das 2. bis 4. Jahrhundert n. Chr. Sie haben zwölf fünfeckige Flächen mit unterschiedlich grossen runden Löchern darin. Niemand weiss mit Sicherheit, wofür sie verwendet wurden. Hypothesen reichen von Kerzenhaltern über Vermessungsinstrumente bis zu religiösen Gegenständen. Das Rätsel ist bis heute ungelöst.",
            );

            info_h(ui, "Eulers Polyedersatz");
            info_p(
                ui,
                "Für jeden konvexen Polyeder gilt eine einfache Beziehung: Ecken minus Kanten plus Flächen ist immer gleich zwei. Leonhard Euler formulierte dieses Gesetz 1758. Für den Dodekaeder: 20 − 30 + 12 = 2. Diese Formel gilt für alle fünf platonischen Körper, für jedes Prisma, für jede Pyramide, für jeden konvexen Körper überhaupt.",
            );
        }
        7 => {
            info_h(ui, "Der Goldene Schnitt im Dodekaeder");
            info_p(
                ui,
                "Jede Fläche des Dodekaeders ist ein regelmässiges Fünfeck — und das regelmässige Fünfeck ist die Heimat des Goldenen Schnitts. Die Diagonale eines solchen Fünfecks verhält sich zu seiner Seite exakt wie φ = (1+√5)/2 ≈ 1.618 (dezimal) zu 1. Diese Proportion durchdringt den gesamten Körper.",
            );
            info_pre(
                ui,
                "  Grösse            Formel              Dezimalwert\n  Volumen           (15+7\u{221a}5)/4          ≈ 7.663\n  Oberfläche        3\u{221a}(25+10\u{221a}5)        ≈ 20.646\n  Umkugelradius     \u{221a}3·\u{03c6}/2              ≈ 1.401\n  Inkugelradius     \u{221a}(25+11\u{221a}5)/(2\u{221a}10)   ≈ 1.114",
            );
            info_p(
                ui,
                "Wer im Rechner φ² = tippt, erhält φ+1. Das ist die definierende Eigenschaft des Goldenen Schnitts — und der Grund, warum φ in so vielen Formeln des Dodekaeders erscheint.",
            );

            info_h(ui, "Dualität — der Spiegel des Ikosaeders");
            info_p(
                ui,
                "Zu jedem platonischen Körper gibt es einen dualen Körper: man ersetzt jede Fläche durch eine Ecke (im Mittelpunkt der Fläche) und verbindet benachbarte neue Ecken mit Kanten. Beim Dodekaeder entsteht so das Ikosaeder — und umgekehrt:",
            );
            info_pre(
                ui,
                "               Dodekaeder   Ikosaeder\n  Flächen            12           20\n  Kanten             30           30\n  Ecken              20           12",
            );
            info_p(
                ui,
                "Flächen und Ecken tauschen die Plätze, die Kantenzahl bleibt gleich. Die 12 erscheint in beiden Körpern — einmal als Flächenzahl, einmal als Eckenzahl.",
            );

            info_h(ui, "120 Symmetrien");
            info_p(
                ui,
                "Der Dodekaeder besitzt die reichste Symmetrie unter allen platonischen Körpern: die Ikosaedergruppe Iₕ mit 120 Elementen — 60 Drehungen und 60 Dreh-Spiegelungen. Zum Vergleich: der Würfel hat nur 48 Symmetrien, das Tetraeder 24.",
            );
            info_p(
                ui,
                "120 = 5! = 2³×3×5. Die drei Primfaktoren 2, 3 und 5 sind exakt dieselben, die in den Flächen des Dodekaeders (Fünfecke) und in der Teilbarkeit von 12 (= 2²×3) zusammenkommen.",
            );
        }
        8 => {
            info_h(ui, "360 Grad und die Babylonier");
            info_p(
                ui,
                "Dass ein Vollkreis 360 Grad hat, ist keine Naturkonstante — es ist eine menschliche Festlegung, und sie geht auf die Babylonier zurück. Die babylonische Mathematik verwendete die Basis 60, und 360 = 6×60. Aber 360 lässt sich auch als 12×30 schreiben, und genau so teilten die Babylonier den Himmel auf: die scheinbare Sonnenbahn (die Ekliptik) wurde in 12 gleiche Abschnitte zu je 30° zerlegt. Jedem Abschnitt wurde ein Sternbild zugeordnet — die zwölf Tierkreiszeichen.",
            );
            info_p(
                ui,
                "Die Wahl von 12 war kein Zufall. Die Babylonier organisierten ihre 60er-Basis intern in Gruppen von 12, weil 60 = 12×5. Die Zwölf war für sie eine natürliche Untereinheit — in der Zeitmessung, im Kalender, in der Astronomie.",
            );

            info_h(ui, "Der Mond und die Zwölf");
            info_p(
                ui,
                "Warum gerade zwölf Abschnitte am Himmel? Weil die Natur selbst eine Zwölfteilung nahelegt: ein Sonnenjahr enthält fast genau 12 Mondzyklen. Ein synodischer Monat dauert etwa 29.53 Tage. 12 Mondzyklen ergeben 354.4 Tage — nur 11 Tage weniger als ein Sonnenjahr von 365.24 Tagen. Diese Beinahe-Übereinstimmung machte die Zwölf zur offensichtlichen Einteilung des Jahres.",
            );

            info_h(ui, "Ordnung am Himmel");
            info_p(
                ui,
                "Die Zwölfteilung des Himmels war für die alten Kulturen weit mehr als ein Koordinatensystem. Ein faszinierendes Detail: der Frühlingspunkt wandert langsam durch die Sternbilder, weil die Erdachse wie ein Kreisel taumelt (Präzession, Periode ca. 25'800 Jahre). Die Sternbilder, durch die der Frühlingspunkt wandert, heissen deshalb auch Zeitalter. Auch hier strukturiert die Zwölf die Zeit: zwölf Sternbilder, zwölf Zeitalter, ein grosser Kreis.",
            );

            info_h(ui, "Die Zwölf anderswo am Himmel");
            info_p(
                ui,
                "Die alten Ägypter teilten Tag und Nacht in je 12 Stunden — daher unsere 24-Stunden-Einteilung. Der chinesische Tierkreis zählt ebenfalls zwölf Zeichen in 12-Jahres-Zyklen, abgeleitet vom 12-jährigen Jupiterumlauf. Beide Traditionen sind unabhängig voneinander entstanden. Die Konvergenz auf die Zahl 12 ist bemerkenswert.",
            );
        }
        9 => {
            info_h(ui, "Pyrit — das Narren-Dodekaeder");
            info_p(
                ui,
                "Pyrit (FeS₂), wegen seines goldenen Glanzes auch als »Narrengold« bekannt, kristallisiert häufig in einer Form, die dem platonischen Dodekaeder zum Verwechseln ähnlich sieht: der Pyritoeder. Er hat zwölf fünfeckige Flächen, 20 Ecken und 30 Kanten — dieselbe Topologie wie der reguläre Dodekaeder aus Kapitel 7. Aber bei genauem Hinsehen sind die Fünfecke nicht regelmässig. In der Kristallographie ist echte fünfzählige Drehsymmetrie bei periodischen Kristallen unmöglich — nur Symmetrien der Ordnung 1, 2, 3, 4 und 6 sind erlaubt. Der Pyritoeder schummelt sich mit unregelmässigen Fünfecken an dieser Regel vorbei.",
            );

            info_h(ui, "Granat — ein anderer Zwölfflächner");
            info_p(
                ui,
                "Die Minerale der Granat-Gruppe kristallisieren bevorzugt als Rhombendodekaeder — ebenfalls ein Körper mit zwölf Flächen, aber ganz anderer Natur: die Flächen sind Rauten, keine Fünfecke. Der Rhombendodekaeder hat 14 Ecken und 24 Kanten und gehört zum kubischen Kristallsystem. Er füllt den Raum lückenlos — die dreidimensionale Entsprechung der Bienenwabe.",
            );
            info_p(
                ui,
                "Die Natur verwendet die Zahl 12 als Flächenzahl für zwei völlig verschiedene Kristallformen — Fünfecke beim Pyrit, Rauten beim Granat. Die Zwölf ist nicht an eine bestimmte Geometrie gebunden.",
            );

            info_h(ui, "Radiolarien — Skelette aus Glas");
            info_p(
                ui,
                "Radiolarien sind einzellige Meeresorganismen, kaum grösser als ein Zehntel Millimeter, die filigrane Skelette aus Siliziumdioxid bilden. Einige Arten formen Skelette mit ikosaedrischer Symmetrie — also der Symmetrie des Ikosaeders, des Duals zum Dodekaeder. Der deutsche Biologe Ernst Haeckel zeichnete diese Organismen 1904 in seinem Werk »Kunstformen der Natur« mit einer Detailtreue, die bis heute beeindruckt.",
            );

            info_h(ui, "Quasikristalle — die Ausnahme, die die Regel bestätigt");
            info_p(
                ui,
                "1982 entdeckte Dan Shechtman in einer Aluminium-Mangan-Legierung ein Muster mit ikosaedrischer Symmetrie — die in normalen Kristallen verboten ist. Die Fachwelt reagierte zunächst mit Ablehnung. Doch die Beobachtung hielt stand, und 2011 erhielt Shechtman den Nobelpreis für Chemie. Diese Quasikristalle haben ikosaedrische Symmetrie, die sowohl Dodekaeder- als auch Ikosaeder-Geometrie enthält. Natürlich vorkommende Quasikristalle — das Mineral Icosahedrit — wurden 2009 in einem Meteoriten in Kamtschatka entdeckt.",
            );
        }
        10 => {
            info_h(ui, "Zwölf an einer Hand");
            info_p(
                ui,
                "Halte eine Hand vor dich, den Daumen abgespreizt, und betrachte die vier Finger. Jeder Finger hat drei Glieder (Phalangen), getrennt durch sichtbare Gelenke. Vier Finger mal drei Glieder — das sind zwölf. Der Daumen kann als Zeiger dienen: er berührt nacheinander jedes Glied der vier Finger und zählt so von eins bis zwölf.",
            );
            info_p(
                ui,
                "Diese Methode ist keine moderne Erfindung. In Teilen Südostasiens, Indiens und des Nahen Ostens wird sie seit Jahrhunderten verwendet. Sie hat einen entscheidenden Vorteil gegenüber dem westlichen Fingerzählen: sie nutzt eine Hand für zwölf Einheiten statt für fünf.",
            );

            info_h(ui, "Von zwölf zu sechzig");
            info_p(
                ui,
                "Die zweite Hand zählt die vollen Durchgänge. Jedes Mal, wenn die erste Hand eine Runde von zwölf vollendet hat, streckt die zweite Hand einen Finger aus. Fünf Finger mal zwölf — das ergibt sechzig. Mit zwei Händen kann man also bis 60 zählen, und das ohne jedes Hilfsmittel.",
            );
            info_p(
                ui,
                "Diese Verbindung von 12 und 60 ist vermutlich kein Zufall: das babylonische Sexagesimalsystem (Basis 60) könnte seinen Ursprung in genau dieser Zählmethode haben. 60 = 12×5 — eine elegante Verschmelzung von Anatomie und Arithmetik.",
            );

            info_h(ui, "Weitere Zwölfer in der Anatomie");
            info_pre(
                ui,
                "  — 12 Rippenpaare (Standardanatomie)\n  — 12 Hirnnervenpaare (I Olfactorius bis XII Hypoglossus)\n  — 12 Brustwirbel (mit den 12 Rippenpaaren verbunden)",
            );
            info_p(
                ui,
                "Anatomische Variationen kommen vor (11 oder 13 Rippenpaare sind selten möglich).",
            );

            info_h(ui, "Hat die Hand das Zahlensystem geformt?");
            info_p(
                ui,
                "Ob die Fingerglieder-Anatomie die Entstehung dozenaler Zahlensysteme beeinflusst hat oder umgekehrt, lässt sich historisch nicht sicher entscheiden. Es könnte auch eine gegenseitige Verstärkung gewesen sein: die Menschen begannen an den Fingergliedern zu zählen, weil die Zwölf in ihrer Kultur bereits wichtig war. Was sicher ist: die menschliche Hand bietet eine natürliche physische Grundlage für die Zwölf.",
            );
        }
        11 => {
            info_h(ui, "Zwölfer im Alltag");
            info_pre(
                ui,
                "  12 Zoll  = 1 Fuss\n  12 Unzen = 1 Troy-Pfund (Edelmetalle)\n  12 Pence = 1 Shilling (brit. Geld bis 1971)\n  12 Stück = 1 Dutzend\n  144      = 12² = 1 Gros",
            );
            info_p(
                ui,
                "Diese Einteilungen sind keine historischen Zufälle — sie wurden gewählt, weil sie das Teilen erleichtern. Ein Fuss lässt sich in zwei gleiche Teile teilen (je 6 Zoll), in drei (je 4 Zoll), in vier (je 3 Zoll) und in sechs (je 2 Zoll). Jede dieser Teilungen geht exakt auf.",
            );
            info_p(
                ui,
                "Ein Meter dagegen lässt sich in zwei gleiche Teile teilen (je 50 cm) und in fünf (je 20 cm), aber ein Drittel Meter ist 33.333… cm — ein unendlicher Bruch, sobald man es exakt nehmen will. Im Handwerk, wo ständig gedrittelt und geviertelt wird, ist die Zwölf praktischer als die Zehn.",
            );

            info_h(ui, "Das metrische System — und sein blinder Fleck");
            info_p(
                ui,
                "Das metrische System hat grosse Stärken: es ist kohärent (alle Einheiten passen zusammen), es skaliert dezimal (Kilo, Mega, Milli, Mikro), und es ist weltweit standardisiert. Diese Vorzüge sind real und gewichtig. Kein vernünftiger Mensch würde vorschlagen, SI abzuschaffen.",
            );
            info_p(
                ui,
                "Aber das metrische System erbt die Schwäche seiner Basis. In Basis 10 ist ein Drittel ein unendlicher Bruch: 0.333… In einem dozenalen metrischen System wäre 1/3 = 0.4 — exakt, kurz, ohne Restfehler. Die Eleganz des metrischen Prinzips bliebe erhalten — nur die Basis wäre besser.",
            );

            info_h(ui, "Tom Pendleburys TGM");
            info_p(
                ui,
                "Tom Pendlebury, Mitglied der Dozenal Society of Great Britain, hat diesen Gedanken konsequent zu Ende gedacht. Sein System heisst TGM — benannt nach seinen drei Grundeinheiten Tim (Zeit), Grafut (Länge), Maz (Masse). Pendlebury begann nicht mit der Länge, sondern mit der Zeit: er teilte die Stunde in 12⁴ gleiche Teile. Aus dem Tim leitete er über die Erdbeschleunigung den Grafut ab (≈ 29.6 cm), die Masseeinheit Maz aus dem Volumen eines Kubik-Grafut Wasser.",
            );
            info_p(
                ui,
                "Das Ergebnis ist ein vollständig kohärentes Einheitensystem, in dem alle Umrechnungen in Potenzen von 12 erfolgen. TGM wurde nie über Enthusiastenkreise hinaus angenommen, demonstriert aber, dass ein dozenales Metriksystem nicht nur möglich, sondern in mancher Hinsicht dem dezimalen überlegen wäre.",
            );

            info_h(ui, "Was dieser Rechner zeigt");
            info_p(
                ui,
                "Wer 1 / 3 = tippt und 0.4 sieht — kurz, exakt, ohne Periodenstrich — versteht in einer Sekunde, was Seiten voller Argumente nicht vermitteln können. Die Frage »Dozenal oder Dezimal?« wird in der Praxis nie entschieden werden. Aber die mathematischen Vorteile der Basis 12 sind objektiv und messbar, und dieser Rechner macht sie erlebbar.",
            );
        }
        _ => {
            info_p(ui, "Kapitel nicht gefunden.");
        }
    }
}
