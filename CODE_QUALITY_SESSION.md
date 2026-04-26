# Code-Quality-Session: Pre-Overlay Fixes

This is a focused session to fix mathematical correctness issues and UX bugs in `src/main.rs` before the overlay refactor begins. All eight fixes below are independent of each other and should be applied as small, separately-verifiable patches.

**Before starting**: read `CLAUDE.md`. Run `cargo check` to establish baseline. Make a feature branch (`git checkout -b code-quality-pre-overlay`).

**After each fix**: run `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`. If any of these fail, fix before moving to the next item.

**At the end**: run `cargo run` (desktop) and `trunk serve` (web) to verify both targets still work. Manual smoke test: enter `5+3=`, verify result is `8`. Try one trig function. Try AC. Try cot. Try √.

---

## Fix 1 — Error state cannot be dismissed

**File**: `src/main.rs`
**Symptom**: When `error_msg` is `Some(...)` (e.g. after a syntax error), no key press clears it. Only a successful new calculation resets it.

**Root cause**: `handle_click()` at line ~77 never resets `self.error_msg`.

**Fix**:
- At the very top of `handle_click()`, add a guard: if `self.error_msg.is_some()`, only the `AC` token should respond. Other tokens should return early without doing anything.
- The `AC` arm (line ~84) should additionally `self.error_msg = None;` to clear the error.

**Verification**: trigger an error (e.g. enter `(` then `=`). Display shows error. Press a digit — nothing happens. Press AC — error gone, display shows `0`.

---

## Fix 2 — Cot expression has unbalanced parentheses

**File**: `src/main.rs`
**Symptom**: `cot(x)` is currently expanded to `1/tan(` (without closing). For simple cases this works because the user adds the closing `)`. For composed expressions, operator precedence can produce wrong results.

**Root cause**: Line 171: `CalcToken::Cot => "1/tan(",`

**Fix**:
- The expansion should produce a fully-bracketed `(1/tan(...))` group. Since the user types the closing paren manually, a clean approach is: emit `Cot` as `(1/tan(` instead of `1/tan(`, and let the existing close-paren-balancing logic handle the closing. Then add an extra `)` after the user's closing paren — OR, simpler: change the post-processing in `calculate_result()` to replace `1/tan(...)` patterns with their bracketed forms after parenthesis balancing.
- Recommended: change line 171 to `CalcToken::Cot => "(1/tan(",` and add a corresponding wrap in the parenthesis balancer to close one extra `)`. Alternatively, detect this case explicitly.
- **Important**: also fix line ~487 where `Cot` is rendered in the input display — it currently shows just `cot`, which is fine for display, but the *evaluation* string is what's broken.

**Verification**: enter `cot(30)*2=`. Should produce roughly `2 * cot(30°)`. Compare against a known value: cot(30°) ≈ 1.732, so result should be ≈ 3.464 (in dozenal: 3.585...).

---

## Fix 3 — ArcCot range is incorrect for negative inputs

**File**: `src/main.rs`
**Symptom**: `arccot(x)` is expanded as `(pi/2 - atan(x))`. This formula yields negative values for negative `x`, but the conventional principal value of arccot lies in `(0, π)` (always positive).

**Root cause**: Lines 178 and 490: `CalcToken::ArcCot => "(pi/2 - atan(",`

**Fix**:
- Replace with the conditional definition. There are two valid conventions:
  - **Convention A** (range `(0, π)`): `arccot(x) = pi/2 - atan(x)` for all x. This is what the code currently does. The result for x < 0 is in `(π/2, π)` — i.e. positive, between 90° and 180°. Wait — let me recheck: if x = -1, atan(-1) = -π/4, so `pi/2 - atan(-1) = pi/2 + pi/4 = 3π/4`. That IS positive and in `(0, π)`. So the current formula is actually correct for Convention A.
  - **Convention B** (range `(-π/2, π/2]`): `arccot(x) = atan(1/x)` for x ≠ 0, with arccot(0) = π/2.
