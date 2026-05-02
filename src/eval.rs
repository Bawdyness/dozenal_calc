// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::logic::{
    DozenalConverter, DozenalDigit, FRAC_EPSILON, RatExpr, Rational, eval_rational,
};
use crate::tokens::{AngleMode, CalcToken, DozenalCalcApp};

/// Maximum number of period digits rendered with an overline. Periods longer than
/// this are truncated and signalled with the State-C raised-dots suffix.
const MAX_PERIOD_DISPLAY: usize = 5;

/// Number of dozenal fractional digits emitted in the f64 fallback (when the
/// rational track collapses). Chosen to fit the display while still showing
/// enough precision to distinguish typical irrational results.
const F64_FRAC_DIGITS: usize = 4;

/// Converts the `input_buffer` token sequence into `RatExpr` atoms for the
/// rational evaluation track. Returns `None` as soon as a non-rational token
/// (transcendental function, irrational constant, etc.) is encountered.
pub fn build_rat_expr(tokens: &[CalcToken]) -> Option<Vec<RatExpr>> {
    let mut exprs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            CalcToken::Digit(_) => {
                let mut int_d: Vec<DozenalDigit> = Vec::new();
                let mut frac_d: Vec<DozenalDigit> = Vec::new();
                let mut in_frac = false;
                loop {
                    if i >= tokens.len() {
                        break;
                    }
                    match tokens[i] {
                        CalcToken::Digit(d) => {
                            if in_frac {
                                frac_d.push(d);
                            } else {
                                int_d.push(d);
                            }
                            i += 1;
                        }
                        CalcToken::Decimal if !in_frac => {
                            in_frac = true;
                            i += 1;
                        }
                        _ => break,
                    }
                }
                let int_val = DozenalConverter::to_decimal_exact(&int_d)?;
                let int_rat = Rational::new(int_val, 1)?;
                let rat = if frac_d.is_empty() {
                    int_rat
                } else {
                    let frac_num = DozenalConverter::to_decimal_exact(&frac_d)?;
                    let frac_den = 12_i128.checked_pow(frac_d.len() as u32)?;
                    int_rat.add(Rational::new(frac_num, frac_den)?)?
                };
                exprs.push(RatExpr::Num(rat));
            }
            CalcToken::Decimal => {
                // Leading decimal point: implicit zero integer part (e.g. ".6")
                i += 1;
                let mut frac_d: Vec<DozenalDigit> = Vec::new();
                while i < tokens.len() {
                    if let CalcToken::Digit(d) = tokens[i] {
                        frac_d.push(d);
                        i += 1;
                    } else {
                        break;
                    }
                }
                if frac_d.is_empty() {
                    return None;
                }
                let frac_num = DozenalConverter::to_decimal_exact(&frac_d)?;
                let frac_den = 12_i128.checked_pow(frac_d.len() as u32)?;
                exprs.push(RatExpr::Num(Rational::new(frac_num, frac_den)?));
            }
            CalcToken::Add => {
                exprs.push(RatExpr::Add);
                i += 1;
            }
            CalcToken::Sub | CalcToken::Negate => {
                exprs.push(RatExpr::Sub);
                i += 1;
            }
            CalcToken::Mul => {
                exprs.push(RatExpr::Mul);
                i += 1;
            }
            CalcToken::Div => {
                exprs.push(RatExpr::Div);
                i += 1;
            }
            CalcToken::ParenOpen => {
                exprs.push(RatExpr::LParen);
                i += 1;
            }
            CalcToken::ParenClose => {
                exprs.push(RatExpr::RParen);
                i += 1;
            }
            CalcToken::ExpTopRight => {
                exprs.push(RatExpr::Pow);
                i += 1;
            }
            CalcToken::OplusBotLeft => {
                exprs.push(RatExpr::OPlus);
                i += 1;
            }
            CalcToken::RatLit(r) => {
                exprs.push(RatExpr::Num(r));
                i += 1;
            }
            _ => return None, // non-rational token → collapse the track
        }
    }
    Some(exprs)
}

