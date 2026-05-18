// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

/// Eine Ziffer im Dozenal-Zahlensystem (Basis 12), Werte 0..=11.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DozenalDigit {
    D0,
    D1,
    D2,
    D3,
    D4,
    D5,
    D6,
    D7,
    D8,
    D9,
    D10,
    D11,
}

impl DozenalDigit {
    /// Numerischer Wert der Ziffer (0..=11).
    #[must_use]
    pub fn to_value(self) -> u32 {
        match self {
            DozenalDigit::D0 => 0,
            DozenalDigit::D1 => 1,
            DozenalDigit::D2 => 2,
            DozenalDigit::D3 => 3,
            DozenalDigit::D4 => 4,
            DozenalDigit::D5 => 5,
            DozenalDigit::D6 => 6,
            DozenalDigit::D7 => 7,
            DozenalDigit::D8 => 8,
            DozenalDigit::D9 => 9,
            DozenalDigit::D10 => 10,
            DozenalDigit::D11 => 11,
        }
    }

    /// Ziffer aus einem Wert. Gibt `None` für Werte ausserhalb `0..=11`.
    pub fn from_value(val: u32) -> Option<Self> {
        match val {
            0 => Some(DozenalDigit::D0),
            1 => Some(DozenalDigit::D1),
            2 => Some(DozenalDigit::D2),
            3 => Some(DozenalDigit::D3),
            4 => Some(DozenalDigit::D4),
            5 => Some(DozenalDigit::D5),
            6 => Some(DozenalDigit::D6),
            7 => Some(DozenalDigit::D7),
            8 => Some(DozenalDigit::D8),
            9 => Some(DozenalDigit::D9),
            10 => Some(DozenalDigit::D10),
            11 => Some(DozenalDigit::D11),
            _ => None,
        }
    }
}

/// Numerical tolerance used when extracting dozenal digits from an `f64` fractional
/// part. Floating-point arithmetic introduces drift below this threshold; treating
/// it as zero avoids spurious trailing-digit noise like `0.6000000000001` showing
/// up as `0.6` instead of bleeding into a fake periodic tail.
pub const FRAC_EPSILON: f64 = 0.000_001;

pub struct DozenalConverter;

impl DozenalConverter {
    /// Exact integer conversion via Horner's method. Returns `None` on i128 overflow.
    pub fn to_decimal_exact(digits: &[DozenalDigit]) -> Option<i128> {
        let mut result: i128 = 0;
        for digit in digits {
            result = result
                .checked_mul(12)?
                .checked_add(i128::from(digit.to_value()))?;
        }
        Some(result)
    }

    /// Wandelt eine Ziffer-Liste in einen `f64`-Wert um (Horner via `12.0_f64.powi`).
    #[must_use]
    pub fn to_decimal(digits: &[DozenalDigit]) -> f64 {
        let mut result = 0.0;
        for (i, digit) in digits.iter().rev().enumerate() {
            result += f64::from(digit.to_value()) * 12.0_f64.powi(i as i32);
        }
        result
    }

    /// Wandelt eine `f64`-Zahl in ihre Dozenal-Ziffer-Folge (Ganzzahl-Anteil) um.
    #[must_use]
    pub fn from_decimal(value: f64) -> Vec<DozenalDigit> {
        let mut digits = Vec::new();
        let mut integer_part = value.floor();
        if integer_part < 1.0 {
            digits.push(DozenalDigit::D0);
        } else {
            while integer_part >= 1.0 {
                let remainder = (integer_part % 12.0) as u32;
                if let Some(d) = DozenalDigit::from_value(remainder) {
                    digits.push(d);
                }
                integer_part = (integer_part / 12.0).floor();
            }
            digits.reverse();
        }
        digits
    }

    /// Extrahiert bis zu `precision` Dozenal-Bruchziffern aus dem Nachkommaanteil.
    /// Bricht früh ab, wenn der Rest unter `FRAC_EPSILON` fällt.
    #[must_use]
    pub fn frac_to_digits(mut frac: f64, precision: usize) -> Vec<DozenalDigit> {
        let mut digits = Vec::new();
        for _ in 0..precision {
            frac *= 12.0;
            let d_val = (frac + FRAC_EPSILON).floor() as u32;
            if let Some(d) = DozenalDigit::from_value(d_val) {
                digits.push(d);
            }
            frac -= f64::from(d_val);
            if frac.abs() < FRAC_EPSILON {
                break;
            }
        }
        digits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversion() {
        let dec = 14.0;
        let doz = DozenalConverter::from_decimal(dec);
        assert_eq!(doz, vec![DozenalDigit::D1, DozenalDigit::D2]);

        let doz_input = vec![DozenalDigit::D1, DozenalDigit::D4];
        let dec_result = DozenalConverter::to_decimal(&doz_input);
        assert_eq!(dec_result, 16.0);
    }

    #[test]
    fn to_decimal_exact_basic() {
        assert_eq!(DozenalConverter::to_decimal_exact(&[]), Some(0));
        assert_eq!(
            DozenalConverter::to_decimal_exact(&[DozenalDigit::D1, DozenalDigit::D0]),
            Some(12)
        );
        assert_eq!(
            DozenalConverter::to_decimal_exact(&[DozenalDigit::D1, DozenalDigit::D1]),
            Some(13)
        );
    }
}
