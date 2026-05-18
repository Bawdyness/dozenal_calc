// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

#![doc = include_str!("../README.md")]

mod digit;
mod eval;
mod expression;
mod rational;
mod token;

pub use digit::{DozenalConverter, DozenalDigit, FRAC_EPSILON};
pub use eval::eval_f64;
pub use expression::{
    F64_FRAC_DIGITS, MAX_PERIOD_DISPLAY, PeriodMeta, build_meval_string, build_rat_expr,
    format_f64_as_decimal, format_f64_result, format_rational_result, resolve_custom_operators,
    resolve_postfix, with_implicit_muls,
};
pub use rational::{RatExpr, Rational, eval_rational};
pub use token::{AngleMode, CalcToken};
