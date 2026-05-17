// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

//! Reaktiver State der Leptos-Web-App.
//!
//! Spiegelt strukturell `crates/dozenal_calc_app/src/state.rs` (egui)
//! und `lib/state.dart` (Flutter), aber jedes Feld lebt als `RwSignal`,
//! sodass Komponenten fine-grained re-rendern.
//!
//! Phase B legt nur Struktur und `calculate_result` an. Tasten-Handling
//! (`handle_click`, `inverse_swap`, Decimal-Filter, Mode-Toggles) folgt
//! in Phase C zusammen mit der Tastatur.

use dozenal_core::{
    AngleMode, CalcToken, DozenalDigit, Rational, build_meval_string, build_rat_expr, eval_f64,
    eval_rational, format_f64_result, format_rational_result, resolve_postfix, with_implicit_muls,
};
use leptos::prelude::*;

/// Aktive Zahl-Basis für Eingabe und Anzeige. Spiegelt das Flutter-`NumeralSystem`.
/// `Dez` wird in Phase D durch die `Doz↔Dez`-Mode-Taste genutzt.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[allow(dead_code)]
pub enum NumeralSystem {
    #[default]
    Doz,
    Dez,
}

impl NumeralSystem {
    pub fn label(self) -> &'static str {
        match self {
            NumeralSystem::Doz => "DOZ",
            NumeralSystem::Dez => "DEZ",
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)] // Einige Felder werden erst von Tastatur (Phase C) / Overlay (Phase D) gelesen.
pub struct CalcState {
    pub input_buffer: RwSignal<Vec<CalcToken>>,
    pub result_buffer: RwSignal<Vec<CalcToken>>,
    pub cursor_pos: RwSignal<usize>,
    pub result_cursor_pos: RwSignal<usize>,
    pub result_field_active: RwSignal<bool>,
    pub result_period_start: RwSignal<Option<usize>>,
    pub result_period_len: RwSignal<usize>,
    pub result_period_capped: RwSignal<bool>,
    /// True wenn die Rational-Schiene kollabiert ist und das Ergebnis aus
    /// dem f64-Fallback stammt — steuert das `≈`-Suffix im Display.
    pub rat_collapsed: RwSignal<bool>,
    pub memory: RwSignal<Vec<CalcToken>>,
    pub memory_rational: RwSignal<Option<Rational>>,
    pub last_ans: RwSignal<Option<Rational>>,
    pub last_result_f64: RwSignal<f64>,
    pub error_msg: RwSignal<Option<String>>,
    pub overlay_open: RwSignal<bool>,
    pub angle_mode: RwSignal<AngleMode>,
    pub numeral_system: RwSignal<NumeralSystem>,
}

impl Default for CalcState {
    fn default() -> Self {
        Self {
            input_buffer: RwSignal::new(Vec::new()),
            result_buffer: RwSignal::new(vec![CalcToken::Digit(DozenalDigit::D0)]),
            cursor_pos: RwSignal::new(0),
            result_cursor_pos: RwSignal::new(0),
            result_field_active: RwSignal::new(false),
            result_period_start: RwSignal::new(None),
            result_period_len: RwSignal::new(0),
            result_period_capped: RwSignal::new(false),
            rat_collapsed: RwSignal::new(false),
            memory: RwSignal::new(Vec::new()),
            memory_rational: RwSignal::new(None),
            last_ans: RwSignal::new(None),
            last_result_f64: RwSignal::new(0.0),
            error_msg: RwSignal::new(None),
            overlay_open: RwSignal::new(false),
            angle_mode: RwSignal::new(AngleMode::Rad),
            numeral_system: RwSignal::new(NumeralSystem::Doz),
        }
    }
}

impl CalcState {
    /// True wenn das Display den `≈`-State-B-Suffix zeigen soll.
    pub fn is_f64_fallback(&self) -> bool {
        self.error_msg.with(Option::is_none) && self.rat_collapsed.get()
    }

    /// Strukturell 1:1 zu `crates/dozenal_calc_app/src/eval.rs::calculate_result`,
    /// nur über Signals statt direkter Feldzuweisung.
    pub fn calculate_result(&self) {
        let buf = self.input_buffer.get();
        if buf.is_empty() {
            self.set_zero_result();
            return;
        }
        let normalized = resolve_postfix(&buf);
        let expanded = with_implicit_muls(&normalized);
        let math_string = build_meval_string(&expanded);
        let rat_result = build_rat_expr(&expanded).and_then(|exprs| eval_rational(&exprs));
        let am = self.angle_mode.get();

        match eval_f64(&math_string, am) {
            Some(result) if result.is_finite() => {
                self.error_msg.set(None);
                self.last_ans.set(rat_result.clone());
                self.last_result_f64.set(result);
                self.rat_collapsed.set(rat_result.is_none());

                if let Some(r) = rat_result {
                    let (out, meta) = format_rational_result(&r);
                    self.result_buffer.set(out);
                    self.result_period_start.set(meta.start);
                    self.result_period_len.set(meta.len);
                    self.result_period_capped.set(meta.capped);
                } else {
                    self.result_buffer.set(format_f64_result(result));
                    self.result_period_start.set(None);
                    self.result_period_len.set(0);
                    self.result_period_capped.set(false);
                }
                self.result_cursor_pos.set(0);
                self.result_field_active.set(true);
            }
            Some(result) if result.is_nan() => {
                self.error_msg.set(Some("DOMAIN ERROR".to_string()));
            }
            Some(_) => {
                self.error_msg.set(Some("DIV BY ZERO".to_string()));
            }
            None => {
                self.error_msg.set(Some("SYNTAX ERROR".to_string()));
            }
        }
    }

    fn set_zero_result(&self) {
        self.error_msg.set(None);
        self.last_ans.set(Rational::from_ints(0, 1));
        self.last_result_f64.set(0.0);
        self.rat_collapsed.set(false);
        self.result_buffer
            .set(vec![CalcToken::Digit(DozenalDigit::D0)]);
        self.result_period_start.set(None);
        self.result_period_len.set(0);
        self.result_period_capped.set(false);
        self.result_cursor_pos.set(0);
        self.result_field_active.set(true);
    }
}
