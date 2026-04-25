mod logic;

use eframe::egui;
use egui::{Color32, Stroke, Vec2, Rect, Pos2, Align2, FontId};
use logic::{DozenalDigit, DozenalConverter};

#[derive(Clone, Copy, PartialEq)]
enum CalcToken {
    Digit(DozenalDigit),
    Add, Sub, Mul, Div,
    ExpTopRight, RootTopLeft, OplusBotLeft, LogBotRight,
    Sin, Cos, Tan, Cot,
    ArcSin, ArcCos, ArcTan, ArcCot, 
    ParenOpen, ParenClose, TriangleRight, TriangleLeft,
    AC, Del, MPlus, Decimal, Equals,
}

// 1. Die Tür für den Desktop (Native)
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dozenal Calc",
        native_options,
        Box::new(|_cc| Box::new(DozenalCalcApp::default())),
    )
}

// 2. Die Tür für den Browser (WebAssembly)
#[cfg(target_arch = "wasm32")]
fn main() {
    // Leitet Fehler in die Entwickler-Konsole des Browsers (F12) um
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // Diese ID muss in deiner index.html stehen
                web_options,
                Box::new(|_cc| Box::new(DozenalCalcApp::default())),
            )
            .await
            .expect("failed to start eframe");
    });
}

struct DozenalCalcApp {
    input_buffer: Vec<CalcToken>,
    result_buffer: Vec<CalcToken>, 
    cursor_pos: usize, 
    memory: Vec<CalcToken>, // Der Notizblock des Rechners
}

impl Default for DozenalCalcApp {
    fn default() -> Self {
        Self { 
            input_buffer: Vec::new(),
            result_buffer: vec![CalcToken::Digit(DozenalDigit::D0)],
            cursor_pos: 0,
            memory: Vec::new(),
        }
    }
}

// --- DER ÜBERSETZER UND DIE LAYOUTS ---
impl DozenalCalcApp {
    
    // --- KLICK-LOGIK ---
    fn handle_click(&mut self, token: CalcToken) {
        match token {
            CalcToken::Digit(digit) => {
                self.input_buffer.insert(self.cursor_pos, CalcToken::Digit(digit)); 
                self.cursor_pos += 1;
            },
            CalcToken::Equals => self.calculate_result(),
            CalcToken::AC => { 
                self.input_buffer.clear();
                self.result_buffer = vec![CalcToken::Digit(DozenalDigit::D0)];
                self.cursor_pos = 0;
            },
            CalcToken::Del => {
                if self.cursor_pos > 0 {
                    self.input_buffer.remove(self.cursor_pos - 1);
                    self.cursor_pos -= 1;
                }
            },
            CalcToken::TriangleLeft => { if self.cursor_pos > 0 { self.cursor_pos -= 1; } },
            CalcToken::TriangleRight => { if self.cursor_pos < self.input_buffer.len() { self.cursor_pos += 1; } },
            CalcToken::MPlus => {
                if self.input_buffer.is_empty() {
                    self.memory = self.result_buffer.clone();
                } else {
                    for m_token in &self.memory {
                        self.input_buffer.insert(self.cursor_pos, *m_token);
                        self.cursor_pos += 1;
                    }
                }
            },
            _ => {
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
            },
        }
    }

