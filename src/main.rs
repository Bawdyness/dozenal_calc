// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

mod logic;

use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};
use logic::{DozenalConverter, DozenalDigit, RatExpr, Rational, eval_rational};

#[derive(Clone, Copy, PartialEq)]
enum CalcToken {
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
enum AngleMode {
    Deg,
    #[default]
    Rad,
    Grad,
}

impl AngleMode {
    fn label(self) -> &'static str {
        match self {
            AngleMode::Deg => "DEG",
            AngleMode::Rad => "RAD",
            AngleMode::Grad => "GRD",
        }
    }

    fn next(self) -> Self {
        match self {
            AngleMode::Deg => AngleMode::Rad,
            AngleMode::Rad => AngleMode::Grad,
            AngleMode::Grad => AngleMode::Deg,
        }
    }

    /// Converts an angle from this mode to radians for meval.
    fn to_rad(self, x: f64) -> f64 {
        match self {
            AngleMode::Deg => x.to_radians(),
            AngleMode::Rad => x,
            AngleMode::Grad => x * std::f64::consts::PI / 200.0,
        }
    }

    /// Converts a result in radians to this mode's unit (for inverse trig).
    fn rad_to_unit(self, x: f64) -> f64 {
        match self {
            AngleMode::Deg => x.to_degrees(),
            AngleMode::Rad => x,
            AngleMode::Grad => x * 200.0 / std::f64::consts::PI,
        }
    }
}

