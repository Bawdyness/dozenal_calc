# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Identity

Dozenal Calculator — a scientific calculator that natively computes in base 12, with custom-designed digit symbols. The project's character is **minimalist and didactic**: it should never feel overloaded, even at the cost of fewer features. The custom digit symbols already place a cognitive demand on first-time users; the UI must not add a second hurdle.

Key design philosophy:
- **Progressive disclosure**: rare/advanced functions live behind an expansion overlay, not on the main keypad.
- **Symmetry and rhythm**: the layout uses 5 sets of 4 keys each, plus a full-width key at the bottom. The expansion overlay mirrors this exact structure.
- **No CAS bloat**: this is not a TI-Nspire competitor. Statistics, complex numbers, programming, and unit conversions are explicit non-goals for v1.

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

**`src/main.rs`** — everything else:
- `CalcToken` enum: all possible button types (digits, arithmetic, trig, specials like `MPlus`, `TriangleLeft/Right`)
- `DozenalCalcApp` struct: holds `input_buffer: Vec<CalcToken>`, `result_buffer: Vec<CalcToken>`, `cursor_pos`, `memory`, `error_msg`
- `handle_click()`: routes button presses; double-clicking a trig function toggles it to its arc-inverse (sin→asin, etc.)
- `calculate_result()`: walks `input_buffer`, converts dozenal digits to decimal values, assembles a string for `meval::eval_str`, converts the `f64` result back into dozenal tokens
- `draw_desktop_layout()` / `draw_mobile_layout()`: two separate egui layouts, switched at 500 px width in the `update()` loop
- `paint_dozenal_digit()`: custom vector drawing for each digit — D0 is a circle, D1/4/7/10 are directional arrows (anchor points), D2–D11 are arc combinations
- `paint_token()`: draws all non-digit buttons as geometric shapes or text

**Calculation pipeline**: `input_buffer` → string expression (dozenal→decimal, special operators expanded inline) → `meval` → `f64` → dozenal `result_buffer`. The `⊕` operator expands to `(a*b)/(a+b)`. Root `√` and `log` operators are positional (left operand is base/degree, right is argument).

**Web entry point**: `index.html` uses Trunk's `<link data-trunk rel="rust">` directive; canvas id `the_canvas_id` must match the string in `main()`.

**CI**: pushing to `main` triggers `.github/workflows/deploy.yml`, which builds with Trunk and deploys to GitHub Pages.

## Math Conventions

**`acot` (arc cotangent)**: Convention A — range `(0, π)`, formula `π/2 − atan(x)`. This yields positive results for all real x, including negative inputs (e.g. `acot(−1) = 3π/4`). Convention B (`atan(1/x)`, range `(−π/2, π/2]`) is explicitly rejected.

**`cot`** and **`acot`** are registered as custom functions via `meval::Context` in `calculate_result()`. They are not meval builtins. The tuple `(ctx, meval::builtin())` merges the custom context with all standard meval functions and constants (`sin`, `cos`, `pi`, etc.).

## Layout Architecture

The keypad is built around a **5-set grid** that is preserved in both desktop and mobile modes, and mirrored in the expansion overlay.

### Main keypad (current)

| Set | Keys |
|---|---|
| 1 — Arithmetic | Add, Sub, Mul, Div |
| 2 — Special operators | ⊕, x², √, log |
| 3 — Trigonometry | Sin, Cos, Tan, Cot |
| 4 — Parentheses & cursor | (, ), ◀, ▶ |
| 5 — System | AC, Del, ., **Expand** |

Plus a full-width `=` key at the very bottom.

**Mobile layout**: Sets 1–4 are arranged vertically side by side; Set 5 sits horizontally below them with some spacing. `=` is full-width at the bottom.

**Desktop layout**: All 5 sets are arranged vertically side by side. `=` is full-width at the bottom.

The `MPlus` token in the current code is to be **replaced** by an `Expand` token. M+ no longer exists as a top-level key — all memory functions live inside the expansion overlay.

