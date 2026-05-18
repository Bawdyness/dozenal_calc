// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

use crate::digit::{DozenalConverter, DozenalDigit, FRAC_EPSILON};
use crate::rational::{RatExpr, Rational};
use crate::token::CalcToken;

/// Maximum number of period digits rendered with an overline. Periods longer than
/// this are truncated and signalled with the State-C raised-dots suffix.
pub const MAX_PERIOD_DISPLAY: usize = 5;

/// Number of dozenal fractional digits emitted in the f64 fallback (when the
/// rational track collapses). Chosen to fit the display while still showing
/// enough precision to distinguish typical irrational results.
pub const F64_FRAC_DIGITS: usize = 4;

/// Converts the `input_buffer` token sequence into `RatExpr` atoms for the
/// rational evaluation track. Returns `None` as soon as a non-rational token
/// (transcendental function, irrational constant, etc.) is encountered.
pub fn build_rat_expr(tokens: &[CalcToken]) -> Option<Vec<RatExpr>> {
    use num_bigint::BigInt;

    let mut exprs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            CalcToken::Digit(_) => {
                let mut int_d: Vec<DozenalDigit> = Vec::new();
                let mut frac_d: Vec<DozenalDigit> = Vec::new();
                let mut in_frac = false;
                loop {
                    if i >= tokens.len() {
                        break;
                    }
                    match &tokens[i] {
                        CalcToken::Digit(d) => {
                            if in_frac {
                                frac_d.push(*d);
                            } else {
                                int_d.push(*d);
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
                let int_rat = Rational::from_ints(int_val, 1)?;
                let rat = if frac_d.is_empty() {
                    int_rat
                } else {
                    let frac_num = DozenalConverter::to_decimal_exact(&frac_d)?;
                    let frac_den = BigInt::from(12).pow(frac_d.len() as u32);
                    int_rat.add(&Rational::new(BigInt::from(frac_num), frac_den)?)
                };
                exprs.push(RatExpr::Num(rat));
            }
            CalcToken::Decimal => {
                i += 1;
                let mut frac_d: Vec<DozenalDigit> = Vec::new();
                while i < tokens.len() {
                    if let CalcToken::Digit(d) = &tokens[i] {
                        frac_d.push(*d);
                        i += 1;
                    } else {
                        break;
                    }
                }
                if frac_d.is_empty() {
                    return None;
                }
                let frac_num = DozenalConverter::to_decimal_exact(&frac_d)?;
                let frac_den = BigInt::from(12).pow(frac_d.len() as u32);
                exprs.push(RatExpr::Num(Rational::new(
                    BigInt::from(frac_num),
                    frac_den,
                )?));
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
                exprs.push(RatExpr::Num(r.clone()));
                i += 1;
            }
            _ => return None,
        }
    }
    Some(exprs)
}

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
pub fn resolve_custom_operators(tokens: &mut Vec<String>) {
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
        // `^` und `%` zählen ebenfalls als vorangehende Operatoren: nach `2^` oder
        // `5%` startet `√` einen frischen Radikanden (unäre Quadratwurzel), nicht
        // eine binäre n-te Wurzel. Ohne diese beiden würde `2^√3` als Konstrukt
        // mit `^` als n-Index fehlinterpretiert → `(3^(1/^))` → SYNTAX ERROR.
        let preceded_by_op = i == 0
            || matches!(
                tokens[i - 1].as_str(),
                "+" | "-" | "*" | "/" | "(" | "^" | "%"
            );
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

fn needs_implicit_mul(tokens: &[CalcToken], i: usize) -> bool {
    if i + 1 >= tokens.len() {
        return false;
    }
    let curr = &tokens[i];
    let next = &tokens[i + 1];
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

fn is_function_token(t: &CalcToken) -> bool {
    matches!(
        t,
        CalcToken::Sin
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
            | CalcToken::Factorial
            | CalcToken::AbsVal
            | CalcToken::Reciprocal
    )
}

/// Range of the operand immediately to the left of `op_pos`. Handles three
/// shapes: a parenthesised sub-expression (with its leading function token
/// if any), a contiguous Digit/Decimal number literal, or a single-token
/// operand (RatLit, constant). Other token kinds — operators, function
/// tokens without their argument — return `None`.
fn left_operand_token_range(tokens: &[CalcToken], op_pos: usize) -> Option<(usize, usize)> {
    if op_pos == 0 {
        return None;
    }
    let prev = &tokens[op_pos - 1];

    if matches!(prev, CalcToken::ParenClose) {
        let mut depth: i32 = 0;
        let mut j = op_pos - 1;
        loop {
            let t = &tokens[j];
            if matches!(t, CalcToken::ParenClose) {
                depth += 1;
            } else if matches!(t, CalcToken::ParenOpen) {
                depth -= 1;
                if depth == 0 {
                    if j > 0 && is_function_token(&tokens[j - 1]) {
                        return Some((j - 1, op_pos));
                    }
                    return Some((j, op_pos));
                }
            }
            if j == 0 {
                break;
            }
            j -= 1;
        }
        return None;
    }

    if matches!(prev, CalcToken::Digit(_) | CalcToken::Decimal) {
        let mut j = op_pos - 1;
        while j > 0 {
            let t = &tokens[j - 1];
            if matches!(t, CalcToken::Digit(_) | CalcToken::Decimal) {
                j -= 1;
            } else {
                break;
            }
        }
        return Some((j, op_pos));
    }

    if matches!(
        prev,
        CalcToken::RatLit(_)
            | CalcToken::ConstPi
            | CalcToken::ConstE
            | CalcToken::ConstPhi
            | CalcToken::ConstSqrt2
    ) {
        return Some((op_pos - 1, op_pos));
    }

    None
}

/// Wandelt postfix-Aufrufe von `Factorial`, `AbsVal` und `Reciprocal` (deren
/// Button-Labels `n!`, `|x|`, `1/x` Postfix-Syntax suggerieren) in die
/// Prefix-Funktions-Aufrufform, die die restliche Pipeline erwartet. So wird
/// aus `[Digit(5), Factorial]` die Folge `[Factorial, (, 5, )]`, die
/// `build_meval_string` als `fact(5)` rendert.
///
/// Postfix-Tokens, die bereits an einer gültigen Präfix-Position stehen (kein
/// Operand links, oder nur Nicht-Operand-Token), bleiben unverändert — so
/// funktioniert auch der Präfix-Eingabe-Pfad weiterhin.
///
/// Mehrere Durchläufe lösen verschachtelte Fälle wie `5!!` korrekt auf: nach
/// dem ersten Rewrite `[Factorial, (, 5, )]` umschlingt der zweite Durchlauf
/// das äußere `!` um den geklammerten Zwischenstand.
#[must_use]
pub fn resolve_postfix(tokens: &[CalcToken]) -> Vec<CalcToken> {
    let mut current = tokens.to_vec();
    loop {
        let mut changed = false;
        for i in 0..current.len() {
            if !matches!(
                &current[i],
                CalcToken::Factorial | CalcToken::AbsVal | CalcToken::Reciprocal
            ) {
                continue;
            }
            let Some((start, end)) = left_operand_token_range(&current, i) else {
                continue;
            };
            let t = current[i].clone();
            let operand: Vec<CalcToken> = current[start..end].to_vec();
            let mut new_current = Vec::with_capacity(current.len() + 2);
            new_current.extend_from_slice(&current[..start]);
            new_current.push(t);
            new_current.push(CalcToken::ParenOpen);
            new_current.extend_from_slice(&operand);
            new_current.push(CalcToken::ParenClose);
            new_current.extend_from_slice(&current[i + 1..]);
            current = new_current;
            changed = true;
            break;
        }
        if !changed {
            break;
        }
    }
    current
}

/// Expands a token sequence by inserting `CalcToken::Mul` wherever algebraic
/// notation implies multiplication (e.g. `π π`, `2(`, `)(`, `2 sin`).
#[must_use]
pub fn with_implicit_muls(tokens: &[CalcToken]) -> Vec<CalcToken> {
    let mut result = Vec::with_capacity(tokens.len() + 4);
    for (i, token) in tokens.iter().enumerate() {
        result.push(token.clone());
        if needs_implicit_mul(tokens, i) {
            result.push(CalcToken::Mul);
        }
    }
    result
}

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

fn token_meval_str(token: &CalcToken) -> &'static str {
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

fn const_value(token: &CalcToken) -> Option<f64> {
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
#[must_use]
pub fn build_meval_string(expanded: &[CalcToken]) -> String {
    let mut int_digits = Vec::new();
    let mut frac_digits = Vec::new();
    let mut in_fraction = false;
    let mut tokens_str: Vec<String> = Vec::new();

    for token in expanded {
        match token {
            CalcToken::Digit(d) => {
                if in_fraction {
                    frac_digits.push(*d);
                } else {
                    int_digits.push(*d);
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

/// Period-start index, capped period length, and overflow flag for a rational result.
pub struct PeriodMeta {
    pub start: Option<usize>,
    pub len: usize,
    pub capped: bool,
}

/// Renders an exact `Rational` as a token sequence with optional period metadata.
#[must_use]
pub fn format_rational_result(r: &Rational) -> (Vec<CalcToken>, PeriodMeta) {
    let (int_d, pre_d, period_d) = r.to_dozenal_periodic();
    let mut buf: Vec<CalcToken> = Vec::new();
    if r.is_negative() {
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

/// Formats an `f64` as a plain decimal string with up to 10 significant
/// fractional digits, trailing zeros stripped. `NaN` and infinities are
/// reported with their conventional symbols (`"NaN"`, `"∞"`, `"-∞"`).
///
/// Universally useful — independent of dozenal semantics — and reused by
/// every render path that needs a base-10 view on a numeric result.
#[must_use]
pub fn format_f64_as_decimal(val: f64) -> String {
    if !val.is_finite() {
        return if val.is_nan() {
            "NaN".to_string()
        } else if val.is_sign_negative() {
            "-∞".to_string()
        } else {
            "∞".to_string()
        };
    }
    let s = format!("{val:.10}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Renders an f64 result as a token sequence with `F64_FRAC_DIGITS` fractional digits
/// (no period — used when the rational track has collapsed).
#[must_use]
pub fn format_f64_result(value: f64) -> Vec<CalcToken> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rational::eval_rational;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| (*x).to_string()).collect()
    }

    fn resolved(v: &[&str]) -> String {
        let mut t = s(v);
        super::resolve_custom_operators(&mut t);
        t.join(" ")
    }

    #[test]
    fn oplus_with_paren_right_operand() {
        let out = resolved(&["5", "⊕", "(", "3", "+", "2", ")"]);
        assert_eq!(out, "((5*(3+2))/(5+(3+2)))");
    }

    #[test]
    fn oplus_with_paren_left_operand() {
        let out = resolved(&["(", "2", "+", "3", ")", "⊕", "5"]);
        assert_eq!(out, "(((2+3)*5)/((2+3)+5))");
    }

    #[test]
    fn sqrt_with_paren_arg() {
        let out = resolved(&["√", "(", "1", "+", "1", ")"]);
        assert_eq!(out, "((1+1)^(1/2))");
    }

    #[test]
    fn log_with_paren_base() {
        let out = resolved(&["64", "log", "(", "2", "+", "2", ")"]);
        assert_eq!(out, "(ln(64)/ln((2+2)))");
    }

    #[test]
    fn nth_root_with_paren_arg() {
        let out = resolved(&["3", "√", "(", "27", ")"]);
        assert_eq!(out, "((27)^(1/3))");
    }

    #[test]
    fn rational_oplus_with_paren() {
        use crate::digit::DozenalDigit::*;
        use crate::rational::Rational;
        use crate::token::CalcToken::{self, *};
        let tokens: Vec<CalcToken> = vec![
            Digit(D5),
            OplusBotLeft,
            ParenOpen,
            Digit(D3),
            Add,
            Digit(D2),
            ParenClose,
        ];
        let exprs = build_rat_expr(&tokens).expect("rat track should not collapse");
        let result = eval_rational(&exprs).expect("rat eval should succeed");
        assert_eq!(
            result,
            Rational::from_ints(5, 2).unwrap(),
            "5⊕(3+2) must equal 5/2"
        );
    }

    #[test]
    fn period_longer_than_display_is_capped() {
        use crate::rational::Rational;
        let one_seventh = Rational::from_ints(1, 7).unwrap();
        let (buf, meta) = format_rational_result(&one_seventh);
        assert!(meta.start.is_some(), "1/7 must have a periodic part");
        assert_eq!(meta.len, MAX_PERIOD_DISPLAY);
        assert!(meta.capped, "true period (6 digits) exceeds display cap");
        let digit_count = buf
            .iter()
            .filter(|t| matches!(t, CalcToken::Digit(_)))
            .count();
        assert_eq!(digit_count, 1 + MAX_PERIOD_DISPLAY);
    }

    #[test]
    fn period_shorter_than_display_is_not_capped() {
        use crate::rational::Rational;
        let one_fifth = Rational::from_ints(1, 5).unwrap();
        let (_buf, meta) = format_rational_result(&one_fifth);
        assert!(meta.start.is_some());
        assert_eq!(meta.len, 4);
        assert!(!meta.capped);
    }

    #[test]
    fn negative_rational_renders_negate_token() {
        use crate::rational::Rational;
        let minus_half = Rational::from_ints(-1, 2).unwrap();
        let (buf, _meta) = format_rational_result(&minus_half);
        assert_eq!(
            buf.first(),
            Some(&CalcToken::Negate),
            "negative result must start with Negate"
        );
    }

    #[test]
    fn implicit_mul_constant_constant() {
        let input = vec![CalcToken::ConstPi, CalcToken::ConstPi];
        let out = with_implicit_muls(&input);
        assert_eq!(
            out,
            vec![CalcToken::ConstPi, CalcToken::Mul, CalcToken::ConstPi]
        );
    }

    #[test]
    fn implicit_mul_digit_paren() {
        use crate::digit::DozenalDigit::D2;
        let input = vec![CalcToken::Digit(D2), CalcToken::ParenOpen];
        let out = with_implicit_muls(&input);
        assert_eq!(
            out,
            vec![CalcToken::Digit(D2), CalcToken::Mul, CalcToken::ParenOpen]
        );
    }

    #[test]
    fn implicit_mul_close_open_paren() {
        let input = vec![CalcToken::ParenClose, CalcToken::ParenOpen];
        let out = with_implicit_muls(&input);
        assert_eq!(
            out,
            vec![CalcToken::ParenClose, CalcToken::Mul, CalcToken::ParenOpen]
        );
    }

    #[test]
    fn implicit_mul_digit_function() {
        use crate::digit::DozenalDigit::D2;
        let input = vec![CalcToken::Digit(D2), CalcToken::Sin];
        let out = with_implicit_muls(&input);
        assert_eq!(
            out,
            vec![CalcToken::Digit(D2), CalcToken::Mul, CalcToken::Sin]
        );
    }

    #[test]
    fn no_implicit_mul_within_number() {
        use crate::digit::DozenalDigit::{D1, D2};
        let input = vec![CalcToken::Digit(D1), CalcToken::Digit(D2)];
        let out = with_implicit_muls(&input);
        assert_eq!(
            out,
            vec![CalcToken::Digit(D1), CalcToken::Digit(D2)],
            "no Mul within a number"
        );
    }

    #[test]
    fn sqrt_after_pow_is_unary() {
        // Regression: 2^√3 muss als 2^(3^(1/2)) parsen, nicht als
        // n-te Wurzel mit `^` als n.
        let out = resolved(&["2", "^", "√", "3"]);
        assert_eq!(out, "2 ^ (3^(1/2))");
    }

    #[test]
    fn sqrt_after_percent_is_unary() {
        let out = resolved(&["5", "%", "√", "9"]);
        assert_eq!(out, "5 % (9^(1/2))");
    }

    #[test]
    fn factorial_postfix_to_prefix() {
        use crate::digit::DozenalDigit::D5;
        let input = vec![CalcToken::Digit(D5), CalcToken::Factorial];
        let out = resolve_postfix(&input);
        assert_eq!(
            out,
            vec![
                CalcToken::Factorial,
                CalcToken::ParenOpen,
                CalcToken::Digit(D5),
                CalcToken::ParenClose,
            ]
        );
    }

    #[test]
    fn factorial_prefix_unchanged() {
        use crate::digit::DozenalDigit::D5;
        // Kein Operand vor Factorial → unverändert lassen.
        let input = vec![CalcToken::Factorial, CalcToken::Digit(D5)];
        let out = resolve_postfix(&input);
        assert_eq!(out, input);
    }

    #[test]
    fn nested_factorial_postfix() {
        use crate::digit::DozenalDigit::D5;
        // 5!! → Factorial(Factorial(5))
        let input = vec![
            CalcToken::Digit(D5),
            CalcToken::Factorial,
            CalcToken::Factorial,
        ];
        let out = resolve_postfix(&input);
        let expected = vec![
            CalcToken::Factorial,
            CalcToken::ParenOpen,
            CalcToken::Factorial,
            CalcToken::ParenOpen,
            CalcToken::Digit(D5),
            CalcToken::ParenClose,
            CalcToken::ParenClose,
        ];
        assert_eq!(out, expected);
    }

    #[test]
    fn abs_postfix_to_prefix() {
        use crate::digit::DozenalDigit::{D2, D5};
        // 25 | → abs(25)
        let input = vec![
            CalcToken::Digit(D2),
            CalcToken::Digit(D5),
            CalcToken::AbsVal,
        ];
        let out = resolve_postfix(&input);
        assert_eq!(
            out,
            vec![
                CalcToken::AbsVal,
                CalcToken::ParenOpen,
                CalcToken::Digit(D2),
                CalcToken::Digit(D5),
                CalcToken::ParenClose,
            ]
        );
    }

    #[test]
    fn reciprocal_postfix_paren_operand() {
        use crate::digit::DozenalDigit::{D2, D3};
        // (2+3) 1/x → recip((2+3))
        let input = vec![
            CalcToken::ParenOpen,
            CalcToken::Digit(D2),
            CalcToken::Add,
            CalcToken::Digit(D3),
            CalcToken::ParenClose,
            CalcToken::Reciprocal,
        ];
        let out = resolve_postfix(&input);
        assert_eq!(
            out,
            vec![
                CalcToken::Reciprocal,
                CalcToken::ParenOpen,
                CalcToken::ParenOpen,
                CalcToken::Digit(D2),
                CalcToken::Add,
                CalcToken::Digit(D3),
                CalcToken::ParenClose,
                CalcToken::ParenClose,
            ]
        );
    }

    #[test]
    fn ratlit_token_evaluates_to_embedded_value() {
        use crate::rational::Rational;
        let prev_ans = Rational::from_ints(5, 7).unwrap();
        let tokens: Vec<CalcToken> = vec![
            CalcToken::RatLit(prev_ans.clone()),
            CalcToken::Add,
            CalcToken::RatLit(prev_ans),
        ];
        let exprs = build_rat_expr(&tokens).expect("rat track should not collapse");
        let result = eval_rational(&exprs).expect("rat eval should succeed");
        assert_eq!(result, Rational::from_ints(10, 7).unwrap());
    }
}