    // --- RECHEN-LOGIK ---
    fn calculate_result(&mut self) {
        let mut int_digits = Vec::new();
        let mut frac_digits = Vec::new();
        let mut in_fraction = false;
        let mut tokens_str: Vec<String> = Vec::new();

        for token in &self.input_buffer {
            match token {
                CalcToken::Digit(d) => {
                    if in_fraction { frac_digits.push(*d); } else { int_digits.push(*d); }
                },
                CalcToken::Decimal => {
                    in_fraction = true;
                },
                _ => {
                    if !int_digits.is_empty() || !frac_digits.is_empty() {
                        let int_val = if int_digits.is_empty() { "0".to_string() } else { DozenalConverter::to_decimal(&int_digits).to_string() };
                        if in_fraction && !frac_digits.is_empty() {
                            let frac_val = DozenalConverter::to_decimal(&frac_digits).to_string();
                            let len = frac_digits.len();
                            tokens_str.push(format!("({}+({}/(12^{})))", int_val, frac_val, len));
                        } else {
                            tokens_str.push(int_val);
                        }
                        int_digits.clear();
                        frac_digits.clear();
                        in_fraction = false;
                    }
                    
                    let s = match token {
                        CalcToken::Add => "+", CalcToken::Sub => "-", 
                        CalcToken::Mul => "*", CalcToken::Div => "/",
                        CalcToken::ParenOpen => "(", CalcToken::ParenClose => ")",
                        CalcToken::Sin => "sin(", CalcToken::Cos => "cos(",
                        CalcToken::Tan => "tan(", CalcToken::Cot => "1/tan(",
                        CalcToken::ExpTopRight => "^", CalcToken::RootTopLeft => "√",
                        CalcToken::OplusBotLeft => "⊕", 
                        CalcToken::LogBotRight => "log",
                        CalcToken::ArcSin => "asin(", 
                        CalcToken::ArcCos => "acos(",
                        CalcToken::ArcTan => "atan(", 
                        CalcToken::ArcCot => "(pi/2 - atan(", 
                        _ => ""
                    };
                    if !s.is_empty() { tokens_str.push(s.to_string()); }
                }
            }
        }
        
        if !int_digits.is_empty() || !frac_digits.is_empty() {
            let int_val = if int_digits.is_empty() { "0".to_string() } else { DozenalConverter::to_decimal(&int_digits).to_string() };
            if in_fraction && !frac_digits.is_empty() {
                let frac_val = DozenalConverter::to_decimal(&frac_digits).to_string();
                let len = frac_digits.len();
                tokens_str.push(format!("({}+({}/(12^{})))", int_val, frac_val, len));
            } else {
                tokens_str.push(int_val);
            }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "⊕") {
            if i > 0 && i + 1 < tokens_str.len() {
                let a = &tokens_str[i - 1];
                let b = &tokens_str[i + 1];
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("(({}*{})/({}+{}))", a, b, a, b)]);
            } else { break; }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "√") {
            if i > 0 && i + 1 < tokens_str.len() {
                let n = &tokens_str[i - 1];
                let x = &tokens_str[i + 1];
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("({}^(1/{}))", x, n)]);
            } else if i == 0 && i + 1 < tokens_str.len() {
                let x = &tokens_str[i + 1];
                tokens_str.splice(i..=(i + 1), vec![format!("({}^(1/2))", x)]);
            } else { break; }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "log") {
            if i > 0 && i + 1 < tokens_str.len() {
                let x = &tokens_str[i - 1];
                let n = &tokens_str[i + 1];
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("(ln({})/ln({}))", x, n)]);
            } else { break; }
        }

        let mut math_string = tokens_str.join(" ");
        let open_parens = math_string.matches('(').count();
        let close_parens = math_string.matches(')').count();
        for _ in 0..(open_parens.saturating_sub(close_parens)) { math_string.push(')'); }

        if let Ok(result) = meval::eval_str(&math_string) {
            let mut new_result = Vec::new();
            let mut val = result;
            if val < 0.0 { new_result.push(CalcToken::Sub); val = val.abs(); }

            let mut integer_part = val.floor() as u64;
            let mut int_digits_res = Vec::new();
            if integer_part == 0 { int_digits_res.push(CalcToken::Digit(DozenalDigit::D0)); }
            else {
                while integer_part > 0 {
                    if let Some(d) = DozenalDigit::from_value((integer_part % 12) as u32) {
                        int_digits_res.push(CalcToken::Digit(d));
                    }
                    integer_part /= 12;
                }
                int_digits_res.reverse();
            }
            new_result.extend(int_digits_res);

            let mut frac_part = val - val.floor();
            if frac_part > 0.000001 {
                new_result.push(CalcToken::Decimal);
                for _ in 0..4 {
                    frac_part *= 12.0;
                    let d_val = (frac_part + 0.000001).floor() as u32;
                    if let Some(d) = DozenalDigit::from_value(d_val) {
                        new_result.push(CalcToken::Digit(d));
                    }
                    frac_part -= d_val as f64;
                    if frac_part.abs() < 0.000001 { break; }
                }
            }
            self.result_buffer = new_result;
            self.input_buffer.clear();
            self.cursor_pos = 0; 
        }
    }

    // --- DESKTOP LAYOUT ---
    fn draw_desktop_layout(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let btn_size = Vec2::splat(50.0);
            let gap = 15.0;

            ui.vertical(|ui| {
                let layout = [
                    [DozenalDigit::D10, DozenalDigit::D11, DozenalDigit::D0],
                    [DozenalDigit::D7,  DozenalDigit::D8,  DozenalDigit::D9],
                    [DozenalDigit::D4,  DozenalDigit::D5,  DozenalDigit::D6],
                    [DozenalDigit::D1,  DozenalDigit::D2,  DozenalDigit::D3],
                ];
                egui::Grid::new("num_block").spacing([10.0, 10.0]).show(ui, |ui| {
                    for row in &layout {
                        for &digit in row {
                            let (rect, resp) = ui.allocate_at_least(btn_size, egui::Sense::click());
                            let color = if resp.is_pointer_button_down_on() { Color32::GOLD } else { Color32::WHITE };
                            paint_dozenal_digit(ui, ui.painter(), rect, digit, color, 2.5);
                            if resp.clicked() { self.handle_click(CalcToken::Digit(digit)); }
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
                        let color = if resp.is_pointer_button_down_on() { Color32::LIGHT_RED } else { Color32::LIGHT_BLUE };
                        paint_token(ui, ui.painter(), rect, token, color, 2.0);
                        if resp.clicked() { self.handle_click(token); }
                        ui.add_space(6.0); 
                    }
                });
                ui.add_space(15.0);
            };

            render_col(&[CalcToken::Add, CalcToken::Sub, CalcToken::Mul, CalcToken::Div]);
            render_col(&[CalcToken::OplusBotLeft, CalcToken::ExpTopRight, CalcToken::RootTopLeft, CalcToken::LogBotRight]);
            render_col(&[CalcToken::Sin, CalcToken::Cos, CalcToken::Tan, CalcToken::Cot]);
            render_col(&[CalcToken::ParenOpen, CalcToken::ParenClose, CalcToken::TriangleLeft, CalcToken::TriangleRight]);
            render_col(&[CalcToken::AC, CalcToken::Del, CalcToken::Decimal, CalcToken::MPlus]);
        });
        ui.add_space(15.0);

        let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 50.0), egui::Sense::click());
        let color = if resp.is_pointer_button_down_on() { Color32::LIGHT_RED } else { Color32::LIGHT_BLUE };
        paint_token(ui, ui.painter(), rect, CalcToken::Equals, color, 2.0);
        if resp.clicked() { self.handle_click(CalcToken::Equals); }
    }

