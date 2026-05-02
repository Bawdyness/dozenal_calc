// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::info_content::{INFO_TITLES, draw_info_chapter};
use crate::logic::DozenalDigit;
use crate::painting::{paint_dozenal_digit, paint_token};
use crate::tokens::{CalcToken, DozenalCalcApp, InfoState};
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};

pub const MOBILE_BREAKPOINT_PX: f32 = 500.0;
const DISPLAY_LINE_H: f32 = 50.0; // height of each display line
const DISPLAY_GAP: f32 = 10.0; // gap between input and result line

/// Formats an f64 as a decimal string with up to 10 significant digits, trailing zeros removed.
pub fn format_decimal_result(val: f64) -> String {
    if !val.is_finite() {
        return if val.is_nan() {
            "NaN".to_string()
        } else {
            "∞".to_string()
        };
    }
    let s = format!("{val:.10}");
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

impl DozenalCalcApp {
    /// Renders the two-line display area (background + all content).
    pub fn draw_display(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        let total_h = DISPLAY_LINE_H * 2.0 + DISPLAY_GAP;
        let (display_rect, _) = ui.allocate_at_least(
            Vec2::new(ui.available_width(), total_h),
            egui::Sense::hover(),
        );
        ui.painter()
            .rect_filled(display_rect, 8.0, Color32::from_black_alpha(150));

        let input_rect = Rect::from_min_size(
            display_rect.min,
            Vec2::new(display_rect.width(), DISPLAY_LINE_H),
        );
        let result_rect = Rect::from_min_size(
            Pos2::new(
                display_rect.min.x,
                display_rect.min.y + DISPLAY_LINE_H + DISPLAY_GAP,
            ),
            Vec2::new(display_rect.width(), DISPLAY_LINE_H),
        );

        // Subtle separator between the two lines
        let sep_y = display_rect.min.y + DISPLAY_LINE_H + DISPLAY_GAP / 2.0;
        ui.painter().line_segment(
            [
                Pos2::new(display_rect.min.x + 8.0, sep_y),
                Pos2::new(display_rect.max.x - 8.0, sep_y),
            ],
            Stroke::new(0.5, Color32::from_gray(40)),
        );

        // Indicators — anchored to corners of full display_rect
        if !self.memory.is_empty() {
            ui.painter().text(
                display_rect.left_top() + Vec2::new(10.0, 10.0),
                Align2::LEFT_TOP,
                "M",
                FontId::monospace(14.0),
                Color32::GOLD,
            );
        }
        ui.painter().text(
            display_rect.right_top() + Vec2::new(-8.0, 8.0),
            Align2::RIGHT_TOP,
            self.angle_mode.label(),
            FontId::monospace(11.0),
            Color32::from_gray(180),
        );
        if self.display_dec {
            ui.painter().text(
                display_rect.right_top() + Vec2::new(-8.0, 22.0),
                Align2::RIGHT_TOP,
                "DEC",
                FontId::monospace(11.0),
                Color32::from_rgb(100, 200, 255),
            );
        }

        // --- UPPER LINE: input buffer (always visible) ---
        {
            let mut x_pos = input_rect.left() + 30.0;
            for i in 0..=self.input_buffer.len() {
                if i == self.cursor_pos && !self.result_field_active {
                    ui.painter().line_segment(
                        [
                            Pos2::new(x_pos - 15.0, input_rect.center().y - 15.0),
                            Pos2::new(x_pos - 15.0, input_rect.center().y + 15.0),
                        ],
                        Stroke::new(2.0, Color32::RED),
                    );
                }
                if i < self.input_buffer.len() {
                    let token = &self.input_buffer[i];
                    let rect = Rect::from_center_size(
                        Pos2::new(x_pos, input_rect.center().y),
                        Vec2::splat(30.0),
                    );
                    if let CalcToken::Digit(d) = token {
                        paint_dozenal_digit(ui, ui.painter(), rect, *d, Color32::WHITE, 2.0);
                        x_pos += 35.0;
                    } else {
                        let text = match token {
                            CalcToken::Add => "+",
                            CalcToken::Sub | CalcToken::Negate => "-",
                            CalcToken::Mul => "×",
                            CalcToken::Div => "÷",
                            CalcToken::ParenOpen => "(",
                            CalcToken::ParenClose => ")",
                            CalcToken::Sin => "sin",
                            CalcToken::Cos => "cos",
                            CalcToken::Tan => "tan",
                            CalcToken::Cot => "cot",
                            CalcToken::ExpTopRight => "^",
                            CalcToken::RootTopLeft => "√",
                            CalcToken::OplusBotLeft => "⊕",
                            CalcToken::LogBotRight => "log",
                            CalcToken::ArcSin => "sin⁻¹",
                            CalcToken::ArcCos => "cos⁻¹",
                            CalcToken::ArcTan => "tan⁻¹",
                            CalcToken::ArcCot => "cot⁻¹",
                            CalcToken::Expand => "…",
                            CalcToken::Decimal => ".",
                            CalcToken::RatLit(_) => "Ans",
                            CalcToken::ConstPi => "π",
                            CalcToken::ConstE => "e",
                            CalcToken::ConstPhi => "φ",
                            CalcToken::ConstSqrt2 => "√2",
                            CalcToken::Sinh => "sinh",
                            CalcToken::Cosh => "cosh",
                            CalcToken::Tanh => "tanh",
                            CalcToken::Coth => "coth",
                            CalcToken::ArSinh => "sinh⁻¹",
                            CalcToken::ArCosh => "cosh⁻¹",
                            CalcToken::ArTanh => "tanh⁻¹",
                            CalcToken::ArCoth => "coth⁻¹",
                            CalcToken::Factorial => "n!",
                            CalcToken::AbsVal => "|x|",
                            CalcToken::Reciprocal => "1/x",
                            CalcToken::Mod => "mod",
                            _ => "?",
                        };
                        ui.painter().text(
                            rect.center(),
                            Align2::CENTER_CENTER,
                            text,
                            FontId::monospace(24.0),
                            Color32::LIGHT_GREEN,
                        );
                        x_pos += match token {
                            CalcToken::ArcSin
                            | CalcToken::ArcCos
                            | CalcToken::ArcTan
                            | CalcToken::ArcCot
                            | CalcToken::ArSinh
                            | CalcToken::ArCosh
                            | CalcToken::ArTanh
                            | CalcToken::ArCoth => 65.0,
                            CalcToken::Sin
                            | CalcToken::Cos
                            | CalcToken::Tan
                            | CalcToken::Cot
                            | CalcToken::Sinh
                            | CalcToken::Cosh
                            | CalcToken::Tanh
                            | CalcToken::Coth
                            | CalcToken::LogBotRight
                            | CalcToken::RatLit(_)
                            | CalcToken::ConstSqrt2
                            | CalcToken::Reciprocal
                            | CalcToken::Mod => 45.0,
                            _ => 30.0,
                        };
                    }
                }
            }
        }

        // --- LOWER LINE: result buffer (always visible) ---
        let error_msg = self.error_msg.clone();
        if let Some(msg) = error_msg {
            ui.painter().text(
                result_rect.center(),
                Align2::CENTER_CENTER,
                &msg,
                FontId::monospace(30.0),
                Color32::LIGHT_RED,
            );
        } else if self.display_dec {
            let val = self
                .last_ans
                .map_or(self.last_result_f64, super::logic::Rational::to_f64);
            let s = format_decimal_result(val);
            ui.painter().text(
                result_rect.center(),
                Align2::CENTER_CENTER,
                s,
                FontId::monospace(26.0),
                Color32::WHITE,
            );
        } else {
            let n = self.result_buffer.len();
            let right_anchor = result_rect.right() - 40.0;
            let left_margin = result_rect.left() + 10.0;

            let widths: Vec<f32> = self
                .result_buffer
                .iter()
                .map(|t| {
                    if matches!(t, CalcToken::Decimal) {
                        25.0_f32
                    } else {
                        50.0_f32
                    }
                })
                .collect();

            let decimal_idx = self
                .result_buffer
                .iter()
                .position(|t| matches!(t, CalcToken::Decimal));

            // How many tokens to show, always starting from index 0 (integer part first).
            // Fractional digits are dropped from the right when the display is too narrow.
            let display_n = {
                let available = (right_anchor - left_margin).max(0.0);
                let base_end = decimal_idx.map_or(n, |d| d + 1);
                let base_width: f32 = widths[..base_end].iter().sum();
                if base_width >= available {
                    // Even integer is tight — show as many integer tokens as fit
                    let mut w = 0.0_f32;
                    let mut count = 0usize;
                    for &tw in widths.iter().take(n) {
                        let next = w + tw;
                        if next > available && count > 0 {
                            break;
                        }
                        w = next;
                        count += 1;
                    }
                    count.max(1)
                } else {
                    // Integer fits; pack in fractional tokens left-to-right
                    let frac_start = decimal_idx.map_or(n, |d| d + 1);
                    let mut remaining = available - base_width;
                    let mut count = base_end;
                    for &tw in widths.iter().take(n).skip(frac_start) {
                        if remaining < tw {
                            break;
                        }
                        remaining -= tw;
                        count += 1;
                    }
                    count.max(1)
                }
            };

            // Positions for display_n tokens, right-aligned
            let mut positions = vec![Pos2::ZERO; display_n];
            {
                let mut x = right_anchor;
                for i in (0..display_n).rev() {
                    positions[i] = Pos2::new(x, result_rect.center().y);
                    x -= widths[i];
                }
            }

            // Overline above period digits (visible portion only)
            let period_shown = if let Some(ps) = self
                .result_period_start
                .filter(|&ps| self.result_period_len > 0 && ps < display_n)
            {
                let vis_end = (ps + self.result_period_len).min(display_n);
                let last = vis_end - 1;
                let ox_start = positions[ps].x - 20.0;
                let ox_end = positions[last].x + 20.0;
                let oy = result_rect.center().y - 23.0;
                ui.painter().line_segment(
                    [Pos2::new(ox_start, oy), Pos2::new(ox_end, oy)],
                    Stroke::new(1.5, Color32::WHITE),
                );
                // State C: period capped in logic or extends beyond display width
                if self.result_period_capped || ps + self.result_period_len > display_n {
                    let dot_x = ox_end + 6.0;
                    for k in 0..3_u8 {
                        ui.painter().circle_filled(
                            Pos2::new(dot_x + f32::from(k) * 6.0, oy),
                            2.0,
                            Color32::WHITE,
                        );
                    }
                }
                true
            } else {
                false
            };

            // Result cursor
            if self.result_field_active && display_n > 0 {
                let rcp = self.result_cursor_pos.min(display_n - 1);
                let cx = positions[rcp].x - 20.0;
                ui.painter().line_segment(
                    [
                        Pos2::new(cx, result_rect.center().y - 15.0),
                        Pos2::new(cx, result_rect.center().y + 15.0),
                    ],
                    Stroke::new(2.0, Color32::RED),
                );
            }

            // Draw result tokens (only the visible display_n)
            for (idx, token) in self.result_buffer[..display_n].iter().enumerate() {
                let rect = Rect::from_center_size(positions[idx], Vec2::splat(40.0));
                match token {
                    CalcToken::Digit(d) => {
                        paint_dozenal_digit(ui, ui.painter(), rect, *d, Color32::WHITE, 2.5);
                    }
                    CalcToken::Sub | CalcToken::Negate => {
                        ui.painter().line_segment(
                            [rect.left_center(), rect.right_center()],
                            Stroke::new(2.5, Color32::WHITE),
                        );
                    }
                    CalcToken::Decimal => {
                        ui.painter().circle_filled(
                            rect.center_bottom() + Vec2::new(12.0, -5.0),
                            3.0,
                            Color32::WHITE,
                        );
                    }
                    _ => {}
                }
            }

            // State B: f64 fallback or display-truncated fractional result.
            // Three dots at the bottom of the digit line (not center, not overline height).
            let has_frac = decimal_idx.is_some();
            let is_state_b =
                has_frac && !period_shown && (self.last_ans.is_none() || display_n < n);
            if is_state_b {
                let dot_x = positions[display_n - 1].x + 30.0;
                let dot_y = result_rect.center().y + 15.0;
                for k in 0..3_u8 {
                    ui.painter().circle_filled(
                        Pos2::new(dot_x + f32::from(k) * 6.0, dot_y),
                        2.0,
                        Color32::WHITE,
                    );
                }
            }
        }
    }

    // --- DESKTOP LAYOUT ---
    pub fn draw_desktop_layout(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let btn_size = Vec2::splat(50.0);
            let gap = 15.0;

            ui.vertical(|ui| {
                let layout = [
                    [DozenalDigit::D10, DozenalDigit::D11, DozenalDigit::D0],
                    [DozenalDigit::D7, DozenalDigit::D8, DozenalDigit::D9],
                    [DozenalDigit::D4, DozenalDigit::D5, DozenalDigit::D6],
                    [DozenalDigit::D1, DozenalDigit::D2, DozenalDigit::D3],
                ];
                egui::Grid::new("num_block")
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        for row in &layout {
                            for &digit in row {
                                let (rect, resp) =
                                    ui.allocate_at_least(btn_size, egui::Sense::click());
                                let color = if resp.is_pointer_button_down_on() {
                                    Color32::GOLD
                                } else {
                                    Color32::WHITE
                                };
                                paint_dozenal_digit(ui, ui.painter(), rect, digit, color, 2.5);
                                if resp.clicked() {
                                    self.handle_click(CalcToken::Digit(digit));
                                }
                            }
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(gap);

            let mut render_col = |tokens: &[CalcToken]| {
                ui.vertical(|ui| {
                    for &token in tokens {
                        let (rect, resp) = ui.allocate_at_least(btn_size, egui::Sense::click());
                        let color = if resp.is_pointer_button_down_on() {
                            Color32::LIGHT_RED
                        } else {
                            Color32::LIGHT_BLUE
                        };
                        paint_token(ui, ui.painter(), rect, token, color, 2.0);
                        if self.is_armed(token) {
                            ui.painter().circle_filled(
                                rect.right_top() + Vec2::new(-5.0, 5.0),
                                3.0,
                                Color32::GOLD,
                            );
                        }
                        if resp.clicked() {
                            self.handle_click(token);
                        }
                        ui.add_space(6.0);
                    }
                });
                ui.add_space(15.0);
            };

            render_col(&[
                CalcToken::Add,
                CalcToken::Sub,
                CalcToken::Mul,
                CalcToken::Div,
            ]);
            render_col(&[
                CalcToken::OplusBotLeft,
                CalcToken::ExpTopRight,
                CalcToken::RootTopLeft,
                CalcToken::LogBotRight,
            ]);
            render_col(&[
                CalcToken::Sin,
                CalcToken::Cos,
                CalcToken::Tan,
                CalcToken::Cot,
            ]);
            render_col(&[
                CalcToken::ParenOpen,
                CalcToken::ParenClose,
                CalcToken::TriangleLeft,
                CalcToken::TriangleRight,
            ]);
            render_col(&[
                CalcToken::AC,
                CalcToken::Del,
                CalcToken::Decimal,
                CalcToken::Expand,
            ]);
        });
        ui.add_space(15.0);

        let (rect, resp) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 50.0), egui::Sense::click());
        let color = if resp.is_pointer_button_down_on() {
            Color32::LIGHT_RED
        } else {
            Color32::LIGHT_BLUE
        };
        paint_token(ui, ui.painter(), rect, CalcToken::Equals, color, 2.0);
        if resp.clicked() {
            self.handle_click(CalcToken::Equals);
        }
    }

    // --- HANDY LAYOUT (Voll Responsive mit Safe-Area) ---
    pub fn draw_mobile_layout(&mut self, ui: &mut egui::Ui) {
        let spacing = 8.0;
        let num_spacing_y = 10.0;

        let total_h = ui.available_height();
        let safe_bottom_padding = 60.0;
        let usable_h = total_h - safe_bottom_padding;
        let btn_height = ((usable_h - 280.0) / 10.0).clamp(25.0, 60.0);

        let num_btn_width = (ui.available_width() - (2.0 * spacing)) / 3.0;
        let num_btn_size = Vec2::new(num_btn_width, btn_height);

        let ops_btn_width = (ui.available_width() - (3.0 * spacing)) / 4.0;
        let ops_btn_size = Vec2::new(ops_btn_width, btn_height);

        let render_btn =
            |app: &mut Self, ui: &mut egui::Ui, token: CalcToken, color_normal: Color32| {
                let (rect, resp) = ui.allocate_at_least(ops_btn_size, egui::Sense::click());
                let color = if resp.is_pointer_button_down_on() {
                    Color32::LIGHT_RED
                } else {
                    color_normal
                };
                paint_token(ui, ui.painter(), rect, token, color, 2.0);
                if app.is_armed(token) {
                    ui.painter().circle_filled(
                        rect.right_top() + Vec2::new(-5.0, 5.0),
                        3.0,
                        Color32::GOLD,
                    );
                }
                if resp.clicked() {
                    app.handle_click(token);
                }
            };

        let render_digit = |app: &mut Self, ui: &mut egui::Ui, digit: DozenalDigit| {
            let (rect, resp) = ui.allocate_at_least(num_btn_size, egui::Sense::click());
            let color = if resp.is_pointer_button_down_on() {
                Color32::GOLD
            } else {
                Color32::WHITE
            };
            paint_dozenal_digit(ui, ui.painter(), rect, digit, color, 2.5);
            if resp.clicked() {
                app.handle_click(CalcToken::Digit(digit));
            }
        };

        let num_layout = [
            [DozenalDigit::D10, DozenalDigit::D11, DozenalDigit::D0],
            [DozenalDigit::D7, DozenalDigit::D8, DozenalDigit::D9],
            [DozenalDigit::D4, DozenalDigit::D5, DozenalDigit::D6],
            [DozenalDigit::D1, DozenalDigit::D2, DozenalDigit::D3],
        ];

        egui::Grid::new("mob_numpad")
            .spacing([spacing, num_spacing_y])
            .show(ui, |ui| {
                for row in &num_layout {
                    for &digit in row {
                        render_digit(self, ui, digit);
                    }
                    ui.end_row();
                }
            });

        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        egui::Grid::new("mob_ops_vertical")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                let c = Color32::LIGHT_BLUE;
                render_btn(self, ui, CalcToken::Add, c);
                render_btn(self, ui, CalcToken::OplusBotLeft, c);
                render_btn(self, ui, CalcToken::Sin, c);
                render_btn(self, ui, CalcToken::ParenOpen, c);
                ui.end_row();
                render_btn(self, ui, CalcToken::Sub, c);
                render_btn(self, ui, CalcToken::ExpTopRight, c);
                render_btn(self, ui, CalcToken::Cos, c);
                render_btn(self, ui, CalcToken::ParenClose, c);
                ui.end_row();
                render_btn(self, ui, CalcToken::Mul, c);
                render_btn(self, ui, CalcToken::RootTopLeft, c);
                render_btn(self, ui, CalcToken::Tan, c);
                render_btn(self, ui, CalcToken::TriangleLeft, c);
                ui.end_row();
                render_btn(self, ui, CalcToken::Div, c);
                render_btn(self, ui, CalcToken::LogBotRight, c);
                render_btn(self, ui, CalcToken::Cot, c);
                render_btn(self, ui, CalcToken::TriangleRight, c);
                ui.end_row();
            });

        ui.add_space(10.0);

        egui::Grid::new("mob_sys")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                let c = Color32::LIGHT_BLUE;
                render_btn(self, ui, CalcToken::AC, c);
                render_btn(self, ui, CalcToken::Del, c);
                render_btn(self, ui, CalcToken::Decimal, c);
                render_btn(self, ui, CalcToken::Expand, c);
                ui.end_row();
            });

        ui.add_space(10.0);

        let equals_size = Vec2::new(ui.available_width(), btn_height * 1.2);
        let (rect, resp) = ui.allocate_at_least(equals_size, egui::Sense::click());
        let color = if resp.is_pointer_button_down_on() {
            Color32::LIGHT_RED
        } else {
            Color32::LIGHT_GREEN
        };
        paint_token(ui, ui.painter(), rect, CalcToken::Equals, color, 2.0);
        if resp.clicked() {
            self.handle_click(CalcToken::Equals);
        }
    }

    pub fn draw_overlay(&mut self, ui: &mut egui::Ui, keypad_rect: Rect, is_mobile: bool) {
        ui.painter()
            .rect_filled(keypad_rect, 0.0, Color32::from_black_alpha(180));

        let sets: [[CalcToken; 4]; 5] = [
            [
                CalcToken::Sto,
                CalcToken::Rcl,
                CalcToken::Mc,
                CalcToken::Ans,
            ],
            [
                CalcToken::ConstPi,
                CalcToken::ConstE,
                CalcToken::ConstPhi,
                CalcToken::ConstSqrt2,
            ],
            [
                CalcToken::Sinh,
                CalcToken::Cosh,
                CalcToken::Tanh,
                CalcToken::Coth,
            ],
            [
                CalcToken::Factorial,
                CalcToken::AbsVal,
                CalcToken::Reciprocal,
                CalcToken::Mod,
            ],
            [
                CalcToken::DozDec,
                CalcToken::Drg,
                CalcToken::Info,
                CalcToken::Close,
            ],
        ];

        let spacing = 6.0;

        if is_mobile {
            let btn_w = (keypad_rect.width() - spacing * 3.0) / 4.0;
            let extra_gap = spacing * 2.0;
            let btn_h = (keypad_rect.height() - spacing * 4.0 - extra_gap) / 5.0;

            for (col_idx, set) in sets[..4].iter().enumerate() {
                for (row_idx, &token) in set.iter().enumerate() {
                    let x = keypad_rect.left() + col_idx as f32 * (btn_w + spacing);
                    let y = keypad_rect.top() + row_idx as f32 * (btn_h + spacing);
                    let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(btn_w, btn_h));
                    self.paint_overlay_btn(ui, rect, token);
                }
            }
            let y10 = keypad_rect.top() + 4.0 * (btn_h + spacing) + extra_gap;
            for (col_idx, &token) in sets[4].iter().enumerate() {
                let x = keypad_rect.left() + col_idx as f32 * (btn_w + spacing);
                let rect = Rect::from_min_size(Pos2::new(x, y10), Vec2::new(btn_w, btn_h));
                self.paint_overlay_btn(ui, rect, token);
            }
        } else {
            let btn_w = (keypad_rect.width() - spacing * 4.0) / 5.0;
            let btn_h = (keypad_rect.height() - spacing * 3.0) / 4.0;

            for (col_idx, set) in sets.iter().enumerate() {
                for (row_idx, &token) in set.iter().enumerate() {
                    let x = keypad_rect.left() + col_idx as f32 * (btn_w + spacing);
                    let y = keypad_rect.top() + row_idx as f32 * (btn_h + spacing);
                    let rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(btn_w, btn_h));
                    self.paint_overlay_btn(ui, rect, token);
                }
            }
        }
    }

    /// Renders a single overlay button (shared between desktop and mobile overlay).
    pub fn paint_overlay_btn(&mut self, ui: &mut egui::Ui, rect: Rect, token: CalcToken) {
        let resp = ui.allocate_rect(rect, egui::Sense::click());
        let color = if resp.is_pointer_button_down_on() {
            Color32::LIGHT_RED
        } else {
            Color32::LIGHT_BLUE
        };
        paint_token(ui, ui.painter(), rect, token, color, 2.0);
        if self.is_armed(token) {
            ui.painter()
                .circle_filled(rect.right_top() + Vec2::new(-5.0, 5.0), 3.0, Color32::GOLD);
        }
        if resp.clicked() {
            self.handle_click(token);
        }
    }

    /// Renders the keypad area (desktop or mobile) and the overlay if open.
    pub fn draw_keypad(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let is_mobile = ctx.screen_rect().width() < MOBILE_BREAKPOINT_PX;
        let keypad_top = ui.cursor().top();
        if is_mobile {
            self.draw_mobile_layout(ui);
        } else {
            self.draw_desktop_layout(ui);
        }
        if self.overlay_open {
            let keypad_rect = Rect::from_min_max(
                Pos2::new(ui.min_rect().left(), keypad_top),
                ui.min_rect().right_bottom(),
            );
            self.draw_overlay(ui, keypad_rect, is_mobile);
        }
    }

    /// Renders the floating info modal window (no-op when `info_state` is Closed).
    pub fn draw_info_modal(&mut self, ctx: &egui::Context) {
        if self.info_state == InfoState::Closed {
            return;
        }
        egui::Window::new("Info")
            .resizable(false)
            .collapsible(false)
            .default_width(340.0)
            .show(ctx, |ui| match self.info_state {
                InfoState::Closed => {}
                InfoState::List => {
                    ui.label(
                        egui::RichText::new("Dozenal — Zwölf Kapitel")
                            .strong()
                            .size(14.0),
                    );
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(2.0);
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for (i, title) in INFO_TITLES.iter().enumerate() {
                                let label = format!("{:>2}. {}", i + 1, title);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 26.0],
                                        egui::Button::new(label).frame(false),
                                    )
                                    .clicked()
                                {
                                    self.info_state = InfoState::Chapter(i);
                                }
                                ui.separator();
                            }
                        });
                    ui.add_space(4.0);
                    if ui.button("Schliessen").clicked() {
                        self.info_state = InfoState::Closed;
                    }
                }
                InfoState::Chapter(n) => {
                    ui.horizontal(|ui| {
                        if ui.button("← Zurück").clicked() {
                            self.info_state = InfoState::List;
                        }
                        ui.label(
                            egui::RichText::new(format!("{}. {}", n + 1, INFO_TITLES[n]))
                                .strong()
                                .size(12.0),
                        );
                    });
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(460.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            draw_info_chapter(ui, n);
                            ui.add_space(8.0);
                        });
                }
            });
    }
}
