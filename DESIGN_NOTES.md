# DESIGN_NOTES.md

A working journal of design discussions for the Dozenal Calculator.

This file is **not** authoritative — `CLAUDE.md` is. This file captures *open questions*, *deliberations in progress*, and *ideas under consideration*. Once a decision is finalized, it moves to `CLAUDE.md` and the corresponding section here is marked as resolved.

## Status of known bugs

| Bug | Status | Notes |
|---|---|---|
| Brightness too high on light system theme | ✅ Fixed by user | — |
| Error message can't be dismissed | ✅ Fixed (`code-quality-pre-overlay`) | Guard in `handle_click` + `AC` clears `error_msg`. |
| M+ not fully functional | ✅ Resolved by design | M+ removed entirely; Set 6 covers memory with STO/RCL/MC/Ans (single slot). |
| `cot` / inverse trig display unwieldy | ✅ Fixed (`code-quality-pre-overlay`) | Display width for Arc tokens bumped to 65 px; labels now `sin⁻¹` etc. consistent with buttons. Remaining sub-issue: visual "armed inverse" marker (open). |
| Cot expression syntactically incorrect | ✅ Fixed (`code-quality-pre-overlay`) | `cot`/`acot` registered as custom meval functions. |
| ArcCot convention undefined | ✅ Fixed (`code-quality-pre-overlay`) | Convention A pinned (`acot(x) = π/2 − atan(x)`, range `(0,π)`). |
| `√` mid-expression uses wrong operand | ✅ Fixed (`code-quality-pre-overlay`) | Operator-context detection. |
| Unary minus not supported | ✅ Confirmed working (`code-quality-pre-overlay`) | meval handles unary minus natively. |
| Integer overflow for large results | ✅ Fixed (`code-quality-pre-overlay`) | `from_decimal` now uses f64 arithmetic throughout. |
| Negative results break re-insertion | ✅ Fixed (`code-quality-pre-overlay`) | `CalcToken::Negate` added; negative results stored as `[Negate, digits…]`. |

## Resolved design questions (moved to CLAUDE.md)

The following were under deliberation and have now been finalized. Decisions live in `CLAUDE.md`.

### ✅ Set nomenclature and overlay structure

- Sets 1–5 are the main keypad, Sets 6–10 are the overlay.
- Sets are strictly 4 keys each. The `=` bar is a standalone component, not part of any set.
- The overlay has no full-width bottom bar — Close lives in 10.4.
- Overlay depth is exactly one level. No nested overlays. This is a designed-in invariant, not an implementation limitation.

### ✅ Overlay set contents

| Set | Position 1 | Position 2 | Position 3 | Position 4 |
|---|---|---|---|---|
| 6 — Memory | STO | RCL | MC | Ans |
| 7 — Constants | π | e | φ | √2 |
| 8 — Hyperbolic | sinh | cosh | tanh | coth |
| 9 — Extended | n! | \|x\| | 1/x | mod |
| 10 — Modes & Meta | Doz↔Dec | DRG | Info | Close |

Double-click inverses available in Sets 3 (trig) and 8 (hyperbolic). Set 9 has no inverses currently — these slots are intentional reserve.

### ✅ Memory model

One memory slot. STO/RCL/MC/Ans, no auto-accumulation. A multi-slot model would have required either a "wait for slot number" interaction state or a slot-rotation mechanic on existing keys; both were rejected as adding cognitive load disproportionate to benefit. Multi-slot is a possible v2 feature.

### ✅ Cursor behavior

A single rule: arrows operate on the active field. Active field is the input before `=`, the result after `=`, and switches back to input on any new entry. No mode toggle, no dedicated activity-switch key.

### ✅ Ans auto-insertion

After `=`, any operator click prepends `Ans` to a fresh `input_buffer`. Includes `-` (no ambiguity check — convention is "minus from Ans"; users wanting a fresh negative number press AC first). Digits, constants, parentheses, and functions start a fresh buffer without auto-Ans. The explicit Ans key (6.4) remains available for inserting Ans mid-expression.

### ✅ Periodic decimals (replaces former "scrollable constants" idea)

The earlier proposal to hardcode ~1000-digit dozenal strings for irrational constants and scroll through them is **discarded**. Justification: too few benefits relative to complexity, and the truly interesting didactic content (periodicity in base 12) is better served by a feature that works for *any* rational result, not just hand-picked constants.

Replacement: a parallel rational evaluation track produces an `Option<Rational>` alongside the f64 result. When the rational track succeeds, the display shows the periodic representation with overline. Cap at 5 period digits with `…` for longer periods. Full spec in `CLAUDE.md`.

### ✅ Info Modal at position 10.3

A self-contained reading surface for didactic content. Not a second overlay. Contains mathematical curiosities, constants in base 12, brief usage hints. Plain text + static tables, scrolls vertically, has only a close button.

## Open design questions

### 1. Visual indicator for "armed" inverse mode

Confirmed in concept (small marker, not a label change). Concrete form still open:
- A small filled dot near a corner of the key?
- A thin colored border?
- A subtle background tint?

To be decided when the patch is written. Should be consistent across all keys with inverse capability (trig + overlay hyperbolics).

