// SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0
// Copyright (c) 2026 Eric Naville

use crate::logic::DozenalDigit;
use crate::tokens::CalcToken;
use eframe::egui;
use egui::{Align2, Color32, FontId, Pos2, Rect, Stroke, Vec2};

// --- ZEICHEN-ROUTINEN ---
pub fn paint_token(
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

pub fn paint_dozenal_digit(
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

// --- INFO-MODAL HILFSROUTINEN ---

pub fn info_h(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(text).strong().size(17.0));
}

pub fn info_p(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).size(15.0));
}

pub fn info_pre(ui: &mut egui::Ui, text: &str) {
    ui.add_space(2.0);
    ui.label(egui::RichText::new(text).monospace().size(13.0));
}

pub fn draw_digit_legend(ui: &mut egui::Ui) {
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

pub fn draw_chapter4_svg(ui: &mut egui::Ui) {
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

pub fn draw_chapter5_svg(ui: &mut egui::Ui) {
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
