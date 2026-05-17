// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use dozenal_core::{AngleMode, CalcToken, DozenalDigit, Rational};

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
    pub result_period_start: Option<usize>,
    pub result_period_len: usize,
    pub result_period_capped: bool,
    pub cursor_pos: usize,
    pub memory: Vec<CalcToken>,
    pub memory_rational: Option<Rational>,
    pub last_ans: Option<Rational>,
    pub last_result_f64: f64,
    pub result_cursor_pos: usize,
    pub result_field_active: bool,
    pub error_msg: Option<String>,
    pub overlay_open: bool,
    pub angle_mode: AngleMode,
    pub display_dec: bool,
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