- **Re-evaluation**: Looking again at the formula `pi/2 - atan(x)`: for x = -1, it gives `pi/2 - (-pi/4) = 3pi/4 ≈ 2.356`. That is positive. So **the current code is mathematically correct for Convention A**.
- **Real issue**: the formula in line 178 is fine. What MAY be confusing is that some calculators (and some textbooks) use Convention B. Verify with a test case: enter `arccot(-1)=`, expected result with Convention A is `3π/4 ≈ 2.356` rad or `135°`. If that matches, no fix needed.
- **Action**: **add a unit test** in `src/logic.rs` (or a new test module) that pins down the convention. If the project intends Convention A, document it in code comments. If Convention B is preferred, change the formula to `atan(1/x)` with a special case for x=0.
- **Recommendation**: keep Convention A (current behavior), add a comment explaining the choice, and document it in `CLAUDE.md` under "Math conventions".

**Verification**: `arccot(1) = π/4`, `arccot(-1) = 3π/4`, `arccot(0) = π/2`. Test all three.

---

## Fix 4 — Square root in middle of expression is computed wrongly

**File**: `src/main.rs`
**Symptom**: When `√` appears mid-expression without an explicit index (e.g. `5 + √9`), the code at line 205-214 takes the *previous* token as the index, producing `5^(1/9)` instead of `5 + 3`.

**Root cause**: The `while let Some(i) = ...position(t == "√")` loop assumes index-then-radical, but the user might mean default-square-root with the index implicit.

**Fix**:
- Detect whether the token *before* `√` is a number/expression versus an operator. If it's an operator (`+`, `-`, `*`, `/`, `(`), treat `√` as default square root: `√x → x^(1/2)`.
- If the token before is a number/closing paren, treat as `n√x → x^(1/n)` (current behavior).
- Concretely: in the loop, check `tokens_str[i-1]` against operator strings. If it's an operator, splice differently.

**Verification**:
- `9 √ 8 =` should produce `8^(1/9)` ≈ `1.259...`
- `5 + √ 9 =` should produce `5 + 3 = 8`
- `√ 16 =` (at start) should produce `4` — current behavior, must still work

---

## Fix 5 — Negative numbers cannot be entered as input

**File**: `src/main.rs`
**Symptom**: User cannot type `-5 + 3`. Pressing `-` at the start triggers a syntax error because the parser sees a binary operator without a left operand.

**Root cause**: There is no unary minus handling. The `Sub` token is always treated as binary subtraction.

**Fix** (choose ONE approach):

**Approach A** — preprocess the token stream: in `calculate_result()`, before assembling `math_string`, walk the buffer and detect `Sub` tokens that appear at position 0 or directly after another operator/open-paren. Wrap their following operand in `(0 - x)` or prefix with `-`.

**Approach B** — let `meval` handle it: meval already supports leading `-` as unary. Just emit the `-` directly in those cases. Verify by testing `meval::eval_str("-5 + 3")` — if it returns 8 (no test, just a quick local script), then `Sub` at position 0 just needs to emit `-` instead of `- ` and meval handles the rest.

**Recommendation**: **Approach B is simpler.** Most expression evaluators accept leading `-`. Test first; if meval rejects it, fall back to Approach A.

**Verification**:
- `-5 + 3 =` should produce `-2`.
- `5 * -3 =` should produce `-15` (unary minus after `*`).
- `5 - 3 =` should still produce `2` (binary subtraction unchanged).

---

## Fix 6 — Integer overflow when result exceeds u64::MAX

**File**: `src/main.rs`
**Symptom**: At line 238, `let mut integer_part = val.floor() as u64;` produces undefined behavior (saturation in release, panic in debug) when `val > u64::MAX ≈ 1.8e19`.

**Root cause**: The `as` cast for f64 → u64 has no overflow protection. Calculations like `12^20` exceed u64.

**Fix**:
- Before the cast, check `if val > u64::MAX as f64 { self.error_msg = Some("OVERFLOW".to_string()); return; }`.
- Alternatively, switch the integer part to `f64`-based loop division by 12.0 instead of integer modulo. This is slightly slower but has no overflow ceiling (only the f64 precision limit at ~10^308). Recommended for a scientific calculator.

