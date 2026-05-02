// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::logic::{DozenalDigit, Rational};

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
    Negate, // unary minus in result_buffer; distinct from Sub (binary) to survive re-insertion
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

#[derive(Clone, Copy, PartialEq, Default)]
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

    /// Converts an angle from this mode to radians for meval.
    pub fn to_rad(self, x: f64) -> f64 {
        match self {
            AngleMode::Deg => x.to_radians(),
            AngleMode::Rad => x,
            AngleMode::Grad => x * std::f64::consts::PI / 200.0,
        }
    }

    /// Converts a result in radians to this mode's unit (for inverse trig).
    pub fn rad_to_unit(self, x: f64) -> f64 {
        match self {
            AngleMode::Deg => x.to_degrees(),
            AngleMode::Rad => x,
            AngleMode::Grad => x * 200.0 / std::f64::consts::PI,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default)]
pub enum InfoState {
    #[default]
    Closed,
    List,
    Chapter(usize),
}

pub struct DozenalCalcApp {
    pub input_buffer: Vec<CalcToken>,
    pub result_buffer: Vec<CalcToken>,
    // Period metadata — parallel to result_buffer; None when no period or f64 fallback.
    pub result_period_start: Option<usize>, // index in result_buffer where the period begins
    pub result_period_len: usize,           // digits in period, capped at 5 for display
    pub result_period_capped: bool,         // true when the true period exceeds 5 digits
    pub cursor_pos: usize,
    pub memory: Vec<CalcToken>,
    pub memory_rational: Option<Rational>, // exact value for STO/RCL roundtrip
    pub last_ans: Option<Rational>,
    pub last_result_f64: f64, // kept for decimal display mode
    pub result_cursor_pos: usize,
    pub result_field_active: bool, // true after = until next input modifies input_buffer
    pub error_msg: Option<String>,
    pub overlay_open: bool,
    pub angle_mode: AngleMode,
    pub display_dec: bool, // true = show result in decimal; dozenal is the default
    pub info_state: InfoState,
}

impl Default for DozenalCalcApp {
    fn default() -> Self {
        Self {
            input_buffer: Vec::new(),
            result_buffer: vec![CalcToken::Digit(DozenalDigit::D0)],
            result_period_start: None,
            result_period_len: 0,
            result_period_capped: false,
            cursor_pos: 0,
            memory: Vec::new(),
            memory_rational: None,
            last_ans: None,
            last_result_f64: 0.0,
            result_cursor_pos: 0,
            result_field_active: false,
            error_msg: None,
            overlay_open: false,
            angle_mode: AngleMode::Rad,
            display_dec: false,
            info_state: InfoState::Closed,
        }
    }
}