// --- HANDY LAYOUT (Symmetrisch angepasst) ---
    fn draw_mobile_layout(&mut self, ui: &mut egui::Ui) {
        let spacing = 8.0;
        
        // Berechnung für den gedrittelten Zahlenblock (3 Spalten)
        let num_btn_width = (ui.available_width() - (2.0 * spacing)) / 3.0;
        let num_btn_size = Vec2::new(num_btn_width, 45.0);

        // Berechnung für den geviertelten Operationsblock (4 Spalten)
        let ops_btn_width = (ui.available_width() - (3.0 * spacing)) / 4.0;
        let ops_btn_size = Vec2::new(ops_btn_width, 45.0);

        // 1. Hilfsarbeiter für die Operationstasten (4 Spalten)
        let render_btn = |app: &mut Self, ui: &mut egui::Ui, token: CalcToken, color_normal: Color32| {
            let (rect, resp) = ui.allocate_at_least(ops_btn_size, egui::Sense::click());
            let color = if resp.is_pointer_button_down_on() { Color32::LIGHT_RED } else { color_normal };
            paint_token(ui, ui.painter(), rect, token, color, 2.0);
            if resp.clicked() { app.handle_click(token); }
        };

        // 2. Hilfsarbeiter für die Dozenal-Ziffern (3 Spalten)
        let render_digit = |app: &mut Self, ui: &mut egui::Ui, digit: DozenalDigit| {
            let (rect, resp) = ui.allocate_at_least(num_btn_size, egui::Sense::click());
            let color = if resp.is_pointer_button_down_on() { Color32::GOLD } else { Color32::WHITE };
            paint_dozenal_digit(ui, ui.painter(), rect, digit, color, 2.5);
            if resp.clicked() { app.handle_click(CalcToken::Digit(digit)); }
        };

        // --- ZAHLENBLOCK OBEN (Gedrittelt) ---
        let num_layout = [
            [DozenalDigit::D10, DozenalDigit::D11, DozenalDigit::D0],
            [DozenalDigit::D7,  DozenalDigit::D8,  DozenalDigit::D9],
            [DozenalDigit::D4,  DozenalDigit::D5,  DozenalDigit::D6],
            [DozenalDigit::D1,  DozenalDigit::D2,  DozenalDigit::D3],
        ];
        
        egui::Grid::new("mob_numpad").spacing([spacing, 20.0]).show(ui, |ui| {
            for row in &num_layout {
                for &digit in row {
                    render_digit(self, ui, digit);
                }
                ui.end_row();
            }
        });

        ui.add_space(15.0);
        ui.separator();
        ui.add_space(15.0);

        // --- OPERATIONEN (Geviertelt) ---
        egui::Grid::new("mob_ops_vertical").spacing([spacing, spacing]).show(ui, |ui| {
            let c = Color32::LIGHT_BLUE;
            
            // Zeile 1
            render_btn(self, ui, CalcToken::Add, c);           render_btn(self, ui, CalcToken::OplusBotLeft, c);  
            render_btn(self, ui, CalcToken::Sin, c);           render_btn(self, ui, CalcToken::ParenOpen, c);     
            ui.end_row();

            // Zeile 2
            render_btn(self, ui, CalcToken::Sub, c);           render_btn(self, ui, CalcToken::ExpTopRight, c);   
            render_btn(self, ui, CalcToken::Cos, c);           render_btn(self, ui, CalcToken::ParenClose, c);    
            ui.end_row();

            // Zeile 3
            render_btn(self, ui, CalcToken::Mul, c);           render_btn(self, ui, CalcToken::RootTopLeft, c);   
            render_btn(self, ui, CalcToken::Tan, c);           render_btn(self, ui, CalcToken::TriangleLeft, c);  
            ui.end_row();

            // Zeile 4
            render_btn(self, ui, CalcToken::Div, c);           render_btn(self, ui, CalcToken::LogBotRight, c);   
            render_btn(self, ui, CalcToken::Cot, c);           render_btn(self, ui, CalcToken::TriangleRight, c); 
            ui.end_row();
        });

        ui.add_space(20.0); 

        // --- SYSTEMTASTEN (Geviertelt) ---
        egui::Grid::new("mob_sys").spacing([spacing, spacing]).show(ui, |ui| {
            let c = Color32::LIGHT_BLUE;
            render_btn(self, ui, CalcToken::AC, c); 
            render_btn(self, ui, CalcToken::Del, c); 
            render_btn(self, ui, CalcToken::Decimal, c); 
            render_btn(self, ui, CalcToken::MPlus, c); 
            ui.end_row();
        });

        ui.add_space(10.0);

        // --- BREITE GLEICHHEITSTASTE ---
        let equals_size = Vec2::new(ui.available_width(), 60.0);
        let (rect, resp) = ui.allocate_at_least(equals_size, egui::Sense::click());
        let color = if resp.is_pointer_button_down_on() { Color32::LIGHT_RED } else { Color32::LIGHT_GREEN };
        paint_token(ui, ui.painter(), rect, CalcToken::Equals, color, 2.0);
        if resp.clicked() { self.handle_click(CalcToken::Equals); }
    }
}

