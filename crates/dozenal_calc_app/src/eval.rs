// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::state::DozenalCalcApp;
use dozenal_core::{
    build_meval_string, build_rat_expr, eval_f64, eval_rational, format_f64_result,
    format_rational_result, resolve_postfix, with_implicit_muls,
};

impl DozenalCalcApp {
    pub fn calculate_result(&mut self) {
        // resolve_postfix sortiert Postfix-Aufrufe (n!, |x|, 1/x) zu Präfix-Aufrufen
        // um, damit `build_meval_string` sie als `fact(…)`, `abs(…)`, `recip(…)`
        // formen kann. Beide Eingabewege (User tippt `5 !` oder `! 5`) funktionieren.
        let normalized = resolve_postfix(&self.input_buffer);
        let expanded = with_implicit_muls(&normalized);
        let math_string = build_meval_string(&expanded);
        let rat_result = build_rat_expr(&expanded).and_then(|exprs| eval_rational(&exprs));

        match eval_f64(&math_string, self.angle_mode) {
            Some(result) if result.is_finite() => {
                self.error_msg = None;
                self.last_ans.clone_from(&rat_result);
                self.last_result_f64 = result;

                if let Some(r) = rat_result {
                    let (buf, meta) = format_rational_result(&r);
                    self.result_buffer = buf;
                    self.result_period_start = meta.start;
                    self.result_period_len = meta.len;
                    self.result_period_capped = meta.capped;
                } else {
                    self.result_buffer = format_f64_result(result);
                    self.result_period_start = None;
                    self.result_period_len = 0;
                    self.result_period_capped = false;
                }

                self.result_cursor_pos = 0;
                self.result_field_active = true;
            }
            Some(result) if result.is_nan() => {
                self.error_msg = Some("DOMAIN ERROR".to_string());
            }
            Some(_) => {
                // Infinite — typischerweise Division durch Null.
                self.error_msg = Some("DIV BY ZERO".to_string());
            }
            None => {
                self.error_msg = Some("SYNTAX ERROR".to_string());
            }
        }
    }
}
