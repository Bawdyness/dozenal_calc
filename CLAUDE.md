# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Identity

Dozenal Calculator — a scientific calculator that natively computes in base 12, with custom-designed digit symbols. The project's character is **minimalist and didactic**: it should never feel overloaded, even at the cost of fewer features. The custom digit symbols already place a cognitive demand on first-time users; the UI must not add a second hurdle.

Key design philosophy:
- **Progressive disclosure**: rare/advanced functions live behind an expansion overlay, not on the main keypad.
- **Symmetry and rhythm**: the layout uses 5 sets of 4 keys each (Sets 1–5), plus a full-width `=` key at the bottom. The expansion overlay mirrors this exact structure with Sets 6–10.
- **No CAS bloat**: this is not a TI-Nspire competitor. Statistics, complex numbers, programming, and unit conversions are explicit non-goals for v1.

## Layout Invariants

These are hard structural rules. They define the project's visual identity and are not subject to incremental drift.

- **Sets are strictly 4-key groups.** Always. No 3-key sets, no 5-key sets, no exceptions.
- **The `=` bar at the bottom of the main keypad is a standalone component, not part of any set.** It does not count toward Set 5.
- **The overlay has no separate full-width bottom bar.** Close lives in position 10.4. The visual symmetry to the `=` bar is intentionally broken: the overlay is a *selection* mode, not a *trigger* mode, and a vollwidth bar would imply a primary action it doesn't have.
- **Overlay depth is exactly one level.** There is the main keypad and exactly one expansion overlay. A second overlay (nested overlays with further sets) is explicitly not a design goal and will not be implemented. Content that would need more space is either dropped or moved to other expression forms (info modal, README, tooltips). This boundary is constitutive for the minimalist character of the project.

## Commands

```
# Run desktop app
cargo run

# Run tests
cargo test

# Local web dev server (requires `trunk` installed: cargo install trunk)
trunk serve

# Production web build (deployed to GitHub Pages)
trunk build --release --public-url /dozenal_calc/

# Format code
cargo fmt --all

# Lint with strict rules
cargo clippy --all-targets --all-features -- -D warnings
```

Tests live only in `src/logic.rs`. To install trunk: `cargo install trunk`.

## Architecture

Two source files, two build targets (native desktop + WebAssembly via Trunk).

**`src/logic.rs`** — pure data layer:
- `DozenalDigit` enum: D0–D11 representing base-12 digits
- `DozenalConverter`: converts between `Vec<DozenalDigit>` and `f64` (`to_decimal` / `from_decimal`)
- `Rational`: exact rational arithmetic for the periodic-decimal track (see "Periodic Decimals" below)

**`src/main.rs`** — everything else:
- `CalcToken` enum: all possible button types (digits, arithmetic, trig, specials like `Expand`, `TriangleLeft/Right`, `Negate`)
- `DozenalCalcApp` struct: holds `input_buffer: Vec<CalcToken>`, `result_buffer: Vec<CalcToken>`, `cursor_pos`, `result_cursor_pos`, `memory`, `last_ans: Option<Rational>`, `error_msg`, `overlay_open: bool`, `angle_mode: AngleMode`, `display_base: DisplayBase`
- `handle_click()`: routes button presses; double-clicking a function key toggles it to its inverse where applicable
- `calculate_result()`: runs two parallel evaluation tracks — the f64 track via `meval` and the rational track via `Rational` arithmetic
- `draw_desktop_layout()` / `draw_mobile_layout()`: two separate egui layouts, switched at 500 px width in the `update()` loop
- `draw_overlay()`: renders the expansion overlay over the dimmed main keypad
- `paint_dozenal_digit()`: custom vector drawing for each digit
- `paint_token()`: draws all non-digit buttons as geometric shapes or text

**Calculation pipeline**: `input_buffer` → (a) string expression for `meval` → `f64` → dozenal `result_buffer`; in parallel (b) `Rational` evaluation → `Option<Rational>`. If (b) succeeds, the display shows the periodic representation with overline; otherwise the f64 result is shown with 4 dozenal fractional digits.