// --- HAUPT-UPDATE SCHLEIFE (Das Fenster) ---
impl eframe::App for DozenalCalcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Dozenal Calc");
                ui.add_space(10.0);

                // --- DISPLAY ---
                let (display_rect, _) = ui.allocate_at_least(Vec2::new(ui.available_width(), 80.0), egui::Sense::hover());
                ui.painter().rect_filled(display_rect, 8.0, Color32::from_black_alpha(150));
                
                if !self.memory.is_empty() {
                    ui.painter().text(display_rect.left_top() + Vec2::new(10.0, 10.0), Align2::LEFT_TOP, "M", FontId::monospace(14.0), Color32::GOLD);
                }
                
                if self.input_buffer.is_empty() {
                    let mut x_pos = display_rect.right() - 40.0;
                    for token in self.result_buffer.iter().rev() {
                        let rect = Rect::from_center_size(Pos2::new(x_pos, display_rect.center().y), Vec2::splat(40.0));
                        match token {
                            CalcToken::Digit(d) => paint_dozenal_digit(ui, ui.painter(), rect, *d, Color32::WHITE, 2.5),
                            CalcToken::Sub => { ui.painter().line_segment([rect.left_center(), rect.right_center()], Stroke::new(2.5, Color32::WHITE)); },
                            CalcToken::Decimal => { ui.painter().circle_filled(rect.center_bottom() + Vec2::new(12.0, -5.0), 3.0, Color32::WHITE); },
                            _ => {}
                        }
                        x_pos -= match token { CalcToken::Decimal => 25.0, _ => 50.0 };
                    }
                } else {
                    let mut x_pos = display_rect.left() + 30.0;
                    for i in 0..=self.input_buffer.len() {
                        if i == self.cursor_pos {
                            ui.painter().line_segment(
                                [Pos2::new(x_pos - 15.0, display_rect.center().y - 15.0), Pos2::new(x_pos - 15.0, display_rect.center().y + 15.0)], 
                                Stroke::new(2.0, Color32::RED)
                            );
                        }

                        if i < self.input_buffer.len() {
                            let token = &self.input_buffer[i];
                            let rect = Rect::from_center_size(Pos2::new(x_pos, display_rect.center().y), Vec2::splat(30.0));
                            match token {
                                CalcToken::Digit(d) => {
                                    paint_dozenal_digit(ui, ui.painter(), rect, *d, Color32::WHITE, 2.0);
                                    x_pos += 35.0;
                                },
                                _ => {
                                    let text = match token {
                                        CalcToken::Add => "+", CalcToken::Sub => "-", CalcToken::Mul => "×", CalcToken::Div => "÷",
                                        CalcToken::ParenOpen => "(", CalcToken::ParenClose => ")",
                                        CalcToken::Sin => "sin", CalcToken::Cos => "cos", CalcToken::Tan => "tan", CalcToken::Cot => "cot",
                                        CalcToken::ExpTopRight => "^", CalcToken::RootTopLeft => "√", CalcToken::OplusBotLeft => "⊕",
                                        CalcToken::LogBotRight => "log", CalcToken::ArcSin => "asin(", CalcToken::ArcCos => "acos(",
                                        CalcToken::ArcTan => "atan(", CalcToken::ArcCot => "(pi/2 - atan(", 
                                        CalcToken::MPlus => "M+", CalcToken::Decimal => ".", _ => "Op",
                                    };
                                    ui.painter().text(rect.center(), Align2::CENTER_CENTER, text, FontId::monospace(24.0), Color32::LIGHT_GREEN);
                                    x_pos += match token {
                                        CalcToken::Sin | CalcToken::Cos | CalcToken::Tan | CalcToken::Cot | CalcToken::LogBotRight | CalcToken::ArcSin | CalcToken::ArcCos | CalcToken::ArcTan | CalcToken::ArcCot => 45.0,
                                        _ => 30.0,
                                    };
                                }
                            }
                        }
                    }
                }
                ui.add_space(20.0);

                // --- DER NEUE RESPONSIVE SCHALTER ---
                let is_mobile = ctx.screen_rect().width() < 500.0;
                if is_mobile {
                    self.draw_mobile_layout(ui);
                } else {
                    self.draw_desktop_layout(ui);
                }
            }); 
        }); 
    } 
}

