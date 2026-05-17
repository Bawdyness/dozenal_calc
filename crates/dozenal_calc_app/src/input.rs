// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::state::{DozenalCalcApp, InfoState};
use dozenal_core::{CalcToken, DozenalDigit};
use eframe::egui;

impl DozenalCalcApp {
    // --- KLICK-LOGIK ---
    pub fn handle_click(&mut self, token: CalcToken) {
        if self.error_msg.is_some() && token != CalcToken::AC {
            // Mode and navigation keys stay blocked during errors.
            // Any input token clears the error and resets display to zero.
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
            self.error_msg = None;
            self.input_buffer.clear();
            self.result_buffer = vec![CalcToken::Digit(DozenalDigit::D0)];
            self.result_period_start = None;
            self.result_period_len = 0;
            self.result_period_capped = false;
            self.cursor_pos = 0;
            self.result_field_active = false;
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
        // When coming from result-field-active, starting a new expression clears the old one.
        // Mode keys, overlay controls, and Equals are transparent (don't start a new expression).
        let starts_new_expr = self.result_field_active
            && !matches!(
                token,
                CalcToken::TriangleLeft
                    | CalcToken::TriangleRight
                    | CalcToken::AC      // clears everything in its own match arm
                    | CalcToken::Equals  // re-evaluates current expression
                    | CalcToken::Drg
                    | CalcToken::DozDec
                    | CalcToken::Info
                    | CalcToken::Expand
                    | CalcToken::Close
                    | CalcToken::Sto
                    | CalcToken::Mc
            );
        if starts_new_expr {
            self.input_buffer.clear();
            self.cursor_pos = 0;
            if is_operator {
                // Ans auto-insertion for operator-first new expressions
                if let Some(r) = self.last_ans {
                    self.input_buffer.push(CalcToken::RatLit(r));
                } else {
                    for &t in &self.result_buffer.clone() {
                        self.input_buffer.push(t);
                    }
                }
                self.cursor_pos = self.input_buffer.len();
            }
        }

        // Any action other than arrows switches activity back to the input field.
        if !matches!(token, CalcToken::TriangleLeft | CalcToken::TriangleRight) {
            self.result_field_active = false;
        }
        match token {
            CalcToken::Digit(digit) => {
                self.input_buffer
                    .insert(self.cursor_pos, CalcToken::Digit(digit));
                self.cursor_pos += 1;
            }
            CalcToken::Decimal => {
                // Schutz gegen `1.2.3`: nur einfügen, wenn das Literal unter dem
                // Cursor noch keinen Dezimalpunkt enthält. Bidirektionaler Walk,
                // damit auch der mitten im Literal navigierte Fall greift.
                if !self.has_decimal_in_current_literal() {
                    self.input_buffer
                        .insert(self.cursor_pos, CalcToken::Decimal);
                    self.cursor_pos += 1;
                }
            }
            CalcToken::Equals => self.calculate_result(),
            CalcToken::AC => {
                self.input_buffer.clear();
                self.result_buffer = vec![CalcToken::Digit(DozenalDigit::D0)];
                self.result_period_start = None;
                self.result_period_len = 0;
                self.result_period_capped = false;
                self.cursor_pos = 0;
                self.error_msg = None;
            }
            CalcToken::Del => {
                if self.cursor_pos > 0 {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            }
            CalcToken::TriangleLeft => {
                if self.result_field_active {
                    if self.result_cursor_pos > 0 {
                        self.result_cursor_pos -= 1;
                    }
                } else if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            CalcToken::TriangleRight => {
                if self.result_field_active {
                    if self.result_cursor_pos < self.result_buffer.len() {
                        self.result_cursor_pos += 1;
                    }
                } else if self.cursor_pos < self.input_buffer.len() {
                    self.cursor_pos += 1;
                }
            }
            CalcToken::Expand => {
                self.overlay_open = true;
            }
            CalcToken::Close => {
                self.overlay_open = false;
            }
            // Set 6 — Memory
            CalcToken::Sto => {
                self.memory = self.result_buffer.clone();
                self.memory_rational = self.last_ans;
                self.overlay_open = false;
            }
            CalcToken::Rcl => {
                if !self.memory.is_empty() {
                    if let Some(r) = self.memory_rational {
                        // Exact rational — insert a single RatLit token
                        self.input_buffer
                            .insert(self.cursor_pos, CalcToken::RatLit(r));
                        self.cursor_pos += 1;
                    } else {
                        // f64 fallback — insert digit tokens from memory buffer
                        for &m in &self.memory.clone() {
                            self.input_buffer.insert(self.cursor_pos, m);
                            self.cursor_pos += 1;
                        }
                    }
                }
                self.overlay_open = false;
            }
            CalcToken::Mc => {
                self.memory.clear();
                self.memory_rational = None;
                self.overlay_open = false;
            }
            CalcToken::Ans => {
                if let Some(r) = self.last_ans {
                    // Exact rational — insert a single RatLit token
                    self.input_buffer
                        .insert(self.cursor_pos, CalcToken::RatLit(r));
                    self.cursor_pos += 1;
                } else {
                    // f64 fallback — insert digit tokens from result buffer
                    for &m in &self.result_buffer.clone() {
                        self.input_buffer.insert(self.cursor_pos, m);
                        self.cursor_pos += 1;
                    }
                }
                self.overlay_open = false;
            }
            // Set 7 — Constants: insert as symbolic token (collapses rational track correctly)
            CalcToken::ConstPi
            | CalcToken::ConstE
            | CalcToken::ConstPhi
            | CalcToken::ConstSqrt2 => {
                self.input_buffer.insert(self.cursor_pos, token);
                self.cursor_pos += 1;
                self.overlay_open = false;
            }
            // Set 9 — Extended: insert as function tokens where applicable
            CalcToken::Mod => {
                self.input_buffer.insert(self.cursor_pos, token);
                self.cursor_pos += 1;
                self.overlay_open = false;
            }
            _ => {
                // Mode keys — not inserted into buffer
                if token == CalcToken::Drg {
                    self.angle_mode = self.angle_mode.next();
                    self.overlay_open = false;
                    return;
                }
                if token == CalcToken::DozDec {
                    self.display_dec = !self.display_dec;
                    self.overlay_open = false;
                    return;
                }
                if token == CalcToken::Info {
                    self.info_state = InfoState::List;
                    self.overlay_open = false;
                    return;
                }
                let mut toggled = false;
                if self.cursor_pos > 0 {
                    let prev_idx = self.cursor_pos - 1;
                    let prev_token = self.input_buffer[prev_idx];
                    let swap = match (token, prev_token) {
                        (CalcToken::Sin, CalcToken::Sin) => Some(CalcToken::ArcSin),
                        (CalcToken::Sin, CalcToken::ArcSin) => Some(CalcToken::Sin),
                        (CalcToken::Cos, CalcToken::Cos) => Some(CalcToken::ArcCos),
                        (CalcToken::Cos, CalcToken::ArcCos) => Some(CalcToken::Cos),
                        (CalcToken::Tan, CalcToken::Tan) => Some(CalcToken::ArcTan),
                        (CalcToken::Tan, CalcToken::ArcTan) => Some(CalcToken::Tan),
                        (CalcToken::Cot, CalcToken::Cot) => Some(CalcToken::ArcCot),
                        (CalcToken::Cot, CalcToken::ArcCot) => Some(CalcToken::Cot),
                        (CalcToken::Sinh, CalcToken::Sinh) => Some(CalcToken::ArSinh),
                        (CalcToken::Sinh, CalcToken::ArSinh) => Some(CalcToken::Sinh),
                        (CalcToken::Cosh, CalcToken::Cosh) => Some(CalcToken::ArCosh),
                        (CalcToken::Cosh, CalcToken::ArCosh) => Some(CalcToken::Cosh),
                        (CalcToken::Tanh, CalcToken::Tanh) => Some(CalcToken::ArTanh),
                        (CalcToken::Tanh, CalcToken::ArTanh) => Some(CalcToken::Tanh),
                        (CalcToken::Coth, CalcToken::Coth) => Some(CalcToken::ArCoth),
                        (CalcToken::Coth, CalcToken::ArCoth) => Some(CalcToken::Coth),
                        _ => None,
                    };
                    if let Some(new_token) = swap {
                        self.input_buffer[prev_idx] = new_token;
                        toggled = true;
                    }
                }
                if !toggled {
                    self.input_buffer.insert(self.cursor_pos, token);
                    self.cursor_pos += 1;
                }
                // Overlay tokens close the overlay after insertion
                if matches!(
                    token,
                    CalcToken::Sinh
                        | CalcToken::Cosh
                        | CalcToken::Tanh
                        | CalcToken::Coth
                        | CalcToken::ArSinh
                        | CalcToken::ArCosh
                        | CalcToken::ArTanh
                        | CalcToken::ArCoth
                        | CalcToken::Factorial
                        | CalcToken::AbsVal
                        | CalcToken::Reciprocal
                ) {
                    self.overlay_open = false;
                }
            }
        }
    }

    /// Returns true when the token just before `cursor_pos` is the same function —
    /// meaning a second click would toggle to its inverse. Used for the armed marker.
    pub fn is_armed(&self, token: CalcToken) -> bool {
        let invertible = matches!(
            token,
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
                | CalcToken::ArCoth
        );
        if !invertible || self.cursor_pos == 0 {
            return false;
        }
        self.input_buffer[self.cursor_pos - 1] == token
    }

    /// True wenn das Zahl-Literal unter dem Cursor schon einen Dezimalpunkt enthält.
    /// Bidirektionaler Walk durch zusammenhängende Digit/Decimal-Tokens, damit auch
    /// der "mitten ins Literal navigiert"-Fall greift (z. B. Cursor zwischen `1` und `.2`).
    fn has_decimal_in_current_literal(&self) -> bool {
        let mut i = self.cursor_pos;
        while i > 0 {
            i -= 1;
            let t = self.input_buffer[i];
            if matches!(t, CalcToken::Decimal) {
                return true;
            }
            if !matches!(t, CalcToken::Digit(_)) {
                return false;
            }
        }
        let mut i = self.cursor_pos;
        while i < self.input_buffer.len() {
            let t = self.input_buffer[i];
            if matches!(t, CalcToken::Decimal) {
                return true;
            }
            if !matches!(t, CalcToken::Digit(_)) {
                return false;
            }
            i += 1;
        }
        false
    }

    pub fn handle_keyboard(&mut self, ctx: &egui::Context) {
        // Collect tokens to dispatch outside the closure (borrow-checker)
        let mut tokens: Vec<CalcToken> = Vec::new();
        ctx.input(|i| {
            for event in &i.events {
                match event {
                    egui::Event::Text(s) if !i.modifiers.ctrl && !i.modifiers.mac_cmd => {
                        for ch in s.chars() {
                            let t = match ch {
                                '0' => Some(CalcToken::Digit(DozenalDigit::D0)),
                                '1' => Some(CalcToken::Digit(DozenalDigit::D1)),
                                '2' => Some(CalcToken::Digit(DozenalDigit::D2)),
                                '3' => Some(CalcToken::Digit(DozenalDigit::D3)),
                                '4' => Some(CalcToken::Digit(DozenalDigit::D4)),
                                '5' => Some(CalcToken::Digit(DozenalDigit::D5)),
                                '6' => Some(CalcToken::Digit(DozenalDigit::D6)),
                                '7' => Some(CalcToken::Digit(DozenalDigit::D7)),
                                '8' => Some(CalcToken::Digit(DozenalDigit::D8)),
                                '9' => Some(CalcToken::Digit(DozenalDigit::D9)),
                                'a' | 'A' => Some(CalcToken::Digit(DozenalDigit::D10)),
                                'b' | 'B' => Some(CalcToken::Digit(DozenalDigit::D11)),
                                '+' => Some(CalcToken::Add),
                                '-' => Some(CalcToken::Sub),
                                '*' => Some(CalcToken::Mul),
                                '/' => Some(CalcToken::Div),
                                '^' => Some(CalcToken::ExpTopRight),
                                '.' => Some(CalcToken::Decimal),
                                '=' => Some(CalcToken::Equals),
                                _ => None,
                            };
                            if let Some(t) = t {
                                tokens.push(t);
                            }
                        }
                    }
                    egui::Event::Key {
                        key, pressed: true, ..
                    } => {
                        let t = match key {
                            egui::Key::Enter => Some(CalcToken::Equals),
                            egui::Key::Backspace => Some(CalcToken::Del),
                            egui::Key::Escape => Some(CalcToken::AC),
                            egui::Key::ArrowLeft => Some(CalcToken::TriangleLeft),
                            egui::Key::ArrowRight => Some(CalcToken::TriangleRight),
                            _ => None,
                        };
                        if let Some(t) = t {
                            tokens.push(t);
                        }
                    }
                    _ => {}
                }
            }
        });
        for t in tokens {
            self.handle_click(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dozenal_core::DozenalDigit::{D1, D2, D3};

    #[test]
    fn decimal_not_inserted_twice_in_same_literal() {
        let mut app = DozenalCalcApp::default();
        app.handle_click(CalcToken::Digit(D1));
        app.handle_click(CalcToken::Decimal);
        app.handle_click(CalcToken::Digit(D2));
        app.handle_click(CalcToken::Decimal); // soll ignoriert werden
        app.handle_click(CalcToken::Digit(D3));
        assert_eq!(
            app.input_buffer,
            vec![
                CalcToken::Digit(D1),
                CalcToken::Decimal,
                CalcToken::Digit(D2),
                CalcToken::Digit(D3),
            ]
        );
    }

    #[test]
    fn decimal_allowed_after_operator_starts_new_literal() {
        let mut app = DozenalCalcApp::default();
        app.handle_click(CalcToken::Digit(D1));
        app.handle_click(CalcToken::Decimal);
        app.handle_click(CalcToken::Digit(D2));
        app.handle_click(CalcToken::Add);
        app.handle_click(CalcToken::Decimal); // neues Literal, OK
        app.handle_click(CalcToken::Digit(D3));
        let decimals = app
            .input_buffer
            .iter()
            .filter(|t| matches!(t, CalcToken::Decimal))
            .count();
        assert_eq!(decimals, 2);
    }

    #[test]
    fn decimal_blocked_when_navigated_into_literal() {
        // Bidirektionaler Walk: 1.2 mit Cursor zwischen 1 und . darf kein
        // zweites . einfügen.
        let mut app = DozenalCalcApp::default();
        app.handle_click(CalcToken::Digit(D1));
        app.handle_click(CalcToken::Decimal);
        app.handle_click(CalcToken::Digit(D2));
        // Cursor zurück auf Position 1 (zwischen D1 und Decimal)
        app.handle_click(CalcToken::TriangleLeft);
        app.handle_click(CalcToken::TriangleLeft);
        app.handle_click(CalcToken::Decimal); // soll ignoriert werden
        assert_eq!(
            app.input_buffer,
            vec![
                CalcToken::Digit(D1),
                CalcToken::Decimal,
                CalcToken::Digit(D2)
            ]
        );
    }
}