/// Returns the index range of the left-hand operand for a custom operator at `op_pos`.
/// If the preceding token is `)`, walks back to its matching `(` so the entire bracketed
/// subexpression is treated as one operand. Otherwise returns the single preceding token.
fn left_operand_range(tokens: &[String], op_pos: usize) -> Option<std::ops::Range<usize>> {
    if op_pos == 0 {
        return None;
    }
    if tokens[op_pos - 1] != ")" {
        return Some((op_pos - 1)..op_pos);
    }
    let mut depth: i32 = 0;
    let mut j = op_pos;
    while j > 0 {
        j -= 1;
        match tokens[j].as_str() {
            ")" => depth += 1,
            "(" => {
                depth -= 1;
                if depth == 0 {
                    return Some(j..op_pos);
                }
            }
            _ => {}
        }
    }
    None
}

/// Returns the index range of the right-hand operand for a custom operator at `op_pos`.
/// If the following token is `(`, walks forward to its matching `)`; otherwise returns
/// the single following token. The range is half-open and includes any closing `)`.
fn right_operand_range(tokens: &[String], op_pos: usize) -> Option<std::ops::Range<usize>> {
    if op_pos + 1 >= tokens.len() {
        return None;
    }
    if tokens[op_pos + 1] != "(" {
        return Some((op_pos + 1)..(op_pos + 2));
    }
    let mut depth: i32 = 0;
    for (j, tok) in tokens.iter().enumerate().skip(op_pos + 1) {
        match tok.as_str() {
            "(" => depth += 1,
            ")" => {
                depth -= 1;
                if depth == 0 {
                    return Some((op_pos + 1)..(j + 1));
                }
            }
            _ => {}
        }
    }
    None
}

/// Resolves the three custom operators (`⊕`, `√`, `log`) by rewriting them into pure
/// meval-compatible infix using `left_operand_range` / `right_operand_range`. Operates
/// in place on a `Vec<String>` token list. Pure function — testable without meval.
pub(crate) fn resolve_custom_operators(tokens: &mut Vec<String>) {
    while let Some(i) = tokens.iter().position(|t| t == "⊕") {
        let Some(left) = left_operand_range(tokens, i) else {
            break;
        };
        let Some(right) = right_operand_range(tokens, i) else {
            break;
        };
        let a = tokens[left.clone()].join("");
        let b = tokens[right.clone()].join("");
        let replaced = format!("(({a}*{b})/({a}+{b}))");
        tokens.splice(left.start..right.end, vec![replaced]);
    }
    while let Some(i) = tokens.iter().position(|t| t == "√") {
        let Some(right) = right_operand_range(tokens, i) else {
            break;
        };
        let preceded_by_op =
            i == 0 || matches!(tokens[i - 1].as_str(), "+" | "-" | "*" | "/" | "(");
        let x = tokens[right.clone()].join("");
        if preceded_by_op {
            tokens.splice(i..right.end, vec![format!("({x}^(1/2))")]);
        } else {
            let Some(left) = left_operand_range(tokens, i) else {
                break;
            };
            let n = tokens[left.clone()].join("");
            tokens.splice(left.start..right.end, vec![format!("({x}^(1/{n}))")]);
        }
    }
    while let Some(i) = tokens.iter().position(|t| t == "log") {
        let Some(left) = left_operand_range(tokens, i) else {
            break;
        };
        let Some(right) = right_operand_range(tokens, i) else {
            break;
        };
        let x = tokens[left.clone()].join("");
        let n = tokens[right.clone()].join("");
        tokens.splice(left.start..right.end, vec![format!("(ln({x})/ln({n}))")]);
    }
}

/// Returns true when an implicit `*` should be inserted between tokens[i] and tokens[i+1].
fn needs_implicit_mul(tokens: &[CalcToken], i: usize) -> bool {
    if i + 1 >= tokens.len() {
        return false;
    }
    let curr = &tokens[i];
    let next = &tokens[i + 1];
    // curr must produce a value and not be part of an ongoing number literal
    let curr_ends_value = match curr {
        CalcToken::Digit(_) => !matches!(next, CalcToken::Digit(_) | CalcToken::Decimal),
        CalcToken::ParenClose
        | CalcToken::RatLit(_)
        | CalcToken::ConstPi
        | CalcToken::ConstE
        | CalcToken::ConstPhi
        | CalcToken::ConstSqrt2 => true,
        _ => false,
    };
    if !curr_ends_value {
        return false;
    }
    // next must begin a new subexpression (not an operator)
    matches!(
        next,
        CalcToken::Digit(_)
            | CalcToken::ParenOpen
            | CalcToken::RatLit(_)
            | CalcToken::ConstPi
            | CalcToken::ConstE
            | CalcToken::ConstPhi
            | CalcToken::ConstSqrt2
            | CalcToken::Sin
            | CalcToken::Cos
            | CalcToken::Tan
            | CalcToken::Cot
            | CalcToken::ArcSin
            | CalcToken::ArcCos
            | CalcToken::ArcTan
            | CalcToken::ArcCot
            | CalcToken::Sinh
            | CalcToken::Cosh
            | CalcToken::Tanh
            | CalcToken::Coth
            | CalcToken::ArSinh
            | CalcToken::ArCosh
            | CalcToken::ArTanh
            | CalcToken::ArCoth
            | CalcToken::AbsVal
            | CalcToken::Reciprocal
    )
}

