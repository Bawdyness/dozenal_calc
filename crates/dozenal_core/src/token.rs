// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

use crate::digit::DozenalDigit;
use crate::rational::Rational;

/// Alle möglichen Tasten-Tokens des Taschenrechners. UI-agnostisch — keine
/// egui-/Leptos-/Flutter-Abhängigkeit, nur die semantische Bedeutung.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum CalcToken {
    // Main keypad
    Digit(DozenalDigit),
    Add,
    Sub,
    Mul,
    Div,
    ExpTopRight,
    RootTopLeft,
    OplusBotLeft,
    LogBotRight,
    Sin,
    Cos,
    Tan,
    Cot,
    ArcSin,
    ArcCos,
    ArcTan,
    ArcCot,
    ParenOpen,
    ParenClose,
    TriangleRight,
    TriangleLeft,
    AC,
    Del,
    Decimal,
    Equals,
    Expand,
    /// Unary minus in result_buffer; distinct from Sub (binary) to survive re-insertion.
    Negate,
    /// Exact rational literal inserted by Ans / RCL. Carries the value through the pipeline
    /// so periodicity survives a STO→RCL or Ans re-use roundtrip without precision loss.
    RatLit(Rational),
    // Overlay Set 6 — Memory
    Sto,
    Rcl,
    Mc,
    Ans,
    // Overlay Set 7 — Constants
    ConstPi,
    ConstE,
    ConstPhi,
    ConstSqrt2,
    // Overlay Set 8 — Hyperbolic
    Sinh,
    Cosh,
    Tanh,
    Coth,
    ArSinh,
    ArCosh,
    ArTanh,
    ArCoth,
    // Overlay Set 9 — Extended
    Factorial,
    AbsVal,
    Reciprocal,
    Mod,
    // Overlay Set 10 — Modes & Meta
    DozDec,
    Drg,
    Info,
    Close,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum AngleMode {
    Deg,
    #[default]
    Rad,
    Grad,
}

impl AngleMode {
    pub fn label(self) -> &'static str {
        match self {
            AngleMode::Deg => "DEG",
            AngleMode::Rad => "RAD",
            AngleMode::Grad => "GRD",
        }
    }

    pub fn next(self) -> Self {
        match self {
            AngleMode::Deg => AngleMode::Rad,
            AngleMode::Rad => AngleMode::Grad,
            AngleMode::Grad => AngleMode::Deg,
        }
    }

    /// Konvertiert einen Winkel von diesem Modus in Radian.
    pub fn to_rad(self, x: f64) -> f64 {
        match self {
            AngleMode::Deg => x.to_radians(),
            AngleMode::Rad => x,
            AngleMode::Grad => x * std::f64::consts::PI / 200.0,
        }
    }

    /// Konvertiert ein Resultat in Radian zurück in die Einheit dieses Modus.
    pub fn rad_to_unit(self, x: f64) -> f64 {
        match self {
            AngleMode::Deg => x.to_degrees(),
            AngleMode::Rad => x,
            AngleMode::Grad => x * 200.0 / std::f64::consts::PI,
        }
    }
}
