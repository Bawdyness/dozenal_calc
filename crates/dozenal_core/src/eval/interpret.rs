// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

//! Funktions-Dispatch des f64-Evaluators.
//!
//! Trennt die Funktions-Semantik (sin, cot, fact, …) vom Parsing.
//! Numerisch gehärtete Implementierungen für hyperbolische Funktionen
//! und `fact` — siehe Modul-Doku der einzelnen Helper.

use crate::token::AngleMode;

pub(super) fn apply_func(name: &str, x: f64, angle_mode: AngleMode) -> Option<f64> {
    Some(match name {
        "sin" => angle_mode.to_rad(x).sin(),
        "cos" => angle_mode.to_rad(x).cos(),
        "tan" => angle_mode.to_rad(x).tan(),
        "cot" => 1.0 / angle_mode.to_rad(x).tan(),
        "asin" => angle_mode.rad_to_unit(x.asin()),
        "acos" => angle_mode.rad_to_unit(x.acos()),
        "atan" => angle_mode.rad_to_unit(x.atan()),
        // Convention A: acot range (0, π), acot(x) = π/2 − atan(x).
        "acot" => angle_mode.rad_to_unit(std::f64::consts::FRAC_PI_2 - x.atan()),
        "sinh" => sinh_safe(x),
        "cosh" => cosh_safe(x),
        "tanh" => tanh_safe(x),
        "coth" => cosh_safe(x) / sinh_safe(x),
        "arsinh" => arsinh_safe(x),
        "arcosh" => (x + (x * x - 1.0).sqrt()).ln(),
        "artanh" => 0.5 * ((1.0 + x) / (1.0 - x)).ln(),
        "arcoth" => 0.5 * ((x + 1.0) / (x - 1.0)).ln(),
        "ln" => x.ln(),
        "fact" => fact_safe(x),
        "abs" => x.abs(),
        "recip" => 1.0 / x,
        _ => return None,
    })
}

fn sinh_safe(x: f64) -> f64 {
    (x.exp() - (-x).exp()) / 2.0
}

fn cosh_safe(x: f64) -> f64 {
    f64::midpoint(x.exp(), (-x).exp())
}

/// `tanh` saturiert ab |x| > 20 — jenseits davon rundet das Ergebnis ohnehin
/// auf ±1, und ab |x| ≈ 709 läuft `exp` über zu `∞`, was die Formel
/// (∞−0)/(∞+0) = NaN ergeben würde.
fn tanh_safe(x: f64) -> f64 {
    if x > 20.0 {
        return 1.0;
    }
    if x < -20.0 {
        return -1.0;
    }
    let ex = x.exp();
    let emx = (-x).exp();
    (ex - emx) / (ex + emx)
}

/// `arsinh` mit symmetrischer Formel für negative x via `arsinh(−x) = −arsinh(x)`.
/// Die naive Form `ln(x + √(x²+1))` leidet bei negativem x an catastrophic
/// cancellation: |x| − √(x²+1) ≈ 0 → ln(≈0) → −∞.
fn arsinh_safe(x: f64) -> f64 {
    if x < 0.0 {
        -(-x + (x * x + 1.0).sqrt()).ln()
    } else {
        (x + (x * x + 1.0).sqrt()).ln()
    }
}

/// `fact` über BigInt, damit auch grosse n korrekt sind, bevor die Konvertierung
/// nach f64 die Genauigkeit reduziert. Schützt gegen NaN/∞-Eingaben.
fn fact_safe(x: f64) -> f64 {
    use num_bigint::BigInt;
    use num_traits::ToPrimitive;
    if x.is_nan() || x.is_infinite() {
        return f64::NAN;
    }
    let n = x.round() as i64;
    if n < 0 {
        return f64::NAN;
    }
    let mut r = BigInt::from(1);
    for i in 1..=n {
        r *= BigInt::from(i);
    }
    r.to_f64().unwrap_or(f64::NAN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tanh_saturates_at_extremes() {
        assert_eq!(tanh_safe(100.0), 1.0);
        assert_eq!(tanh_safe(-100.0), -1.0);
        // Nicht-saturierter Bereich bleibt korrekt
        assert!((tanh_safe(0.0)).abs() < 1e-12);
    }

    #[test]
    fn arsinh_handles_negative_without_cancellation() {
        // arsinh(-1) = -arsinh(1) ≈ -0.8814
        assert!((arsinh_safe(-1.0) + 0.881_373_587).abs() < 1e-6);
        assert!((arsinh_safe(-100.0) + (100.0_f64 + (10001.0_f64).sqrt()).ln()).abs() < 1e-9);
    }

    #[test]
    fn fact_basic_values() {
        assert_eq!(fact_safe(0.0), 1.0);
        assert_eq!(fact_safe(1.0), 1.0);
        assert_eq!(fact_safe(5.0), 120.0);
        assert_eq!(fact_safe(10.0), 3_628_800.0);
    }

    #[test]
    fn fact_negative_or_nan_yields_nan() {
        assert!(fact_safe(-1.0).is_nan());
        assert!(fact_safe(f64::NAN).is_nan());
        assert!(fact_safe(f64::INFINITY).is_nan());
    }
}