/// Expands a token sequence by inserting `CalcToken::Mul` wherever algebraic
/// notation implies multiplication (e.g. `π π`, `2(`, `)(`, `2 sin`).
fn with_implicit_muls(tokens: &[CalcToken]) -> Vec<CalcToken> {
    let mut result = Vec::with_capacity(tokens.len() + 4);
    for (i, &token) in tokens.iter().enumerate() {
        result.push(token);
        if needs_implicit_mul(tokens, i) {
            result.push(CalcToken::Mul);
        }
    }
    result
}

/// Flushes any accumulated digit buffer into a single meval-compatible string token
/// (e.g. `"5"` or `"(5+(6/(12^1)))"` for `5.6` dozenal). No-op when both buffers are empty.
fn flush_number_literal(
    int_digits: &mut Vec<DozenalDigit>,
    frac_digits: &mut Vec<DozenalDigit>,
    in_fraction: &mut bool,
    out: &mut Vec<String>,
) {
    if int_digits.is_empty() && frac_digits.is_empty() {
        return;
    }
    let int_val = if int_digits.is_empty() {
        "0".to_string()
    } else {
        DozenalConverter::to_decimal(int_digits).to_string()
    };
    if *in_fraction && !frac_digits.is_empty() {
        let frac_val = DozenalConverter::to_decimal(frac_digits).to_string();
        let len = frac_digits.len();
        out.push(format!("({int_val}+({frac_val}/(12^{len})))"));
    } else {
        out.push(int_val);
    }
    int_digits.clear();
    frac_digits.clear();
    *in_fraction = false;
}

/// Maps a non-digit, non-decimal `CalcToken` to its meval-string form. Returns `""`
/// for tokens that emit nothing on their own (handled separately: digits, decimal,
/// `RatLit`, irrational constants).
fn token_meval_str(token: CalcToken) -> &'static str {
    match token {
        CalcToken::Add => "+",
        CalcToken::Sub | CalcToken::Negate => "-",
        CalcToken::Mul => "*",
        CalcToken::Div => "/",
        CalcToken::Mod => "%",
        CalcToken::ParenOpen => "(",
        CalcToken::ParenClose => ")",
        CalcToken::Sin => "sin(",
        CalcToken::Cos => "cos(",
        CalcToken::Tan => "tan(",
        CalcToken::Cot => "cot(",
        CalcToken::ExpTopRight => "^",
        CalcToken::RootTopLeft => "√",
        CalcToken::OplusBotLeft => "⊕",
        CalcToken::LogBotRight => "log",
        CalcToken::ArcSin => "asin(",
        CalcToken::ArcCos => "acos(",
        CalcToken::ArcTan => "atan(",
        CalcToken::ArcCot => "acot(",
        CalcToken::Sinh => "sinh(",
        CalcToken::Cosh => "cosh(",
        CalcToken::Tanh => "tanh(",
        CalcToken::Coth => "coth(",
        CalcToken::ArSinh => "arsinh(",
        CalcToken::ArCosh => "arcosh(",
        CalcToken::ArTanh => "artanh(",
        CalcToken::ArCoth => "arcoth(",
        CalcToken::Factorial => "fact(",
        CalcToken::AbsVal => "abs(",
        CalcToken::Reciprocal => "recip(",
        _ => "",
    }
}