**Web entry point**: `index.html` uses Trunk's `<link data-trunk rel="rust">` directive; canvas id `the_canvas_id` must match the string in `main()`.

**CI**: pushing to `main` triggers `.github/workflows/deploy.yml`, which builds with Trunk and deploys to GitHub Pages.

## Math Conventions

**`acot` (arc cotangent)**: Convention A — range `(0, π)`, formula `π/2 − atan(x)`. This yields positive results for all real x, including negative inputs (e.g. `acot(−1) = 3π/4`). Convention B (`atan(1/x)`, range `(−π/2, π/2]`) is explicitly rejected.

**`cot`** and **`acot`** are registered as custom functions via `meval::Context` in `calculate_result()`. They are not meval builtins. The tuple `(ctx, meval::builtin())` merges the custom context with all standard meval functions and constants (`sin`, `cos`, `pi`, etc.).

**Hyperbolic inverses (`arsinh`, `arcosh`, `artanh`, `arcoth`)**: standard principal-value definitions. `arcoth(x) = (1/2) * ln((x+1)/(x-1))` defined for `|x| > 1`; outside this range the f64 track produces NaN and the result is shown as an error.

## Periodic Decimals

The calculator displays periodic dozenal expansions with an overline (Periodenstrich) over the repeating digits. This is a defining didactic feature: in base 12, fractions like `1/5`, `1/7`, `1/B` reveal periodic structures that are different from base 10 and worth showing visibly.

**Parallel evaluation**: On `=`, the `input_buffer` is evaluated twice — once via `meval` to `f64` (existing path), and once via the rational track to `Option<Rational>`. If the rational track returns `Some(p/q)`, the display is generated from the exact fraction. If `None`, the f64 result is displayed with 4 dozenal fractional digits and no overline.

**What stays rational**:
- Integer dozenal literals (`5`, `B`, `100`)
- Finite-fractional dozenal literals (`0.6` = 6/12, `1.16` = 1 + 1/12 + 6/144)
- The four basic operations (`+`, `−`, `*`, `/`)
- Parentheses and negation
- Integer powers (positive or negative integer exponent)
- The `⊕` operator on rational operands (`(a*b)/(a+b)` stays rational)

**What collapses the rational track to `None`**:
- Any transcendental function (`sin`, `cos`, `tan`, `cot`, their inverses, hyperbolics, `log`, `ln`, exponentials)
- Irrational roots (e.g. `√2`, `√n` where n is not a perfect square)
- The constants π, e, φ, √2 (Set 7 keys)
- Division by zero
- `i128` overflow in any intermediate step (use `checked_*` arithmetic throughout)

**Datatype**: `Rational { num: i128, den: i128 }` defined in `logic.rs`. Always reduced via gcd, with `den > 0` enforced. All operations use `checked_add`, `checked_sub`, `checked_mul`, `checked_div`. Failure to compute (overflow, divide by zero) returns `None` and collapses the track.

**Period detection**: classical school algorithm over remainders. After reducing `p/q`, the integer part is `p / q`, and the fractional part is computed by repeatedly multiplying the remainder by 12 and recording the remainders seen. Two outcomes:
- A remainder of 0 appears → finite expansion. `period_len = 0`.
- A remainder repeats → period found. `pre_len` and `period_len` are derived from the position of first repetition.

**Display rules** (only when the rational track succeeds):
- Finite (`period_len = 0`): standard display, no overline.
- Period 1–5 digits: pre-period digits shown normally, the entire period rendered with an overline above all of its digits.
- Period > 5 digits: pre-period digits shown normally, the first 5 period digits rendered with overline, followed by `…` to indicate continuation.
- The overline never extends over pre-period digits — only over the actual repeating cycle.

**Input scope**: the overline is output-only. The user has no input mechanism for typing periodic decimals (no Periodenstrich-key on either keypad).

