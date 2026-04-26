# DESIGN_NOTES.md

A working journal of design discussions for the Dozenal Calculator.

This file is **not** authoritative — `CLAUDE.md` is. This file captures *open questions*, *deliberations in progress*, and *ideas under consideration*. Once a decision is finalized, it moves to `CLAUDE.md` and the corresponding section here is marked as resolved.

## Status of known bugs

| Bug | Status | Notes |
|---|---|---|
| Brightness too high on light system theme | ✅ Fixed by user | — |
| Error message can't be dismissed | ✅ Fixed (`code-quality-pre-overlay`) | Guard in `handle_click` + `AC` clears `error_msg`. |
| M+ not fully functional | 🔧 Will be replaced | M+ disappears as a top-level key; memory becomes part of the expansion overlay. |
| `cot` / inverse trig display unwieldy | ✅ Fixed (`code-quality-pre-overlay`) | Display width for Arc tokens bumped to 65 px; labels now `sin⁻¹` etc. consistent with buttons. Remaining sub-issue: visual "armed inverse" marker (open). |
| Cot expression syntactically incorrect | ✅ Fixed (`code-quality-pre-overlay`) | `cot`/`acot` registered as custom meval functions; no longer rely on paren-balancer. |
| ArcCot convention undefined | ✅ Fixed (`code-quality-pre-overlay`) | Convention A pinned (`acot(x) = π/2 − atan(x)`, range `(0,π)`). Unit tests added. |
| `√` mid-expression uses wrong operand | ✅ Fixed (`code-quality-pre-overlay`) | Operator-context detection: `√` after `+`/`-`/`*`/`/`/`(` or at position 0 → square root; after a number → n-th root. |
| Unary minus not supported | ✅ Confirmed working (`code-quality-pre-overlay`) | meval handles unary minus natively. Unit test added. |
| Integer overflow for large results | ✅ Fixed (`code-quality-pre-overlay`) | `from_decimal` now uses f64 arithmetic throughout; no u64 cast. |
| Negative results break re-insertion | ✅ Fixed (`code-quality-pre-overlay`) | `CalcToken::Negate` added; negative results stored as `[Negate, digits…]` instead of `[Sub, digits…]`. |

## Open design questions

### 1. Expansion overlay — content of the 5 sets

Confirmed:
- **Set 1 (Memory)**: STO, RCL, MC, Ans
- **Bottom full-width**: Close

Under deliberation:
- **Set 2 (Constants)** — candidates: π, τ, e, φ, √2, γ (Euler-Mascheroni), ζ(3), ln(2), 1/12 demos. User noted the dozenal-specific elegance of φ (because F(12) = 144 = 12² is unique in the Fibonacci sequence) and wants constants to demonstrate that irrationals stay irrational in base 12.
- **Set 3 (Hyperbolic)** — sinh, cosh, tanh, coth — with double-click to inverse (arsinh, etc.). Since double-click halves the needed slots, this set may be combined with logarithms (e.g. log/10ˣ, ln/eˣ) freeing one set for something else.
- **Set 4 (Extended ops)** — candidates: factorial (n!), modulo, |x|, 1/x. Or: didactic dozenal tools (divisor function, prime factorization, fraction periodicity).
- **Set 5 (Display & conversion)** — candidates: Doz↔Dec quick-toggle, FIX (digit count), DRG (deg/rad/grad). User has not yet weighed in on which of these matter.

The user is currently weighing the question: **"What would I want to find in this calculator that I can't find in any other?"** — choosing between standard scientific functionality vs. dozenal-specific didactic tools.

### 2. Memory slot count

- Option A: 1 slot (single M)
- Option B: 3 slots (M1, M2, M3) — TI-30 eco RS style

Pro 3 slots: more useful, authentic TI feel. Con: requires a "wait for slot number" interaction state (after pressing STO, next digit click selects slot).

### 3. Constants and the "scrollable digits" mode

The user wants to demonstrate that irrational constants remain non-periodic in base 12. The proposal: store constants like π, e, φ, √2 with **~1000 precomputed dozenal digits** as hardcoded strings, and allow horizontal scrolling in the display when one of these constants is the result.

Decision pending:
- Do we implement this scroll mode for **constants only**, or for **all results**?
- For constants only: trivial — hardcode the strings, add a scroll handler. No new dependencies.
- For all results: requires arbitrary-precision arithmetic. Realistic only via `astro-float` (pure Rust, WASM-compatible). Significant refactor.

Recommendation: start with constants-only. Defer arbitrary-precision compute to v2, if ever.

### 4. Visual indicator for "armed" inverse mode

Confirmed in concept (small marker, not a label change). Concrete form still open:
- A small filled dot near a corner of the key?
- A thin colored border?
- A subtle background tint?

To be decided when the patch is written. Should be consistent across all keys with inverse capability (trig + overlay hyperbolics + others).

## Ideas parked for later

These have been mentioned but are explicitly **not** in scope right now:

- **Statistics functions** (mean, std dev, regression) — out of scope for v1. Could be a separate "Stats overlay" in v2.
- **Complex numbers** — out of scope.
- **Unit conversions** (TGM, metric, imperial) — out of scope. Would deserve its own overlay if added.
- **Programming/scripting** — explicitly out of scope. This is not a CAS.
- **Theme toggle** (dark/light user choice) — not currently planned. The brightness issue was solved differently.

## Mathematical curiosities specific to dozenal

Captured for use as didactic content (display, tooltips, easter eggs, or in the README):

- **F(12) = 144 = 12²**. The 12th Fibonacci number is the only Fibonacci square besides F(1)=1, F(2)=1, F(12)=144. (Cohn, 1964 — proven unique.)
- **12 is the smallest abundant number**: σ(12) − 12 = 16 > 12.
- **12 is highly composite**: more divisors (6) than any smaller integer.
- **Short fractions in base 12**: 1/2 = 0.6, 1/3 = 0.4, 1/4 = 0.3, 1/6 = 0.2, 1/8 = 0.16, 1/9 = 0.14. Compare base 10 where 1/3 and 1/6 are infinite.
- **Constants in base 12** (first ~14 digits):
  - π ≈ 3.184809493B91866...
  - e ≈ 2.875236069821...
  - φ ≈ 1.74BB6772802A4...
  - √2 ≈ 1.4B79170A07B85...

## Workflow

This project uses a parallel workflow:
- **This chat (Claude Desktop App)**: design discussion, concept iteration, documentation.
- **Claude Code (terminal)**: code implementation, with `CLAUDE.md` as the binding instruction set.
- **DESIGN_NOTES.md (this file)**: bridge between the two — captures what's still in flux.

When a decision is finalized in chat, it should be moved into `CLAUDE.md` and removed from this file's "open questions" section.

## Patch order (when implementation starts)

The user has chosen "all together at the end" rather than incremental fixes. Once the overlay design is finalized, the order will be:

1. **Refactor for overlay infrastructure**: replace `MPlus` token with `Expand`, add overlay state, add `Close` token, add overlay drawing routine.
2. **Implement Set 1 (Memory)**: STO, RCL, MC, Ans logic.
3. **Implement remaining sets** as their content is decided.
4. **Add error-reset logic** (`AC` and others clear `error_msg`; non-AC keys are inactive when error is shown).
5. **Add inverse-armed visual marker** for double-click affordance.
6. **Fix display width** for inverse trig labels (`sin⁻¹`, etc.).
7. **(Optional)** constants scroll mode.

Each step verified locally with `cargo run` and `trunk serve` before merge.