/// Returns the f64 value for irrational constant tokens; `None` for everything else.
fn const_value(token: CalcToken) -> Option<f64> {
    match token {
        CalcToken::ConstPi => Some(std::f64::consts::PI),
        CalcToken::ConstE => Some(std::f64::consts::E),
        CalcToken::ConstPhi => Some(1.618_033_988_749_895),
        CalcToken::ConstSqrt2 => Some(std::f64::consts::SQRT_2),
        _ => None,
    }
}

/// Builds the final meval-ready expression string from an already-`with_implicit_muls`-expanded
/// token sequence. Resolves `⊕`, `√`, `log` into pure infix and balances any unclosed parens.
fn build_meval_string(expanded: &[CalcToken]) -> String {
    let mut int_digits = Vec::new();
    let mut frac_digits = Vec::new();
    let mut in_fraction = false;
    let mut tokens_str: Vec<String> = Vec::new();

    for &token in expanded {
        match token {
            CalcToken::Digit(d) => {
                if in_fraction {
                    frac_digits.push(d);
                } else {
                    int_digits.push(d);
                }
            }
            CalcToken::Decimal => in_fraction = true,
            _ => {
                flush_number_literal(
                    &mut int_digits,
                    &mut frac_digits,
                    &mut in_fraction,
                    &mut tokens_str,
                );
                if let CalcToken::RatLit(r) = token {
                    tokens_str.push(r.to_f64().to_string());
                } else if let Some(v) = const_value(token) {
                    tokens_str.push(v.to_string());
                } else {
                    let s = token_meval_str(token);
                    if !s.is_empty() {
                        tokens_str.push(s.to_string());
                    }
                }
            }
        }
    }
    flush_number_literal(
        &mut int_digits,
        &mut frac_digits,
        &mut in_fraction,
        &mut tokens_str,
    );

    resolve_custom_operators(&mut tokens_str);

    let mut math_string = tokens_str.join(" ");
    let open_parens = math_string.matches('(').count();
    let close_parens = math_string.matches(')').count();
    for _ in 0..(open_parens.saturating_sub(close_parens)) {
        math_string.push(')');
    }
    math_string
}

/// Builds the meval evaluation context with angle-mode-aware trig and the custom
/// helpers (`cot`, `acot`, hyperbolics, `fact`, `abs`, `recip`) needed by our token
/// surface. Returned context is merged with `meval::builtin()` at the call site.
fn make_meval_context(am: AngleMode) -> meval::Context<'static> {
    let mut ctx = meval::Context::new();
    ctx.func("sin", move |x| am.to_rad(x).sin());
    ctx.func("cos", move |x| am.to_rad(x).cos());
    ctx.func("tan", move |x| am.to_rad(x).tan());
    ctx.func("cot", move |x| 1.0 / am.to_rad(x).tan());
    ctx.func("asin", move |x| am.rad_to_unit(x.asin()));
    ctx.func("acos", move |x| am.rad_to_unit(x.acos()));
    ctx.func("atan", move |x| am.rad_to_unit(x.atan()));
    // Convention A: acot range (0,π), acot(x) = π/2 − atan(x).
    ctx.func("acot", move |x| {
        am.rad_to_unit(std::f64::consts::FRAC_PI_2 - x.atan())
    });
    ctx.func("coth", |x: f64| x.cosh() / x.sinh());
    ctx.func("arsinh", f64::asinh);
    ctx.func("arcosh", f64::acosh);
    ctx.func("artanh", f64::atanh);
    ctx.func("arcoth", |x: f64| 0.5 * ((x + 1.0) / (x - 1.0)).ln());
    ctx.func("fact", |x: f64| {
        let n = x.round() as u64;
        (1..=n).fold(1u64, u64::saturating_mul) as f64
    });
    ctx.func("abs", f64::abs);
    ctx.func("recip", |x: f64| 1.0 / x);
    ctx
}

/// Period-start index, capped period length, and overflow flag for a rational result.
struct PeriodMeta {
    start: Option<usize>,
    len: usize,
    capped: bool,
}

