# Info-Sektion-Layout für die Leptos-Web-Version

Stand der Recherche: Mai 2026. Anwendungsfall: 12 didaktische Kapitel (Dozenalität, Zwölfeck, Dodekaeder, …), Prosa + Tabellen + einfache SVGs, ~400–600 Wörter pro Kapitel, fix bei 12, Sprache Deutsch. Deployment Leptos 0.8 + Trunk auf GitHub Pages (statisch).

## TL;DR

**Empfehlung: Option 1 — Long-Read mit Sticky-TOC (auf Desktop) bzw. Top-Sheet-TOC (auf Mobile), kombiniert mit Hash-Anchor-Navigation pro Kapitel (`/#/info/3-fibonacci`).** Das ist das idiomatische Web-Pattern für Reading-Surfaces dieser Größe (vgl. mdBook, VitePress, Stripe-Docs), passt zur Stimmung „Reading-Surface, nicht Calculator-Interface", liefert Direkt-Verlinkbarkeit *gratis* via Anker, und ist in Leptos mit ~60 Zeilen Hash-Layer + IntersectionObserver-Scrollspy umsetzbar — ohne `leptos_router`. Optionen 2 und 3 lösen Teilaufgaben gut, aber keine löst alle Kriterien zugleich.

## Vergleichstabelle

Bewertung: ★★★ sehr gut · ★★ akzeptabel · ★ schwach · ✗ Show-Stopper.

| Kriterium | (1) Long-Read + Sticky-TOC | (2) Eigene Hash-Route pro Kapitel | (3) Akkordeon |
|---|---|---|---|
| Reading-Komfort (durchgängig) | ★★★ — Scroll durch alle 12 Kapitel möglich, Lesefluss intakt | ★★ — pro Kapitel ok, aber Weiterlesen erfordert Klick | ★ — Klick-pro-Kapitel zerstückelt Lese-Fluss, lange Kapitel mit Tabellen werden unhandlich |
| Direkt-Verlinkbarkeit | ★★★ — `#/info/3-fibonacci` springt zu Anker, Browser scrollt | ★★★ — pro Route ein Link | ★ — Anker auf geschlossene Sektion sind möglich, aber Aufklappen muss synchronisiert werden |
| Browser-Back-Button | ★★ — Back navigiert Hash-Sprünge zurück (nativ über `pushState`/`location.hash`) | ★★★ — Kapitel-für-Kapitel-Back, sehr „web-nativ" | ★ — Back verlässt die Info-Sektion ganz; Aufklapp-Aktionen sind kein History-Eintrag |
| Mobile (Portrait + Landscape) | ★★★ — TOC als Top-Sheet einklappbar; Lesefluss bleibt | ★★ — funktioniert, aber ständige Back/Forward-Taps; sticky-Header isst Höhe | ★★ — kompakt, aber kein Mehrwert über Hash-Anker |
| SEO / Indexierbarkeit | ★★★ — eine Seite, alle 12 H2-Headings im DOM, vorbildlich für Search-Indexer | ★ — Hash-Routen werden von Crawlern nicht als separate URLs gewertet | ★★ — alle Inhalte im DOM, aber `display:none` ist crawler-grenzwertig |
| Initial-Render-Performance | ★★ — 12 `<section>` initial im DOM (~6 KB HTML, vernachlässigbar) | ★★★ — nur ein Kapitel initial gerendert | ★★★ — Headers + erstes geöffnetes Kapitel |
| Implementations-Komplexität (Leptos) | ★★ — ~60 LOC Hash-Layer + ~30 LOC Scrollspy + CSS-Grid | ★★ — ~60 LOC Hash-Layer + Match auf `chapter_id` Signal | ★★★ — Pure Signal + `<details>` Element, fast nichts zu schreiben |
| Accessibility | ★★★ — Skip-Link, `<nav>`-Landmark, `aria-current`, Heading-Hierarchie geradlinig | ★★ — Heading-Hierarchie pro Kapitel ok, aber zwischen Kapiteln kein Lese-Kontinuum für Screen-Reader | ★ — `<details>`/`<summary>` ist accessible, aber Screen-Reader muss jeden Eintrag aufklappen |

**Show-Stopper:** Keiner. Alle drei Optionen sind funktional realisierbar. Aber: Option 2 (eigene Routen) verliert merklich gegen Option 1, weil die 12 Kapitel zusammen einen Bogen erzählen (Dozenalität → Zwölfeck → Dodekaeder → Anwendungen) — den der durchgängige Scroll abbildet, das Routing aber zerschneidet.

