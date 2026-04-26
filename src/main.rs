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
    show_info: bool,
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
            show_info: false,
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
                    self.show_info = true;
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
}

// --- HAUPT-UPDATE SCHLEIFE (Das Fenster) ---
impl eframe::App for DozenalCalcApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                let is_mobile = ctx.screen_rect().width() < 500.0;
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
        if self.show_info {
            let mut open = self.show_info;
            egui::Window::new("Dozenal — Why base 12?")
                .open(&mut open)
                .resizable(false)
                .collapsible(false)
                .vscroll(true)
                .default_width(340.0)
                .show(ctx, |ui| {
                    ui.label(egui::RichText::new("Why base 12?").strong());
                    ui.add_space(4.0);
                    ui.label("12 has more divisors (1, 2, 3, 4, 6, 12) than any smaller integer — that's what \"highly composite\" means. Base 10 only has 1, 2, 5, 10.");
                    ui.add_space(4.0);
                    ui.label("12 is also the smallest abundant number: the sum of its proper divisors (1+2+3+4+6 = 16) exceeds 12 itself.");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Short fractions in base 12").strong());
                    ui.add_space(4.0);
                    ui.label("1/2 = 0.6      1/3 = 0.4\n1/4 = 0.3      1/6 = 0.2\n1/8 = 0.16     1/9 = 0.14");
                    ui.add_space(4.0);
                    ui.label("Compare base 10, where 1/3 = 0.333… and 1/6 = 0.1666…");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Periodic fractions (base 12)").strong());
                    ui.add_space(4.0);
                    ui.label("1/5  = 0.[2497]         period 4\n1/7  = 0.[186A35]       period 6\n1/B  = 0.[1]            period 1\n1/11 = 0.[0A35186]…     period 6");
                    ui.add_space(4.0);
                    ui.label("The overline over a digit group means it repeats infinitely. A trailing … means the period was longer than 5 digits.");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Constants in base 12 (first 14 digits)").strong());
                    ui.add_space(4.0);
                    ui.label("π  ≈ 3.184809493B9186…\ne  ≈ 2.875236069821…\nφ  ≈ 1.74BB6772802A4…\n√2 ≈ 1.4B79170A07B85…");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("F(12) = 144 = 12²").strong());
                    ui.add_space(4.0);
                    ui.label("The 12th Fibonacci number is 144 — the only Fibonacci number that is a perfect square besides F(1) = F(2) = 1 (proven by Cohn, 1964).");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Usage hints").strong());
                    ui.add_space(4.0);
                    ui.label("• Double-click sin/cos/tan/cot for their inverses.\n• Double-click sinh/cosh/tanh/coth (overlay) for inverses.\n• ◀ ▶ move the cursor inside the input expression.\n• Doz↔Dec (overlay) toggles the result display between dozenal and decimal.\n• DRG (overlay) cycles Rad → Grad → Deg → Rad.");
                });
            self.show_info = open;
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