/// Renders an exact `Rational` as a token sequence with optional period metadata.
fn format_rational_result(r: Rational) -> (Vec<CalcToken>, PeriodMeta) {
    let (int_d, pre_d, period_d) = r.to_dozenal_periodic();
    let mut buf: Vec<CalcToken> = Vec::new();
    if r.num < 0 {
        buf.push(CalcToken::Negate);
    }
    buf.extend(int_d.into_iter().map(CalcToken::Digit));
    if !pre_d.is_empty() || !period_d.is_empty() {
        buf.push(CalcToken::Decimal);
    }
    buf.extend(pre_d.into_iter().map(CalcToken::Digit));
    let start = if period_d.is_empty() {
        None
    } else {
        Some(buf.len())
    };
    let capped = period_d.len() > MAX_PERIOD_DISPLAY;
    let len = period_d.len().min(MAX_PERIOD_DISPLAY);
    buf.extend(
        period_d
            .into_iter()
            .take(MAX_PERIOD_DISPLAY)
            .map(CalcToken::Digit),
    );
    (buf, PeriodMeta { start, len, capped })
}

/// Renders an f64 result as a token sequence with `F64_FRAC_DIGITS` fractional digits
/// (no period — used when the rational track has collapsed).
fn format_f64_result(value: f64) -> Vec<CalcToken> {
    let mut buf: Vec<CalcToken> = Vec::new();
    let mut val = value;
    if val < 0.0 {
        buf.push(CalcToken::Negate);
        val = val.abs();
    }
    buf.extend(
        DozenalConverter::from_decimal(val)
            .into_iter()
            .map(CalcToken::Digit),
    );
    let frac_part = val - val.floor();
    if frac_part > FRAC_EPSILON {
        buf.push(CalcToken::Decimal);
        buf.extend(
            DozenalConverter::frac_to_digits(frac_part, F64_FRAC_DIGITS)
                .into_iter()
                .map(CalcToken::Digit),
        );
    }
    buf
}