**`Ans` interaction**: when `Ans` is inserted into a new calculation (either explicitly via Set 6.4 or implicitly via auto-insertion, see "Cursor and Ans behavior"), it must carry the exact `Rational` from the previous result, not the f64 approximation. Periodicity is preserved through chained calculations as long as the rational track stays alive.

**Layer separation**: `Rational` arithmetic and period detection live in `logic.rs`. The overline rendering is `main.rs`'s responsibility (extension of the result-display routine in `paint_token` or a dedicated draw function).

## Layout Architecture

The keypad uses a **5-set grid** that is preserved in both desktop and mobile modes, and mirrored in the expansion overlay (Sets 6–10).

### Main keypad (Sets 1–5)

| Set | Position 1 | Position 2 | Position 3 | Position 4 |
|---|---|---|---|---|
| 1 — Arithmetic | Add | Sub | Mul | Div |
| 2 — Special operators | ⊕ | x² | √ | log |
| 3 — Trigonometry | sin | cos | tan | cot |
| 4 — Parentheses & cursor | ( | ) | ◀ | ▶ |
| 5 — System | AC | Del | . | Expand |

Plus the full-width `=` key at the very bottom (not part of any set).

**Mobile layout**: Sets 1–4 are arranged vertically side by side; Set 5 sits horizontally below them with some spacing. `=` is full-width at the bottom.

**Desktop layout**: All 5 sets are arranged vertically side by side. `=` is full-width at the bottom.

### Expansion overlay (Sets 6–10)

The overlay opens when `Expand` (5.4) is pressed. It uses **exactly the same spatial arrangement as the main keypad** — not just the same key count, but the same layout logic per platform:

- **Desktop overlay**: Sets 6–10 arranged vertically side by side, exactly as Sets 1–5 are on desktop.
- **Mobile overlay**: Sets 6–9 arranged vertically side by side, Set 10 sitting horizontally below them with the same spacing used for Set 5 in the mobile main keypad.

This is a hard requirement. The overlay must be a visual mirror of the main keypad at all times — same proportions, same axis, same breakpoint behavior. Any implementation that arranges overlay sets differently from main keypad sets (e.g. all-horizontal on desktop, or all-horizontal on mobile) is incorrect.

Close lives in position 10.4 — there is no separate full-width bar in the overlay.

| Set | Position 1 | Position 2 | Position 3 | Position 4 |
|---|---|---|---|---|
| 6 — Memory | STO | RCL | MC | Ans |
| 7 — Constants | π | e | φ | √2 |
| 8 — Hyperbolic | sinh | cosh | tanh | coth |
| 9 — Extended | n! | \|x\| | 1/x | mod |
| 10 — Modes & Meta | Doz↔Dec | DRG | Info | Close |

**Doppelklick-Inversen im Overlay**:
- Set 8 (Hyperbolic): `sinh ↔ arsinh`, `cosh ↔ arcosh`, `tanh ↔ artanh`, `coth ↔ arcoth`
- Set 9: no inverses currently — these double-click slots are intentional reserve for future expansion without changing the layout.

### Visual behavior of the overlay

- **Background**: semi-transparent black (`Color32::from_black_alpha(180)` or similar) over the main keypad area. The main layout dims and remains visible for context.
- **Overlay keys themselves**: fully opaque, sharply rendered, same look-and-feel as main keys.
- **Click routing**: while the overlay is open, the main keypad is inactive. Only overlay keys respond.
- **Display and input area**: remain visible and unaffected — the user can see what they're entering while choosing an overlay function.

### The Info Modal (10.3)

Position 10.3 opens an info modal containing didactic content about dozenality. The modal is a self-contained UI element, **not** a second overlay with keys. It contains:
- Mathematical curiosities specific to base 12 (F(12) = 144 = 12², highly composite, abundant, short fractions)
- Selected constants in base 12 with their first ~14 digits
- Brief usage hints (double-click for inverses, cursor navigation)
- A close button

The modal scrolls vertically if content exceeds viewport height. It has no calculator buttons — it is a reading surface, not an interactive layer.

## Interaction Conventions

### Cursor and Ans behavior