// 1. Die Tür für den Desktop (Native)
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([400.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Dozenal Calc",
        native_options,
        Box::new(|cc| {
            // --- HIER ZWINGEN WIR DEN DESKTOP IN DEN DUNKELMODUS ---
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(DozenalCalcApp::default())
        }),
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
                Box::new(|cc| {
                    // --- HIER ZWINGEN WIR DAS HANDY/WEB IN DEN DUNKELMODUS ---
                    cc.egui_ctx.set_visuals(egui::Visuals::dark());
                    Box::new(DozenalCalcApp::default())
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[derive(Clone, Copy, PartialEq, Default)]
enum InfoState {
    #[default]
    Closed,
    List,
    Chapter(usize),
}

struct DozenalCalcApp {
    input_buffer: Vec<CalcToken>,
    result_buffer: Vec<CalcToken>,
    // Period metadata — parallel to result_buffer; None when no period or f64 fallback.
    result_period_start: Option<usize>, // index in result_buffer where the period begins
    result_period_len: usize,           // digits in period, capped at 5 for display
    result_period_capped: bool,         // true when the true period exceeds 5 digits
    cursor_pos: usize,
    memory: Vec<CalcToken>,
    memory_rational: Option<Rational>, // exact value for STO/RCL roundtrip
    last_ans: Option<Rational>,
    last_result_f64: f64, // kept for decimal display mode
    result_cursor_pos: usize,
    result_field_active: bool, // true after = until next input modifies input_buffer
    error_msg: Option<String>,
    overlay_open: bool,
    angle_mode: AngleMode,
    display_dec: bool, // true = show result in decimal; dozenal is the default
    info_state: InfoState,
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

/// Converts the `input_buffer` token sequence into `RatExpr` atoms for the
/// rational evaluation track. Returns `None` as soon as a non-rational token
/// (transcendental function, irrational constant, etc.) is encountered.
fn build_rat_expr(tokens: &[CalcToken]) -> Option<Vec<RatExpr>> {
    let mut exprs = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            CalcToken::Digit(_) => {
                let mut int_d: Vec<DozenalDigit> = Vec::new();
                let mut frac_d: Vec<DozenalDigit> = Vec::new();
                let mut in_frac = false;
                loop {
                    if i >= tokens.len() {
                        break;
                    }
                    match tokens[i] {
                        CalcToken::Digit(d) => {
                            if in_frac {
                                frac_d.push(d);
                            } else {
                                int_d.push(d);
                            }
                            i += 1;
                        }
                        CalcToken::Decimal if !in_frac => {
                            in_frac = true;
                            i += 1;
                        }
                        _ => break,
                    }
                }
                let int_val = DozenalConverter::to_decimal_exact(&int_d)?;
                let int_rat = Rational::new(int_val, 1)?;
                let rat = if frac_d.is_empty() {
                    int_rat
                } else {
                    let frac_num = DozenalConverter::to_decimal_exact(&frac_d)?;
                    let frac_den = 12_i128.checked_pow(frac_d.len() as u32)?;
                    int_rat.add(Rational::new(frac_num, frac_den)?)?
                };
                exprs.push(RatExpr::Num(rat));
            }
            CalcToken::Decimal => {
                // Leading decimal point: implicit zero integer part (e.g. ".6")
                i += 1;
                let mut frac_d: Vec<DozenalDigit> = Vec::new();
                while i < tokens.len() {
                    if let CalcToken::Digit(d) = tokens[i] {
                        frac_d.push(d);
                        i += 1;
                    } else {
                        break;
                    }
                }
                if frac_d.is_empty() {
                    return None;
                }
                let frac_num = DozenalConverter::to_decimal_exact(&frac_d)?;
                let frac_den = 12_i128.checked_pow(frac_d.len() as u32)?;
                exprs.push(RatExpr::Num(Rational::new(frac_num, frac_den)?));
            }
            CalcToken::Add => {
                exprs.push(RatExpr::Add);
                i += 1;
            }
            CalcToken::Sub | CalcToken::Negate => {
                exprs.push(RatExpr::Sub);
                i += 1;
            }
            CalcToken::Mul => {
                exprs.push(RatExpr::Mul);
                i += 1;
            }
            CalcToken::Div => {
                exprs.push(RatExpr::Div);
                i += 1;
            }
            CalcToken::ParenOpen => {
                exprs.push(RatExpr::LParen);
                i += 1;
            }
            CalcToken::ParenClose => {
                exprs.push(RatExpr::RParen);
                i += 1;
            }
            CalcToken::ExpTopRight => {
                exprs.push(RatExpr::Pow);
                i += 1;
            }
            CalcToken::OplusBotLeft => {
                exprs.push(RatExpr::OPlus);
                i += 1;
            }
            CalcToken::RatLit(r) => {
                exprs.push(RatExpr::Num(r));
                i += 1;
            }
            _ => return None, // non-rational token → collapse the track
        }
    }
    Some(exprs)
}

/// Formats an f64 as a decimal string with up to 10 significant digits, trailing zeros removed.
fn format_decimal_result(val: f64) -> String {
    if !val.is_finite() {
        return if val.is_nan() {
            "NaN".to_string()
        } else {
            "∞".to_string()
        };
    }
    let s = format!("{:.10}", val);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

// --- DER ÜBERSETZER UND DIE LAYOUTS ---
impl DozenalCalcApp {
    // --- KLICK-LOGIK ---
    fn handle_click(&mut self, token: CalcToken) {
        if self.error_msg.is_some() && token != CalcToken::AC {
            return;
        }
        // Ans auto-insertion: if we just evaluated (result_field_active) and the user
        // presses an operator as the first token of a new expression, prepend Ans.
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
        if self.result_field_active && is_operator && self.input_buffer.is_empty() {
            if let Some(r) = self.last_ans {
                self.input_buffer.push(CalcToken::RatLit(r));
            } else {
                for &t in &self.result_buffer.clone() {
                    self.input_buffer.push(t);
                }
            }
            self.cursor_pos = self.input_buffer.len();
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
    fn is_armed(&self, token: CalcToken) -> bool {
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

    // --- RECHEN-LOGIK ---
    fn calculate_result(&mut self) {
        let mut int_digits = Vec::new();
        let mut frac_digits = Vec::new();
        let mut in_fraction = false;
        let mut tokens_str: Vec<String> = Vec::new();

        for token in &self.input_buffer {
            match token {
                CalcToken::Digit(d) => {
                    if in_fraction {
                        frac_digits.push(*d);
                    } else {
                        int_digits.push(*d);
                    }
                }
                CalcToken::Decimal => {
                    in_fraction = true;
                }
                _ => {
                    if !int_digits.is_empty() || !frac_digits.is_empty() {
                        let int_val = if int_digits.is_empty() {
                            "0".to_string()
                        } else {
                            DozenalConverter::to_decimal(&int_digits).to_string()
                        };
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
                        CalcToken::Add => "+",
                        CalcToken::Sub | CalcToken::Negate => "-",
                        CalcToken::Mul => "*",
                        CalcToken::Div => "/",
                        CalcToken::Mod => "%",
                        CalcToken::ParenOpen => "(",
                        CalcToken::ParenClose => ")",
                        CalcToken::Sin => "sin(",
                        CalcToken::Cos => "cos(",
                        CalcToken::Tan => "tan(",
                        CalcToken::Cot => "cot(",
                        CalcToken::ExpTopRight => "^",
                        CalcToken::RootTopLeft => "√",
                        CalcToken::OplusBotLeft => "⊕",
                        CalcToken::LogBotRight => "log",
                        CalcToken::ArcSin => "asin(",
                        CalcToken::ArcCos => "acos(",
                        CalcToken::ArcTan => "atan(",
                        CalcToken::ArcCot => "acot(",
                        CalcToken::Sinh => "sinh(",
                        CalcToken::Cosh => "cosh(",
                        CalcToken::Tanh => "tanh(",
                        CalcToken::Coth => "coth(",
                        CalcToken::ArSinh => "arsinh(",
                        CalcToken::ArCosh => "arcosh(",
                        CalcToken::ArTanh => "artanh(",
                        CalcToken::ArCoth => "arcoth(",
                        CalcToken::Factorial => "fact(",
                        CalcToken::AbsVal => "abs(",
                        CalcToken::Reciprocal => "recip(",
                        _ => "",
                    };
                    // RatLit and irrational constants push their f64 values directly.
                    let const_val: Option<f64> = match token {
                        CalcToken::ConstPi => Some(std::f64::consts::PI),
                        CalcToken::ConstE => Some(std::f64::consts::E),
                        CalcToken::ConstPhi => Some(1.618_033_988_749_895),
                        CalcToken::ConstSqrt2 => Some(std::f64::consts::SQRT_2),
                        _ => None,
                    };
                    if let CalcToken::RatLit(r) = token {
                        tokens_str.push(r.to_f64().to_string());
                    } else if let Some(v) = const_val {
                        tokens_str.push(v.to_string());
                    } else if !s.is_empty() {
                        tokens_str.push(s.to_string());
                    }
                }
            }
        }

        if !int_digits.is_empty() || !frac_digits.is_empty() {
            let int_val = if int_digits.is_empty() {
                "0".to_string()
            } else {
                DozenalConverter::to_decimal(&int_digits).to_string()
            };
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
                tokens_str.splice(
                    (i - 1)..=(i + 1),
                    vec![format!("(({}*{})/({}+{}))", a, b, a, b)],
                );
            } else {
                break;
            }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "√") {
            if i + 1 >= tokens_str.len() {
                break;
            }
            // If preceded by an operator/open-paren (or at position 0), treat as √x (square root).
            // If preceded by a number/closing-paren, treat as n√x (n-th root).
            let preceded_by_op =
                i == 0 || matches!(tokens_str[i - 1].as_str(), "+" | "-" | "*" | "/" | "(");
            if preceded_by_op {
                let x = tokens_str[i + 1].clone();
                tokens_str.splice(i..=(i + 1), vec![format!("({}^(1/2))", x)]);
            } else {
                let n = tokens_str[i - 1].clone();
                let x = tokens_str[i + 1].clone();
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("({}^(1/{}))", x, n)]);
            }
        }

        while let Some(i) = tokens_str.iter().position(|t| t == "log") {
            if i > 0 && i + 1 < tokens_str.len() {
                let x = &tokens_str[i - 1];
                let n = &tokens_str[i + 1];
                tokens_str.splice((i - 1)..=(i + 1), vec![format!("(ln({})/ln({}))", x, n)]);
            } else {
                break;
            }
        }

        let mut math_string = tokens_str.join(" ");
        let open_parens = math_string.matches('(').count();
        let close_parens = math_string.matches(')').count();
        for _ in 0..(open_parens.saturating_sub(close_parens)) {
            math_string.push(')');
        }

        // --- Rational track (runs independently of meval) ---
        let rat_result = build_rat_expr(&self.input_buffer).and_then(|exprs| eval_rational(&exprs));

        // Copy angle mode so the closures can capture it by value (no borrow of self).
        let am = self.angle_mode;
        let mut ctx = meval::Context::new();
        // Angle-mode-aware trig — shadow meval builtins so DRG takes effect.
        ctx.func("sin", move |x| am.to_rad(x).sin());
        ctx.func("cos", move |x| am.to_rad(x).cos());
        ctx.func("tan", move |x| am.to_rad(x).tan());
        ctx.func("cot", move |x| 1.0 / am.to_rad(x).tan());
        ctx.func("asin", move |x| am.rad_to_unit(x.asin()));
        ctx.func("acos", move |x| am.rad_to_unit(x.acos()));
        ctx.func("atan", move |x| am.rad_to_unit(x.atan()));
        // Convention A: acot range (0,π), acot(x) = π/2 − atan(x).
        ctx.func("acot", move |x| {
            am.rad_to_unit(std::f64::consts::FRAC_PI_2 - x.atan())
        });
        ctx.func("coth", |x: f64| x.cosh() / x.sinh());
        ctx.func("arsinh", |x: f64| x.asinh());
        ctx.func("arcosh", |x: f64| x.acosh());
        ctx.func("artanh", |x: f64| x.atanh());
        ctx.func("arcoth", |x: f64| 0.5 * ((x + 1.0) / (x - 1.0)).ln());
        ctx.func("fact", |x: f64| {
            let n = x.round() as u64;
            (1..=n).fold(1u64, |acc, i| acc.saturating_mul(i)) as f64
        });
        ctx.func("abs", |x: f64| x.abs());
        ctx.func("recip", |x: f64| 1.0 / x);
        match meval::eval_str_with_context(&math_string, (ctx, meval::builtin())) {
            Ok(result) if result.is_finite() => {
                self.error_msg = None;
                self.last_ans = rat_result;
                self.last_result_f64 = result;

                if let Some(r) = rat_result {
                    // --- Rational track: exact display with possible overline ---
                    let (int_d, pre_d, period_d) = r.to_dozenal_periodic();
                    let mut buf: Vec<CalcToken> = Vec::new();
                    if r.num < 0 {
                        buf.push(CalcToken::Negate);
                    }
                    buf.extend(int_d.into_iter().map(CalcToken::Digit));
                    if !pre_d.is_empty() || !period_d.is_empty() {
                        buf.push(CalcToken::Decimal);
                    }
                    buf.extend(pre_d.into_iter().map(CalcToken::Digit));
                    let period_start = if period_d.is_empty() {
                        None
                    } else {
                        Some(buf.len())
                    };
                    let period_capped = period_d.len() > 5;
                    let period_len = period_d.len().min(5);
                    buf.extend(period_d.into_iter().take(5).map(CalcToken::Digit));
                    self.result_buffer = buf;
                    self.result_period_start = period_start;
                    self.result_period_len = period_len;
                    self.result_period_capped = period_capped;
                } else {
                    // --- f64 fallback: 4 fractional dozenal digits, no overline ---
                    let mut buf: Vec<CalcToken> = Vec::new();
                    let mut val = result;
                    if val < 0.0 {
                        buf.push(CalcToken::Negate);
                        val = val.abs();
                    }
                    buf.extend(
                        DozenalConverter::from_decimal(val)
                            .into_iter()
                            .map(CalcToken::Digit),
                    );
                    let frac_part = val - val.floor();
                    if frac_part > 0.000001 {
                        buf.push(CalcToken::Decimal);
                        buf.extend(
                            DozenalConverter::frac_to_digits(frac_part, 4)
                                .into_iter()
                                .map(CalcToken::Digit),
                        );
                    }
                    self.result_buffer = buf;
                    self.result_period_start = None;
                    self.result_period_len = 0;
                    self.result_period_capped = false;
                }

                self.input_buffer.clear();
                self.cursor_pos = 0;
                self.result_cursor_pos = 0;
                self.result_field_active = true;
            }
            Ok(result) if result.is_nan() => {
                // NaN comes from out-of-domain inputs (e.g. arcosh(0), artanh(1))
                self.error_msg = Some("DOMAIN ERROR".to_string());
            }
            Ok(_) => {
                // Infinite result — most likely division by zero
                self.error_msg = Some("DIV BY ZERO".to_string());
            }
            Err(_) => {
                // Meval konnte den String gar nicht erst berechnen (Syntaxfehler)
                self.error_msg = Some("SYNTAX ERROR".to_string());
            }
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
    fn draw_mobile_layout(&mut self, ui: &mut egui::Ui) {
        let spacing = 8.0;
        let num_spacing_y = 10.0;

        // --- DYNAMISCHE HÖHE MIT PUFFER ---
        let total_h = ui.available_height();

        // Der Sicherheits-Puffer speziell für die untere Browserleiste
        let safe_bottom_padding = 60.0;
        let usable_h = total_h - safe_bottom_padding;

        // Abzug für Header/Display (190) und Verteilung auf 10 Reihen
        let btn_height = ((usable_h - 190.0) / 10.0).clamp(25.0, 60.0);

        let num_btn_width = (ui.available_width() - (2.0 * spacing)) / 3.0;
        let num_btn_size = Vec2::new(num_btn_width, btn_height);

        let ops_btn_width = (ui.available_width() - (3.0 * spacing)) / 4.0;
        let ops_btn_size = Vec2::new(ops_btn_width, btn_height);

        // 1. Hilfsarbeiter
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

        // 2. Hilfsarbeiter
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

        // --- ZAHLENBLOCK OBEN ---
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

        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);

        // --- OPERATIONEN ---
        egui::Grid::new("mob_ops_vertical")
            .spacing([spacing, spacing])
            .show(ui, |ui| {
                let c = Color32::LIGHT_BLUE;
                // Zeile 1
                render_btn(self, ui, CalcToken::Add, c);
                render_btn(self, ui, CalcToken::OplusBotLeft, c);
                render_btn(self, ui, CalcToken::Sin, c);
                render_btn(self, ui, CalcToken::ParenOpen, c);
                ui.end_row();
                // Zeile 2
                render_btn(self, ui, CalcToken::Sub, c);
                render_btn(self, ui, CalcToken::ExpTopRight, c);
                render_btn(self, ui, CalcToken::Cos, c);
                render_btn(self, ui, CalcToken::ParenClose, c);
                ui.end_row();
                // Zeile 3
                render_btn(self, ui, CalcToken::Mul, c);
                render_btn(self, ui, CalcToken::RootTopLeft, c);
                render_btn(self, ui, CalcToken::Tan, c);
                render_btn(self, ui, CalcToken::TriangleLeft, c);
                ui.end_row();
                // Zeile 4
                render_btn(self, ui, CalcToken::Div, c);
                render_btn(self, ui, CalcToken::LogBotRight, c);
                render_btn(self, ui, CalcToken::Cot, c);
                render_btn(self, ui, CalcToken::TriangleRight, c);
                ui.end_row();
            });

        ui.add_space(10.0);

        // --- SYSTEMTASTEN ---
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

        // --- BREITE GLEICHHEITSTASTE ---
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

    fn draw_overlay(&mut self, ui: &mut egui::Ui, keypad_rect: Rect, is_mobile: bool) {
        // Dim the main keypad
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
            // Sets 6-9: 4 columns × 4 rows (mirrors the 4-column ops grid in mobile main layout)
            // Set 10: single row below, like Set 5 in the mobile main layout
            let btn_w = (keypad_rect.width() - spacing * 3.0) / 4.0;
            let extra_gap = spacing * 2.0; // extra visual separation before set-10 row
            // 5 button rows total: 4 for the main grid + 1 for set 10
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
            // Desktop: 5 columns × 4 rows — mirrors Sets 1–5 on desktop
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
    fn paint_overlay_btn(&mut self, ui: &mut egui::Ui, rect: Rect, token: CalcToken) {
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

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
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

// --- HAUPT-UPDATE SCHLEIFE (Das Fenster) ---
impl eframe::App for DozenalCalcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);
        egui::TopBottomPanel::bottom("footer").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.centered_and_justified(|ui| {
                ui.label(
                    egui::RichText::new(
                        "© 2026 Eric Naville \u{00b7} PolyForm Noncommercial License 1.0.0",
                    )
                    .size(10.0)
                    .color(egui::Color32::from_gray(120)),
                );
            });
            ui.add_space(4.0);
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Dozenal Calc");
                ui.add_space(10.0);

                // --- DISPLAY ---
                let (display_rect, _) = ui
                    .allocate_at_least(Vec2::new(ui.available_width(), 80.0), egui::Sense::hover());
                ui.painter()
                    .rect_filled(display_rect, 8.0, Color32::from_black_alpha(150));

                if !self.memory.is_empty() {
                    ui.painter().text(
                        display_rect.left_top() + Vec2::new(10.0, 10.0),
                        Align2::LEFT_TOP,
                        "M",
                        FontId::monospace(14.0),
                        Color32::GOLD,
                    );
                }

                // Angle mode indicator — top right
                ui.painter().text(
                    display_rect.right_top() + Vec2::new(-8.0, 8.0),
                    Align2::RIGHT_TOP,
                    self.angle_mode.label(),
                    FontId::monospace(11.0),
                    Color32::from_gray(180),
                );
                // DEC mode indicator — below angle mode
                if self.display_dec {
                    ui.painter().text(
                        display_rect.right_top() + Vec2::new(-8.0, 22.0),
                        Align2::RIGHT_TOP,
                        "DEC",
                        FontId::monospace(11.0),
                        Color32::from_rgb(100, 200, 255),
                    );
                }

                if self.input_buffer.is_empty() {
                    if self.display_dec {
                        // Decimal display mode: show the f64 result as a plain decimal string.
                        let val = self
                            .last_ans
                            .map(|r| r.to_f64())
                            .unwrap_or(self.last_result_f64);
                        let s = format_decimal_result(val);
                        ui.painter().text(
                            display_rect.center(),
                            Align2::CENTER_CENTER,
                            s,
                            FontId::monospace(26.0),
                            Color32::WHITE,
                        );
                    } else {
                        let n = self.result_buffer.len();
                        // Pre-compute x positions (right-aligned, mirrors the original right-to-left layout)
                        let mut positions = vec![Pos2::ZERO; n];
                        {
                            let mut x = display_rect.right() - 40.0;
                            for i in (0..n).rev() {
                                positions[i] = Pos2::new(x, display_rect.center().y);
                                x -= match self.result_buffer[i] {
                                    CalcToken::Decimal => 25.0,
                                    _ => 50.0,
                                };
                            }
                        }
                        // Draw overline above period digits (before tokens so digits render on top)
                        if let Some(ps) = self
                            .result_period_start
                            .filter(|&ps| self.result_period_len > 0 && ps < n)
                        {
                            {
                                let last = (ps + self.result_period_len - 1).min(n - 1);
                                let ox_start = positions[ps].x - 20.0;
                                let ox_end = positions[last].x + 20.0;
                                let oy = display_rect.center().y - 23.0;
                                ui.painter().line_segment(
                                    [Pos2::new(ox_start, oy), Pos2::new(ox_end, oy)],
                                    Stroke::new(1.5, Color32::WHITE),
                                );
                                if self.result_period_capped {
                                    ui.painter().text(
                                        Pos2::new(ox_end + 3.0, display_rect.center().y),
                                        Align2::LEFT_CENTER,
                                        "…",
                                        FontId::monospace(16.0),
                                        Color32::WHITE,
                                    );
                                }
                            }
                        }
                        // Draw result cursor when result field is active
                        if self.result_field_active && n > 0 {
                            let rcp = self.result_cursor_pos.min(n - 1);
                            let cx = positions[rcp].x - 20.0;
                            ui.painter().line_segment(
                                [
                                    Pos2::new(cx, display_rect.center().y - 15.0),
                                    Pos2::new(cx, display_rect.center().y + 15.0),
                                ],
                                Stroke::new(2.0, Color32::RED),
                            );
                        }
                        // Draw tokens
                        for (idx, token) in self.result_buffer.iter().enumerate() {
                            let rect = Rect::from_center_size(positions[idx], Vec2::splat(40.0));
                            match token {
                                CalcToken::Digit(d) => paint_dozenal_digit(
                                    ui,
                                    ui.painter(),
                                    rect,
                                    *d,
                                    Color32::WHITE,
                                    2.5,
                                ),
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
                    }
                } else {
                    let mut x_pos = display_rect.left() + 30.0;
                    for i in 0..=self.input_buffer.len() {
                        if i == self.cursor_pos {
                            ui.painter().line_segment(
                                [
                                    Pos2::new(x_pos - 15.0, display_rect.center().y - 15.0),
                                    Pos2::new(x_pos - 15.0, display_rect.center().y + 15.0),
                                ],
                                Stroke::new(2.0, Color32::RED),
                            );
                        }

                        if i < self.input_buffer.len() {
                            let token = &self.input_buffer[i];
                            let rect = Rect::from_center_size(
                                Pos2::new(x_pos, display_rect.center().y),
                                Vec2::splat(30.0),
                            );
                            match token {
                                CalcToken::Digit(d) => {
                                    paint_dozenal_digit(
                                        ui,
                                        ui.painter(),
                                        rect,
                                        *d,
                                        Color32::WHITE,
                                        2.0,
                                    );
                                    x_pos += 35.0;
                                }
                                _ => {
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
                }

                if let Some(msg) = &self.error_msg {
                    ui.painter().text(
                        display_rect.center(),
                        Align2::CENTER_CENTER,
                        msg,
                        FontId::monospace(30.0),
                        Color32::LIGHT_RED,
                    );
                }
                ui.add_space(20.0);

                // --- DER NEUE RESPONSIVE SCHALTER ---
                const MOBILE_BREAKPOINT_PX: f32 = 500.0;
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
            });
        });

        // Info modal — rendered outside the CentralPanel so it floats above
        if self.info_state != InfoState::Closed {
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
}

// --- INFO-MODAL ---

const INFO_TITLES: [&str; 12] = [
    "Bedienung des Rechners",
    "Was ist das Dozenalsystem?",
    "Fibonacci, Quadratzahlen und Kuriositäten",
    "Das Zwölfeck — Grundlagen",
    "Das Zwölfeck — Winkel und Diagonalen",
    "Das Zwölfeck — Flächen und Verhältnisse",
    "Der Dodekaeder — zwölf Fünfecke im Raum",
    "Der Dodekaeder — φ, Dualität und Symmetrie",
    "Zwölf Tierkreiszeichen und der Himmel",
    "Zwölf Flächen in Kristallen und Lebewesen",
    "Zwölf Glieder an der Hand",
    "Zoll, Fuss, Pfund — und warum sie dozenal Sinn ergeben",
];

fn info_h(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(text).strong());
}

fn info_p(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).size(12.5));
}

fn info_pre(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).monospace().size(11.0));
}

fn draw_digit_legend(ui: &mut egui::Ui) {
    let sym_size = Vec2::splat(24.0);
    let digits = [
        DozenalDigit::D0,
        DozenalDigit::D1,
        DozenalDigit::D2,
        DozenalDigit::D3,
        DozenalDigit::D4,
        DozenalDigit::D5,
        DozenalDigit::D6,
        DozenalDigit::D7,
        DozenalDigit::D8,
        DozenalDigit::D9,
        DozenalDigit::D10,
        DozenalDigit::D11,
    ];
    ui.horizontal(|ui| {
        for col in 0..2 {
            ui.vertical(|ui| {
                for row in 0..6 {
                    let i = col * 6 + row;
                    ui.horizontal(|ui| {
                        let (r, _) = ui.allocate_exact_size(sym_size, egui::Sense::hover());
                        paint_dozenal_digit(ui, ui.painter(), r, digits[i], Color32::WHITE, 1.5);
                        ui.label(
                            egui::RichText::new(format!("= {i}"))
                                .monospace()
                                .size(11.0)
                                .color(Color32::from_gray(200)),
                        );
                    });
                    ui.add_space(2.0);
                }
            });
            if col == 0 {
                ui.add_space(20.0);
            }
        }
    });
}

fn draw_chapter4_svg(ui: &mut egui::Ui) {
    // Dodekagon with inscribed triangle (teal), square (blue), hexagon (purple).
    // Layout follows the SVG from INFO_MODAL_CONTENT.md (viewBox 680×520, r=200).
    let avail_w = ui.available_width();
    let scale = avail_w / 680.0;
    let r = 200.0_f32;
    let draw_h = (r * 2.0 + 56.0) * scale;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, draw_h), egui::Sense::hover());
    let p = ui.painter();

    // Center the dodekagon; 30px SVG margin above top vertex
    let cx = rect.center().x;
    let cy = rect.min.y + (30.0 + r) * scale;

    // Vertices: angle starts at −90° (top), 30° steps
    let verts: Vec<Pos2> = (0..12)
        .map(|i| {
            let a = (i as f32 * 30.0 - 90.0).to_radians();
            Pos2::new(cx + r * scale * a.cos(), cy + r * scale * a.sin())
        })
        .collect();

    // Inscribed hexagon (every 2nd vertex: 0,2,4,6,8,10) — purple
    let hex: Vec<Pos2> = [0usize, 2, 4, 6, 8, 10].iter().map(|&i| verts[i]).collect();
    p.add(egui::Shape::convex_polygon(
        hex,
        Color32::from_rgba_unmultiplied(175, 169, 236, 25),
        Stroke::new(1.5, Color32::from_rgb(83, 74, 183)),
    ));

    // Inscribed square (every 3rd vertex: 0,3,6,9) — blue
    let sq: Vec<Pos2> = [0usize, 3, 6, 9].iter().map(|&i| verts[i]).collect();
    p.add(egui::Shape::convex_polygon(
        sq,
        Color32::from_rgba_unmultiplied(133, 183, 235, 25),
        Stroke::new(1.5, Color32::from_rgb(24, 95, 165)),
    ));

    // Inscribed triangle (every 4th vertex: 0,4,8) — teal
    let tri: Vec<Pos2> = [0usize, 4, 8].iter().map(|&i| verts[i]).collect();
    p.add(egui::Shape::convex_polygon(
        tri,
        Color32::from_rgba_unmultiplied(159, 225, 203, 25),
        Stroke::new(1.5, Color32::from_rgb(15, 110, 86)),
    ));

    // Dodekagon outline
    p.add(egui::Shape::closed_line(
        verts.clone(),
        Stroke::new(2.0, Color32::from_gray(210)),
    ));

    // Corner dots: primary (0,3,6,9 — square vertices) larger/brighter
    for (i, &v) in verts.iter().enumerate() {
        if i % 3 == 0 {
            p.circle_filled(v, 3.5, Color32::WHITE);
        } else {
            p.circle_filled(v, 2.5, Color32::from_gray(140));
        }
    }

    // Legend below the drawing (rendered as egui widgets at readable size)
    ui.add_space(4.0);
    let legend = [
        (
            Color32::from_rgba_unmultiplied(159, 225, 203, 80),
            Color32::from_rgb(15, 110, 86),
            "Dreieck (jede 4. Ecke)",
        ),
        (
            Color32::from_rgba_unmultiplied(133, 183, 235, 80),
            Color32::from_rgb(24, 95, 165),
            "Quadrat (jede 3. Ecke)",
        ),
        (
            Color32::from_rgba_unmultiplied(175, 169, 236, 80),
            Color32::from_rgb(83, 74, 183),
            "Sechseck (jede 2. Ecke)",
        ),
    ];
    for (fill, border, label) in legend {
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::new(14.0, 14.0), egui::Sense::hover());
            ui.painter().rect_filled(r, 2.0, fill);
            ui.painter().rect_stroke(r, 2.0, Stroke::new(1.0, border));
            ui.label(
                egui::RichText::new(label)
                    .size(11.0)
                    .color(Color32::from_gray(200)),
            );
        });
    }
}

