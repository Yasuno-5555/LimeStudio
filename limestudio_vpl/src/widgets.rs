use crate::style::colors;
use dirtydata_core::ir::Node;
use dirtydata_core::types::ConfidenceScore;
use eframe::egui;

/// Trust UI Widget: Modulation Ring
pub fn modulation_ring(ui: &mut egui::Ui, value: f32, modulation: f32, label: &str) {
    let size = egui::vec2(40.0, 40.0);
    let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());

    let center = rect.center();
    let radius = rect.width() / 2.0 - 2.0;

    ui.painter().circle_stroke(
        center,
        radius,
        egui::Stroke::new(2.0, egui::Color32::from_hex(colors::GRID).unwrap()),
    );

    let angle = value * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let end = center + egui::vec2(angle.cos(), angle.sin()) * radius;
    ui.painter().line_segment(
        [center, end],
        egui::Stroke::new(3.0, egui::Color32::from_hex(colors::CALM_LIME).unwrap()),
    );

    if modulation.abs() > 0.001 {
        let mod_angle = (value + modulation).clamp(0.0, 1.0) * std::f32::consts::TAU
            - std::f32::consts::FRAC_PI_2;
        let mod_end = center + egui::vec2(mod_angle.cos(), mod_angle.sin()) * radius;
        ui.painter().line_segment(
            [end, mod_end],
            egui::Stroke::new(2.0, egui::Color32::from_hex(colors::MUTED_AMBER).unwrap()),
        );
    }

    ui.painter().text(
        rect.left_bottom() + egui::vec2(0.0, 12.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::monospace(11.0),
        egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap(),
    );
}

/// Trust UI Widget: Latency & Safety Monitor
pub fn safety_monitor(ui: &mut egui::Ui, latency: u32, has_denormals: bool) {
    ui.horizontal(|ui| {
        let latency_color = if latency > 0 {
            colors::CALM_LIME
        } else {
            colors::TEXT_SECONDARY
        };
        ui.label(
            egui::RichText::new(format!("LATENCY: {} samples", latency))
                .color(egui::Color32::from_hex(latency_color).unwrap())
                .monospace(),
        );

        if has_denormals {
            ui.label(
                egui::RichText::new("⚠ DENORMAL RISK")
                    .color(egui::Color32::from_hex(colors::MUTED_AMBER).unwrap())
                    .monospace(),
            );
        }
    });
}

/// Trust UI Widget: Confidence Badge
pub fn confidence_badge(ui: &mut egui::Ui, score: ConfidenceScore) {
    let size = egui::vec2(12.0, 12.0);
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::hover());

    let color = match score {
        ConfidenceScore::Verified => egui::Color32::from_hex(colors::CALM_LIME).unwrap(),
        ConfidenceScore::Inferred => egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap(),
        ConfidenceScore::Suspicious => egui::Color32::from_hex(colors::MUTED_AMBER).unwrap(),
        ConfidenceScore::Unknown => egui::Color32::from_rgb(150, 150, 150),
    };

    let painter = ui.painter();
    painter.rect_filled(rect.shrink(2.0), 0.0, color);

    response.on_hover_ui(|ui| {
        ui.heading(format!("Confidence Score: {:?}", score));
    });
}

#[derive(Debug, Clone, Copy)]
pub struct MeterState {
    pub peak: f32,
    pub rms: f32,
}

pub fn level_meter(ui: &mut egui::Ui, state: &MeterState) {
    let size = egui::vec2(16.0, 60.0);
    let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());

    let painter = ui.painter();
    let bg_color = egui::Color32::from_hex(colors::SURFACE).unwrap();
    painter.rect_filled(rect, 2.0, bg_color);

    let color_bytes = crate::style::oklab::mix_oklab(
        colors::CALM_LIME,
        colors::MUTED_AMBER,
        state.peak.clamp(0.0, 1.0),
    );
    let color = egui::Color32::from_rgb(color_bytes[0], color_bytes[1], color_bytes[2]);

    let rms_height = (state.rms.clamp(0.0, 1.0) * rect.height()).max(1.0);
    let rms_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 2.0, rect.bottom() - rms_height),
        egui::pos2(rect.right() - 2.0, rect.bottom()),
    );
    painter.rect_filled(rms_rect, 1.0, color.gamma_multiply(0.6));

    let peak_y = rect.bottom() - (state.peak.clamp(0.0, 1.0) * rect.height());
    painter.line_segment(
        [
            egui::pos2(rect.left() + 1.0, peak_y),
            egui::pos2(rect.right() - 1.0, peak_y),
        ],
        egui::Stroke::new(2.0, color),
    );
}

pub fn node_hover_card(ui: &mut egui::Ui, node: &Node) {
    let title = node
        .config
        .get("name")
        .and_then(|v| v.as_string())
        .map(|s| s.as_str())
        .unwrap_or("Unknown Node");
    let subtitle = "NODE (REALITY)";

    hover_card(ui, title, subtitle, |ui| {
        ui.vertical(|ui| {
            ui.label(
                egui::RichText::new("PORTS")
                    .small()
                    .color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()),
            );
            for port in &node.ports {
                ui.monospace(format!("  {} ({:?})", port.name, port.direction));
            }
        });
    });
}

pub fn hover_card(
    ui: &mut egui::Ui,
    title: &str,
    subtitle: &str,
    content: impl FnOnce(&mut egui::Ui),
) {
    ui.vertical(|ui| {
        ui.set_max_width(240.0);
        let frame = egui::Frame::none()
            .fill(egui::Color32::from_hex(colors::SURFACE).unwrap())
            .rounding(egui::Rounding::same(4.0))
            .stroke(egui::Stroke::new(
                1.0,
                egui::Color32::from_hex(colors::GRID).unwrap(),
            ))
            .inner_margin(egui::Margin::same(8.0));

        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(title).strong());
                ui.label(egui::RichText::new(subtitle).small());
                ui.add_space(4.0);
                content(ui);
            });
        });
    });
}
