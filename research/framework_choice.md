# Rust-DOM-Framework-Wahl für den Dozenal-Taschenrechner

Stand der Recherche: Mai 2026. Anwendungsfall: Mid-Size SPA (Calculator-Keypad, Two-Line-Display, 12-Kapitel-Info-Modal, Custom-Glyphen in `<canvas>`/`<svg>`-Insets), Deployment auf GitHub Pages, kein Backend.

## TL;DR

**Empfehlung: Leptos 0.8.** Es ist 2025/2026 das aktivste, am besten unterstützte und am stärksten optimierte Rust-Web-Framework, hat Fine-grained-Reactivity ohne VDOM-Overhead, und Trunk/cargo-leptos liefern reproduzierbare GitHub-Pages-Deployments. Yew ist die konservative Alternative, falls Reaktivität via Hooks-Modell (React-ähnlich) gewünscht wird; Sycamore und Dioxus haben für diesen spezifischen Use-Case je ein klares Aber.

## Vergleichstabelle

Note: Bundle-Zahlen sind Größenordnungen für eine Mid-Size-App nach `opt-level='z'`, `lto=true`, `strip=true`, `wasm-opt -Oz`, **brotli-komprimiert**. Genaue Zahlen sind App-spezifisch (vom Dependency-Tree dominiert).

