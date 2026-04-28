// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::logic::{DozenalConverter, DozenalDigit, RatExpr, Rational, eval_rational};
use crate::tokens::{CalcToken, DozenalCalcApp};

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

impl DozenalCalcApp {
    // --- RECHEN-LOGIK ---
    pub fn calculate_result(&mut self) {
        let expanded = with_implicit_muls(&self.input_buffer);
        let mut int_digits = Vec::new();
        let mut frac_digits = Vec::new();
        let mut in_fraction = false;
        let mut tokens_str: Vec<String> = Vec::new();

        for token in &expanded {
            match token {
                CalcToken::Digit(d) => {
                    if in_fraction {
                        frac_digits.push(*d);
                    } else {
                        int_digits.push(*d);
                    }
                }
                CalcToken::Decimal => {
                    in_fraction = true;
                }
                _ => {
                    if !int_digits.is_empty() || !frac_digits.is_empty() {
                        let int_val = if int_digits.is_empty() {
                            "0".to_string()
                        } else {
                            DozenalConverter::to_decimal(&int_digits).to_string()
                        };
                        if in_fraction && !frac_digits.is_empty() {
                            let frac_val = DozenalConverter::to_decimal(&frac_digits).to_string();
                            let len = frac_digits.len();
                            tokens_str.push(format!("({}+({}/(12^{})))", int_val, frac_val, len));
                        } else {
                            tokens_str.push(int_val);
                        }
                        int_digits.clear();
                        frac_digits.clear();
                        in_fraction = false;
                    }

                    let s = match token {
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
                    };
                    // RatLit and irrational constants push their f64 values directly.
                    let const_val: Option<f64> = match token {
                        CalcToken::ConstPi => Some(std::f64::consts::PI),
                        CalcToken::ConstE => Some(std::f64::consts::E),
                        CalcToken::ConstPhi => Some(1.618_033_988_749_895),
                        CalcToken::ConstSqrt2 => Some(std::f64::consts::SQRT_2),
                        _ => None,
                    };
                    if let CalcToken::RatLit(r) = token {
                        tokens_str.push(r.to_f64().to_string());
                    } else if let Some(v) = const_val {
                        tokens_str.push(v.to_string());
                    } else if !s.is_empty() {
                        tokens_str.push(s.to_string());
                    }
                }
            }
        }

        if !int_digits.is_empty() || !frac_digits.is_empty() {
            let int_val = if int_digits.is_empty() {
                "0".to_string()
            } else {
                DozenalConverter::to_decimal(&int_digits).to_string()
            };
            if in_fraction && !frac_digits.is_empty() {
                let frac_val = DozenalConverter::to_decimal(&frac_digits).to_string();
                let len = frac_digits.len();
                tokens_str.push(format!("({}+({}/(12^{})))", int_val, frac_val, len));
            } else {
                tokens_str.push(int_val);
            }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "⊕") {
            if i > 0 && i + 1 < tokens_str.len() {
                let a = &tokens_str[i - 1];
                let b = &tokens_str[i + 1];
                tokens_str.splice(
                    (i - 1)..=(i + 1),
                    vec![format!("(({}*{})/({}+{}))", a, b, a, b)],
                );
            } else {
                break;
            }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "√") {
            if i + 1 >= tokens_str.len() {
                break;
            }
            // If preceded by an operator/open-paren (or at position 0), treat as √x (square root).
            // If preceded by a number/closing-paren, treat as n√x (n-th root).
            let preceded_by_op =
                i == 0 || matches!(tokens_str[i - 1].as_str(), "+" | "-" | "*" | "/" | "(");
            if preceded_by_op {
                let x = tokens_str[i + 1].clone();
                tokens_str.splice(i..=(i + 1), vec![format!("({}^(1/2))", x)]);
            } else {
                let n = tokens_str[i - 1].clone();
                let x = tokens_str[i + 1].clone();
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("({}^(1/{}))", x, n)]);
            }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "log") {
            if i > 0 && i + 1 < tokens_str.len() {
                let x = &tokens_str[i - 1];
                let n = &tokens_str[i + 1];
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("(ln({})/ln({}))", x, n)]);
            } else {
                break;
            }
        }

        let mut math_string = tokens_str.join(" ");
        let open_parens = math_string.matches('(').count();
        let close_parens = math_string.matches(')').count();
        for _ in 0..(open_parens.saturating_sub(close_parens)) {
            math_string.push(')');
        }

        // --- Rational track (runs independently of meval) ---
        let rat_result = build_rat_expr(&expanded).and_then(|exprs| eval_rational(&exprs));

        // Copy angle mode so the closures can capture it by value (no borrow of self).
        let am = self.angle_mode;
        let mut ctx = meval::Context::new();
        // Angle-mode-aware trig — shadow meval builtins so DRG takes effect.
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
        ctx.func("arsinh", |x: f64| x.asinh());
        ctx.func("arcosh", |x: f64| x.acosh());
        ctx.func("artanh", |x: f64| x.atanh());
        ctx.func("arcoth", |x: f64| 0.5 * ((x + 1.0) / (x - 1.0)).ln());
        ctx.func("fact", |x: f64| {
            let n = x.round() as u64;
            (1..=n).fold(1u64, |acc, i| acc.saturating_mul(i)) as f64
        });
        ctx.func("abs", |x: f64| x.abs());
        ctx.func("recip", |x: f64| 1.0 / x);
        match meval::eval_str_with_context(&math_string, (ctx, meval::builtin())) {
            Ok(result) if result.is_finite() => {
                self.error_msg = None;
                self.last_ans = rat_result;
                self.last_result_f64 = result;

                if let Some(r) = rat_result {
                    // --- Rational track: exact display with possible overline ---
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
                    let period_start = if period_d.is_empty() {
                        None
                    } else {
                        Some(buf.len())
                    };
                    let period_capped = period_d.len() > 5;
                    let period_len = period_d.len().min(5);
                    buf.extend(period_d.into_iter().take(5).map(CalcToken::Digit));
                    self.result_buffer = buf;
                    self.result_period_start = period_start;
                    self.result_period_len = period_len;
                    self.result_period_capped = period_capped;
                } else {
                    // --- f64 fallback: 4 fractional dozenal digits, no overline ---
                    let mut buf: Vec<CalcToken> = Vec::new();
                    let mut val = result;
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
                    if frac_part > 0.000001 {
                        buf.push(CalcToken::Decimal);
                        buf.extend(
                            DozenalConverter::frac_to_digits(frac_part, 4)
                                .into_iter()
                                .map(CalcToken::Digit),
                        );
                    }
                    self.result_buffer = buf;
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
}
