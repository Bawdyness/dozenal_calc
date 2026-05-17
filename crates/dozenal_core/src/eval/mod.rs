// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

//! f64-Ausdrucks-Evaluator (Lexer + Recursive-Descent-Parser + Funktions-Dispatch).
//!
//! Ersetzt die früher genutzte `meval`-Crate (unmaintained seit 2020). Eingebaut
//! sind die Konstanten `pi` und `e`, die Trig-Familie (sin/cos/tan/cot + Inverse)
//! mit Winkelmodus, die hyperbolische Familie (numerisch gehärtet — siehe
//! `interpret`-Modul), `ln`, `fact`, `abs`, `recip`, sowie die Operatoren
//! `+ − × / ^ %` und Klammern.

mod interpret;
mod lexer;
mod parser;

use crate::token::AngleMode;

/// Wertet einen f64-Ausdrucks-String aus.
///
/// Gibt `None` zurück bei Syntaxfehler (unbekannter Identifier, unmatched
/// Klammer, Tokenisierungsfehler etc.). NaN und ±∞ bleiben als f64-Wert
/// erhalten — der Aufrufer entscheidet, ob das als `DOMAIN ERROR` oder
/// `DIV BY ZERO` interpretiert wird.
pub fn eval_f64(expr: &str, angle_mode: AngleMode) -> Option<f64> {
    let toks = lexer::tokenize(expr).ok()?;
    parser::Parser::new(&toks, angle_mode).parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval_rad(expr: &str) -> f64 {
        eval_f64(expr, AngleMode::Rad).expect("must evaluate")
    }

    #[test]
    fn arithmetic_precedence() {
        assert_eq!(eval_rad("1 + 2 * 3"), 7.0);
        assert_eq!(eval_rad("(1 + 2) * 3"), 9.0);
        assert_eq!(eval_rad("10 - 3 - 2"), 5.0);
        assert_eq!(eval_rad("8 / 4 / 2"), 1.0);
    }

    #[test]
    fn unary_minus() {
        assert_eq!(eval_rad("-5+3"), -2.0);
        assert_eq!(eval_rad("5*-3"), -15.0);
        assert_eq!(eval_rad("5-3"), 2.0);
        assert_eq!(eval_rad("-(2+3)"), -5.0);
    }

    #[test]
    fn power_is_right_associative() {
        // 2^3^2 = 2^(3^2) = 2^9 = 512, nicht (2^3)^2 = 64
        assert_eq!(eval_rad("2^3^2"), 512.0);
        // 2^-3 = 0.125
        assert!((eval_rad("2^-3") - 0.125).abs() < 1e-12);
    }

    #[test]
    fn pi_and_e_constants() {
        assert!((eval_rad("pi") - std::f64::consts::PI).abs() < 1e-15);
        assert!((eval_rad("e") - std::f64::consts::E).abs() < 1e-15);
    }

    #[test]
    fn acot_convention_a() {
        let pi = std::f64::consts::PI;
        assert!((eval_rad("acot(1)") - pi / 4.0).abs() < 1e-10);
        assert!((eval_rad("acot(-1)") - 3.0 * pi / 4.0).abs() < 1e-10);
        assert!((eval_rad("acot(0)") - pi / 2.0).abs() < 1e-10);
    }

    #[test]
    fn cot_in_denominator() {
        // cot(π/4) = 1; 6 / cot(π/4) = 6
        assert!((eval_rad("cot(pi/4)") - 1.0).abs() < 1e-10);
        assert!((eval_rad("6/cot(pi/4)") - 6.0).abs() < 1e-10);
    }

    #[test]
    fn sqrt_via_pow_half() {
        assert!((eval_rad("(16^(1/2))") - 4.0).abs() < 1e-10);
        assert!((eval_rad("(8^(1/9))") - 8_f64.powf(1.0 / 9.0)).abs() < 1e-10);
    }

    #[test]
    fn hyperbolic_basics() {
        assert!((eval_rad("sinh(0)")).abs() < 1e-15);
        assert!((eval_rad("cosh(0)") - 1.0).abs() < 1e-15);
        assert!((eval_rad("tanh(0)")).abs() < 1e-15);
        // arsinh(sinh(2)) = 2
        assert!((eval_rad("arsinh(sinh(2))") - 2.0).abs() < 1e-9);
    }

    #[test]
    fn factorial() {
        assert_eq!(eval_rad("fact(0)"), 1.0);
        assert_eq!(eval_rad("fact(5)"), 120.0);
        assert_eq!(eval_rad("fact(10)"), 3_628_800.0);
    }

    #[test]
    fn abs_and_recip() {
        assert_eq!(eval_rad("abs(-7)"), 7.0);
        assert!((eval_rad("recip(4)") - 0.25).abs() < 1e-15);
    }

    #[test]
    fn angle_mode_changes_trig() {
        // sin(90°) = 1
        let v = eval_f64("sin(90)", AngleMode::Deg).unwrap();
        assert!((v - 1.0).abs() < 1e-12);
        // sin(100 grad) = sin(90°) = 1
        let v = eval_f64("sin(100)", AngleMode::Grad).unwrap();
        assert!((v - 1.0).abs() < 1e-12);
    }

    #[test]
    fn syntax_error_returns_none() {
        assert!(eval_f64("1 +", AngleMode::Rad).is_none());
        assert!(eval_f64("(1 + 2", AngleMode::Rad).is_none());
        assert!(eval_f64("unknown_fn(1)", AngleMode::Rad).is_none());
        assert!(eval_f64("", AngleMode::Rad).is_none());
    }

    #[test]
    fn division_by_zero_yields_infinity() {
        // Aufrufer entscheidet was damit zu tun ist; eval_f64 selbst gibt ∞ zurück.
        let v = eval_f64("1/0", AngleMode::Rad).unwrap();
        assert!(v.is_infinite());
    }

    #[test]
    fn domain_error_yields_nan() {
        // arcosh(0) ist außerhalb der Domain → NaN
        let v = eval_f64("arcosh(0)", AngleMode::Rad).unwrap();
        assert!(v.is_nan());
    }
}