| # | Kriterium | Leptos 0.8 | Dioxus 0.7 | Sycamore 0.9 | Yew 0.23 |
|---|---|---|---|---|---|
| 1 | Bundle (br, Mid-Size) | ~100-180 KB | ~120-200 KB; Splitting möglich | ~90-160 KB | ~180-280 KB |
| 2 | Touch/Mobile UX | OK (web-sys), keine Auto-Polish | Beste Mobile-Story (eigene mobile Targets) | OK (web-sys) | OK (web-sys), keine Auto-Polish |
| 3 | Accessibility | radix-leptos verfügbar (WCAG 2.1 AA) | Erste-Klasse Component-Lib mit ARIA | Eigenständig zu bauen | Eigenständig zu bauen |
| 4 | Hash-Routing | **Nicht out-of-the-box** (Issue #2184 offen) | Ja, via `hash` Segments + 404-Fallback | Nur `HistoryIntegration` dokumentiert | Ja, `HashRouter` (offiziell als „last resort") |
| 5 | Canvas/SVG | `NodeRef<Canvas>` + `web-sys`, idiomatisch | `web-sys` + Native-WGPU-Pfad | `NodeRef` + `web-sys` | `NodeRef` + `web-sys` |
| 6 | SSG / Static-Hosting | CSR-Mode + Trunk `--public-url` einfach; SSG-Modus existiert | `dx build --release`, `index_on_404`, Static-Site-Templates | Static-Builds via Trunk; SSR-Streaming ja | Trunk-Standard, ausgereift |
| 7 | Reaktivität | Signals (Solid-style), `count()` API | VDOM mit fiber-batching; Hooks | Signals (Reactivity v3, `'static + Copy`) | VDOM (React-style), `use_state` |
| 8 | Aktivität (2025/26) | Sehr hoch — Releases im Monatsrhythmus, 0.8.18 (Apr 26) | Sehr hoch — v0.7.9 (8. Mai 26), Native-Mobile | **Niedrig** — v0.9.2 ist Sept 25, 33 Issues offen | Mittel — v0.23 (März 25), stabil |
| 9 | Lernkurve (egui-Erfahrung) | Mittel — Signals neu, aber egui-User kennen reaktives Denken | Hoch — RSX-Macro, plus Plattformmodell | Mittel — Sycamore-Sprache abweichend | Niedrig — VDOM/Hooks vertraut von React |

## Bundle-Daten

Konkrete Zahlen aus den Quellen:

- **Dioxus Hello World**: ~70 KB gzip bzw. ~60 KB brotli laut Dioxus-Doku. Aspirativ <50 KB mit aggressiver Optimierung; nightly-Features senken Hello-World auf <100 KB. Mid-Size Apps liegen typischerweise bei 120-200 KB brotli, mit `wasm-split` (Code-Splitting per Route in 0.7) reduzierbar.
- **Leptos**: Keine offizielle Hello-World-Zahl in der Doku. Ein User-Bericht im Repo reduzierte ein Hackernews-Beispiel von 135 KB → 69 KB (`brotli`, unklar) durch Sub-Crate-Refactor. Leptos 0.7+ hat `#[lazy]` und `#[lazy_route]` für WASM-Splitting. Krausest's js-framework-benchmark führt Leptos als schnellstes Rust/Wasm-Framework — Bundle-Werte aus dem aktuellen Lauf sind nicht direkt zugänglich (Tabelle hinter dem `current.html`-Endpoint nicht extrahierbar).
- **Sycamore**: Keine offiziellen Bundle-Zahlen aus 2025/26. Historisch (v0.8) als kompakt eingeordnet; v0.9-Reactivity-v3 hat keine nachgewiesene Bundle-Regression.
- **Yew**: GitHub-Issue #1 zu rustmart-yew-example zeigt 488 KB unkomprimiert / 136 KB komprimiert. Dies entspricht einer kleinen E-Commerce-Demo, nicht einer Mid-Size-App. Yew ist in den Krausest-Benchmarks regelmäßig **größer** als Leptos/Dioxus/Sycamore-Beispiele.

**Bewertung**: Bundle-Größen liegen für alle vier auf ähnlicher Größenordnung. Der Dependency-Tree der App (z. B. `meval`, `gloo`, `web-sys`-Features) bestimmt das Resultat stärker als die Framework-Wahl. Yew ist tendenziell der teuerste Kandidat (Virtual-DOM-Runtime + größere Helper-APIs).

## Pro/Contra pro Framework

### Leptos 0.8 (leptos-rs)
**Pro**
- Aktivste Rust-Web-Codebase 2025/2026, monatliche Releases, 20.8k Stars, 5.259 Commits.
- Fine-grained Reactivity wie SolidJS: keine VDOM-Diffs, direkte DOM-Updates → besser für Calc-Use-Case mit isolierten Updates (einzelne Display-Tokens ändern, Keypad bleibt statisch).
- WASM-Splitting (`#[lazy]`, `#[lazy_route]`) seit 0.7 stabil.
- `radix-leptos-primitives` als WCAG-2.1-AA-Komponentenbibliothek verfügbar.
- Beste rust-analyzer-Integration der vier Frameworks, gute Fehlermeldungen im View-Macro.

**Contra**
- **Hash-Routing nicht out-of-the-box** (Issue #2184 offen seit Januar 2024). Für die 12-Kapitel-Navigation muss entweder via Trunk `--public-url` mit `404.html`-Trick gearbeitet werden oder ein eigenes Hash-Routing-Layer gebaut werden.
- Server-Functions/SSR-Architektur dominiert die Doku — als reine CSR-App auf GitHub Pages „läuft man gegen den Strom".
- Signal-Closures (`count()` vs. `move || count.get()`) sind ein häufiger Stolperstein für Einsteiger.

### Dioxus 0.7 (DioxusLabs)
**Pro**
- Höchste Release-Frequenz aller vier (v0.7.9 vom 8. Mai 2026), 36.1k Stars.
- Eigene Komponentenbibliothek (`DioxusLabs/components`, Radix-basiert, WAI-ARIA-konform, 28 Foundation-Komponenten).
- WASM-Splitting nativ per Router-Variant (`wasm-split` Feature).
- Hash-Segments und `index_on_404 = true` in `Dioxus.toml` machen GitHub-Pages-Deployment offiziell unterstützt.
- VDOM-Fiber-Architektur ist React-nahe und erlaubt einen schrittweisen Lernpfad.

**Contra**
- Multi-Plattform-Fokus (Web + Desktop + iOS + Android + TUI) erzeugt **Komplexität die wir nicht brauchen** — Flutter-Port adressiert Mobile bereits.
- Bundle-Footprint im Mid-Size-Bereich tendenziell größer als Leptos/Sycamore wegen VDOM-Runtime, trotz Splitting.
- 599 offene Issues (vs. 77 bei Leptos, 33 bei Sycamore, 82 bei Yew) → höheres Bug-Rauschen, aber auch breitere Nutzerbasis.

### Sycamore 0.9 (sycamore-rs)
**Pro**
- Konzeptionell sehr nah an Leptos (Solid-style Signals), v0.9 Reactivity-v3 macht Signals `'static + Copy` — sauberstes Ergonomic-Modell der vier.
- Compile-time-checked Templates.
- Kompakte WASM-Bundles historisch.
- Kleinerer API-Surface — leichter komplett zu überblicken.

**Contra**
- **K.o.-relevant: Maintenance-Lücke**. v0.9.0 erschien November 2024 **nach >2 Jahren Pause** seit v0.8.2. Letzter Release v0.9.2 ist September 2025 — über 8 Monate vor Stand dieser Recherche. Der Maintainer hat „regelmäßigeren Release-Zyklus" zugesagt, die Datenlage erfüllt das aber bisher nicht.
- Eigene View-Syntax (nicht JSX-ähnlich), Closures müssen in `(...)` gewrappt werden.
- **Kein dokumentiertes Hash-Routing**, nur `HistoryIntegration`.
- Kleinere Community (3.3k Stars, 626 Discord-Mitglieder per v0.9-Ankündigung).
- Komponentenbibliotheken/Ökosystem deutlich schmaler als Leptos/Dioxus.

### Yew 0.23 (yewstack)
**Pro**
- Größte Community-Mindshare (32.6k Stars).
- VDOM/Hooks-Modell ist React-vertraut → niedrige Lernkurve für Web-Entwickler.
- Mit Trunk ausgereift und stabil.
- **Bestes Hash-Routing-Story der vier**: offizieller `HashRouter` (für genau unseren Use-Case dokumentiert: „last resort … for static hosting").
- Reife Doku, viele Tutorials.

**Contra**
- Releases vergleichsweise selten (0.23 im März 2025 ist letzter Stand).
- VDOM-Overhead → tendenziell größere Bundles und langsamere Updates (Krausest: deutlich langsamer als Leptos/Dioxus/Sycamore).
- Kein Erste-Klasse-Component-Lib, ARIA muss von Hand verdrahtet werden.
- Yew wurde „im SPA-Zeitalter" entwickelt — die Roadmap wirkt eher pflege- als wachstumsorientiert. Strategisch der konservativste, aber auch stagnierendste Pick.
- Keine WASM-Splitting-Story vergleichbar mit Leptos `#[lazy]` oder Dioxus `wasm-split`.

## Empfehlung mit Begründung

**Leptos 0.8** ist die richtige Wahl. Der Use-Case — ein Calculator mit häufigen, **lokal begrenzten** Updates des Display-Buffers und einer ansonsten statischen Tastatur — ist genau das Szenario, in dem Fine-grained-Reactivity gegen VDOM-Diffing einen messbaren Vorteil hat. Die Codebase ist 2025/2026 die aktivste im Ökosystem (monatliche Patch-Releases, 5.259 Commits, klare Roadmap), und `radix-leptos-primitives` liefert WCAG-2.1-AA-Komponenten falls Modal/Dialog/Focus-Trapping benötigt wird (12-Kapitel-Info-Modal). Canvas-/SVG-Insets für die Dozenal-Glyphen sind über `NodeRef<Canvas>` + `web-sys` idiomatisch — genau das, was die bestehende egui-Painting-Logik in Trait-Methoden mit `wasm-bindgen` umsetzen würde.

Der einzige relevante Schwachpunkt — **fehlendes Hash-Routing** — ist mit dem in der GitHub-Community etablierten `404.html`-Redirect-Trick adressierbar (oder durch einen schlanken Custom-Hash-Router über `web_sys::window().location().hash()` und ein `RwSignal<Chapter>`). Für 12 Info-Kapitel ist kein vollständiger Router nötig; ein `enum Chapter { ... }` plus `RwSignal` reicht völlig.

Leptos ist außerdem das Framework, das das Publikum „andere Rust-Entwickler nutzen es gerne als Vorlage" am ehesten erreicht — Stars und Discord-Aktivität bestätigen das.

## Risiken der Empfehlung

Drei Risiken:

1. **Lernkurve Signals**: Wer aus egui kommt, hat zwar reaktives Denken intus, aber Leptos-Signal-Capturing über Closures hat eigene Stolperfallen (`count()` löst Subscription aus, `count.get_untracked()` nicht). Plan: erste Komponente bewusst klein halten, Pattern aus den offiziellen `examples/counter` und `examples/router` übernehmen.

2. **Hash-Routing-Workaround**: Falls die GitHub-Pages-Deployment nach Sub-Page-URLs wie `/dozenal_calc/info/3` aussehen soll, sind 12 Kapitel je eine `index.html`-Kopie nötig (`index_on_404`-Äquivalent) oder ein Hash-Layer. Risiko: ~1 Tag Yak-Shaving im Deploy-Workflow.

3. **Breaking-Changes-Tempo**: Monatliche Releases bedeuten gelegentliche Semver-Breaks (0.7 → 0.8 war einer). Mitigation: `leptos = "=0.8.x"` pinnen und bewusst upgraden.

## Quellen

- Leptos: [leptos.dev](https://leptos.dev/) · [leptos-rs/leptos](https://github.com/leptos-rs/leptos) · [Release v0.8.0](https://github.com/leptos-rs/leptos/releases/tag/v0.8.0) · [Optimizing WASM Binary Size](https://book.leptos.dev/deployment/binary_size.html) · [Issue #2184 – Hash-Routing](https://github.com/leptos-rs/leptos/issues/2184) · [radix-leptos-primitives](https://crates.io/crates/radix-leptos-primitives) · [Canvas Discussion #3490](https://github.com/leptos-rs/leptos/discussions/3490)
- Dioxus: [dioxuslabs.com](https://dioxuslabs.com/) · [DioxusLabs/dioxus](https://github.com/DioxusLabs/dioxus) · [Dioxus 0.7 Release](https://dioxuslabs.com/blog/release-070/) · [Optimizing Guide](https://dioxuslabs.com/learn/0.7/guides/tips/optimizing/) · [DioxusLabs/components (Radix-basiert)](https://github.com/DioxusLabs/components) · [dioxus-static-site Template](https://github.com/srid/dioxus-static-site)
- Sycamore: [sycamore.dev](https://sycamore.dev/) · [sycamore-rs/sycamore](https://github.com/sycamore-rs/sycamore) · [Announcing v0.9.0](https://sycamore.dev/post/announcing-v0-9-0) · [Router Book](https://sycamore.dev/book/router)
- Yew: [yew.rs](https://yew.rs/) · [yewstack/yew](https://github.com/yewstack/yew) · [yewstack/yew_router](https://github.com/yewstack/yew_router) · [HashRouter Doc](https://api.yew.rs/next/yew_router/router/struct.HashRouter.html) · [Release 0.23 Announcement](https://yew.rs/blog) · [Bundle-Size Issue rustmart-yew-example](https://github.com/sheshbabu/rustmart-yew-example/issues/1)
- Vergleichende: [Krausest js-framework-benchmark](https://krausest.github.io/js-framework-benchmark/) · [flosse/rust-web-framework-comparison](https://github.com/flosse/rust-web-framework-comparison) · [LogRocket: Top Rust Web Frameworks](https://blog.logrocket.com/top-rust-web-frameworks/) · [Reintech: Leptos vs Yew vs Dioxus 2026](https://reintech.io/blog/leptos-vs-yew-vs-dioxus-rust-frontend-framework-comparison-2026) · [Medium: 5 Best Rust Frontend 2026](https://medium.com/@priya.raimagiya/the-5-best-rust-frontend-frameworks-reviewed-for-2026-4694dad8f181) · [Medium: Syntax Comparison](https://medium.com/@vedantpandey/leptos-vs-dioxus-vs-sycamore-vs-svelte-part-1-syntax-comparison-c58ed631896c)
- Allgemein: [wasm-bindgen canvas example](https://rustwasm.github.io/docs/wasm-bindgen/examples/2d-canvas.html) · [Rust Wasm Book – Code Size](https://rustwasm.github.io/book/game-of-life/code-size.html) · [GitHub Discussion #36010 – BrowserRouter + GH Pages](https://github.com/orgs/community/discussions/36010)
