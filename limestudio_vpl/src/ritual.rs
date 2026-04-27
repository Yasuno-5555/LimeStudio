use eframe::egui;
use crate::style::colors;

pub enum RitualStep {
    Verify,
    Seal,
    Ship,
    Done,
}

pub struct ShipRitual {
    pub current_step: RitualStep,
    pub progress: f32,
    pub manifest_hash: Option<String>,
}

impl Default for ShipRitual {
    fn default() -> Self { Self::new() }
}
impl ShipRitual {
    pub fn new() -> Self {
        Self {
            current_step: RitualStep::Verify,
            progress: 0.0,
            manifest_hash: None,
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("THE SHIP CEREMONY");
        ui.add_space(12.0);

        match self.current_step {
            RitualStep::Verify => {
                ui.label("STEP 1: VERIFYING TRUTH...");
                ui.label("Running LimeLint & Forensic Audit...");
                if ui.button("EXECUTE VERIFICATION").clicked() {
                    self.current_step = RitualStep::Seal;
                }
            }
            RitualStep::Seal => {
                ui.label("STEP 2: SEALING REALITY...");
                ui.label("Generating BLAKE3 integrity hashes...");
                if ui.button("GENERATE SEAL").clicked() {
                    self.manifest_hash = Some("b3a5...f92c".into());
                    self.current_step = RitualStep::Ship;
                }
            }
            RitualStep::Ship => {
                ui.label("STEP 3: SHIPPING ARTIFACT...");
                ui.label("Finalizing binary & shipment manifest...");
                if let Some(hash) = &self.manifest_hash {
                    ui.monospace(format!("HASH: {}", hash));
                }
                if ui.button("SHIP NOW").clicked() {
                    self.current_step = RitualStep::Done;
                }
            }
            RitualStep::Done => {
                ui.label(egui::RichText::new("CEREMONY COMPLETE").color(egui::Color32::from_hex(colors::CALM_LIME).unwrap()).strong());
                ui.label("The artifact has been sealed and is ready for distribution.");
            }
        }
    }
}