## Empfehlung mit Begründung

**Option 1 (Long-Read mit Sticky-TOC + Hash-Anker).** Drei Gründe machen das eindeutig:

1. **Web-Idiomatik.** Lange didaktische Inhalte mit fester, kleiner Kapitelzahl sind im modernen Web als Long-Read mit Sidebar-TOC etabliert: mdBook, VitePress, Stripe-Docs, Smashing-Magazine-Longreads. Niemand baut für 12 thematisch zusammenhängende Kapitel 12 separate Routen — das wäre ein PHP-Pattern aus 2008. Der Eigentümer hat explizit gesagt, dass die Web-Version *nicht* den Flutter-Navigator-Stack imitieren soll; Option 1 ist genau die idiomatische Alternative.

2. **Direkt-Verlinkbarkeit ohne Routing-Overhead.** `#/info/3-fibonacci` ist ein Anker auf eine `<section id="3-fibonacci">`. Der Browser scrollt nativ dorthin, ohne dass die App reagieren muss. Eigene Routen pro Kapitel (Option 2) bauen das gleiche Verhalten mit mehr Code nach und verlieren dabei den durchgängigen Lesefluss.

3. **Accessibility-Best-Practice.** Eine `<nav aria-label="Kapitelübersicht">` mit `aria-current="location"` auf dem aktiven Eintrag plus Skip-Link zum Hauptinhalt ist die WCAG-konforme Long-Read-Lösung. Screen-Reader-Nutzer können entweder via Landmark-Navigation direkt ins Hauptthema springen oder via TOC zu einem bestimmten Kapitel.

Akkordeon (Option 3) ist nur dort sinnvoll, wo Nutzer *nur einen* Eintrag öffnen (FAQs). Hier soll der gesamte Inhalt gelesen werden können — Akkordeon zwingt zu N Klicks und unterdrückt den Lesefluss.

## Leptos-Implementierungs-Skizze

### Komponenten-Hierarchie

```text
<App>
└── <Calculator>   // bestehender Keypad-Code
    └── <InfoPage> // erscheint, wenn hash.starts_with("#/info")
        ├── <Toc current_chapter=read_signal />   // <nav>, sticky auf desktop, top-sheet auf mobile
        └── <ChapterList>                          // <main>
            ├── <Chapter id="1-bedienung" title="Bedienung des Rechners" />
            ├── <Chapter id="2-dozenalsystem" title="Was ist das Dozenalsystem?" />
            └── ... (12 total)
```

`<InfoPage>` ist ein eigenständiger View — kein Modal mit Overlay-Hintergrund. Sie ersetzt visuell die Calculator-View für die Dauer des Besuchs.

### Hash-Routing in ~60 Zeilen

`leptos_router` wird **nicht** benötigt. Ein dünner Layer reicht:

```rust
// src/info/hash_route.rs
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{window, HashChangeEvent};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Route {
    Calc,                    // ""  oder  "#/"
    Info(Option<String>),    // "#/info" oder "#/info/3-fibonacci"
}

impl Route {
    fn parse(hash: &str) -> Self {
        // "#/info/3-fibonacci"  ->  Info(Some("3-fibonacci"))
        let s = hash.strip_prefix('#').unwrap_or(hash);
        let s = s.strip_prefix('/').unwrap_or(s);
        match s.strip_prefix("info") {
            None => Route::Calc,
            Some(rest) => {
                let anchor = rest.strip_prefix('/').filter(|x| !x.is_empty()).map(String::from);
                Route::Info(anchor)
            }
        }
    }
}

pub fn use_route() -> ReadSignal<Route> {
    let (route, set_route) = signal(read_current_hash());
    // hashchange-Event-Listener registrieren
    let closure = Closure::<dyn Fn(HashChangeEvent)>::new(move |_| {
        set_route.set(read_current_hash());
    });
    window().unwrap()
        .add_event_listener_with_callback("hashchange", closure.as_ref().unchecked_ref())
        .unwrap();
    closure.forget();  // SAFETY: Listener lebt für die ganze App-Lifetime, kein Leak in der Praxis
    route
}

fn read_current_hash() -> Route {
    let hash = window().unwrap().location().hash().unwrap_or_default();
    Route::parse(&hash)
}

pub fn navigate(route: &Route) {
    let hash = match route {
        Route::Calc => "",
        Route::Info(None) => "#/info",
        Route::Info(Some(id)) => &format!("#/info/{id}"),
    };
    window().unwrap().location().set_hash(hash).ok();
}
```