The arrow keys `◀`/`▶` (positions 4.3 and 4.4) operate on the **active field**. Field activity follows the calculation lifecycle:

- **Before `=`**: the input field is active. Arrows move the input cursor through `input_buffer`.
- **After `=`**: the result field is active. Arrows move the result cursor through `result_buffer`.
- **On any new input** (digit, operator, function, AC, Del — anything that modifies `input_buffer`): activity returns to the input field.

This is one rule, applied uniformly. No mode toggle, no dedicated activity-switch key.

**Ans auto-insertion**: After a successful `=`, if the user's next click is an *operator* (`+`, `-`, `*`, `/`, `⊕`, `^`, `√`, `log`, `x²`), `Ans` is automatically inserted as the first token of a fresh `input_buffer`, followed by the operator. The user sees `Ans -` (or similar) appear and continues typing. This applies to `-` as well — there is no ambiguity check; `-` after `=` always means "minus from Ans". Users who want to start a new negative number after a result must press `AC` first.

If the user instead clicks a digit, constant, opening parenthesis, or function (sin, cos, …) after `=`, the `input_buffer` is started fresh without auto-Ans.

The explicit `Ans` key (6.4) remains available and is *not* redundant: it allows inserting Ans at any position in an expression (e.g. as a second operand: `5 + Ans`), not only at the start.

### Double-click for inverse functions

A second click on a function key replaces the just-inserted token with its inverse. Established for trigonometry and extended to all overlay functions where an inverse exists:

- Trig (Set 3): `sin ↔ asin`, `cos ↔ acos`, `tan ↔ atan`, `cot ↔ acot`
- Hyperbolics (Set 8): `sinh ↔ arsinh`, `cosh ↔ arcosh`, `tanh ↔ artanh`, `coth ↔ arcoth`
- Set 9 keys (n!, |x|, 1/x, mod): no inverses currently — slots reserved for future use.

### Visual indicator for "armed" inverse mode

When the previous token in `input_buffer` is the same function (so the next click would toggle to inverse), the corresponding key shows a subtle visual marker (small dot or thin border accent — exact form TBD, see DESIGN_NOTES.md). The label itself does not change.

### Error state behavior

When `error_msg` is `Some(...)`:
- All keys except `AC` are inactive (overlay-open state included).
- `AC` clears the error and the input/result buffers.
- `error_msg` is reset to `None` on any successful calculation and on `AC`.

## Memory Model

- One memory slot. STO stores, RCL recalls, MC clears, Ans holds the last result.
- Memory does NOT auto-accumulate (Casio-style M+ is rejected).
- The Memory indicator (`M` shown top-left of display when memory is non-empty) reflects this single slot.
- Memory stores the exact `Rational` when available, falling back to `f64` otherwise — so periodicity survives a STO/RCL roundtrip.
- A future v2 may introduce multiple slots if user demand arises; this is *not* a v1 feature.

## Display Conventions

- Default precision: **4 dozenal fractional digits** for f64 results.
- Periodic decimals (when the rational track succeeds): see "Periodic Decimals" section above.
- Cursor: red vertical bar inside the active field, navigable with `◀` / `▶`.
- Memory indicator: `M` shown top-left of display when memory is non-empty.
- Angle mode indicator: `DEG` / `RAD` / `GRAD` shown top-right of display, controlled by Set 10.2 (DRG).
- Display base indicator: a subtle marker shown when in decimal mode (Doz is the default; Dec is a temporary inspection mode triggered by Set 10.1).
- Error display: red text replacing the normal display content.

## Code Quality Principles

This project uses **tool-enforced quality** wherever possible. Read this section once, then trust the tools.

### Tool-enforced rules (do not duplicate manually)

The following are handled by `rustfmt.toml`, `clippy.toml`, and `Cargo.toml [lints]`:
- Indentation, line breaks, import ordering, trailing commas → `rustfmt`
- Variable naming, dead code, unused imports, common anti-patterns → `clippy`
- Cognitive complexity, function length warnings → `clippy` (configured thresholds)
- Disallowed macros (`dbg!` etc.) → `clippy`