### Expansion overlay (planned, 20 keys + close key)

The overlay opens when `Expand` is pressed. It uses the **same 5-set grid structure** as the main keypad. The overlay's full-width bottom key is the **Close** key (mirrors the position of `=`).

Set contents are still under design (see DESIGN_NOTES.md). Confirmed structural decisions:
- Memory set: STO, RCL, MC, Ans (Memory functions follow TI-30 eco RS conventions, scaled down).
- Other sets: under deliberation. Likely candidates include hyperbolic functions, constants, and didactic dozenal-specific tools.

### Visual behavior of the overlay

- **Background**: semi-transparent black (`Color32::from_black_alpha(180)` or similar) over the main keypad area. The main layout dims and remains visible for context.
- **Overlay keys themselves**: fully opaque, sharply rendered, same look-and-feel as main keys.
- **Click routing**: while the overlay is open, the main keypad is inactive. Only overlay keys (and the close key) respond.
- **Display and input area**: remain visible and unaffected — the user can see what they're entering while choosing an overlay function.

## Interaction Conventions

### Double-click for inverse functions

A second click on a function key replaces the just-inserted token with its inverse. This pattern is established for trigonometry (`sin → asin`, `cos → acos`, etc.) and **must be extended** to all overlay functions where an inverse exists:

- Hyperbolics: `sinh ↔ arsinh`, `cosh ↔ arcosh`, `tanh ↔ artanh`, `coth ↔ arcoth`
- Logarithms/exponentials: where applicable (e.g. `log ↔ 10ˣ`, `ln ↔ eˣ`)
- Powers/roots: where applicable (e.g. `x² ↔ √x`)

Rationale: nobody actually computes `sin(sin(x))`, so the second-click slot is free real estate. This **doubles the available functions per key** without adding visual complexity.

### Visual indicator for "armed" inverse mode

When the previous token in `input_buffer` is the same function (so the next click would toggle to inverse), the corresponding key should show a **subtle visual marker** — for example a small dot or a thin border accent. The label itself does NOT change. The marker is a passive affordance, not a mode switch.

### Error state behavior

When `error_msg` is `Some(...)`:
- All keys except `AC` (and possibly `Del`) are **inactive**.
- `AC` clears the error AND the input/result, returning to a clean state.
- `error_msg` is reset to `None` on any successful calculation, on `AC`, and on entry of any new digit (TBD — see DESIGN_NOTES.md).

The current code only resets `error_msg` on successful calculation, which is the root cause of the "error won't go away" bug.

## Memory Model

Following the TI-30 eco RS philosophy, but condensed:
- Memory operations live exclusively in the expansion overlay (no top-level memory keys).
- Memory does NOT auto-accumulate (M+ Casio-style is rejected). Memory is explicitly stored via `STO` and recalled via `RCL`.
- The `Ans` token holds the last successful result and can be inserted into a new calculation.
- `MC` clears the memory.

Number of memory slots is still under deliberation (see DESIGN_NOTES.md). Likely 1 or 3.

## Display Conventions

- Default precision: **4 dozenal fractional digits** for computed results.
- Cursor: red vertical bar inside the input field, navigable with `◀` / `▶`.
- Memory indicator: `M` shown top-left of display when memory is non-empty.
- Error display: red text replacing the normal display content.
- (Planned) For irrational constants, a horizontal-scroll mode showing precomputed high-precision strings (e.g. ~1000 dozenal digits) — implemented via hardcoded constants, not a runtime arbitrary-precision engine. See DESIGN_NOTES.md.

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
- When new computational logic is added, it goes into `logic.rs` first, then `main.rs` calls it. If a function in `main.rs` does math beyond trivial arithmetic for layout, that's a smell — propose moving it.

**Single Responsibility, applied loosely**:
- Functions whose names contain "and" usually do two things; consider splitting.
- A function that mixes UI rendering and state mutation (`handle_click` is a deliberate exception) should be reconsidered.
- The `paint_*` functions are pure rendering — keep them that way. They take state as input and produce visual output, never mutate.

