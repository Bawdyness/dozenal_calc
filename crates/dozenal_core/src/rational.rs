// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, ToPrimitive, Zero};

use crate::digit::{DozenalConverter, DozenalDigit};

/// Exact rational number. Invariants: `den > 0`, always in lowest terms.
///
/// Backed by `BigInt`, so arithmetic never overflows; the only remaining
/// failure modes are division by zero and `0` raised to a negative power.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rational {
    pub num: BigInt,
    pub den: BigInt,
}

impl Rational {
    /// Returns `None` on division by zero.
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(num: BigInt, den: BigInt) -> Option<Self> {
        if den.is_zero() {
            return None;
        }
        let g = num.gcd(&den);
        let sign = if den.is_negative() {
            -BigInt::one()
        } else {
            BigInt::one()
        };
        Some(Self {
            num: &sign * (&num / &g),
            den: &sign * (&den / &g),
        })
    }

    /// Convenience-Konstruktor für i128-Literale — kürzer als `BigInt::from(…)`
    /// an Aufrufstellen, die ohnehin kleine Zahlen verwenden (Tests, Beispiele).
    pub fn from_ints(num: i128, den: i128) -> Option<Self> {
        Self::new(BigInt::from(num), BigInt::from(den))
    }

    pub fn zero() -> Self {
        Self {
            num: BigInt::zero(),
            den: BigInt::one(),
        }
    }

    pub fn one() -> Self {
        Self {
            num: BigInt::one(),
            den: BigInt::one(),
        }
    }

    /// # Panics
    /// Theoretisch unmöglich — `den` ist das Produkt zweier positiver Werte (invariant).
    pub fn add(&self, rhs: &Self) -> Self {
        let num = &self.num * &rhs.den + &rhs.num * &self.den;
        let den = &self.den * &rhs.den;
        Self::new(num, den).expect("den != 0 by invariant")
    }

    /// # Panics
    /// Theoretisch unmöglich — `den` ist das Produkt zweier positiver Werte (invariant).
    pub fn sub(&self, rhs: &Self) -> Self {
        let num = &self.num * &rhs.den - &rhs.num * &self.den;
        let den = &self.den * &rhs.den;
        Self::new(num, den).expect("den != 0 by invariant")
    }

    /// # Panics
    /// Theoretisch unmöglich — `den` ist das Produkt zweier positiver Werte (invariant).
    pub fn mul(&self, rhs: &Self) -> Self {
        let num = &self.num * &rhs.num;
        let den = &self.den * &rhs.den;
        Self::new(num, den).expect("den != 0 by invariant")
    }

    /// Returns `None` on division by zero.
    pub fn div(&self, rhs: &Self) -> Option<Self> {
        if rhs.num.is_zero() {
            return None;
        }
        let num = &self.num * &rhs.den;
        let den = &self.den * &rhs.num;
        Self::new(num, den)
    }

    /// Integer power (negative exponent allowed). Returns `None` on `0^negative`.
    pub fn pow(&self, exp: i32) -> Option<Self> {
        if exp == 0 {
            return Some(Self::one());
        }
        if exp < 0 {
            if self.num.is_zero() {
                return None;
            }
            // x^(-n) = (1/x)^n
            let inv = Self::new(self.den.clone(), self.num.clone())?;
            return inv.pow(-exp);
        }
        let mut result = Self::one();
        let mut base = self.clone();
        let mut e = exp as u32;
        while e > 0 {
            if e & 1 == 1 {
                result = result.mul(&base);
            }
            base = base.mul(&base);
            e >>= 1;
        }
        Some(result)
    }

    /// Parallel-resistor operator: `(a·b)/(a+b)`. Returns `None` on `a+b = 0`.
    pub fn oplus(&self, rhs: &Self) -> Option<Self> {
        let sum = self.add(rhs);
        if sum.num.is_zero() {
            return None;
        }
        let product = self.mul(rhs);
        product.div(&sum)
    }

    pub fn to_f64(&self) -> f64 {
        // BigInt → f64 ist verlustbehaftet bei sehr großen Werten;
        // num-traits liefert das nächste darstellbare f64.
        let num = self.num.to_f64().unwrap_or(f64::NAN);
        let den = self.den.to_f64().unwrap_or(f64::NAN);
        num / den
    }