fn draw_chapter5_svg(ui: &mut egui::Ui) {
    // Dodekagon with 6 colored diagonal types.
    // Layout follows the SVG from INFO_MODAL_CONTENT.md (viewBox 680×560, r=200).
    let avail_w = ui.available_width();
    // Keep the dodekagon at ~70 % of width; legend sits below.
    let dod_w = avail_w * 0.72;
    let scale = dod_w / 400.0; // dodekagon diameter = 400 SVG units
    let r = 200.0_f32;
    let draw_h = (r * 2.0 + 50.0) * scale;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(avail_w, draw_h), egui::Sense::hover());
    let p = ui.painter();

    let cx = rect.min.x + avail_w * 0.44;
    let cy = rect.min.y + (25.0 + r) * scale;

    let verts: Vec<Pos2> = (0..12)
        .map(|i| {
            let a = (i as f32 * 30.0 - 90.0).to_radians();
            Pos2::new(cx + r * scale * a.cos(), cy + r * scale * a.sin())
        })
        .collect();

    // Subtle dodekagon outline
    p.add(egui::Shape::closed_line(
        verts.clone(),
        Stroke::new(1.0, Color32::from_gray(110)),
    ));
    // Subtle all-vertex dots
    for &v in &verts {
        p.circle_filled(v, 2.0, Color32::from_gray(100));
    }

    // Six diagonal / side types with their colors
    let diagonals: [([usize; 2], Color32); 6] = [
        ([0, 1], Color32::from_rgb(95, 94, 90)),  // s: side — gray
        ([1, 3], Color32::from_rgb(15, 110, 86)), // d₂ — teal
        ([0, 3], Color32::from_rgb(24, 95, 165)), // d₃ — blue
        ([1, 5], Color32::from_rgb(83, 74, 183)), // d₄ — purple
        ([0, 5], Color32::from_rgb(153, 60, 29)), // d₅ — coral
        ([0, 6], Color32::from_rgb(163, 45, 45)), // d₆ — red
    ];
    for ([a, b], color) in diagonals {
        p.line_segment([verts[a], verts[b]], Stroke::new(2.5, color));
    }

    // Highlight the 5 involved vertices
    for &i in &[0usize, 1, 3, 5, 6] {
        p.circle_filled(verts[i], 4.0, Color32::WHITE);
    }

    // Legend below — colored line + formula + approx value
    ui.add_space(6.0);
    let legend_items: [([usize; 2], Color32, &str, &str); 6] = [
        ([0, 1], Color32::from_rgb(95, 94, 90), "s = 1", "≈ 1.000"),
        (
            [1, 3],
            Color32::from_rgb(15, 110, 86),
            "d₂ = \u{221a}(2+\u{221a}3)",
            "≈ 1.932",
        ),
        (
            [0, 3],
            Color32::from_rgb(24, 95, 165),
            "d₃ = 1+\u{221a}3",
            "≈ 2.732",
        ),
        (
            [1, 5],
            Color32::from_rgb(83, 74, 183),
            "d₄ = (3\u{221a}2+\u{221a}6)/2",
            "≈ 3.346",
        ),
        (
            [0, 5],
            Color32::from_rgb(153, 60, 29),
            "d₅ = 2+\u{221a}3",
            "≈ 3.732",
        ),
        (
            [0, 6],
            Color32::from_rgb(163, 45, 45),
            "d₆ = \u{221a}6+\u{221a}2",
            "≈ 3.864",
        ),
    ];
    egui::Grid::new("ch5_diag_legend")
        .num_columns(3)
        .spacing([6.0, 2.0])
        .show(ui, |ui| {
            for (_, color, formula, approx) in legend_items {
                let (lr, _) = ui.allocate_exact_size(Vec2::new(24.0, 12.0), egui::Sense::hover());
                ui.painter().line_segment(
                    [lr.left_center(), lr.right_center()],
                    Stroke::new(2.5, color),
                );
                ui.label(egui::RichText::new(formula).monospace().size(10.5));
                ui.label(
                    egui::RichText::new(approx)
                        .size(10.0)
                        .color(Color32::from_gray(155)),
                );
                ui.end_row();
            }
        });
}

