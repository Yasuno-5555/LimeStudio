use eframe::egui;
use crate::style::colors;

/// Trust UI Widget: Modulation Ring
/// かわいいノブではなく、情報を見せる。
pub fn modulation_ring(ui: &mut egui::Ui, value: f32, modulation: f32, label: &str) {
    let size = egui::vec2(40.0, 40.0);
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::hover());
    
    let center = rect.center();
    let radius = rect.width() / 2.0 - 2.0;
    
    // Base circle (Surface)
    ui.painter().circle_stroke(
        center,
        radius,
        egui::Stroke::new(2.0, egui::Color32::from_hex(colors::GRID).unwrap())
    );
    
    // Value arc (Calm Lime)
    // Simplified as a line for now, but should be a solid matte arc
    let angle = value * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let end = center + egui::vec2(angle.cos(), angle.sin()) * radius;
    ui.painter().line_segment(
        [center, end],
        egui::Stroke::new(3.0, egui::Color32::from_hex(colors::CALM_LIME).unwrap())
    );
    
    // Modulation range (Muted Amber)
    if modulation.abs() > 0.001 {
        // Show the influence of modulation
        let mod_angle = (value + modulation).clamp(0.0, 1.0) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let mod_end = center + egui::vec2(mod_angle.cos(), mod_angle.sin()) * radius;
        ui.painter().line_segment(
            [end, mod_end],
            egui::Stroke::new(2.0, egui::Color32::from_hex(colors::MUTED_AMBER).unwrap())
        );
    }
    
    ui.painter().text(
        rect.left_bottom() + egui::vec2(0.0, 12.0),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::monospace(11.0),
        egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()
    );
}

/// Trust UI Widget: Latency & Safety Monitor
pub fn safety_monitor(ui: &mut egui::Ui, latency: u32, has_denormals: bool) {
    ui.horizontal(|ui| {
        let latency_color = if latency > 0 { colors::CALM_LIME } else { colors::TEXT_SECONDARY };
        ui.label(egui::RichText::new(format!("LATENCY: {} samples", latency))
            .color(egui::Color32::from_hex(latency_color).unwrap())
            .monospace());
        
        if has_denormals {
            ui.label(egui::RichText::new("⚠ DENORMAL RISK")
                .color(egui::Color32::from_hex(colors::MUTED_AMBER).unwrap())
                .monospace());
        }
    });
}