    /// Decomposes the fraction into its base-12 representation.
    /// Returns `(integer_digits, pre_period_digits, period_digits)`.
    /// `period_digits` is empty iff the expansion is finite.
    /// The period is capped at 100 digits to bound computation.
    pub fn to_dozenal_periodic(&self) -> (Vec<DozenalDigit>, Vec<DozenalDigit>, Vec<DozenalDigit>) {
        let abs_num = self.num.abs();
        let den = self.den.clone();
        let twelve = BigInt::from(12);

        let int_part = &abs_num / &den;
        let mut rem = &abs_num % &den;

        let int_digits = int_part.to_f64().map_or_else(
            // Sehr großer Integer-Anteil: aus dem Bruch raus konvertieren.
            || big_int_to_dozenal_digits(&int_part),
            DozenalConverter::from_decimal,
        );

        let mut frac_digits: Vec<DozenalDigit> = Vec::new();
        let mut seen: std::collections::HashMap<BigInt, usize> = std::collections::HashMap::new();

        loop {
            if rem.is_zero() {
                return (int_digits, frac_digits, Vec::new());
            }
            if let Some(&first_pos) = seen.get(&rem) {
                let period = frac_digits.split_off(first_pos);
                return (int_digits, frac_digits, period);
            }
            if frac_digits.len() >= 100 {
                return (int_digits, frac_digits, Vec::new());
            }
            seen.insert(rem.clone(), frac_digits.len());
            rem *= &twelve;
            let digit_val = (&rem / &den).to_u32().unwrap_or(0);
            rem %= &den;
            if let Some(d) = DozenalDigit::from_value(digit_val) {
                frac_digits.push(d);
            }
        }
    }
}

/// Hilfsfunktion: konvertiert eine beliebig große positive BigInt in
/// eine Dozenal-Ziffer-Folge per Horner-Division.
fn big_int_to_dozenal_digits(value: &BigInt) -> Vec<DozenalDigit> {
    if value.is_zero() {
        return vec![DozenalDigit::D0];
    }
    let twelve = BigInt::from(12);
    let mut digits = Vec::new();
    let mut v = value.clone();
    while !v.is_zero() {
        let rem = (&v % &twelve).to_u32().unwrap_or(0);
        if let Some(d) = DozenalDigit::from_value(rem) {
            digits.push(d);
        }
        v /= &twelve;
    }
    digits.reverse();
    digits
}