fn draw_info_chapter(ui: &mut egui::Ui, chapter: usize) {
    match chapter {
        0 => {
            info_h(ui, "Die Ziffern");
            info_p(
                ui,
                "Dieser Rechner verwendet eigene Symbole für alle zwölf Ziffern. Vier Ankerziffern sind stilisierte Pfeilspitzen, die in die vier Himmelsrichtungen zeigen — 1 (oben), 4 (links), 7 (rechts), A (unten). Sie teilen den Zahlenkreis in vier Dreiergruppen, wie die Stunden 12, 3, 6 und 9 auf einem Zifferblatt.",
            );
            info_p(
                ui,
                "Alle Ziffern dazwischen bestehen aus Halbkreisen und Vollkreisen. Die Null ist ein einfacher Kreis, B (= elf) ein gefüllter Kreis.",
            );
            draw_digit_legend(ui);

            info_h(ui, "Grundbedienung");
            info_p(
                ui,
                "Tippe Zahlen und Operatoren wie auf einem gewöhnlichen Taschenrechner. Drücke die breite Taste am unteren Rand, um das Ergebnis zu berechnen. AC löscht die gesamte Eingabe und das Ergebnis, Del entfernt das Zeichen links vom Cursor.",
            );

            info_h(ui, "Cursor und Navigation");
            info_p(
                ui,
                "Der rote Strich im Eingabefeld ist der Cursor. Mit ◀ und ▶ bewegst du ihn, um mitten in einer Formel Zeichen einzufügen oder zu löschen. Nach einer Berechnung wandert der Cursor ins Ergebnisfeld — die Pfeile bewegen dann den Ergebnis-Cursor. Sobald du eine neue Eingabe machst, springt der Cursor zurück ins Eingabefeld.",
            );

            info_h(ui, "Weiterrechnen");
            info_p(
                ui,
                "Nach einer Berechnung kannst du direkt mit einem Operator weitermachen. Tippst du zum Beispiel + 5 =, verwendet der Rechner automatisch das letzte Ergebnis als ersten Operanden. Wenn du stattdessen eine ganz neue Rechnung beginnen willst, drücke zuerst AC.",
            );

            info_h(ui, "Doppelklick für Umkehrfunktionen");
            info_p(
                ui,
                "Ein zweiter Klick auf eine Funktionstaste wandelt sie in ihre Umkehrfunktion um: sin wird zu sin⁻¹, cos zu cos⁻¹, und so weiter. Das gilt auch für die hyperbolischen Funktionen im Erweiterungsfeld. Ein kleiner goldener Punkt auf der Taste zeigt an, dass der nächste Klick umkehrt.",
            );

            info_h(ui, "Spezialoperatoren");
            info_p(
                ui,
                "x² quadriert die vorangehende Zahl. √ berechnet die Quadratwurzel — steht links davon eine Zahl, wird diese als Wurzelgrad verwendet: 3√27 ergibt die dritte Wurzel von 27. log berechnet den Logarithmus zur Basis der vorangehenden Zahl. ⊕ berechnet die Paralleladdition: a ⊕ b = (a·b)/(a+b), nützlich für Parallelschaltungen von Widerständen.",
            );

            info_h(ui, "Erweiterungsfeld");
            info_p(
                ui,
                "Die Taste … rechts unten öffnet das Erweiterungsfeld mit weiteren Funktionen: Speicher, Konstanten (π, e, φ, √2), hyperbolische Funktionen, erweiterte Operatoren und Einstellungen. Es schliesst sich über die Taste rechts unten im Erweiterungsfeld selbst.",
            );
            info_pre(
                ui,
                "  6 — Speicher:    STO   RCL   MC    Ans\n  7 — Konstanten:  π     e     φ     √2\n  8 — Hyperbel:   sinh  cosh  tanh  coth\n  9 — Erweitert:  n!    |x|   1/x   mod\n  10 — Modi:      Doz↔  DRG   Info  ×",
            );

            info_h(ui, "Speicher");
            info_p(
                ui,
                "STO speichert das aktuelle Ergebnis, RCL fügt den gespeicherten Wert in die Eingabe ein, MC löscht den Speicher. Ein kleines M im Display zeigt an, dass etwas gespeichert ist. Ans fügt das Ergebnis der letzten Berechnung ein — exakte rationale Werte werden vollständig mitgespeichert, Periodizität bleibt erhalten.",
            );

            info_h(ui, "Periodenstrich");
            info_p(
                ui,
                "Wenn das Ergebnis ein periodischer Bruch ist, zeigt der Rechner die sich wiederholenden Ziffern mit einem Strich darüber an. Beispiel: 1/5 ergibt 0.2497 mit Strich über allen vier Ziffern. Bei Perioden mit mehr als fünf Stellen werden nur die ersten fünf gezeigt, gefolgt von …",
            );

            info_h(ui, "Anzeige und Winkelmodus");
            info_p(
                ui,
                "Doz↔Dec im Erweiterungsfeld schaltet die Anzeige zwischen dozenal (Basis 12) und dezimal (Basis 10) um — praktisch, um ein Ergebnis in vertrauter Schreibweise zu überprüfen. DRG wechselt den Winkelmodus für trigonometrische Funktionen: Rad → Grad → Gon → Rad, angezeigt oben rechts im Display.",
            );
        }
        1 => {
            info_h(ui, "Das Prinzip");
            info_p(
                ui,
                "Im Dezimalsystem hat jede Stelle den zehnfachen Wert der Stelle rechts davon: Einer, Zehner, Hunderter. Im Dozenalsystem ist die Basis nicht zehn, sondern zwölf. Die Stellenwerte sind Potenzen von 12: Einer, Zwölfer, Hundertvierundvierziger. Die Zahl »100« bedeutet hier nicht zehn mal zehn, sondern zwölf mal zwölf — also 144 im Dezimalen.",
            );
            info_p(
                ui,
                "Dafür braucht man zwölf Ziffern statt zehn. Für die Werte zehn und elf kommen zwei neue hinzu, die dieser Rechner mit eigenen Symbolen darstellt (A = zehn, B = elf). Beispiel: 2B (dozenal) = 2·12 + 11 = 35 (dezimal).",
            );

            info_h(ui, "Warum gerade zwölf?");
            info_p(
                ui,
                "Der Grund ist Teilbarkeit. Zwölf hat sechs Teiler: 1, 2, 3, 4, 6 und 12. Zehn hat nur vier: 1, 2, 5 und 10. Das klingt nach einem kleinen Unterschied, aber die Auswirkung auf den Alltag ist erheblich — vor allem beim Bruchrechnen.",
            );

            info_h(ui, "Stammbrüche im Vergleich");
            info_pre(
                ui,
                "  Bruch   Basis 10    Basis 12\n  1/2     0.5         0.6\n  1/3     0.333…      0.4\n  1/4     0.25        0.3\n  1/5     0.2         0.2497…\n  1/6     0.166…      0.2\n  1/8     0.125       0.16\n  1/9     0.111…      0.14\n  1/10    0.1         0.1249…\n  1/12    0.0833…     0.1",
            );
            info_p(
                ui,
                "In Basis 10 sind Drittel und Sechstel unendliche Dezimalbrüche. In Basis 12 sind sie kurz und exakt. Dafür werden Fünftel und Zehntel periodisch — ein fairer Tausch, wenn man bedenkt, wie viel häufiger man durch drei und vier teilt als durch fünf.",
            );

            info_h(ui, "Die Regel dahinter");
            info_p(
                ui,
                "Welche Brüche endlich sind und welche periodisch werden, folgt einem einfachen Gesetz: ein Bruch 1/n hat in einer Basis b genau dann eine endliche Darstellung, wenn alle Primfaktoren von n auch Primfaktoren von b sind. Die Primfaktoren von 12 sind 2 und 3. Also ist jeder Bruch endlich, dessen Nenner nur aus Zweien und Dreien zusammengesetzt ist. Alles andere — Nenner mit einer 5, 7 oder 11 — wird periodisch. Der Rechner zeigt diese Periodizität mit einem Strich über den sich wiederholenden Ziffern an.",
            );

            info_h(ui, "Spuren in der Geschichte");
            info_p(
                ui,
                "Die Zwölf als Ordnungsgrösse ist älter als jedes Zahlensystem. Die Babylonier rechneten in Basis 60, aber organisierten ihre Ziffern in Gruppen von 12. Im Handel zählte man in Dutzenden (12) und Gros (144 = 12²). Der Tag hat 2×12 Stunden, das Jahr 12 Monate, der Vollkreis 360 = 30×12 Grad.",
            );
            info_p(
                ui,
                "Die Dozenal Society of America (gegründet 1944, heute mit weiteren Ablegern) setzt sich dafür ein, die Vorzüge der Basis 12 bekannter zu machen. Dieser Rechner steht in dieser Tradition — nicht als Forderung nach einer Systemumstellung, sondern als Werkzeug zum Erforschen und Staunen.",
            );
        }
        2 => {
            info_h(ui, "144 — wo sich zwei Welten treffen");
            info_p(
                ui,
                "Die Fibonacci-Folge beginnt mit 1, 1, und jede weitere Zahl ist die Summe der beiden vorangehenden: 1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, … Sie wächst exponentiell. Die Quadratzahlen — 1, 4, 9, 16, 25, 36, … — wachsen dagegen nur quadratisch. Zwei so unterschiedliche Folgen haben fast keinen Grund, sich jemals zu treffen. Und doch tun sie es: die zwölfte Fibonacci-Zahl ist 144, und 144 = 12².",
            );
            info_p(
                ui,
                "J. H. E. Cohn bewies 1964, dass dies kein Zufall ist, sondern ein Unikat: abgesehen von F(1) = F(2) = 1 gibt es keine weitere Fibonacci-Zahl, die zugleich eine perfekte Quadratzahl ist. Die Zwölf steht an einer einmaligen Kreuzung zweier fundamentaler Zahlenfolgen.",
            );

            info_h(ui, "Der Goldene Schnitt");
            info_p(
                ui,
                "φ = (1+√5)/2 ≈ 1.618 ist der Grenzwert des Verhältnisses aufeinanderfolgender Fibonacci-Zahlen: F(n+1)/F(n) → φ. In Basis 12: φ ≈ 1.74BB677… — der Rechner hat φ als Konstante im Erweiterungsfeld.",
            );
            info_p(
                ui,
                "Wer φ² = tippt, wird sehen, dass das Ergebnis genau φ+1 ist — die definierende Eigenschaft des Goldenen Schnitts. Diese Identität macht φ zu einer algebraisch einzigartigen Konstante.",
            );

            info_h(ui, "12 = 2² × 3 — eine Primfaktorzerlegung mit Folgen");
            info_p(
                ui,
                "Zwölf ist eine hochzusammengesetzte Zahl (highly composite number): sie hat mehr Teiler als jede kleinere natürliche Zahl. Die Teiler von 12 sind 1, 2, 3, 4, 6, 12 — das sind sechs Stück. Srinivasa Ramanujan definierte und untersuchte diese Klasse von Zahlen in einer berühmten Arbeit von 1915 in den Proceedings of the London Mathematical Society.",
            );
            info_p(
                ui,
                "Zwölf ist auch die kleinste abundante Zahl: die Summe ihrer echten Teiler (1+2+3+4+6 = 16) übertrifft die Zahl selbst. Bei den meisten kleinen Zahlen ist es umgekehrt — bei 10 ergibt 1+2+5 = 8, was kleiner ist als 10. Zwölf ist die erste Zahl, bei der die Teiler »überquellen«.",
            );

            info_h(ui, "Platons ideale Stadt");
            info_p(
                ui,
                "In seinen »Gesetzen« (Buch V) stellt Platon die Frage, wie viele Bürger eine ideale Stadt haben sollte. Seine Antwort: 5040. Das Argument ist nicht mystisch, sondern praktisch: eine Stadt muss ihre Bürger ständig in gleich grosse Gruppen einteilen. 5040 ist durch jede Zahl von 1 bis 12 teilbar (mit der einzigen Ausnahme von 11).",
            );
            info_p(
                ui,
                "Was Platon intuitiv beschreibt, ist dieselbe Einsicht, die dem Dozenalsystem zugrunde liegt: im Alltag sind die kleinen Teiler die wichtigen. 5040 = 7! ist die grosse Schwester der Zwölf — dieselbe Teilbarkeitsphilosophie auf eine ganze Stadtbevölkerung angewendet.",
            );
        }
        3 => {
            info_h(ui, "Was ist ein regelmässiges Zwölfeck?");
            info_p(
                ui,
                "Ein regelmässiges Zwölfeck (Dodekagon) ist ein Vieleck mit zwölf gleich langen Seiten und zwölf gleich grossen Innenwinkeln. Jeder dieser Innenwinkel beträgt 150° — oder, im Dozenalsystem ausgedrückt, 106°. Es ist eine der ältesten und am häufigsten verwendeten geometrischen Formen: man findet sie in Zifferblättern, Münzen, Bauornamentik und Pflastermustern.",
            );

            info_h(ui, "Das Schweizer Taschenmesser der Vielecke");
            info_p(
                ui,
                "Was das Zwölfeck einzigartig macht, ist nicht seine Form an sich, sondern was alles in ihm steckt. Verbindet man jede vierte Ecke, entsteht ein gleichseitiges Dreieck — exakt, nicht angenähert. Jede dritte Ecke ergibt ein Quadrat. Jede zweite Ecke ein regelmässiges Sechseck. Alle drei Figuren liegen perfekt im selben Kreis, der auch das Zwölfeck umschliesst.",
            );
            info_p(
                ui,
                "Das bedeutet: das Zwölfeck enthält die drei fundamentalen regulären Vielecke der Geometrie als exakte Teilfiguren. Kein anderes Vieleck mit so wenigen Ecken kann das von sich behaupten. Eine direkte Folge der Teilbarkeit von 12 durch 2, 3, 4 und 6.",
            );

            info_h(ui, "Konstruierbar mit Zirkel und Lineal");
            info_p(
                ui,
                "Nicht jedes regelmässige Vieleck lässt sich mit Zirkel und Lineal exakt konstruieren. Das Zwölfeck dagegen ist konstruierbar: man beginnt mit einem Kreis, teilt ihn in sechs gleiche Teile (das gelingt, weil das Sechseck konstruierbar ist), halbiert dann jeden dieser Bögen, und hat zwölf gleichmässig verteilte Punkte auf dem Kreis.",
            );
            info_p(
                ui,
                "Die mathematische Grundlage: ein reguläres n-Eck ist genau dann konstruierbar, wenn n ein Produkt einer Zweierpotenz und verschiedener Fermat-Primzahlen ist (Gauss, 1796). Für 12 = 2² × 3 ist das erfüllt, weil 3 eine Fermat-Primzahl ist.",
            );

            info_h(ui, "Symmetrie");
            info_p(
                ui,
                "Das regelmässige Zwölfeck hat 24 Symmetrien: 12 Drehungen (um 0°, 30°, 60°, …, 330°) und 12 Spiegelungen (6 durch gegenüberliegende Ecken, 6 durch gegenüberliegende Seitenmitten). In der Sprache der Algebra bilden diese 24 Symmetrien die Diedergruppe D₁₂. Jedes reguläre n-Eck hat genau 2n Symmetrien.",
            );

            draw_chapter4_svg(ui);
        }
        4 => {
            info_h(ui, "54 Diagonalen");
            info_p(
                ui,
                "Eine Diagonale verbindet zwei nicht benachbarte Ecken eines Vielecks. Die Formel n(n−3)/2 liefert für das Zwölfeck 12×9/2 = 54 Diagonalen. Das klingt nach einem unübersichtlichen Netz — aber die Struktur ist bemerkenswert geordnet.",
            );

            info_h(ui, "Sechs verschiedene Längen");
            info_p(
                ui,
                "Jede Diagonale überspringt eine bestimmte Anzahl von Ecken. Da das Zwölfeck symmetrisch ist, haben alle Diagonalen, die gleich viele Ecken überspringen, dieselbe Länge. Es gibt fünf mögliche Sprungweiten (1 bis 5 Ecken), plus den Durchmesser mit 6 — also sechs verschiedene Längentypen. Bei Seitenlänge s = 1:",
            );
            info_pre(
                ui,
                "  Typ      Überspringt   Länge (exakt)      Näherung\n  s (Seite)  —           1                  1.000\n  d₂         1 Ecke      \u{221a}(2+\u{221a}3)           1.932\n  d₃         2 Ecken     1+\u{221a}3              2.732\n  d₄         3 Ecken     (3\u{221a}2+\u{221a}6)/2       3.346\n  d₅         4 Ecken     2+\u{221a}3              3.732\n  d₆ (⌀)    5 Ecken     \u{221a}6+\u{221a}2             3.864",
            );

            info_h(ui, "Verborgene Muster");
            info_p(
                ui,
                "Die dritte und die fünfte Diagonale unterscheiden sich um genau 1: d₃ = 1+√3 und d₅ = 2+√3. Die Differenz ist die Seitenlänge selbst — eine geometrische Tatsache, keine rechnerische.",
            );
            info_p(
                ui,
                "Der Durchmesser d₆ ist exakt doppelt so lang wie die kürzeste Diagonale d₂: √6+√2 = 2·√(2+√3). Durchmesser und kürzeste Diagonale stehen im Verhältnis 2:1 — dieselbe Proportion wie die Oktave in der Musik.",
            );

            info_h(ui, "Das 15-Grad-Raster");
            info_p(
                ui,
                "Alle Winkel, die im Zwölfeck auftreten — zwischen Seiten, zwischen Diagonalen — sind Vielfache von 15°. Das liegt daran, dass die zwölf Ecken den Vollkreis in zwölf Sektoren zu je 30° teilen. 15° = 1/24 des Vollkreises. Dozenal: 15° = 13°doz, und 30° = 26°doz. Alle auftretenden Winkel lassen sich dozenal als ganzzahlige Vielfache von 13° schreiben.",
            );

            draw_chapter5_svg(ui);
        }
        5 => {
            info_h(ui, "Die Fläche des Zwölfecks");
            info_p(
                ui,
                "Ein regelmässiges Zwölfeck mit Seitenlänge s hat die Fläche A = 3s²(2+√3). Die Herleitung ist anschaulich: man zerlegt das Zwölfeck vom Mittelpunkt aus in 12 gleichschenklige Dreiecke, berechnet die Fläche eines einzelnen Dreiecks und multipliziert mit 12.",
            );
            info_p(
                ui,
                "Bei s = 1 ergibt das A ≈ 11.196 (dezimal). Zum Vergleich: der Umkreis hat die Fläche πR² ≈ 11.725. Das Zwölfeck füllt seinen Umkreis zu mehr als 95% — deutlich besser als ein Sechseck (83%) und weit besser als ein Quadrat (64%) oder ein Dreieck (41%).",
            );

            info_h(ui, "3/π — ein elegantes Verhältnis");
            info_p(
                ui,
                "Das Verhältnis der Zwölfeck-Fläche zur Umkreis-Fläche vereinfacht sich zu 3/π. Die Herleitung nutzt sin²(15°) = (2−√3)/4, wodurch sich im Flächenverhältnis der Faktor (2+√3)(2−√3) zu 1 kürzt, und es bleibt genau 3/π übrig.",
            );
            info_p(
                ui,
                "3/π ≈ 0.9549 (dezimal) — das Zwölfeck erfasst 95.5% der Kreisfläche. Tippe 3 / π = im Rechner, um es zu verifizieren.",
            );

            info_h(ui, "Vier Vielecke im Vergleich");
            info_p(
                ui,
                "Alle folgenden Figuren teilen denselben Umkreis. Formel: A = (n/2)·R²·sin(2π/n).",
            );
            info_pre(
                ui,
                "  Figur          Anteil    Formel\n  Dreieck        41.3%     3\u{221a}3/(4\u{03c0})\n  Quadrat        63.7%     2/\u{03c0}\n  Sechseck       82.7%     3\u{221a}3/(2\u{03c0})\n  Zwölfeck       95.5%     3/\u{03c0}",
            );
            info_p(
                ui,
                "Das Sechseck hat exakt die doppelte Fläche des Dreiecks (beide enthalten den Faktor 3√3). Und jeder Schritt bringt einen grösseren Flächenzuwachs, weil die Ecken den Kreis immer enger umschliessen.",
            );

            info_h(ui, "Archimedes und die Kreiszahl");
            info_p(
                ui,
                "Archimedes berechnete π über Vielecke. Er verwendete ein 96-Eck — 96 = 12×8 = 12×2³. Er begann mit dem Sechseck (das trivial konstruierbar ist) und verdoppelte die Eckenzahl dreimal: 6→12→24→48→96. Der Ausgangspunkt seiner Methode war also das Zwölfeck.",
            );
            info_p(
                ui,
                "Sein Ergebnis: 3 + 10/71 < π < 3 + 1/7. Ein 96-Eck füllt den Umkreis zu 99.93%. Von den 95.5% des Zwölfecks zu 99.93% sind es nur drei Verdoppelungsschritte — ein bemerkenswertes Tempo der Konvergenz.",
            );
        }
        6 => {
            info_h(ui, "Zwölf Flächen");
            info_p(
                ui,
                "Der Dodekaeder ist ein Körper aus zwölf regelmässigen Fünfecken. Jede Fläche ist identisch, jede Kante gleich lang, und an jeder Ecke treffen genau drei Fünfecke zusammen. Insgesamt hat er 12 Flächen, 30 Kanten und 20 Ecken. Er ist einer der fünf platonischen Körper — die einzigen konvexen Körper, deren Flächen ausschliesslich aus identischen regelmässigen Vielecken bestehen.",
            );

            info_h(ui, "Die fünf platonischen Körper");
            info_pre(
                ui,
                "  Körper           Flächen  Ecken  Kanten  Form\n  Tetraeder            4      4       6   Dreiecke\n  Hexaeder/Würfel      6      8      12   Quadrate\n  Oktaeder             8      6      12   Dreiecke\n  Dodekaeder          12     20      30   Fünfecke\n  Ikosaeder           20     12      30   Dreiecke",
            );
            info_p(
                ui,
                "Der Dodekaeder ist der einzige platonische Körper mit fünfeckigen Flächen. Platon ordnete in seiner Kosmologie die vier anderen Körper den Elementen zu — den Dodekaeder dem Kosmos selbst.",
            );

            info_h(ui, "Wie sieht er aus?");
            info_p(
                ui,
                "Wer Rollenspiele spielt, kennt ihn als D12 — den zwölfseitigen Würfel. Er liegt angenehm in der Hand und kommt zuverlässig auf einer Fläche zum Liegen. Der Fussball ist kein Dodekaeder: er ist ein abgestumpftes Ikosaeder aus 12 Fünfecken und 20 Sechsecken.",
            );

            info_h(ui, "Gallorömische Pentagondodekaeder");
            info_p(
                ui,
                "Über hundert kleine Bronzeobjekte in Form des Dodekaeders wurden in Nordeuropa gefunden, datiert auf das 2. bis 4. Jahrhundert n. Chr. Sie haben zwölf fünfeckige Flächen mit unterschiedlich grossen runden Löchern darin. Niemand weiss mit Sicherheit, wofür sie verwendet wurden. Hypothesen reichen von Kerzenhaltern über Vermessungsinstrumente bis zu religiösen Gegenständen. Das Rätsel ist bis heute ungelöst.",
            );

            info_h(ui, "Eulers Polyedersatz");
            info_p(
                ui,
                "Für jeden konvexen Polyeder gilt eine einfache Beziehung: Ecken minus Kanten plus Flächen ist immer gleich zwei. Leonhard Euler formulierte dieses Gesetz 1758. Für den Dodekaeder: 20 − 30 + 12 = 2. Diese Formel gilt für alle fünf platonischen Körper, für jedes Prisma, für jede Pyramide, für jeden konvexen Körper überhaupt.",
            );
        }
        7 => {
            info_h(ui, "Der Goldene Schnitt im Dodekaeder");
            info_p(
                ui,
                "Jede Fläche des Dodekaeders ist ein regelmässiges Fünfeck — und das regelmässige Fünfeck ist die Heimat des Goldenen Schnitts. Die Diagonale eines solchen Fünfecks verhält sich zu seiner Seite exakt wie φ = (1+√5)/2 ≈ 1.618 (dezimal) zu 1. Diese Proportion durchdringt den gesamten Körper.",
            );
            info_pre(
                ui,
                "  Grösse            Formel              Dezimalwert\n  Volumen           (15+7\u{221a}5)/4          ≈ 7.663\n  Oberfläche        3\u{221a}(25+10\u{221a}5)        ≈ 20.646\n  Umkugelradius     \u{221a}3·\u{03c6}/2              ≈ 1.401\n  Inkugelradius     \u{221a}(25+11\u{221a}5)/(2\u{221a}10)   ≈ 1.114",
            );
            info_p(
                ui,
                "Wer im Rechner φ² = tippt, erhält φ+1. Das ist die definierende Eigenschaft des Goldenen Schnitts — und der Grund, warum φ in so vielen Formeln des Dodekaeders erscheint.",
            );

            info_h(ui, "Dualität — der Spiegel des Ikosaeders");
            info_p(
                ui,
                "Zu jedem platonischen Körper gibt es einen dualen Körper: man ersetzt jede Fläche durch eine Ecke (im Mittelpunkt der Fläche) und verbindet benachbarte neue Ecken mit Kanten. Beim Dodekaeder entsteht so das Ikosaeder — und umgekehrt:",
            );
            info_pre(
                ui,
                "               Dodekaeder   Ikosaeder\n  Flächen            12           20\n  Kanten             30           30\n  Ecken              20           12",
            );
            info_p(
                ui,
                "Flächen und Ecken tauschen die Plätze, die Kantenzahl bleibt gleich. Die 12 erscheint in beiden Körpern — einmal als Flächenzahl, einmal als Eckenzahl.",
            );

            info_h(ui, "120 Symmetrien");
            info_p(
                ui,
                "Der Dodekaeder besitzt die reichste Symmetrie unter allen platonischen Körpern: die Ikosaedergruppe Iₕ mit 120 Elementen — 60 Drehungen und 60 Dreh-Spiegelungen. Zum Vergleich: der Würfel hat nur 48 Symmetrien, das Tetraeder 24.",
            );
            info_p(
                ui,
                "120 = 5! = 2³×3×5. Die drei Primfaktoren 2, 3 und 5 sind exakt dieselben, die in den Flächen des Dodekaeders (Fünfecke) und in der Teilbarkeit von 12 (= 2²×3) zusammenkommen.",
            );
        }
        8 => {
            info_h(ui, "360 Grad und die Babylonier");
            info_p(
                ui,
                "Dass ein Vollkreis 360 Grad hat, ist keine Naturkonstante — es ist eine menschliche Festlegung, und sie geht auf die Babylonier zurück. Die babylonische Mathematik verwendete die Basis 60, und 360 = 6×60. Aber 360 lässt sich auch als 12×30 schreiben, und genau so teilten die Babylonier den Himmel auf: die scheinbare Sonnenbahn (die Ekliptik) wurde in 12 gleiche Abschnitte zu je 30° zerlegt. Jedem Abschnitt wurde ein Sternbild zugeordnet — die zwölf Tierkreiszeichen.",
            );
            info_p(
                ui,
                "Die Wahl von 12 war kein Zufall. Die Babylonier organisierten ihre 60er-Basis intern in Gruppen von 12, weil 60 = 12×5. Die Zwölf war für sie eine natürliche Untereinheit — in der Zeitmessung, im Kalender, in der Astronomie.",
            );

            info_h(ui, "Der Mond und die Zwölf");
            info_p(
                ui,
                "Warum gerade zwölf Abschnitte am Himmel? Weil die Natur selbst eine Zwölfteilung nahelegt: ein Sonnenjahr enthält fast genau 12 Mondzyklen. Ein synodischer Monat dauert etwa 29.53 Tage. 12 Mondzyklen ergeben 354.4 Tage — nur 11 Tage weniger als ein Sonnenjahr von 365.24 Tagen. Diese Beinahe-Übereinstimmung machte die Zwölf zur offensichtlichen Einteilung des Jahres.",
            );

            info_h(ui, "Ordnung am Himmel");
            info_p(
                ui,
                "Die Zwölfteilung des Himmels war für die alten Kulturen weit mehr als ein Koordinatensystem. Ein faszinierendes Detail: der Frühlingspunkt wandert langsam durch die Sternbilder, weil die Erdachse wie ein Kreisel taumelt (Präzession, Periode ca. 25'800 Jahre). Die Sternbilder, durch die der Frühlingspunkt wandert, heissen deshalb auch Zeitalter. Auch hier strukturiert die Zwölf die Zeit: zwölf Sternbilder, zwölf Zeitalter, ein grosser Kreis.",
            );

            info_h(ui, "Die Zwölf anderswo am Himmel");
            info_p(
                ui,
                "Die alten Ägypter teilten Tag und Nacht in je 12 Stunden — daher unsere 24-Stunden-Einteilung. Der chinesische Tierkreis zählt ebenfalls zwölf Zeichen in 12-Jahres-Zyklen, abgeleitet vom 12-jährigen Jupiterumlauf. Beide Traditionen sind unabhängig voneinander entstanden. Die Konvergenz auf die Zahl 12 ist bemerkenswert.",
            );
        }
        9 => {
            info_h(ui, "Pyrit — das Narren-Dodekaeder");
            info_p(
                ui,
                "Pyrit (FeS₂), wegen seines goldenen Glanzes auch als »Narrengold« bekannt, kristallisiert häufig in einer Form, die dem platonischen Dodekaeder zum Verwechseln ähnlich sieht: der Pyritoeder. Er hat zwölf fünfeckige Flächen, 20 Ecken und 30 Kanten — dieselbe Topologie wie der reguläre Dodekaeder aus Kapitel 7. Aber bei genauem Hinsehen sind die Fünfecke nicht regelmässig. In der Kristallographie ist echte fünfzählige Drehsymmetrie bei periodischen Kristallen unmöglich — nur Symmetrien der Ordnung 1, 2, 3, 4 und 6 sind erlaubt. Der Pyritoeder schummelt sich mit unregelmässigen Fünfecken an dieser Regel vorbei.",
            );

            info_h(ui, "Granat — ein anderer Zwölfflächner");
            info_p(
                ui,
                "Die Minerale der Granat-Gruppe kristallisieren bevorzugt als Rhombendodekaeder — ebenfalls ein Körper mit zwölf Flächen, aber ganz anderer Natur: die Flächen sind Rauten, keine Fünfecke. Der Rhombendodekaeder hat 14 Ecken und 24 Kanten und gehört zum kubischen Kristallsystem. Er füllt den Raum lückenlos — die dreidimensionale Entsprechung der Bienenwabe.",
            );
            info_p(
                ui,
                "Die Natur verwendet die Zahl 12 als Flächenzahl für zwei völlig verschiedene Kristallformen — Fünfecke beim Pyrit, Rauten beim Granat. Die Zwölf ist nicht an eine bestimmte Geometrie gebunden.",
            );

            info_h(ui, "Radiolarien — Skelette aus Glas");
            info_p(
                ui,
                "Radiolarien sind einzellige Meeresorganismen, kaum grösser als ein Zehntel Millimeter, die filigrane Skelette aus Siliziumdioxid bilden. Einige Arten formen Skelette mit ikosaedrischer Symmetrie — also der Symmetrie des Ikosaeders, des Duals zum Dodekaeder. Der deutsche Biologe Ernst Haeckel zeichnete diese Organismen 1904 in seinem Werk »Kunstformen der Natur« mit einer Detailtreue, die bis heute beeindruckt.",
            );

            info_h(ui, "Quasikristalle — die Ausnahme, die die Regel bestätigt");
            info_p(
                ui,
                "1982 entdeckte Dan Shechtman in einer Aluminium-Mangan-Legierung ein Muster mit ikosaedrischer Symmetrie — die in normalen Kristallen verboten ist. Die Fachwelt reagierte zunächst mit Ablehnung. Doch die Beobachtung hielt stand, und 2011 erhielt Shechtman den Nobelpreis für Chemie. Diese Quasikristalle haben ikosaedrische Symmetrie, die sowohl Dodekaeder- als auch Ikosaeder-Geometrie enthält. Natürlich vorkommende Quasikristalle — das Mineral Icosahedrit — wurden 2009 in einem Meteoriten in Kamtschatka entdeckt.",
            );
        }
        10 => {
            info_h(ui, "Zwölf an einer Hand");
            info_p(
                ui,
                "Halte eine Hand vor dich, den Daumen abgespreizt, und betrachte die vier Finger. Jeder Finger hat drei Glieder (Phalangen), getrennt durch sichtbare Gelenke. Vier Finger mal drei Glieder — das sind zwölf. Der Daumen kann als Zeiger dienen: er berührt nacheinander jedes Glied der vier Finger und zählt so von eins bis zwölf.",
            );
            info_p(
                ui,
                "Diese Methode ist keine moderne Erfindung. In Teilen Südostasiens, Indiens und des Nahen Ostens wird sie seit Jahrhunderten verwendet. Sie hat einen entscheidenden Vorteil gegenüber dem westlichen Fingerzählen: sie nutzt eine Hand für zwölf Einheiten statt für fünf.",
            );

            info_h(ui, "Von zwölf zu sechzig");
            info_p(
                ui,
                "Die zweite Hand zählt die vollen Durchgänge. Jedes Mal, wenn die erste Hand eine Runde von zwölf vollendet hat, streckt die zweite Hand einen Finger aus. Fünf Finger mal zwölf — das ergibt sechzig. Mit zwei Händen kann man also bis 60 zählen, und das ohne jedes Hilfsmittel.",
            );
            info_p(
                ui,
                "Diese Verbindung von 12 und 60 ist vermutlich kein Zufall: das babylonische Sexagesimalsystem (Basis 60) könnte seinen Ursprung in genau dieser Zählmethode haben. 60 = 12×5 — eine elegante Verschmelzung von Anatomie und Arithmetik.",
            );

            info_h(ui, "Weitere Zwölfer in der Anatomie");
            info_pre(
                ui,
                "  — 12 Rippenpaare (Standardanatomie)\n  — 12 Hirnnervenpaare (I Olfactorius bis XII Hypoglossus)\n  — 12 Brustwirbel (mit den 12 Rippenpaaren verbunden)",
            );
            info_p(
                ui,
                "Anatomische Variationen kommen vor (11 oder 13 Rippenpaare sind selten möglich).",
            );

            info_h(ui, "Hat die Hand das Zahlensystem geformt?");
            info_p(
                ui,
                "Ob die Fingerglieder-Anatomie die Entstehung dozenaler Zahlensysteme beeinflusst hat oder umgekehrt, lässt sich historisch nicht sicher entscheiden. Es könnte auch eine gegenseitige Verstärkung gewesen sein: die Menschen begannen an den Fingergliedern zu zählen, weil die Zwölf in ihrer Kultur bereits wichtig war. Was sicher ist: die menschliche Hand bietet eine natürliche physische Grundlage für die Zwölf.",
            );
        }
        11 => {
            info_h(ui, "Zwölfer im Alltag");
            info_pre(
                ui,
                "  12 Zoll  = 1 Fuss\n  12 Unzen = 1 Troy-Pfund (Edelmetalle)\n  12 Pence = 1 Shilling (brit. Geld bis 1971)\n  12 Stück = 1 Dutzend\n  144      = 12² = 1 Gros",
            );
            info_p(
                ui,
                "Diese Einteilungen sind keine historischen Zufälle — sie wurden gewählt, weil sie das Teilen erleichtern. Ein Fuss lässt sich in zwei gleiche Teile teilen (je 6 Zoll), in drei (je 4 Zoll), in vier (je 3 Zoll) und in sechs (je 2 Zoll). Jede dieser Teilungen geht exakt auf.",
            );
            info_p(
                ui,
                "Ein Meter dagegen lässt sich in zwei gleiche Teile teilen (je 50 cm) und in fünf (je 20 cm), aber ein Drittel Meter ist 33.333… cm — ein unendlicher Bruch, sobald man es exakt nehmen will. Im Handwerk, wo ständig gedrittelt und geviertelt wird, ist die Zwölf praktischer als die Zehn.",
            );

            info_h(ui, "Das metrische System — und sein blinder Fleck");
            info_p(
                ui,
                "Das metrische System hat grosse Stärken: es ist kohärent (alle Einheiten passen zusammen), es skaliert dezimal (Kilo, Mega, Milli, Mikro), und es ist weltweit standardisiert. Diese Vorzüge sind real und gewichtig. Kein vernünftiger Mensch würde vorschlagen, SI abzuschaffen.",
            );
            info_p(
                ui,
                "Aber das metrische System erbt die Schwäche seiner Basis. In Basis 10 ist ein Drittel ein unendlicher Bruch: 0.333… In einem dozenalen metrischen System wäre 1/3 = 0.4 — exakt, kurz, ohne Restfehler. Die Eleganz des metrischen Prinzips bliebe erhalten — nur die Basis wäre besser.",
            );

            info_h(ui, "Tom Pendleburys TGM");
            info_p(
                ui,
                "Tom Pendlebury, Mitglied der Dozenal Society of Great Britain, hat diesen Gedanken konsequent zu Ende gedacht. Sein System heisst TGM — benannt nach seinen drei Grundeinheiten Tim (Zeit), Grafut (Länge), Maz (Masse). Pendlebury begann nicht mit der Länge, sondern mit der Zeit: er teilte die Stunde in 12⁴ gleiche Teile. Aus dem Tim leitete er über die Erdbeschleunigung den Grafut ab (≈ 29.6 cm), die Masseeinheit Maz aus dem Volumen eines Kubik-Grafut Wasser.",
            );
            info_p(
                ui,
                "Das Ergebnis ist ein vollständig kohärentes Einheitensystem, in dem alle Umrechnungen in Potenzen von 12 erfolgen. TGM wurde nie über Enthusiastenkreise hinaus angenommen, demonstriert aber, dass ein dozenales Metriksystem nicht nur möglich, sondern in mancher Hinsicht dem dezimalen überlegen wäre.",
            );

            info_h(ui, "Was dieser Rechner zeigt");
            info_p(
                ui,
                "Wer 1 / 3 = tippt und 0.4 sieht — kurz, exakt, ohne Periodenstrich — versteht in einer Sekunde, was Seiten voller Argumente nicht vermitteln können. Die Frage »Dozenal oder Dezimal?« wird in der Praxis nie entschieden werden. Aber die mathematischen Vorteile der Basis 12 sind objektiv und messbar, und dieser Rechner macht sie erlebbar.",
            );
        }
        _ => {
            info_p(ui, "Kapitel nicht gefunden.");
        }
    }
}