impl DozenalCalcApp {
    // --- RECHEN-LOGIK ---
    pub fn calculate_result(&mut self) {
        let expanded = with_implicit_muls(&self.input_buffer);
        let math_string = build_meval_string(&expanded);
        let rat_result = build_rat_expr(&expanded).and_then(|exprs| eval_rational(&exprs));
        let ctx = make_meval_context(self.angle_mode);

        match meval::eval_str_with_context(&math_string, (ctx, meval::builtin())) {
            Ok(result) if result.is_finite() => {
                self.error_msg = None;
                self.last_ans = rat_result;
                self.last_result_f64 = result;

                if let Some(r) = rat_result {
                    let (buf, meta) = format_rational_result(r);
                    self.result_buffer = buf;
                    self.result_period_start = meta.start;
                    self.result_period_len = meta.len;
                    self.result_period_capped = meta.capped;
                } else {
                    self.result_buffer = format_f64_result(result);
                    self.result_period_start = None;
                    self.result_period_len = 0;
                    self.result_period_capped = false;
                }

                // input_buffer intentionally kept — upper line stays visible after =.
                // It is cleared when the user starts a new expression (handled in handle_click).
                self.result_cursor_pos = 0;
                self.result_field_active = true;
            }
            Ok(result) if result.is_nan() => {
                // NaN comes from out-of-domain inputs (e.g. arcosh(0), artanh(1))
                self.error_msg = Some("DOMAIN ERROR".to_string());
            }
            Ok(_) => {
                // Infinite result — most likely division by zero
                self.error_msg = Some("DIV BY ZERO".to_string());
            }
            Err(_) => {
                self.error_msg = Some("SYNTAX ERROR".to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    fn eval(expr: &str) -> f64 {
        let mut ctx = meval::Context::new();
        ctx.func("cot", |x: f64| 1.0 / x.tan());
        ctx.func("acot", |x: f64| std::f64::consts::FRAC_PI_2 - x.atan());
        meval::eval_str_with_context(expr, (ctx, meval::builtin())).unwrap()
    }

    // acot Convention A: range (0, π), formula π/2 - atan(x)
    #[test]
    fn acot_convention_a() {
        let pi = std::f64::consts::PI;
        assert!((eval("acot(1)") - pi / 4.0).abs() < 1e-10);
        assert!((eval("acot(-1)") - 3.0 * pi / 4.0).abs() < 1e-10);
        assert!((eval("acot(0)") - pi / 2.0).abs() < 1e-10);
    }

    #[test]
    fn cot_basic() {
        // cot(π/4) = 1
        assert!((eval("cot(pi/4)") - 1.0).abs() < 1e-10);
        // cot in denominator: 6 / cot(π/4) = 6
        assert!((eval("6/cot(pi/4)") - 6.0).abs() < 1e-10);
    }

    #[test]
    fn sqrt_mid_expression() {
        // √ at start → square root
        assert!((eval("(16^(1/2))") - 4.0).abs() < 1e-10);
        // n√x syntax: 9^(1/8) ≈ 1.2968...
        assert!((eval("(8^(1/9))") - 8_f64.powf(1.0 / 9.0)).abs() < 1e-10);
    }

    #[test]
    fn unary_minus() {
        assert!((eval("-5+3") - (-2.0)).abs() < 1e-10);
        assert!((eval("5*-3") - (-15.0)).abs() < 1e-10);
        assert!((eval("5-3") - 2.0).abs() < 1e-10);
    }

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| (*x).to_string()).collect()
    }

    fn resolved(v: &[&str]) -> String {
        let mut t = s(v);
        super::resolve_custom_operators(&mut t);
        t.join(" ")
    }

    // Regression: ⊕ used to splice a fixed 3-element window, breaking on bracketed
    // operands. (a⊕b) where b = (3+2) became "((5*()/(5+())".
    #[test]
    fn oplus_with_paren_right_operand() {
        let out = resolved(&["5", "⊕", "(", "3", "+", "2", ")"]);
        assert_eq!(out, "((5*(3+2))/(5+(3+2)))");
        assert!((eval(&out) - (5.0 * 5.0) / (5.0 + 5.0)).abs() < 1e-10);
    }

    #[test]
    fn oplus_with_paren_left_operand() {
        let out = resolved(&["(", "2", "+", "3", ")", "⊕", "5"]);
        assert_eq!(out, "(((2+3)*5)/((2+3)+5))");
        assert!((eval(&out) - 25.0 / 10.0).abs() < 1e-10);
    }

    #[test]
    fn sqrt_with_paren_arg() {
        let out = resolved(&["√", "(", "1", "+", "1", ")"]);
        assert_eq!(out, "((1+1)^(1/2))");
        assert!((eval(&out) - 2_f64.sqrt()).abs() < 1e-10);
    }

    #[test]
    fn log_with_paren_base() {
        // log(64, 2+2) = log_4(64) = 3
        let out = resolved(&["64", "log", "(", "2", "+", "2", ")"]);
        assert_eq!(out, "(ln(64)/ln((2+2)))");
        assert!((eval(&out) - 3.0).abs() < 1e-10);
    }

    // Regression: rational track must also handle ⊕ with bracketed operand.
    // Bug symptom: 5⊕(3+2) showed 1.5186A periodic instead of exact 2.6 (= 5/2).
    #[test]
    fn rational_oplus_with_paren() {
        use crate::logic::{DozenalDigit::*, Rational, eval_rational};
        use crate::tokens::CalcToken::{self, *};
        let tokens: Vec<CalcToken> = vec![
            Digit(D5),
            OplusBotLeft,
            ParenOpen,
            Digit(D3),
            Add,
            Digit(D2),
            ParenClose,
        ];
        let exprs = super::build_rat_expr(&tokens).expect("rat track should not collapse");
        let result = eval_rational(&exprs).expect("rat eval should succeed");
        assert_eq!(
            result,
            Rational::new(5, 2).unwrap(),
            "5⊕(3+2) must equal 5/2"
        );
    }

    #[test]
    fn nth_root_with_paren_arg() {
        // 3√(27) → cube root of 27 = 3
        let out = resolved(&["3", "√", "(", "27", ")"]);
        assert_eq!(out, "((27)^(1/3))");
        assert!((eval(&out) - 3.0).abs() < 1e-10);
    }

    // Period > MAX_PERIOD_DISPLAY (= 5) is truncated and flagged.
    // 1/7 in dozenal has a 6-digit period (186A35) — must trip the cap.
    #[test]
    fn period_longer_than_display_is_capped() {
        use crate::logic::Rational;
        let one_seventh = Rational::new(1, 7).unwrap();
        let (buf, meta) = super::format_rational_result(one_seventh);
        assert!(meta.start.is_some(), "1/7 must have a periodic part");
        assert_eq!(meta.len, super::MAX_PERIOD_DISPLAY);
        assert!(meta.capped, "true period (6 digits) exceeds display cap");
        // Buffer should contain int "0", decimal point, then exactly 5 period digits.
        // Pre-period digits are absent for 1/7 (gcd(7,12)=1 → pure period from position 1).
        let digit_count = buf
            .iter()
            .filter(|t| matches!(t, crate::tokens::CalcToken::Digit(_)))
            .count();
        assert_eq!(digit_count, 1 + super::MAX_PERIOD_DISPLAY); // "0" + 5 period digits
    }

    // Period ≤ MAX_PERIOD_DISPLAY is shown completely with no cap flag.
    // 1/5 in dozenal has period "2497" (4 digits).
    #[test]
    fn period_shorter_than_display_is_not_capped() {
        use crate::logic::Rational;
        let one_fifth = Rational::new(1, 5).unwrap();
        let (_buf, meta) = super::format_rational_result(one_fifth);
        assert!(meta.start.is_some());
        assert_eq!(meta.len, 4);
        assert!(!meta.capped);
    }

    // Negative rationals must emit a leading Negate token.
    #[test]
    fn negative_rational_renders_negate_token() {
        use crate::logic::Rational;
        use crate::tokens::CalcToken;
        let minus_half = Rational::new(-1, 2).unwrap();
        let (buf, _meta) = super::format_rational_result(minus_half);
        assert_eq!(
            buf.first(),
            Some(&CalcToken::Negate),
            "negative result must start with Negate"
        );
    }

    // Implicit multiplication: π π → π * π
    #[test]
    fn implicit_mul_constant_constant() {
        use crate::tokens::CalcToken::{ConstPi, Mul};
        let input = vec![ConstPi, ConstPi];
        let out = super::with_implicit_muls(&input);
        assert_eq!(out, vec![ConstPi, Mul, ConstPi]);
    }

    // Implicit multiplication: 2 ( ... ) → 2 * ( ... )
    #[test]
    fn implicit_mul_digit_paren() {
        use crate::logic::DozenalDigit::D2;
        use crate::tokens::CalcToken::{Digit, Mul, ParenOpen};
        let input = vec![Digit(D2), ParenOpen];
        let out = super::with_implicit_muls(&input);
        assert_eq!(out, vec![Digit(D2), Mul, ParenOpen]);
    }

    // Implicit multiplication: ) ( → ) * (
    #[test]
    fn implicit_mul_close_open_paren() {
        use crate::tokens::CalcToken::{Mul, ParenClose, ParenOpen};
        let input = vec![ParenClose, ParenOpen];
        let out = super::with_implicit_muls(&input);
        assert_eq!(out, vec![ParenClose, Mul, ParenOpen]);
    }

    // Implicit multiplication: 2 sin → 2 * sin
    #[test]
    fn implicit_mul_digit_function() {
        use crate::logic::DozenalDigit::D2;
        use crate::tokens::CalcToken::{Digit, Mul, Sin};
        let input = vec![Digit(D2), Sin];
        let out = super::with_implicit_muls(&input);
        assert_eq!(out, vec![Digit(D2), Mul, Sin]);
    }

    // No implicit mul inside a number literal: 1 2 stays "12", not "1 * 2".
    #[test]
    fn no_implicit_mul_within_number() {
        use crate::logic::DozenalDigit::{D1, D2};
        use crate::tokens::CalcToken::Digit;
        let input = vec![Digit(D1), Digit(D2)];
        let out = super::with_implicit_muls(&input);
        assert_eq!(out, vec![Digit(D1), Digit(D2)], "no Mul within a number");
    }

    // RatLit roundtrip: an Ans-injected exact rational must evaluate as itself.
    #[test]
    fn ratlit_token_evaluates_to_embedded_value() {
        use crate::logic::{Rational, eval_rational};
        use crate::tokens::CalcToken::{self, Add, RatLit};
        let prev_ans = Rational::new(5, 7).unwrap();
        // Simulate: Ans + Ans → 10/7
        let tokens: Vec<CalcToken> = vec![RatLit(prev_ans), Add, RatLit(prev_ans)];
        let exprs = super::build_rat_expr(&tokens).expect("rat track should not collapse");
        let result = eval_rational(&exprs).expect("rat eval should succeed");
        assert_eq!(result, Rational::new(10, 7).unwrap());
    }
}
