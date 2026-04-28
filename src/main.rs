// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

mod eval;
mod input;
mod layout;
mod logic;
mod painting;
mod tokens;

use eframe::egui;
use egui::Color32;
use tokens::DozenalCalcApp;

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
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Box::new(DozenalCalcApp::default())
        }),
    )
}

// 2. Die Tür für den Browser (WebAssembly)
#[cfg(target_arch = "wasm32")]
fn main() {
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                web_options,
                Box::new(|cc| {
                    cc.egui_ctx.set_visuals(egui::Visuals::dark());
                    Box::new(DozenalCalcApp::default())
                }),
            )
            .await
            .expect("failed to start eframe");
    });
}

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
                    .color(Color32::from_gray(120)),
                );
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Dozenal Calc");
                ui.add_space(10.0);
                self.draw_display(ui, ctx);
                ui.add_space(20.0);
                self.draw_keypad(ui, ctx);
            });
        });

        self.draw_info_modal(ctx);
    }
}