In `<App>` wird das Signal abgefragt:

```rust
view! {
    {move || match route.get() {
        Route::Calc => view! { <Calculator/> }.into_any(),
        Route::Info(anchor) => view! { <InfoPage initial_anchor=anchor/> }.into_any(),
    }}
}
```

### Mobile/Desktop-Layout

CSS Grid mit Media-Query, kein JS-Branching nötig:

```css
.info-page { display: grid; grid-template-columns: 240px 1fr; gap: 2rem; }
.info-page > nav { position: sticky; top: 0; max-height: 100vh; overflow-y: auto; padding: 1rem; }
.info-page > main { padding: 2rem; max-width: 70ch; line-height: 1.6; }

@media (max-width: 700px) {
  .info-page { grid-template-columns: 1fr; }
  .info-page > nav {
    position: sticky; top: 0;
    max-height: 4rem;       /* zugeklappt: nur Aktiv-Titel sichtbar */
    transition: max-height 200ms ease;
  }
  .info-page > nav[aria-expanded="true"] { max-height: 60vh; }
}
```

Auf Mobile wird die TOC zum **Top-Sheet**: ein zugeklappter Streifen mit aktuellem Kapitel + Pfeil; Tap entfaltet die volle Liste. Das vermeidet das Smashing-Magazine-Anti-Pattern „sticky Sidebar isst die Hälfte des Screens" auf Portrait-Phones.

### Scrollspy (Anker-Hervorhebung)

Mit `IntersectionObserver` aus `web-sys`:

```rust
let observer = IntersectionObserver::new_with_callback_and_options(
    &Closure::<dyn Fn(Vec<IntersectionObserverEntry>)>::new(move |entries| {
        for entry in entries {
            if entry.is_intersecting() {
                let id = entry.target().get_attribute("id").unwrap_or_default();
                set_active_chapter.set(id);
            }
        }
    }).into_js_value().unchecked_ref(),
    IntersectionObserverInit::new().root_margin("-30% 0px -60% 0px"),
).unwrap();
```

Die `root_margin`-Heuristik markiert das Kapitel als aktiv, das die obere Bildschirmdrittel überquert — Standardpattern aus CSS-Tricks und Maxime Heckels „Scrollspy demystified".

### SVG-Einbettung

**Inline.** Die Kapitel 4–7 enthalten geometrische Diagramme (Zwölfeck, Diagonalen, Dodekaeder-Ansichten). Inline-SVG hat drei Vorteile gegenüber externen Assets:

- Skaliert mit dem umgebenden Text (CSS `width: 100%; max-width: 400px`).
- Erbt `currentColor` für Stroke/Fill — paßt automatisch zum Dark-Theme.
- Kein zusätzlicher HTTP-Request, kein FOUC.

Die Bilder werden als `const SVG_ZWOELFECK: &str = include_str!("../assets/zwoelfeck.svg");` eingebunden und mit `inner_html` ausgespielt — die SVG-Dateien sind unter Eigentümer-Kontrolle, kein XSS-Vektor. Die existierenden `chapter_*_svg.svg`-Dateien (Commit 5eecf99) lassen sich direkt wiederverwenden.

## Accessibility-Checkliste