// --- ZEICHEN-ROUTINEN ---
fn paint_token(
    _ui: &egui::Ui,
    p: &egui::Painter,
    rect: Rect,
    token: CalcToken,
    color: Color32,
    width: f32,
) {
    let s = Stroke::new(width, color);
    let c = rect.center(); // Das exakte Zentrum des Buttons

    // Dynamische Größe basierend auf dem Feld
    let min_edge = rect.width().min(rect.height());
    let q = min_edge / 4.0;

    p.rect_stroke(rect, 4.0, Stroke::new(1.0, Color32::from_gray(80)));

    match token {
        CalcToken::Add => {
            p.line_segment([c - Vec2::new(q, 0.0), c + Vec2::new(q, 0.0)], s);
            p.line_segment([c - Vec2::new(0.0, q), c + Vec2::new(0.0, q)], s);
        }
        CalcToken::Sub => {
            p.line_segment([c - Vec2::new(q, 0.0), c + Vec2::new(q, 0.0)], s);
        }
        CalcToken::Mul => {
            p.line_segment([c - Vec2::new(q, q), c + Vec2::new(q, q)], s);
            p.line_segment([c - Vec2::new(q, -q), c + Vec2::new(q, -q)], s);
        }
        CalcToken::Div => {
            p.line_segment([c - Vec2::new(q, -q), c + Vec2::new(q, -q)], s);
        }

        // --- DIE NEUEN, ZENTRIERTEN X-OPERATIONEN ---
        CalcToken::ExpTopRight => {
            // X exakt in der Mitte, Größe passt sich dynamisch an (45% der Button-Größe)
            p.text(
                c,
                Align2::CENTER_CENTER,
                "x",
                FontId::monospace(min_edge * 0.45),
                color,
            );
            // Quadrat in die Ecke oben rechts geschoben
            p.rect_stroke(
                Rect::from_center_size(
                    c + Vec2::new(q * 1.3, -q * 1.3),
                    Vec2::splat(min_edge * 0.18),
                ),
                1.0,
                s,
            );
        }
        CalcToken::RootTopLeft => {
            p.text(
                c,
                Align2::CENTER_CENTER,
                "x",
                FontId::monospace(min_edge * 0.45),
                color,
            );
            // Quadrat oben links
            p.rect_stroke(
                Rect::from_center_size(
                    c + Vec2::new(-q * 1.3, -q * 1.3),
                    Vec2::splat(min_edge * 0.18),
                ),
                1.0,
                s,
            );
        }
        CalcToken::LogBotRight => {
            p.text(
                c,
                Align2::CENTER_CENTER,
                "x",
                FontId::monospace(min_edge * 0.45),
                color,
            );
            // Quadrat unten rechts
            p.rect_stroke(
                Rect::from_center_size(
                    c + Vec2::new(q * 1.3, q * 1.3),
                    Vec2::splat(min_edge * 0.18),
                ),
                1.0,
                s,
            );
        }
        CalcToken::OplusBotLeft => {
            p.text(
                c,
                Align2::CENTER_CENTER,
                "x",
                FontId::monospace(min_edge * 0.45),
                color,
            );
            // Quadrat unten links
            let sq_c = c + Vec2::new(-q * 1.3, q * 1.3);
            let sq_size = min_edge * 0.18;
            p.rect_stroke(Rect::from_center_size(sq_c, Vec2::splat(sq_size)), 1.0, s);

            // Das kleine Plus im Quadrat wird ebenfalls dynamisch gezeichnet
            let cross = sq_size * 0.3;
            p.line_segment(
                [sq_c + Vec2::new(0.0, -cross), sq_c + Vec2::new(0.0, cross)],
                Stroke::new(1.0, color),
            );
            p.line_segment(
                [sq_c + Vec2::new(-cross, 0.0), sq_c + Vec2::new(cross, 0.0)],
                Stroke::new(1.0, color),
            );
        }

        CalcToken::TriangleRight => {
            let points = vec![
                c - Vec2::new(q, q),
                c - Vec2::new(q, -q),
                c + Vec2::new(q, 0.0),
            ];
            p.add(egui::Shape::closed_line(points, s));
        }
        CalcToken::TriangleLeft => {
            let points = vec![
                c + Vec2::new(q, q),
                c + Vec2::new(q, -q),
                c - Vec2::new(q, 0.0),
            ];
            p.add(egui::Shape::closed_line(points, s));
        }
        _ => {
            let text = match token {
                CalcToken::Sin => "sin",
                CalcToken::Cos => "cos",
                CalcToken::Tan => "tan",
                CalcToken::Cot => "cot",
                CalcToken::ArcSin => "sin⁻¹",
                CalcToken::ArcCos => "cos⁻¹",
                CalcToken::ArcTan => "tan⁻¹",
                CalcToken::ArcCot => "cot⁻¹",
                CalcToken::ParenOpen => "(",
                CalcToken::ParenClose => ")",
                CalcToken::Sinh => "sinh",
                CalcToken::Cosh => "cosh",
                CalcToken::Tanh => "tanh",
                CalcToken::Coth => "coth",
                CalcToken::ArSinh => "sinh⁻¹",
                CalcToken::ArCosh => "cosh⁻¹",
                CalcToken::ArTanh => "tanh⁻¹",
                CalcToken::ArCoth => "coth⁻¹",
                CalcToken::AC => "AC",
                CalcToken::Del => "DEL",
                CalcToken::Decimal => ".",
                CalcToken::Equals => "=",
                CalcToken::Expand => "…",
                CalcToken::Sto => "STO",
                CalcToken::Rcl => "RCL",
                CalcToken::Mc => "MC",
                CalcToken::Ans => "Ans",
                CalcToken::ConstPi => "π",
                CalcToken::ConstE => "e",
                CalcToken::ConstPhi => "φ",
                CalcToken::ConstSqrt2 => "√2",
                CalcToken::Factorial => "n!",
                CalcToken::AbsVal => "|x|",
                CalcToken::Reciprocal => "1/x",
                CalcToken::Mod => "mod",
                CalcToken::DozDec => "Doz",
                CalcToken::Drg => "DRG",
                CalcToken::Info => "Info",
                CalcToken::Close => "…",
                _ => "",
            };
            // Auch die anderen Texte (wie "sin", "cos") passen sich jetzt an den Button an
            p.text(
                c,
                Align2::CENTER_CENTER,
                text,
                FontId::monospace(min_edge * 0.35),
                color,
            );
        }
    }
}

