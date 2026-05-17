// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Info-Surface — Long-Read mit Sticky-TOC und Hash-Anker-Routing.
//!
//! Entspricht der Empfehlung in `research/info_layout.md`: alle 12 Kapitel
//! als Sektionen untereinander, linke Seitenleiste mit Titeln, Klick scrollt
//! zum entsprechenden Anker via `scroll_into_view`. Browser-Zurück und
//! Direktverlinkung über die Hash-URL funktionieren von Haus aus.
//!
//! Phase E liefert die Architektur und Kapitel-Teaser. Der Volltext aus
//! `INFO_MODAL_CONTENT.md` (zwölf Kapitel, je 400–600 Wörter, teils mit
//! Tabellen und SVG-Illustrationen) wird in einem separaten Schritt
//! portiert; bis dahin steht je Kapitel ein Teaser plus Hinweis.

use crate::router::{Route, navigate};
use leptos::prelude::*;

#[derive(Clone, Copy)]
struct ChapterMeta {
    slug: &'static str,
    number: u8,
    title: &'static str,
    teaser: &'static str,
}

const CHAPTERS: &[ChapterMeta] = &[
    ChapterMeta {
        slug: "bedienung",
        number: 1,
        title: "Bedienung des Rechners",
        teaser: "Die zwölf Ziffern, Cursor-Steuerung, Speicher, Modi und das Lesen periodischer Ergebnisse — alles, was auf dem Rechner gebraucht wird.",
    },
    ChapterMeta {
        slug: "dozenalsystem",
        number: 2,
        title: "Was ist das Dozenalsystem?",
        teaser: "Eine Einführung in die Basis 12: warum Brüche wie 1/3 oder 1/4 hier finit werden, wo das System historisch auftaucht, und warum es Dezimalrechnen nicht ersetzt — aber ergänzt.",
    },
    ChapterMeta {
        slug: "zahlentheorie",
        number: 3,
        title: "Fibonacci, Quadratzahlen und andere Kuriositäten",
        teaser: "F(12) = 144 ist sowohl Quadrat- als auch Fibonacci-Zahl. Der goldene Schnitt, Ramanujan-Identitäten und weitere Überraschungen rund um die Zwölf.",
    },
    ChapterMeta {
        slug: "zwoelfeck-grundlagen",
        number: 4,
        title: "Das Zwölfeck — Grundlagen",
        teaser: "Konstruktion, Symmetrie und die fünf regelmässigen Polygone, die das Zwölfeck enthält. Warum es zwischen Drei- und Sechseck eine besondere Stellung einnimmt.",
    },
    ChapterMeta {
        slug: "zwoelfeck-winkel",
        number: 5,
        title: "Das Zwölfeck — Winkel und Diagonalen",
        teaser: "54 Diagonalen, exakte Längen in Vielfachen von Sinus und Kosinus, und das geheime 15°-Raster, das alle inneren Linien strukturiert.",
    },
    ChapterMeta {
        slug: "zwoelfeck-flaechen",
        number: 6,
        title: "Das Zwölfeck — Flächen und Verhältnisse",
        teaser: "A = 3s²(2+√3). Wie das Zwölfeck Pi annähert, und welche Flächenverhältnisse zwischen einbeschriebenen Polygonen auftauchen.",
    },
    ChapterMeta {
        slug: "dodekaeder",
        number: 7,
        title: "Der Dodekaeder — zwölf Fünfecke im Raum",
        teaser: "Die platonischen Körper, Euler-Formel und das Dodekaeder als Krönung der fünfeckigen Symmetrie im dreidimensionalen Raum.",
    },
    ChapterMeta {
        slug: "dodekaeder-mathematik",
        number: 8,
        title: "Der Dodekaeder — φ, Dualität und Symmetrie",
        teaser: "Wie der goldene Schnitt im Dodekaeder steckt, die Dualität mit dem Ikosaeder, und die 120-elementige Symmetriegruppe.",
    },
    ChapterMeta {
        slug: "tierkreis",
        number: 9,
        title: "Zwölf Tierkreiszeichen und der Himmel",
        teaser: "Wie babylonische Astronomie auf zwölf Monate, zwölf Stunden und zwölf Tierkreiszeichen kam — und warum der Kalender heute noch dieser Struktur folgt.",
    },
    ChapterMeta {
        slug: "natur",
        number: 10,
        title: "Zwölf Flächen in Kristallen und Lebewesen",
        teaser: "Pyrit, Granat, Radiolarien — wo der dodekaedrische Bauplan in der Natur auftaucht, und welche Energie-Minima ihn begünstigen.",
    },
    ChapterMeta {
        slug: "hand",
        number: 11,
        title: "Zwölf Glieder an der Hand",
        teaser: "Phalanx-Zählung: vier Finger × drei Glieder = zwölf, der Daumen als Zähler. Die anatomische Basis dozenaler Zahlsysteme.",
    },
    ChapterMeta {
        slug: "messwesen",
        number: 12,
        title: "Zoll, Fuss, Pfund — und warum sie dozenal Sinn ergeben",
        teaser: "Imperial-Einheiten, das TGM-System, und die durch-die-Hintertür-Persistenz der Zwölf im modernen Messwesen.",
    },
];

