// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

use crate::digit::{DozenalConverter, DozenalDigit};

/// Exact rational number. Invariants: `den > 0`, always in lowest terms.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rational {
    pub num: i128,
    pub den: i128,
}

fn gcd(a: i128, b: i128) -> i128 {
    let (mut a, mut b) = (a.abs(), b.abs());
    while b != 0 {
        let t = b;
        b = a % b;
        a = t;
    }
    a
}

impl Rational {
    /// Returns `None` on division by zero.
    pub fn new(num: i128, den: i128) -> Option<Self> {
        if den == 0 {
            return None;
        }
        let sign = if den < 0 { -1 } else { 1 };
        let g = gcd(num, den);
        Some(Self {
            num: sign * num / g,
            den: sign * den / g,
        })
    }

    pub fn zero() -> Self {
        Self { num: 0, den: 1 }
    }

    pub fn one() -> Self {
        Self { num: 1, den: 1 }
    }

    /// Checked addition. Returns `None` on overflow.
    pub fn add(self, rhs: Self) -> Option<Self> {
        let ad = self.num.checked_mul(rhs.den)?;
        let bc = rhs.num.checked_mul(self.den)?;
        let num = ad.checked_add(bc)?;
        let den = self.den.checked_mul(rhs.den)?;
        Self::new(num, den)
    }

    /// Checked subtraction. Returns `None` on overflow.
    pub fn sub(self, rhs: Self) -> Option<Self> {
        let ad = self.num.checked_mul(rhs.den)?;
        let bc = rhs.num.checked_mul(self.den)?;
        let num = ad.checked_sub(bc)?;
        let den = self.den.checked_mul(rhs.den)?;
        Self::new(num, den)
    }

    /// Checked multiplication. Returns `None` on overflow.
    pub fn mul(self, rhs: Self) -> Option<Self> {
        let num = self.num.checked_mul(rhs.num)?;
        let den = self.den.checked_mul(rhs.den)?;
        Self::new(num, den)
    }

    /// Checked division. Returns `None` on division by zero or overflow.
    pub fn div(self, rhs: Self) -> Option<Self> {
        if rhs.num == 0 {
            return None;
        }
        let num = self.num.checked_mul(rhs.den)?;
        let den = self.den.checked_mul(rhs.num)?;
        Self::new(num, den)
    }

    /// Integer power (negative exponent allowed). Returns `None` on overflow or 0^neg.
    pub fn pow(self, exp: i32) -> Option<Self> {
        if exp == 0 {
            return Some(Self::one());
        }
        if exp < 0 {
            if self.num == 0 {
                return None;
            }
            return Self::new(self.den, self.num)?.pow(-exp);
        }
        let mut result = Self::one();
        let mut base = self;
        let mut e = exp as u32;
        while e > 0 {
            if e & 1 == 1 {
                result = result.mul(base)?;
            }
            base = base.mul(base)?;
            e >>= 1;
        }
        Some(result)
    }

    /// Parallel-resistor operator: (a*b)/(a+b). Returns `None` on a+b=0 or overflow.
    pub fn oplus(self, rhs: Self) -> Option<Self> {
        let product = self.mul(rhs)?;
        let sum = self.add(rhs)?;
        product.div(sum)
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    /// Decomposes the fraction into its base-12 representation.
    /// Returns `(integer_digits, pre_period_digits, period_digits)`.
    /// `period_digits` is empty iff the expansion is finite.
    /// The period is capped at 100 digits to bound computation.
    pub fn to_dozenal_periodic(self) -> (Vec<DozenalDigit>, Vec<DozenalDigit>, Vec<DozenalDigit>) {
        let abs_num = self.num.abs();
        let den = self.den;

        let int_part = abs_num / den;
        let mut rem = abs_num % den;

        let int_digits = DozenalConverter::from_decimal(int_part as f64);

        let mut frac_digits: Vec<DozenalDigit> = Vec::new();
        let mut seen: std::collections::HashMap<i128, usize> = std::collections::HashMap::new();

        loop {
            if rem == 0 {
                return (int_digits, frac_digits, Vec::new());
            }
            if let Some(&first_pos) = seen.get(&rem) {
                let period = frac_digits.split_off(first_pos);
                return (int_digits, frac_digits, period);
            }
            if frac_digits.len() >= 100 {
                return (int_digits, frac_digits, Vec::new());
            }
            seen.insert(rem, frac_digits.len());
            // Bei Nennern > i128::MAX / 12 (≈ 1.4 · 10^37) würde rem * 12 wrappen.
            // Bis zur BigInt-Migration: abbrechen und non-periodisch zurückgeben.
            let Some(scaled) = rem.checked_mul(12) else {
                return (int_digits, frac_digits, Vec::new());
            };
            rem = scaled;
            let digit_val = (rem / den) as u32;
            rem %= den;
            if let Some(d) = DozenalDigit::from_value(digit_val) {
                frac_digits.push(d);
            }
        }
    }
}

/// Flat token stream for the rational evaluation track.
#[derive(Clone, Copy, Debug)]
pub enum RatExpr {
    Num(Rational),
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    OPlus,
    LParen,
    RParen,
}

/// Evaluates a `RatExpr` token sequence as an arithmetic expression.
/// Returns `None` if the expression is malformed, overflows, or uses a
/// non-integer exponent (which would make the result irrational).
pub fn eval_rational(exprs: &[RatExpr]) -> Option<Rational> {
    let mut p = RatParser { exprs, pos: 0 };
    let result = p.parse_add_sub()?;
    if p.pos == exprs.len() {
        Some(result)
    } else {
        None
    }
}

struct RatParser<'a> {
    exprs: &'a [RatExpr],
    pos: usize,
}

