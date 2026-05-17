// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

//! Exakte Rational-Arithmetik, Periodenerkennung und Ausdrucks-Auswertung
//! für das Dozenal-Zahlensystem (Basis 12) und andere Basen.
//!
//! Die Crate ist die Logik-Schicht des Dozenal-Taschenrechners, getrennt
//! von der UI-Schicht: keine Abhängigkeit auf egui, Flutter oder
//! Web-Frameworks. Verwendbar in Native-Apps, WASM-Web-Apps und
//! (zukünftig) Embedded-Kontexten.
//!
//! Die zwei Auswertungs-Schienen:
//! - [`Rational`] + [`eval_rational`] — exakte Bruchrechnung mit
//!   Periodenerkennung in beliebiger Basis.
//! - String-basierter f64-Pfad ([`build_meval_string`]) — Fallback, sobald
//!   ein transzendenter Operator den Rational-Pfad kollabiert.
//!
//! `CalcToken` ist die kanonische Tasten-Sprache; sie wird von beliebigen
//! UI-Schichten konsumiert.

mod digit;
mod expression;
mod rational;
mod token;

pub use digit::{DozenalConverter, DozenalDigit, FRAC_EPSILON};
pub use expression::{
    F64_FRAC_DIGITS, MAX_PERIOD_DISPLAY, PeriodMeta, build_meval_string, build_rat_expr,
    format_f64_result, format_rational_result, resolve_custom_operators, resolve_postfix,
    with_implicit_muls,
};
pub use rational::{RatExpr, Rational, eval_rational};
pub use token::{AngleMode, CalcToken};