**Recommendation**: use the f64-based loop. Replace the `while integer_part > 0` loop with one that uses `val.floor()` and successive division by 12.0, accumulating digits via `(quotient.floor() % 12.0) as u32`. The cast to u32 is safe because the modulo is always 0–11.

**Verification**: enter `12 ^ 20 =`. Should display a result, not crash or show wrong value. Compare with manual calculation: `12^20 = 3833759992447475122176` — this exceeds u64 max.

---

## Fix 7 — Negative results break Memory and re-use

**File**: `src/main.rs`
**Symptom**: Negative results are stored as `[Sub, digits...]` in `result_buffer` (line 236). When this is later re-injected (via M+ in current code, or via `Ans` in future overlay), the leading `Sub` is interpreted as binary subtraction, not as a sign.

**Root cause**: There is no representation for "this is a negative number" distinct from "subtract this".

**Fix** (choose ONE):

**Approach A** — add a `Negate` token: introduce `CalcToken::Negate` that displays as `-` but is parsed as unary minus. Use it in `result_buffer` for negative results instead of `Sub`. Update `calculate_result()` to emit `-` (unary) for `Negate` tokens.

**Approach B** — wrap negative results: store as `[ParenOpen, Sub, digits..., ParenClose]`. This makes the binary `Sub` work correctly because `(-5)` evaluates the same as `-5`. Less elegant but no new token.

**Recommendation**: **Approach A** is cleaner and forward-compatible with the upcoming `Ans` token. The `Negate` token can also be used for unary minus on input (Fix 5 alternative).

**Note**: This fix is closely related to Fix 5. If you implement Fix 5 with Approach A (preprocessing) AND Fix 7 with Approach A (Negate token), they share the same token. Coordinate.

**Verification**: compute `5 - 8 =`, get `-3`. Press M+ to store. Clear input. Press M+ to recall. Add `+ 1 =`. Should produce `-2`, not `-3 + 1` parsed as binary.

---

## Fix 8 — Display width too tight for inverse trig labels

**File**: `src/main.rs`
**Symptom**: In the input display (lines ~484-498), all trig and arc-trig tokens get the same horizontal advance (`x_pos += 45.0`). Labels like `cos⁻¹` are visually wider than `cos` and may overlap with the next token.

**Root cause**: Line 495-497, single `45.0` value for all trig tokens.

**Fix**:
- Differentiate between the four base trig (`Sin`, `Cos`, `Tan`, `Cot` → 45.0) and the four arc-variants (`ArcSin`, `ArcCos`, `ArcTan`, `ArcCot` → 60.0 or measured width). Adjust the `match` arm at line 494-497.
- Better: measure the actual text width using egui's text galley — but that's heavier. The simple constant-bump is acceptable for v1.

**Verification**: enter `arccos(0.5)=`, then look at the input area before pressing `=`. The `cos⁻¹` label should not overlap the `(`.

---

## After all eight fixes

1. Run the full quality triple: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`.
2. Run `cargo run` (desktop) — verify the calculator launches and basic operations work.
3. Run `trunk serve` (web) — verify the web build works.
4. Run `cargo build --release` and `trunk build --release --public-url /dozenal_calc/`.
5. Commit with a descriptive message, push the branch, create a PR. **Do not** merge to `main` until manually tested.

## Update DESIGN_NOTES.md

After completion, mark these issues as resolved in `DESIGN_NOTES.md` under "Status of known bugs". The bugs that this session covers:
- Error message can't be dismissed (Fix 1)
- `cot` / inverse trig display unwieldy (Fix 8)

Newly addressed (not previously listed):
- Cot expression syntactic correctness (Fix 2)
- ArcCot convention pinning (Fix 3)
- Square root mid-expression (Fix 4)
- Unary minus support (Fix 5)
- Integer overflow protection (Fix 6)
- Negative-result Memory compatibility (Fix 7)

Add new entries to DESIGN_NOTES.md as appropriate.