// --- ZEICHEN-ROUTINEN ---
fn paint_token(_ui: &egui::Ui, p: &egui::Painter, rect: Rect, token: CalcToken, color: Color32, width: f32) {
    let s = Stroke::new(width, color);
    let c = rect.center();
    
    // DER FIX: Nimm die kürzere Seite des Kastens, damit nichts überlappt!
    let min_edge = rect.width().min(rect.height());
    let q = min_edge / 4.0;
    
    p.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_gray(80)));

    match token {
        CalcToken::Add => { p.line_segment([c - Vec2::new(q, 0.0), c + Vec2::new(q, 0.0)], s); p.line_segment([c - Vec2::new(0.0, q), c + Vec2::new(0.0, q)], s); }
        CalcToken::Sub => { p.line_segment([c - Vec2::new(q, 0.0), c + Vec2::new(q, 0.0)], s); }
        CalcToken::Mul => { p.line_segment([c - Vec2::new(q, q), c + Vec2::new(q, q)], s); p.line_segment([c - Vec2::new(q, -q), c + Vec2::new(q, -q)], s); }
        CalcToken::Div => { p.line_segment([c - Vec2::new(q, -q), c + Vec2::new(q, -q)], s); }
        CalcToken::ExpTopRight => { 
            p.text(c + Vec2::new(-q*0.3, q*0.3), Align2::CENTER_CENTER, "x", FontId::monospace(26.0), color);
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(q*0.9, -q*0.9), Vec2::splat(10.0)), 1.0, s);
        }
        CalcToken::RootTopLeft => { 
            p.text(c + Vec2::new(q*0.3, q*0.3), Align2::CENTER_CENTER, "x", FontId::monospace(26.0), color);
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(-q*0.9, -q*0.9), Vec2::splat(10.0)), 1.0, s);
        }
        CalcToken::OplusBotLeft => { 
            p.text(c + Vec2::new(q*0.3, -q*0.3), Align2::CENTER_CENTER, "x", FontId::monospace(26.0), color);
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(-q*0.9, q*0.9), Vec2::splat(10.0)), 1.0, s);
            p.line_segment([c + Vec2::new(-q*0.9, q*0.6), c + Vec2::new(-q*0.9, q*1.2)], Stroke::new(1.0, color));
            p.line_segment([c + Vec2::new(-q*1.2, q*0.9), c + Vec2::new(-q*0.6, q*0.9)], Stroke::new(1.0, color));
        }
        CalcToken::LogBotRight => { 
            p.text(c + Vec2::new(-q*0.3, -q*0.3), Align2::CENTER_CENTER, "x", FontId::monospace(26.0), color);
            p.rect_stroke(Rect::from_center_size(c + Vec2::new(q*0.9, q*0.9), Vec2::splat(10.0)), 1.0, s);
        }
        CalcToken::TriangleRight => {
            let points = vec![c - Vec2::new(q, q), c - Vec2::new(q, -q), c + Vec2::new(q, 0.0)];
            p.add(egui::Shape::closed_line(points, s));
        }
        CalcToken::TriangleLeft => {
            let points = vec![c + Vec2::new(q, q), c + Vec2::new(q, -q), c - Vec2::new(q, 0.0)];
            p.add(egui::Shape::closed_line(points, s));
        }
        _ => {
            let text = match token {
                CalcToken::Sin => "sin", CalcToken::Cos => "cos", CalcToken::Tan => "tan", CalcToken::Cot => "cot",
                CalcToken::ArcSin => "sin⁻¹", CalcToken::ArcCos => "cos⁻¹", CalcToken::ArcTan => "tan⁻¹", CalcToken::ArcCot => "cot⁻¹",
                CalcToken::ParenOpen => "(", CalcToken::ParenClose => ")",
                CalcToken::AC => "AC", CalcToken::Del => "DEL", CalcToken::Decimal => ".", CalcToken::Equals => "=",
                CalcToken::MPlus => "M+",
                _ => ""
            };
            p.text(c, Align2::CENTER_CENTER, text, FontId::monospace(18.0), color);
        }
    }
}