impl RatParser<'_> {
    fn peek(&self) -> Option<RatExpr> {
        self.exprs.get(self.pos).copied()
    }

    fn parse_add_sub(&mut self) -> Option<Rational> {
        let mut left = self.parse_mul_div()?;
        loop {
            match self.peek() {
                Some(RatExpr::Add) => {
                    self.pos += 1;
                    left = left.add(self.parse_mul_div()?)?;
                }
                Some(RatExpr::Sub) => {
                    self.pos += 1;
                    left = left.sub(self.parse_mul_div()?)?;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_mul_div(&mut self) -> Option<Rational> {
        let mut left = self.parse_pow()?;
        loop {
            match self.peek() {
                Some(RatExpr::Mul) => {
                    self.pos += 1;
                    left = left.mul(self.parse_pow()?)?;
                }
                Some(RatExpr::Div) => {
                    self.pos += 1;
                    left = left.div(self.parse_pow()?)?;
                }
                Some(RatExpr::OPlus) => {
                    self.pos += 1;
                    left = left.oplus(self.parse_pow()?)?;
                }
                _ => break,
            }
        }
        Some(left)
    }

    fn parse_pow(&mut self) -> Option<Rational> {
        match self.peek() {
            Some(RatExpr::Sub) => {
                self.pos += 1;
                let val = self.parse_pow()?;
                return val.mul(Rational::new(-1, 1)?);
            }
            Some(RatExpr::Add) => {
                self.pos += 1;
                return self.parse_pow();
            }
            _ => {}
        }
        let base = self.parse_primary()?;
        if matches!(self.peek(), Some(RatExpr::Pow)) {
            self.pos += 1;
            let exp = self.parse_pow()?;
            if exp.den != 1 {
                return None;
            }
            let e = i32::try_from(exp.num).ok()?;
            base.pow(e)
        } else {
            Some(base)
        }
    }

    fn parse_primary(&mut self) -> Option<Rational> {
        match self.peek()? {
            RatExpr::Num(r) => {
                self.pos += 1;
                Some(r)
            }
            RatExpr::LParen => {
                self.pos += 1;
                let val = self.parse_add_sub()?;
                if matches!(self.peek(), Some(RatExpr::RParen)) {
                    self.pos += 1;
                    Some(val)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rational_new_reduces() {
        let r = Rational::new(6, 9).unwrap();
        assert_eq!(r.num, 2);
        assert_eq!(r.den, 3);
    }

    #[test]
    fn rational_new_negative_den() {
        let r = Rational::new(1, -2).unwrap();
        assert_eq!(r.num, -1);
        assert_eq!(r.den, 2);
    }

    #[test]
    fn rational_new_div_by_zero() {
        assert!(Rational::new(1, 0).is_none());
    }

    #[test]
    fn rational_add() {
        let a = Rational::new(1, 3).unwrap();
        let b = Rational::new(1, 6).unwrap();
        let c = a.add(b).unwrap();
        assert_eq!(c.num, 1);
        assert_eq!(c.den, 2);
    }

    #[test]
    fn rational_sub() {
        let a = Rational::new(3, 4).unwrap();
        let b = Rational::new(1, 4).unwrap();
        let c = a.sub(b).unwrap();
        assert_eq!(c.num, 1);
        assert_eq!(c.den, 2);
    }

    #[test]
    fn rational_mul() {
        let a = Rational::new(2, 3).unwrap();
        let b = Rational::new(3, 4).unwrap();
        let c = a.mul(b).unwrap();
        assert_eq!(c.num, 1);
        assert_eq!(c.den, 2);
    }

    #[test]
    fn rational_div() {
        let a = Rational::new(1, 2).unwrap();
        let b = Rational::new(3, 4).unwrap();
        let c = a.div(b).unwrap();
        assert_eq!(c.num, 2);
        assert_eq!(c.den, 3);
    }

    #[test]
    fn rational_div_by_zero() {
        let a = Rational::new(1, 2).unwrap();
        let z = Rational::zero();
        assert!(a.div(z).is_none());
    }

    #[test]
    fn rational_pow_positive() {
        let a = Rational::new(2, 3).unwrap();
        let r = a.pow(3).unwrap();
        assert_eq!(r.num, 8);
        assert_eq!(r.den, 27);
    }

    #[test]
    fn rational_pow_negative() {
        let a = Rational::new(2, 3).unwrap();
        let r = a.pow(-1).unwrap();
        assert_eq!(r.num, 3);
        assert_eq!(r.den, 2);
    }

    #[test]
    fn rational_pow_zero() {
        let a = Rational::new(5, 7).unwrap();
        let r = a.pow(0).unwrap();
        assert_eq!(r, Rational::one());
    }

    #[test]
    fn rational_oplus() {
        let a = Rational::new(2, 1).unwrap();
        let b = Rational::new(3, 1).unwrap();
        let r = a.oplus(b).unwrap();
        assert_eq!(r.num, 6);
        assert_eq!(r.den, 5);
    }

    #[test]
    fn period_finite_half() {
        let r = Rational::new(1, 2).unwrap();
        let (int, pre, period) = r.to_dozenal_periodic();
        assert_eq!(int, vec![DozenalDigit::D0]);
        assert_eq!(pre, vec![DozenalDigit::D6]);
        assert!(period.is_empty());
    }

    #[test]
    fn period_one_fifth() {
        let r = Rational::new(1, 5).unwrap();
        let (_int, pre, period) = r.to_dozenal_periodic();
        assert!(pre.is_empty());
        assert_eq!(period.len(), 4);
        assert_eq!(period[0], DozenalDigit::D2);
        assert_eq!(period[1], DozenalDigit::D4);
        assert_eq!(period[2], DozenalDigit::D9);
        assert_eq!(period[3], DozenalDigit::D7);
    }

    #[test]
    fn period_one_eleventh() {
        let r = Rational::new(1, 11).unwrap();
        let (_int, pre, period) = r.to_dozenal_periodic();
        assert!(pre.is_empty());
        assert_eq!(period.len(), 1);
        assert_eq!(period[0], DozenalDigit::D1);
    }

    #[test]
    fn period_integer() {
        let r = Rational::new(7, 1).unwrap();
        let (int, pre, period) = r.to_dozenal_periodic();
        assert_eq!(int, vec![DozenalDigit::D7]);
        assert!(pre.is_empty());
        assert!(period.is_empty());
    }

    #[test]
    fn period_huge_denominator_does_not_overflow() {
        // den = i128::MAX provoziert irgendwann `rem * 12 > i128::MAX`.
        // Vor dem Fix: Panik in debug, silent wrap in release.
        // Nach dem Fix: kontrollierter Abbruch mit leerer Periode.
        let r = Rational::new(1, i128::MAX).unwrap();
        let (_int, _pre, _period) = r.to_dozenal_periodic();
        // Erfolg = kein Panik.
    }

    #[test]
    fn period_one_seventh() {
        let r = Rational::new(1, 7).unwrap();
        let (_int, pre, period) = r.to_dozenal_periodic();
        assert!(pre.is_empty());
        assert_eq!(period.len(), 6);
        assert_eq!(period[0], DozenalDigit::D1);
        assert_eq!(period[1], DozenalDigit::D8);
        assert_eq!(period[2], DozenalDigit::D6);
        assert_eq!(period[3], DozenalDigit::D10);
        assert_eq!(period[4], DozenalDigit::D3);
        assert_eq!(period[5], DozenalDigit::D5);
    }

    fn r(n: i128, d: i128) -> RatExpr {
        RatExpr::Num(Rational::new(n, d).unwrap())
    }

    #[test]
    fn eval_add() {
        let exprs = [r(1, 2), RatExpr::Add, r(1, 3)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result.num, 5);
        assert_eq!(result.den, 6);
    }

    #[test]
    fn eval_sub() {
        let exprs = [r(3, 4), RatExpr::Sub, r(1, 4)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result.num, 1);
        assert_eq!(result.den, 2);
    }

    #[test]
    fn eval_mul_div_precedence() {
        let exprs = [r(1, 1), RatExpr::Add, r(2, 1), RatExpr::Mul, r(3, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, Rational::new(7, 1).unwrap());
    }

    #[test]
    fn eval_pow() {
        let exprs = [r(2, 1), RatExpr::Pow, r(10, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, Rational::new(1024, 1).unwrap());
    }

    #[test]
    fn eval_pow_fraction_collapses() {
        let exprs = [
            r(4, 1),
            RatExpr::Pow,
            RatExpr::LParen,
            r(1, 1),
            RatExpr::Div,
            r(2, 1),
            RatExpr::RParen,
        ];
        assert!(eval_rational(&exprs).is_none());
    }

    #[test]
    fn eval_unary_minus() {
        let exprs = [RatExpr::Sub, r(5, 1), RatExpr::Add, r(3, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, Rational::new(-2, 1).unwrap());
    }

    #[test]
    fn eval_parens() {
        let exprs = [
            RatExpr::LParen,
            r(1, 1),
            RatExpr::Add,
            r(2, 1),
            RatExpr::RParen,
            RatExpr::Mul,
            r(4, 1),
        ];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, Rational::new(12, 1).unwrap());
    }

    #[test]
    fn eval_oplus() {
        let exprs = [r(2, 1), RatExpr::OPlus, r(3, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result.num, 6);
        assert_eq!(result.den, 5);
    }

    #[test]
    fn eval_div_by_zero_collapses() {
        let exprs = [r(1, 1), RatExpr::Div, r(0, 1)];
        assert!(eval_rational(&exprs).is_none());
    }
}