/// Flat token stream for the rational evaluation track.
#[derive(Clone, Debug)]
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
/// Returns `None` if the expression is malformed or uses a non-integer
/// exponent (which would make the result irrational).
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
    fn peek(&self) -> Option<&RatExpr> {
        self.exprs.get(self.pos)
    }

    fn parse_add_sub(&mut self) -> Option<Rational> {
        let mut left = self.parse_mul_div()?;
        loop {
            match self.peek() {
                Some(RatExpr::Add) => {
                    self.pos += 1;
                    left = left.add(&self.parse_mul_div()?);
                }
                Some(RatExpr::Sub) => {
                    self.pos += 1;
                    left = left.sub(&self.parse_mul_div()?);
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
                    left = left.mul(&self.parse_pow()?);
                }
                Some(RatExpr::Div) => {
                    self.pos += 1;
                    left = left.div(&self.parse_pow()?)?;
                }
                Some(RatExpr::OPlus) => {
                    self.pos += 1;
                    left = left.oplus(&self.parse_pow()?)?;
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
                return Some(val.mul(&Rational::from_ints(-1, 1)?));
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
            if !exp.den.is_one() {
                return None;
            }
            let e = exp.num.to_i32()?;
            base.pow(e)
        } else {
            Some(base)
        }
    }

    fn parse_primary(&mut self) -> Option<Rational> {
        match self.peek()? {
            RatExpr::Num(r) => {
                let r = r.clone();
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

    fn r(n: i128, d: i128) -> Rational {
        Rational::from_ints(n, d).unwrap()
    }

    #[test]
    fn rational_new_reduces() {
        let r = r(6, 9);
        assert_eq!(r.num, BigInt::from(2));
        assert_eq!(r.den, BigInt::from(3));
    }

    #[test]
    fn rational_new_negative_den() {
        let r = r(1, -2);
        assert_eq!(r.num, BigInt::from(-1));
        assert_eq!(r.den, BigInt::from(2));
    }

    #[test]
    fn rational_new_div_by_zero() {
        assert!(Rational::from_ints(1, 0).is_none());
    }

    #[test]
    fn rational_add() {
        let a = r(1, 3);
        let b = r(1, 6);
        let c = a.add(&b);
        assert_eq!(c.num, BigInt::from(1));
        assert_eq!(c.den, BigInt::from(2));
    }

    #[test]
    fn rational_sub() {
        let a = r(3, 4);
        let b = r(1, 4);
        let c = a.sub(&b);
        assert_eq!(c.num, BigInt::from(1));
        assert_eq!(c.den, BigInt::from(2));
    }

    #[test]
    fn rational_mul() {
        let a = r(2, 3);
        let b = r(3, 4);
        let c = a.mul(&b);
        assert_eq!(c.num, BigInt::from(1));
        assert_eq!(c.den, BigInt::from(2));
    }

    #[test]
    fn rational_div() {
        let a = r(1, 2);
        let b = r(3, 4);
        let c = a.div(&b).unwrap();
        assert_eq!(c.num, BigInt::from(2));
        assert_eq!(c.den, BigInt::from(3));
    }

    #[test]
    fn rational_div_by_zero() {
        let a = r(1, 2);
        let z = Rational::zero();
        assert!(a.div(&z).is_none());
    }

    #[test]
    fn rational_pow_positive() {
        let a = r(2, 3);
        let r = a.pow(3).unwrap();
        assert_eq!(r.num, BigInt::from(8));
        assert_eq!(r.den, BigInt::from(27));
    }

    #[test]
    fn rational_pow_negative() {
        let a = r(2, 3);
        let r = a.pow(-1).unwrap();
        assert_eq!(r.num, BigInt::from(3));
        assert_eq!(r.den, BigInt::from(2));
    }

    #[test]
    fn rational_pow_zero() {
        let a = r(5, 7);
        let res = a.pow(0).unwrap();
        assert_eq!(res, Rational::one());
    }

    #[test]
    fn rational_oplus() {
        let a = r(2, 1);
        let b = r(3, 1);
        let res = a.oplus(&b).unwrap();
        assert_eq!(res.num, BigInt::from(6));
        assert_eq!(res.den, BigInt::from(5));
    }

    #[test]
    fn period_finite_half() {
        let r = r(1, 2);
        let (int, pre, period) = r.to_dozenal_periodic();
        assert_eq!(int, vec![DozenalDigit::D0]);
        assert_eq!(pre, vec![DozenalDigit::D6]);
        assert!(period.is_empty());
    }

    #[test]
    fn period_one_fifth() {
        let r = r(1, 5);
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
        let r = r(1, 11);
        let (_int, pre, period) = r.to_dozenal_periodic();
        assert!(pre.is_empty());
        assert_eq!(period.len(), 1);
        assert_eq!(period[0], DozenalDigit::D1);
    }

    #[test]
    fn period_integer() {
        let r = r(7, 1);
        let (int, pre, period) = r.to_dozenal_periodic();
        assert_eq!(int, vec![DozenalDigit::D7]);
        assert!(pre.is_empty());
        assert!(period.is_empty());
    }

    #[test]
    fn period_huge_denominator_does_not_overflow() {
        // BigInt kann nicht überlaufen — der Test bleibt als
        // Regressions-Schutz und verifiziert, dass der Pfad weiterhin
        // ohne Panik durchläuft.
        let r = r(1, i128::MAX);
        let (_int, _pre, _period) = r.to_dozenal_periodic();
    }

    #[test]
    fn period_one_seventh() {
        let r = r(1, 7);
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

    fn rx(n: i128, d: i128) -> RatExpr {
        RatExpr::Num(r(n, d))
    }

    #[test]
    fn eval_add() {
        let exprs = [rx(1, 2), RatExpr::Add, rx(1, 3)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result.num, BigInt::from(5));
        assert_eq!(result.den, BigInt::from(6));
    }

    #[test]
    fn eval_sub() {
        let exprs = [rx(3, 4), RatExpr::Sub, rx(1, 4)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result.num, BigInt::from(1));
        assert_eq!(result.den, BigInt::from(2));
    }

    #[test]
    fn eval_mul_div_precedence() {
        let exprs = [rx(1, 1), RatExpr::Add, rx(2, 1), RatExpr::Mul, rx(3, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, r(7, 1));
    }

    #[test]
    fn eval_pow() {
        let exprs = [rx(2, 1), RatExpr::Pow, rx(10, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, r(1024, 1));
    }

    #[test]
    fn eval_pow_fraction_collapses() {
        let exprs = [
            rx(4, 1),
            RatExpr::Pow,
            RatExpr::LParen,
            rx(1, 1),
            RatExpr::Div,
            rx(2, 1),
            RatExpr::RParen,
        ];
        assert!(eval_rational(&exprs).is_none());
    }

    #[test]
    fn eval_unary_minus() {
        let exprs = [RatExpr::Sub, rx(5, 1), RatExpr::Add, rx(3, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, r(-2, 1));
    }

    #[test]
    fn eval_parens() {
        let exprs = [
            RatExpr::LParen,
            rx(1, 1),
            RatExpr::Add,
            rx(2, 1),
            RatExpr::RParen,
            RatExpr::Mul,
            rx(4, 1),
        ];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result, r(12, 1));
    }

    #[test]
    fn eval_oplus() {
        let exprs = [rx(2, 1), RatExpr::OPlus, rx(3, 1)];
        let result = eval_rational(&exprs).unwrap();
        assert_eq!(result.num, BigInt::from(6));
        assert_eq!(result.den, BigInt::from(5));
    }

    #[test]
    fn eval_div_by_zero_collapses() {
        let exprs = [rx(1, 1), RatExpr::Div, rx(0, 1)];
        assert!(eval_rational(&exprs).is_none());
    }

    #[test]
    fn no_overflow_on_huge_multiplication() {
        // Vorher: 12^36 sprengte i128. Mit BigInt geht das ohne Kollaps.
        let a = r(12, 1);
        let mut acc = Rational::one();
        for _ in 0..40 {
            acc = acc.mul(&a);
        }
        // 12^40 = sehr große Zahl, aber als Rational dargestellt.
        assert!(!acc.num.is_zero());
        assert_eq!(acc.den, BigInt::one());
    }
}
