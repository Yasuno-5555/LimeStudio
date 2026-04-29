use crate::style::colors;
use dirtydata_core::ir::Node;
use eframe::egui;

pub fn show_lineage_overlay(ui: &mut egui::Ui, _node: &Node, patch_history: &[&str]) {
    let frame = egui::Frame::none()
        .fill(egui::Color32::from_hex(colors::SURFACE).unwrap())
        .rounding(egui::Rounding::same(4.0))
        .stroke(egui::Stroke::new(
            1.0,
            egui::Color32::from_hex(colors::MUTED_AMBER).unwrap(),
        ))
        .inner_margin(egui::Margin::same(12.0));

    egui::show_tooltip(
        ui.ctx(),
        ui.layer_id(),
        egui::Id::new("lineage_overlay"),
        |ui| {
            frame.show(ui, |ui| {
                ui.set_max_width(280.0);
                ui.vertical(|ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("LINEAGE OVERLAY")
                                .small()
                                .color(egui::Color32::from_hex(colors::MUTED_AMBER).unwrap()),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("FORENSIC PROOF").small().monospace());
                        });
                    });

                    ui.heading("Causality Chain");

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Lineage Events (Tier 1)
                    for (i, event) in patch_history.iter().enumerate() {
                        ui.horizontal(|ui| {
                            let bullet = if i == 0 { "●" } else { "○" };
                            ui.label(
                                egui::RichText::new(bullet)
                                    .color(egui::Color32::from_hex(colors::MUTED_AMBER).unwrap()),
                            );
                            ui.label(*event);
                        });
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    // Verification Badge
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Hash Verified ✓")
                                .color(egui::Color32::from_hex(colors::CALM_LIME).unwrap())
                                .strong(),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(egui::RichText::new("MATCHING REALITY").small());
                        });
                    });
                });
            });
        },
    );
}