#[component]
pub fn InfoView(#[prop(into)] anchor: Signal<Option<String>>) -> impl IntoView {
    // Beim Mount und bei Anker-Wechsel: zur Sektion scrollen.
    Effect::new(move |_| {
        let a = anchor.get();
        if let Some(slug) = a {
            scroll_to_chapter(&slug);
        }
    });

    view! {
        <div class="info-page">
            <header class="info-header">
                <button
                    class="info-back"
                    aria-label="Zurück zum Rechner"
                    on:click=move |_| navigate(&Route::Calc)
                >
                    "← Rechner"
                </button>
                <h1 class="info-title">"Über das Dozenalsystem"</h1>
            </header>

            <div class="info-layout">
                <aside class="info-toc" aria-label="Inhaltsverzeichnis">
                    <ol class="info-toc-list">
                        {CHAPTERS.iter().map(|c| {
                            view! {
                                <li>
                                    <a
                                        href={format!("#/info/{}", c.slug)}
                                        class="info-toc-link"
                                    >
                                        <span class="info-toc-num">{format!("{:>2}", c.number)}</span>
                                        <span class="info-toc-title">{c.title}</span>
                                    </a>
                                </li>
                            }
                        }).collect_view()}
                    </ol>
                </aside>

                <article class="info-body">
                    {CHAPTERS.iter().map(|c| {
                        view! {
                            <ChapterSection meta={*c}/>
                        }
                    }).collect_view()}
                    <p class="info-end">
                        "Volltext aller zwölf Kapitel wird in einem separaten Schritt aus "
                        <code>"INFO_MODAL_CONTENT.md"</code>
                        " portiert — die Architektur (Routing, TOC, Scroll-Anker, Hash-URLs) steht."
                    </p>
                </article>
            </div>
        </div>
    }
}

#[component]
fn ChapterSection(meta: ChapterMeta) -> impl IntoView {
    let id = format!("ch-{}", meta.slug);
    view! {
        <section class="info-chapter" id=id>
            <h2 class="info-chapter-title">
                <span class="info-chapter-num">{format!("Kapitel {}", meta.number)}</span>
                <span class="info-chapter-name">{meta.title}</span>
            </h2>
            <p class="info-chapter-teaser">{meta.teaser}</p>
            <p class="info-chapter-stub">
                <em>"[Volltext folgt — Architektur Phase E steht.]"</em>
            </p>
        </section>
    }
}

fn scroll_to_chapter(slug: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            let id = format!("ch-{slug}");
            if let Some(el) = doc.get_element_by_id(&id) {
                el.scroll_into_view();
            }
        }
    }
}
