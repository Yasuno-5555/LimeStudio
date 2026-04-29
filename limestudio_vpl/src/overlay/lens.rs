use crate::style::colors;
use dirtydata_core::ir::Node;
use eframe::egui;

pub fn show_node_lens(ui: &mut egui::Ui, node: &Node, ui_index: u64, generated_code: &str) {
    let frame = egui::Frame::none()
        .fill(egui::Color32::from_hex(colors::SURFACE).unwrap())
        .rounding(egui::Rounding::same(4.0))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_hex(colors::CALM_LIME).unwrap(),
        ))
        .inner_margin(egui::Margin::same(12.0));

    egui::show_tooltip(ui.ctx(), ui.layer_id(), egui::Id::new("node_lens"), |ui| {
        frame.show(ui, |ui| {
            ui.set_max_width(320.0);
            ui.vertical(|ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("VISIBLE LENS")
                            .small()
                            .color(egui::Color32::from_hex(colors::CALM_LIME).unwrap()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("UI_IDX: {}", ui_index))
                                .small()
                                .monospace(),
                        );
                    });
                });

                let name = node
                    .config
                    .get("name")
                    .and_then(|v| v.as_string())
                    .cloned()
                    .unwrap_or_else(|| "Unknown Node".to_string());
                ui.heading(&*name);
                ui.label(
                    egui::RichText::new(format!("STABLE_ID: {}", node.id.0))
                        .small()
                        .monospace(),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Generated Rust (Tier 0)
                ui.label(
                    egui::RichText::new("GENERATED RUST")
                        .small()
                        .color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()),
                );
                ui.add_space(4.0);
                egui::Frame::none()
                    .fill(egui::Color32::from_black_alpha(100))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.monospace(generated_code);
                    });

                ui.add_space(8.0);

                // Graph IR (Tier 0)
                ui.label(
                    egui::RichText::new("GRAPH IR")
                        .small()
                        .color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()),
                );
                for (key, val) in &node.config {
                    if key != "name" {
                        ui.monospace(format!("  {} = {:?}", key, val));
                    }
                }

                ui.add_space(8.0);

                // Source Hash
                let hash = node.id.0.to_string(); // Placeholder for actual BLAKE3 hash
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("SOURCE HASH")
                            .small()
                            .color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.monospace(&hash[..8]);
                    });
                });
            });
        });
    });
}
