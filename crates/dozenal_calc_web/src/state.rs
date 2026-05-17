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

    /// True wenn ein Tap auf `token` den vorherigen Buffer-Token zur Inversen
    /// umtoggeln würde — gemeinsame Quelle für den Armed-Marker (Anzeige) und
    /// die tatsächliche Umschaltung in `handle_click`.
    pub fn is_armed(&self, token: &CalcToken) -> bool {
        let cp = self.cursor_pos.get();
        if cp == 0 {
            return false;
        }
        self.input_buffer
            .with(|buf| inverse_swap(token, &buf[cp - 1]).is_some())
    }

    /// True wenn das Zahl-Literal unter dem Cursor schon einen Dezimalpunkt
    /// enthält. Bidirektionaler Walk, damit auch "mitten ins Literal navigiert"
    /// greift. Verhindert `1.2.3`-Eingaben auf Token-Ebene.
    fn has_decimal_in_current_literal(&self) -> bool {
        let cp = self.cursor_pos.get();
        self.input_buffer.with(|buf| {
            let mut i = cp;
            while i > 0 {
                i -= 1;
                match &buf[i] {
                    CalcToken::Decimal => return true,
                    CalcToken::Digit(_) => {}
                    _ => return false,
                }
            }
            let mut i = cp;
            while i < buf.len() {
                match &buf[i] {
                    CalcToken::Decimal => return true,
                    CalcToken::Digit(_) => i += 1,
                    _ => return false,
                }
            }
            false
        })
    }

    /// Strukturell 1:1 zu `crates/dozenal_calc_app/src/input.rs::handle_click`,
    /// nur über Signals statt direkter Feldzuweisung.
    pub fn handle_click(&self, token: CalcToken) {
        // Error-Guard: bei aktivem Fehler reagiert nur AC. Andere Tokens
        // räumen den Fehler weg und werden danach normal verarbeitet —
        // ausser Mode/Navigation, die bleiben blockiert.
        if self.error_msg.with(Option::is_some) && !matches!(token, CalcToken::AC) {
            if matches!(
                token,
                CalcToken::Drg
                    | CalcToken::DozDec
                    | CalcToken::Info
                    | CalcToken::TriangleLeft
                    | CalcToken::TriangleRight
                    | CalcToken::Expand
                    | CalcToken::Close
            ) {
                return;
            }
            self.error_msg.set(None);
            self.input_buffer.set(Vec::new());
            self.result_buffer
                .set(vec![CalcToken::Digit(DozenalDigit::D0)]);
            self.result_period_start.set(None);
            self.result_period_len.set(0);
            self.result_period_capped.set(false);
            self.cursor_pos.set(0);
            self.result_field_active.set(false);
        }

        let is_operator = matches!(
            token,
            CalcToken::Add
                | CalcToken::Sub
                | CalcToken::Mul
                | CalcToken::Div
                | CalcToken::ExpTopRight
                | CalcToken::RootTopLeft
                | CalcToken::OplusBotLeft
                | CalcToken::LogBotRight
        );

        // Nach `=`: erste Eingabe (außer Mode/Nav/Equals/AC) startet einen
        // neuen Ausdruck. Bei Operator-Start: Ans auto-injection.
        let starts_new_expr = self.result_field_active.get()
            && !matches!(
                token,
                CalcToken::TriangleLeft
                    | CalcToken::TriangleRight
                    | CalcToken::AC
                    | CalcToken::Equals
                    | CalcToken::Drg
                    | CalcToken::DozDec
                    | CalcToken::Info
                    | CalcToken::Expand
                    | CalcToken::Close
                    | CalcToken::Sto
                    | CalcToken::Mc
            );
        if starts_new_expr {
            let mut new_buf: Vec<CalcToken> = Vec::new();
            if is_operator {
                if let Some(r) = self.last_ans.get() {
                    new_buf.push(CalcToken::RatLit(r));
                } else {
                    new_buf.extend(self.result_buffer.get());
                }
            }
            let new_cursor = new_buf.len();
            self.input_buffer.set(new_buf);
            self.cursor_pos.set(new_cursor);
        }

        // Jede Aktion außer Cursor-Arrows lässt die Aktivität zum Input-Feld zurückkehren.
        if !matches!(token, CalcToken::TriangleLeft | CalcToken::TriangleRight) {
            self.result_field_active.set(false);
        }

        self.dispatch(token);
    }

    fn dispatch(&self, token: CalcToken) {
        match token {
            CalcToken::Digit(d) => self.insert_at_cursor(CalcToken::Digit(d)),
            CalcToken::Decimal => {
                if !self.has_decimal_in_current_literal() {
                    self.insert_at_cursor(CalcToken::Decimal);
                }
            }
            CalcToken::Equals => self.calculate_result(),
            CalcToken::AC => {
                self.input_buffer.set(Vec::new());
                self.result_buffer
                    .set(vec![CalcToken::Digit(DozenalDigit::D0)]);
                self.result_period_start.set(None);
                self.result_period_len.set(0);
                self.result_period_capped.set(false);
                self.cursor_pos.set(0);
                self.error_msg.set(None);
                self.rat_collapsed.set(false);
            }
            CalcToken::Del => {
                let cp = self.cursor_pos.get();
                if cp > 0 {
                    self.input_buffer.update(|buf| {
                        buf.remove(cp - 1);
                    });
                    self.cursor_pos.set(cp - 1);
                }
            }
            CalcToken::TriangleLeft => {
                if self.result_field_active.get() {
                    let pos = self.result_cursor_pos.get();
                    if pos > 0 {
                        self.result_cursor_pos.set(pos - 1);
                    }
                } else {
                    let cp = self.cursor_pos.get();
                    if cp > 0 {
                        self.cursor_pos.set(cp - 1);
                    }
                }
            }
            CalcToken::TriangleRight => {
                if self.result_field_active.get() {
                    let pos = self.result_cursor_pos.get();
                    let max = self.result_buffer.with(Vec::len);
                    if pos < max {
                        self.result_cursor_pos.set(pos + 1);
                    }
                } else {
                    let cp = self.cursor_pos.get();
                    let max = self.input_buffer.with(Vec::len);
                    if cp < max {
                        self.cursor_pos.set(cp + 1);
                    }
                }
            }
            CalcToken::Expand => self.overlay_open.update(|b| *b = !*b),
            CalcToken::Close => self.overlay_open.set(false),
            CalcToken::Drg => {
                self.angle_mode.update(|am| *am = am.next());
                self.overlay_open.set(false);
            }
            CalcToken::DozDec => {
                // Toggle des Zahlsystem-Indikators. Die tatsächliche
                // Basis-Konversion des Buffers folgt als Folge-Task —
                // hier wird nur das Anzeige-Label geschwenkt.
                self.numeral_system.update(|n| {
                    *n = match n {
                        NumeralSystem::Doz => NumeralSystem::Dez,
                        NumeralSystem::Dez => NumeralSystem::Doz,
                    };
                });
                self.overlay_open.set(false);
            }
            CalcToken::Info => {
                self.overlay_open.set(false);
                crate::router::navigate(&crate::router::Route::Info { anchor: None });
            }
            // Trig + Hyperbolic: Doppelklick toggelt zur Inversen.
            CalcToken::Sin
            | CalcToken::Cos
            | CalcToken::Tan
            | CalcToken::Cot
            | CalcToken::ArcSin
            | CalcToken::ArcCos
            | CalcToken::ArcTan
            | CalcToken::ArcCot
            | CalcToken::Sinh
            | CalcToken::Cosh
            | CalcToken::Tanh
            | CalcToken::Coth
            | CalcToken::ArSinh
            | CalcToken::ArCosh
            | CalcToken::ArTanh
            | CalcToken::ArCoth => {
                let cp = self.cursor_pos.get();
                let swap = if cp > 0 {
                    self.input_buffer
                        .with(|buf| inverse_swap(&token, &buf[cp - 1]))
                } else {
                    None
                };
                if let Some(new_token) = swap {
                    self.input_buffer.update(|buf| buf[cp - 1] = new_token);
                } else {
                    self.insert_at_cursor(token);
                }
                self.overlay_open.set(false);
            }
            // Memory ops
            CalcToken::Sto => {
                self.memory.set(self.result_buffer.get());
                self.memory_rational.set(self.last_ans.get());
                self.overlay_open.set(false);
            }
            CalcToken::Rcl => {
                if !self.memory.with(Vec::is_empty) {
                    if let Some(r) = self.memory_rational.get() {
                        self.insert_at_cursor(CalcToken::RatLit(r));
                    } else {
                        for m in self.memory.get() {
                            self.insert_at_cursor(m);
                        }
                    }
                }
                self.overlay_open.set(false);
            }
            CalcToken::Mc => {
                self.memory.set(Vec::new());
                self.memory_rational.set(None);
                self.overlay_open.set(false);
            }
            CalcToken::Ans => {
                if let Some(r) = self.last_ans.get() {
                    self.insert_at_cursor(CalcToken::RatLit(r));
                } else {
                    for m in self.result_buffer.get() {
                        self.insert_at_cursor(m);
                    }
                }
                self.overlay_open.set(false);
            }
            // Alle anderen Tokens (Add/Sub/Mul/Div/Op-Set/Parens/Konstanten/n!/|x|/1/x/mod):
            // einfach einfügen; Overlay schliessen, falls überlay-spezifisch.
            other => {
                let close_overlay = matches!(
                    other,
                    CalcToken::Factorial
                        | CalcToken::AbsVal
                        | CalcToken::Reciprocal
                        | CalcToken::Mod
                        | CalcToken::ConstPi
                        | CalcToken::ConstE
                        | CalcToken::ConstPhi
                        | CalcToken::ConstSqrt2
                );
                self.insert_at_cursor(other);
                if close_overlay {
                    self.overlay_open.set(false);
                }
            }
        }
    }

    fn insert_at_cursor(&self, token: CalcToken) {
        let cp = self.cursor_pos.get();
        self.input_buffer.update(|buf| {
            buf.insert(cp, token);
        });
        self.cursor_pos.set(cp + 1);
    }
}