fn paint_dozenal_digit(_ui: &egui::Ui, painter: &egui::Painter, rect: Rect, digit: DozenalDigit, color: Color32, width: f32) {
    let s = Stroke::new(width, color);
    let c = rect.center();
    
    // DER FIX: Auch bei den Ziffern die Größe auf die kürzeste Seite limitieren
    let min_edge = rect.width().min(rect.height());
    let r = min_edge / 2.0; 
    let q = r * 0.5; 

    match digit {
        DozenalDigit::D1 => { 
            let tip = c + Vec2::new(0.0, -q); 
            painter.line_segment([tip, c + Vec2::new(-q, q)], s); 
            painter.line_segment([tip, c + Vec2::new(q, q)], s);  
        }
        DozenalDigit::D4 => { 
            let tip = c + Vec2::new(-q, 0.0); 
            painter.line_segment([tip, c + Vec2::new(q, -q)], s); 
            painter.line_segment([tip, c + Vec2::new(q, q)], s);  
        }
        DozenalDigit::D7 => { 
            let tip = c + Vec2::new(q, 0.0); 
            painter.line_segment([tip, c + Vec2::new(-q, -q)], s); 
            painter.line_segment([tip, c + Vec2::new(-q, q)], s);  
        }
        DozenalDigit::D10 => { 
            let tip = c + Vec2::new(0.0, q); 
            painter.line_segment([tip, c + Vec2::new(-q, -q)], s); 
            painter.line_segment([tip, c + Vec2::new(q, -q)], s);  
        }
        DozenalDigit::D0 => { painter.circle_stroke(c, q, s); } 
        DozenalDigit::D2 => { 
            draw_arc(painter, c + Vec2::new(0.0, -q), q, -90.0, 90.0, s);
            draw_arc(painter, c + Vec2::new(0.0, q), q, 90.0, 270.0, s);
        }
        DozenalDigit::D3 => { 
            draw_arc(painter, c + Vec2::new(0.0, -q), q, -90.0, 90.0, s);
            draw_arc(painter, c + Vec2::new(0.0, q), q, -90.0, 90.0, s);
        }
        DozenalDigit::D5 => { 
            draw_arc(painter, c + Vec2::new(0.0, -q), q, 90.0, 270.0, s);
            draw_arc(painter, c + Vec2::new(0.0, q), q, -90.0, 90.0, s);
        }
        DozenalDigit::D6 => { 
            draw_arc(painter, c + Vec2::new(0.0, -q), q, 90.0, 270.0, s);
            painter.circle_stroke(c + Vec2::new(0.0, q), q, s);
        }
        DozenalDigit::D8 => { 
            painter.circle_stroke(c + Vec2::new(0.0, -q), q, s);
            painter.circle_stroke(c + Vec2::new(0.0, q), q, s);
        }
        DozenalDigit::D9 => { 
            painter.circle_stroke(c + Vec2::new(0.0, -q), q, s);
            draw_arc(painter, c + Vec2::new(0.0, q), q, -90.0, 90.0, s);
        }
        DozenalDigit::D11 => { 
            draw_arc(painter, c + Vec2::new(0.0, -q), q, -90.0, 90.0, s);
            painter.circle_stroke(c + Vec2::new(0.0, q), q, s);
        }
    }
}

fn draw_arc(p: &egui::Painter, center: Pos2, radius: f32, start_deg: f32, end_deg: f32, stroke: Stroke) {
    let points: Vec<Pos2> = (0..=20).map(|i| {
        let angle = (start_deg + (end_deg - start_deg) * (i as f32 / 20.0)).to_radians();
        center + Vec2::new(angle.cos() * radius, angle.sin() * radius)
    }).collect();
    p.add(egui::Shape::line(points, stroke));
}