### 2. DRG cycle behavior and default

Set 10.2 cycles through Deg → Rad → Grad → Deg. Open:
- What is the default at app launch? Likely **Rad** (current implicit behavior, since meval is radian-native), but **Deg** would be friendlier for casual users. Decision pending.
- Where exactly is the mode indicator placed? Top-right of display, but the visual treatment (small text, a colored chip, a single letter D/R/G) is undecided.
- Does changing the mode mid-expression re-interpret already-typed numbers, or only affect new input? Recommended: only affects new input, but worth confirming.

### 3. Doz↔Dec toggle: scope and behavior

Set 10.1 toggles between dozenal and decimal display. Open:
- Does it switch *both* input and result fields, or only the result field?
- Does it persist across calculations, or revert to dozenal on next `=`?
- How are dozenal-only digits (8, 9, A, B) handled in the input field when the user is in decimal mode? Options: (a) those keys are inactive in dec mode, (b) typing them auto-flips back to doz, (c) they remain active and the dec display shows their decimal equivalent.

Likely answer: toggle is a *display* layer, not an input-mode change. Internal storage is always exact (f64 + Rational); display rendering is base-aware. This implies the dozenal-only digit keys remain active and produce the same internal value regardless of display mode. To be confirmed.

### 4. Info Modal — UI and content

✅ Resolved. Navigation is List → Detail → Back: tapping "Info" shows 12 chapter titles; tapping a title opens the full chapter text (scrollable); a back button returns to the list. Complete text in `INFO_MODAL_CONTENT.md`. Hard limit of 12 chapters. Full spec in `CLAUDE.md` under "The Info Modal (10.3)".

### 5. Hyperbolic domain errors

✅ Resolved. Out-of-domain inputs (e.g. `arcosh(0)`, `artanh(1)`) produce NaN via the f64 path; `calculate_result()` catches this and sets `error_msg = Some("DOMAIN ERROR")`. Matches the existing error-message style.

## Ideas parked for later

These have been mentioned but are explicitly **not** in scope:

- **Multi-slot memory** (M1, M2, M3) — possible v2 feature.
- **Statistics functions** — out of scope for v1.
- **Complex numbers** — out of scope.
- **Unit conversions** (TGM, metric, imperial) — out of scope.
- **Programming/scripting** — explicitly out of scope.
- **Theme toggle** (dark/light user choice) — not currently planned.
- **Long-precision constant scrolling** — discarded (replaced by periodic decimals).
- **Nested overlays** — explicitly rejected as a designed-in invariant (see CLAUDE.md "Layout invariants").

## Mathematical curiosities specific to dozenal

Captured for use in the Info Modal and as test material:

- **F(12) = 144 = 12²**. The 12th Fibonacci number is the only Fibonacci square besides F(1) = F(2) = 1, F(12) = 144 (Cohn 1964, proven unique).
- **12 is the smallest abundant number**: σ(12) − 12 = 16 > 12.
- **12 is highly composite**: more divisors (6) than any smaller integer.
- **Short fractions in base 12**: 1/2 = 0.6, 1/3 = 0.4, 1/4 = 0.3, 1/6 = 0.2, 1/8 = 0.16, 1/9 = 0.14. Compare base 10 where 1/3 and 1/6 are infinite.
- **Periodic fractions in base 12** (period length is `ord_q(12)` for q coprime to 12):
  - `1/5` = `0.[2497]` (period 4)
  - `1/7` = `0.[186A35]` (period 6, will be displayed capped at 5: `0.[186A3]…`)
  - `1/B` = `0.[1]` (period 1)
  - `1/11` (= dec 13) = `0.[0A35186]…` (period 6, displayed `0.[0A351]…`)
  - `1/15` (= dec 17) period is 16 → displayed capped at 5
- **Constants in base 12** (first ~14 digits):
  - π ≈ 3.184809493B91866...
  - e ≈ 2.875236069821...
  - φ ≈ 1.74BB6772802A4...
  - √2 ≈ 1.4B79170A07B85...

## Workflow

This project uses a parallel workflow:
- **Claude Desktop App (chat)**: design discussion, concept iteration, documentation.
- **Claude Code (terminal)**: code implementation, with `CLAUDE.md` as the binding instruction set.
- **DESIGN_NOTES.md (this file)**: bridge between the two — captures what's still in flux.

When a decision is finalized in chat, it should be moved into `CLAUDE.md` and removed from this file's "open questions" section.

## Patch order — ✅ all 14 steps completed

1. ✅ Overlay infrastructure
2. ✅ `Rational` type and rational arithmetic
3. ✅ Period detection algorithm
4. ✅ Parallel evaluation track
5. ✅ Overline rendering
6. ✅ Set 6 (Memory)
7. ✅ Set 7 (Constants)
8. ✅ Set 8 (Hyperbolic) + domain errors
9. ✅ Set 9 (Extended)
10. ✅ Set 10 (Modes & Meta)
11. ✅ Cursor activity model
12. ✅ Ans auto-insertion
13. ✅ Inverse-armed visual marker
14. ✅ Info Modal content