/// Gibt den Inversen-Partner für eine Funktions-Token-Doppelklick-Toggle-Paarung
/// zurück, oder `None` wenn keine Inverse definiert ist. Identisch zur Variante
/// in `crates/dozenal_calc_app/src/input.rs::inverse_swap`.
fn inverse_swap(token: &CalcToken, prev: &CalcToken) -> Option<CalcToken> {
    use CalcToken::{
        ArCosh, ArCoth, ArSinh, ArTanh, ArcCos, ArcCot, ArcSin, ArcTan, Cos, Cosh, Cot, Coth, Sin,
        Sinh, Tan, Tanh,
    };
    Some(match (token, prev) {
        (Sin, Sin) => ArcSin,
        (Sin, ArcSin) => Sin,
        (Cos, Cos) => ArcCos,
        (Cos, ArcCos) => Cos,
        (Tan, Tan) => ArcTan,
        (Tan, ArcTan) => Tan,
        (Cot, Cot) => ArcCot,
        (Cot, ArcCot) => Cot,
        (Sinh, Sinh) => ArSinh,
        (Sinh, ArSinh) => Sinh,
        (Cosh, Cosh) => ArCosh,
        (Cosh, ArCosh) => Cosh,
        (Tanh, Tanh) => ArTanh,
        (Tanh, ArTanh) => Tanh,
        (Coth, Coth) => ArCoth,
        (Coth, ArCoth) => Coth,
        _ => return None,
    })
}