**Magic numbers**:
- Layout breakpoints, default precisions, button sizes — anything that appears more than once or has semantic meaning beyond "this number works" — should be a named constant near the top of the relevant file or in a `const` block.
- Example: `if width < 500.0` becomes `const MOBILE_BREAKPOINT_PX: f32 = 500.0;` then `if width < MOBILE_BREAKPOINT_PX`.

**Code duplication**:
- If the same pattern appears in `draw_desktop_layout` and `draw_mobile_layout`, extract a helper function that both call. The current code has some duplication (e.g. button rendering loops); reducing it is welcome when touching those areas.

**Comments**:
- Comments explain *why*, not *what*. `// increment counter` above `i += 1` is noise. `// We start at 1 because index 0 is the header` is signal.
- The existing codebase mixes German and English comments. Preserve this style — do not translate or normalize without explicit request.
- Doc comments (`///`) on public items in `logic.rs` are encouraged.

**Refactor policy**:
- When a patch touches an area where existing code could be cleaner, **propose** the refactor — do not silently include it. The user decides whether the cleanup belongs in the current patch or a separate one.
- "Functional but ugly" is not "fine". Existing rough spots are candidates for cleanup when you're already in the neighborhood — but ask first.
- Never refactor `paint_dozenal_digit`, `paint_token`, or the layout proportions without explicit request. Those define the project's visual identity.

## Coding Conventions for Claude Code

These are operational rules for Claude Code when modifying this repository.

### Compiler errors

- Run `cargo check` **before** starting a patch to know the baseline.
- Run `cargo check` (or the full quality triple above) **after** each patch to verify nothing was broken.
- Read the full error message before acting. Rust's compiler errors are precise — use them, don't guess.
- Do not introduce `unwrap()` or `expect()` without a written justification in the comment above. The existing code uses idiomatic `Option`/`Result` handling — preserve that style.
- Prefer the smallest possible change. If a fix can be 3 lines, do not refactor 30.

### Runtime errors (user-visible)

- The app uses `error_msg: Option<String>` for user-visible errors. Never insert `panic!`, `todo!`, or `unimplemented!` into runtime paths.
- Error messages follow a consistent format: short, all caps, descriptive (`"DIV BY ZERO"`, `"SYNTAX ERROR"`). New error types must follow this convention.
- Errors must be clearable by the user. Whenever a new error path is added, verify that `AC` (and other reset paths) clear it.

### Tests

- Tests live in `src/logic.rs` and cover the dozenal/decimal conversion logic.
- When changing `logic.rs`: run `cargo test` and ensure all pass. Add tests for new logic.
- When changing `main.rs`: there are no automated tests. Verify manually with `cargo run` (desktop) and `trunk serve` (web). Both targets must work — never break one to fix the other.
- Pure functions in `main.rs` may be moved to `logic.rs` if they would benefit from testing.

### Build & deployment

- The CI deploys `main` automatically via `.github/workflows/deploy.yml`. **Never push directly to `main` without local verification.**
- Before pushing: run the quality triple (`fmt`, `clippy`, `test`) plus `cargo build --release` and `trunk build --release --public-url /dozenal_calc/`. If any fails, do not push.
- For larger changes (overlay implementation, memory refactor, etc.): use a feature branch and merge via PR after local verification.
- The web build uses WebAssembly. Avoid dependencies that don't compile to `wasm32-unknown-unknown` (notably anything requiring a C toolchain like GMP). Pure-Rust crates only.

### General code hygiene

- Preserve the `paint_dozenal_digit` and `paint_token` drawing routines as-is unless explicitly asked. The visual identity (custom digit symbols) is the project's signature.
- Preserve the layout proportions in `draw_desktop_layout` and `draw_mobile_layout` unless explicitly asked. Do not "improve" the design.
- New `CalcToken` variants must be added to all `match` statements that handle tokens (compiler will complain if forgotten — use this as a guide).