**Workflow before any commit**:
```
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

If any of the three fails, do not commit. Fix the issues first.

Claude Code MUST run these three commands after each non-trivial patch. The `-D warnings` flag treats warnings as errors, which means clippy actively gatekeeps the code.

### Architectural rules (cannot be enforced by tools)

These are project-specific and require human/AI judgment:

**Layer separation**:
- `logic.rs` is the pure data layer. It must not import `egui`, `eframe`, or any UI crate. It must not contain UI text strings.
- `main.rs` is the UI layer. It may import `logic.rs`, but never the reverse.
- The `Rational` type and period detection belong in `logic.rs`. Overline rendering belongs in `main.rs`.
- When new computational logic is added, it goes into `logic.rs` first, then `main.rs` calls it.

**Single Responsibility, applied loosely**:
- Functions whose names contain "and" usually do two things; consider splitting.
- A function that mixes UI rendering and state mutation (`handle_click` is a deliberate exception) should be reconsidered.
- The `paint_*` functions are pure rendering — keep them that way.

**Magic numbers**:
- Layout breakpoints, default precisions, button sizes — anything that appears more than once or has semantic meaning beyond "this number works" — should be a named constant.
- Example: `if width < 500.0` becomes `const MOBILE_BREAKPOINT_PX: f32 = 500.0;`.

**Code duplication**:
- If the same pattern appears in `draw_desktop_layout` and `draw_mobile_layout`, extract a helper function.

**Comments**:
- Comments explain *why*, not *what*.
- The existing codebase mixes German and English comments. Preserve this style — do not translate or normalize without explicit request.
- Doc comments (`///`) on public items in `logic.rs` are encouraged.

**Refactor policy**:
- When a patch touches an area where existing code could be cleaner, **propose** the refactor — do not silently include it.
- Never refactor `paint_dozenal_digit`, `paint_token`, or the layout proportions without explicit request.

## Coding Conventions for Claude Code

### Compiler errors

- Run `cargo check` **before** starting a patch to know the baseline.
- Run `cargo check` (or the full quality triple above) **after** each patch.
- Do not introduce `unwrap()` or `expect()` without a written justification in the comment above.
- Prefer the smallest possible change.

### Runtime errors (user-visible)

- The app uses `error_msg: Option<String>` for user-visible errors. Never insert `panic!`, `todo!`, or `unimplemented!` into runtime paths.
- Error messages follow a consistent format: short, all caps, descriptive (`"DIV BY ZERO"`, `"SYNTAX ERROR"`, `"OVERFLOW"`).
- Errors must be clearable by the user. Whenever a new error path is added, verify that `AC` clears it.

### Tests

- Tests live in `src/logic.rs` and cover the dozenal/decimal conversion logic plus the rational arithmetic and period detection.
- When changing `logic.rs`: run `cargo test` and ensure all pass. Add tests for new logic, especially edge cases for `Rational` (overflow, divide by zero, finite vs. periodic detection).
- When changing `main.rs`: there are no automated tests. Verify manually with `cargo run` and `trunk serve`.

### Build & deployment

- The CI deploys `main` automatically. Never push directly to `main` without local verification.
- Before pushing: run the quality triple plus `cargo build --release` and `trunk build --release --public-url /dozenal_calc/`.
- For larger changes (overlay implementation, rational track, etc.): use a feature branch and merge via PR.
- The web build uses WebAssembly. Avoid dependencies that don't compile to `wasm32-unknown-unknown`. Pure-Rust crates only. The `Rational` type is implemented from scratch (not via `num-rational`) to avoid dependency creep.

### General code hygiene

- Preserve the `paint_dozenal_digit` and `paint_token` drawing routines as-is unless explicitly asked.
- Preserve the layout proportions in `draw_desktop_layout` and `draw_mobile_layout` unless explicitly asked.
- New `CalcToken` variants must be added to all `match` statements that handle tokens (compiler will complain if forgotten — use this as a guide).
