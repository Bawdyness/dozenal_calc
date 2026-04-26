# Cargo.toml — Lints-Sektion zum Einfügen

Füge den folgenden Block am ENDE deiner bestehenden `Cargo.toml` ein
(nach allen anderen Sektionen wie `[package]`, `[dependencies]` etc.).

Diese Sektion aktiviert strenge Lint-Regeln auf Projekt-Ebene.
Sie wirkt zusammen mit `clippy.toml` und `rustfmt.toml`.

---

```toml
[lints.rust]
# Warnt bei ungenutztem Code, fehlenden Docs auf öffentlichen Items, etc.
unsafe_code = "forbid"
unused = { level = "warn", priority = -1 }

[lints.clippy]
# "pedantic" aktiviert eine Reihe sinnvoller Zusatz-Lints.
# "nursery" sind experimentelle Lints — nützlich, aber gelegentlich falsch positiv.
# Beide auf "warn" statt "deny", damit man sie sehen aber notfalls überschreiben kann.
pedantic = { level = "warn", priority = -1 }
nursery  = { level = "warn", priority = -1 }

# Diese hier sind explizit AUS, weil sie für ein UI-Projekt zu pingelig sind:
module_name_repetitions = "allow"   # Erlaubt z.B. logic::DozenalConverter
similar_names = "allow"             # Erlaubt z.B. int_digits / int_val
cast_precision_loss = "allow"       # f64 ↔ u64 Konvertierungen sind hier OK
cast_possible_truncation = "allow"  # Ditto
cast_sign_loss = "allow"            # Ditto
many_single_char_names = "allow"    # paint_token nutzt c, s, p, q absichtlich
```

---

## Was die Einstellungen bewirken

- `unsafe_code = "forbid"` → niemand darf in diesem Projekt `unsafe`-Blöcke einbauen. Das ist für einen Taschenrechner sowieso unnötig und verhindert eine ganze Klasse von Fehlern.
- `pedantic = "warn"` → aktiviert ~70 zusätzliche clippy-Lints, die "Best Practices" durchsetzen. Beispiele: ineffiziente String-Allocation, vermeidbare Klone, redundante Closures.
- `nursery = "warn"` → noch experimentellere Lints. Manche davon sind gelegentlich übereifrig — aber meist sehr lehrreich.
- Die `allow`-Einträge schalten gezielt ein paar Lints ab, die für dein UI-Projekt zu nervig wären (etwa: clippy würde sonst meckern, dass `c` zu kurz als Variablenname ist, obwohl es in Grafik-Code idiomatisch ist).

## Test nach dem Einfügen

Sobald die `Cargo.toml` ergänzt ist:

```
cargo clippy --all-targets --all-features -- -D warnings
```

Beim ersten Lauf wird clippy wahrscheinlich eine Reihe von Warnungen melden — das ist normal und erwartet. Du kannst dann mit Claude Code zusammen entscheiden, welche du fixen willst und welche du akzeptierst (über `#[allow(clippy::xyz)]`-Attribute im Code).

**Wichtig**: das ist keine Pflicht-Änderung. Wenn du erst sehen willst, was passiert, kannst du auch nur `clippy.toml` und `rustfmt.toml` einfügen und die `Cargo.toml` so lassen wie sie ist. Dann ist clippy weniger streng, aber immer noch nützlich.
