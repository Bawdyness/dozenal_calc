// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::state::DozenalCalcApp;
use dozenal_core::{
    AngleMode, build_meval_string, build_rat_expr, eval_rational, format_f64_result,
    format_rational_result, with_implicit_muls,
};

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

impl DozenalCalcApp {
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

                self.result_cursor_pos = 0;
                self.result_field_active = true;
            }
            Ok(result) if result.is_nan() => {
                self.error_msg = Some("DOMAIN ERROR".to_string());
            }
            Ok(_) => {
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

    #[test]
    fn acot_convention_a() {
        let pi = std::f64::consts::PI;
        assert!((eval("acot(1)") - pi / 4.0).abs() < 1e-10);
        assert!((eval("acot(-1)") - 3.0 * pi / 4.0).abs() < 1e-10);
        assert!((eval("acot(0)") - pi / 2.0).abs() < 1e-10);
    }

    #[test]
    fn cot_basic() {
        assert!((eval("cot(pi/4)") - 1.0).abs() < 1e-10);
        assert!((eval("6/cot(pi/4)") - 6.0).abs() < 1e-10);
    }

    #[test]
    fn sqrt_mid_expression() {
        assert!((eval("(16^(1/2))") - 4.0).abs() < 1e-10);
        assert!((eval("(8^(1/9))") - 8_f64.powf(1.0 / 9.0)).abs() < 1e-10);
    }

    #[test]
    fn unary_minus() {
        assert!((eval("-5+3") - (-2.0)).abs() < 1e-10);
        assert!((eval("5*-3") - (-15.0)).abs() < 1e-10);
        assert!((eval("5-3") - 2.0).abs() < 1e-10);
    }
}