- **Skip-Link** als erstes fokussierbares Element: `<a href="#info-main" class="skip-link">Zum Inhalt springen</a>`. Visuell `position: absolute; left: -9999px;`, bei `:focus` sichtbar.
- **Landmark-Hierarchie**: TOC in `<nav aria-label="Kapitelübersicht">`, Inhalt in `<main id="info-main">`. Damit findet jeder Screen-Reader per Single-Key-Navigation („D" in NVDA) sofort beide.
- **Aktives Kapitel kennzeichnen**: `aria-current="location"` auf dem aktiven TOC-Link. Visuell zusätzlich ein farblicher Marker (nicht *nur* Farbe — auch Fett oder ein Bullet-Glyph, um WCAG 1.4.1 zu erfüllen).
- **Heading-Hierarchie**: `<h1>Info</h1>` ganz oben (auf `<InfoPage>`), `<h2>` für jeden Kapitel-Titel, `<h3>` für Untersektionen in einem Kapitel. Keine Sprünge (`h1 → h3` vermeiden).
- **Fokus-Management bei Hash-Anker**: nach `navigate()` Programm-Fokus auf `<h2>` des Zielkapitels setzen (`element.focus()` mit `tabindex="-1"` am Heading) — sonst springt der Tab-Fokus nach Sprung an die alte Stelle zurück.
- **Sticky-TOC respektiert WCAG 2.4.12** („Focused Component Not Obscured"): da die TOC links sitzt (Desktop) bzw. oben sehr schmal ist (Mobile), verdeckt sie keinen Fokus-Indikator im Hauptinhalt.
- **Reduced Motion**: `scroll-behavior: smooth` nur außerhalb `@media (prefers-reduced-motion: reduce)`.
- **Reflow (WCAG 1.4.10)**: TOC und Inhalt nutzen `max-width` in `ch`, keine festen `px` — Layout bleibt bis 320 px Breite ohne horizontales Scrolling lesbar.

## Quellen

- Leptos & Hash-Routing: [Leptos Issue #2184 Support hashstyle routing](https://github.com/leptos-rs/leptos/issues/2184) · [docs.rs `web_sys::HashChangeEvent`](https://docs.rs/web-sys/latest/web_sys/struct.HashChangeEvent.html) · [Leptos Book: Integrating with JavaScript (web_sys)](https://book.leptos.dev/web_sys.html) · [Static SPAs Exploration of Leptos, Dioxus, Next.js](https://codethoughts.io/posts/2024-07-05-static-spas-exploration-of-leptos-dioxus-and-nextjs/) · [docs.rs `leptos_router`](https://docs.rs/leptos_router/latest/leptos_router/)
- Long-Read / TOC-Pattern: [CSS-Tricks: Table of Contents with IntersectionObserver](https://css-tricks.com/table-of-contents-with-intersectionobserver/) · [CSS-Tricks: Sticky Table of Contents with Scrolling Active States](https://css-tricks.com/sticky-table-of-contents-with-scrolling-active-states/) · [Maxime Heckel: Scrollspy demystified](https://blog.maximeheckel.com/posts/scrollspy-demystified/) · [Smashing Magazine: Designing Sticky Menus — UX Guidelines](https://www.smashingmagazine.com/2023/05/sticky-menus-ux-guidelines/) · [Nielsen Norman Group: Sticky Headers — 5 Ways to Make Them Better](https://www.nngroup.com/articles/sticky-headers/) · [VitePress Default Theme Layout](https://vitepress.dev/reference/default-theme-layout)
- Akkordeon-Trade-offs: [NN/G: Accordions on Desktop — When and How to Use](https://www.nngroup.com/articles/accordions-on-desktop/) · [NN/G: Accordions for Complex Website Content on Desktops](https://www.nngroup.com/articles/accordions-complex-content/) · [Carbon Design System: Accordion Usage](https://carbondesignsystem.com/components/accordion/usage/)
- Accessibility: [W3C WAI: G64 Providing a Table of Contents](https://www.w3.org/TR/WCAG20-TECHS/G64.html) · [W3C WAI ARIA APG: Navigation Landmark Example](https://www.w3.org/WAI/ARIA/apg/patterns/landmarks/examples/navigation.html) · [W3C WAI: ARIA11 — Using ARIA landmarks](https://www.w3.org/WAI/WCAG21/Techniques/aria/ARIA11) · [MDN: ARIA navigation role](https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Reference/Roles/navigation_role) · [W3C: Understanding Success Criterion 1.4.10 Reflow](https://www.w3.org/WAI/WCAG21/Understanding/reflow.html) · [TestParty: Accessible Navigation Patterns — Menus, Breadcrumbs, Skip Links](https://testparty.ai/blog/accessible-navigation-patterns) · [boia.org: How Sticky and Fixed Elements Impact Accessibility](https://www.boia.org/blog/how-sticky-and-fixed-elements-impact-accessibility)
- Mobile-Reading-Pattern: [UX Design: Porting long-scroll content to small-screen](https://uxdesign.cc/porting-long-scroll-content-to-a-small-screen-a-different-approach-to-sticky-in-page-navigation-ca94f15262fe) · [Parallel HQ: What is a Sticky Header? UX Best Practices & 2026 Design Guide](https://www.parallelhq.com/blog/what-sticky-header)