fn paint_dozenal_digit(
    _ui: &egui::Ui,
    painter: &egui::Painter,
    rect: Rect,
    digit: DozenalDigit,
    color: Color32,
    width: f32,
) {
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
        DozenalDigit::D0 => {
            painter.circle_stroke(c, q, s);
        }
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

fn draw_arc(
    p: &egui::Painter,
    center: Pos2,
    radius: f32,
    start_deg: f32,
    end_deg: f32,
    stroke: Stroke,
) {
    let points: Vec<Pos2> = (0..=20)
        .map(|i| {
            let angle = (start_deg + (end_deg - start_deg) * (i as f32 / 20.0)).to_radians();
            center + Vec2::new(angle.cos() * radius, angle.sin() * radius)
        })
        .collect();
    p.add(egui::Shape::line(points, stroke));
}

#[cfg(test)]
mod tests {
    fn eval(expr: &str) -> f64 {
        let mut ctx = meval::Context::new();
        ctx.func("cot", |x: f64| 1.0 / x.tan());
        ctx.func("acot", |x: f64| std::f64::consts::FRAC_PI_2 - x.atan());
        meval::eval_str_with_context(expr, (ctx, meval::builtin())).unwrap()
    }

    // acot Convention A: range (0, π), formula π/2 - atan(x)
    #[test]
    fn acot_convention_a() {
        let pi = std::f64::consts::PI;
        assert!((eval("acot(1)") - pi / 4.0).abs() < 1e-10);
        assert!((eval("acot(-1)") - 3.0 * pi / 4.0).abs() < 1e-10);
        assert!((eval("acot(0)") - pi / 2.0).abs() < 1e-10);
    }

    #[test]
    fn cot_basic() {
        // cot(π/4) = 1
        assert!((eval("cot(pi/4)") - 1.0).abs() < 1e-10);
        // cot in denominator: 6 / cot(π/4) = 6
        assert!((eval("6/cot(pi/4)") - 6.0).abs() < 1e-10);
    }

    #[test]
    fn sqrt_mid_expression() {
        // √ at start → square root
        assert!((eval("(16^(1/2))") - 4.0).abs() < 1e-10);
        // n√x syntax: 9^(1/8) ≈ 1.2968...
        assert!((eval("(8^(1/9))") - 8_f64.powf(1.0 / 9.0)).abs() < 1e-10);
    }

    #[test]
    fn unary_minus() {
        assert!((eval("-5+3") - (-2.0)).abs() < 1e-10);
        assert!((eval("5*-3") - (-15.0)).abs() < 1e-10);
        assert!((eval("5-3") - 2.0).abs() < 1e-10);
    }
}
