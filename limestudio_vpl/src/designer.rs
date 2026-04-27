use eframe::egui;
use crate::style::colors;
use limestudio_core::view::WidgetType;
use limestudio_core::node_discovery::{NodeRegistry};
use dirtydata_core::types::{SignalUnit};

pub struct DesignerState {
    pub registry: std::sync::Arc<NodeRegistry>,
}

impl DesignerState {
    pub fn new(registry: std::sync::Arc<NodeRegistry>) -> Self {
        Self { registry }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui, rect: egui::Rect, layout: &mut limestudio_core::view::DesignerLayout) {
        // Background Grid (The Law)
        let painter = ui.painter();
        let grid_step = 8.0;
        
        // Draw grid lines
        for i in 0..=(rect.width() / grid_step) as i32 {
            let x = rect.left() + i as f32 * grid_step;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], 
                egui::Stroke::new(0.3, egui::Color32::from_hex(colors::GRID).unwrap())
            );
        }

        ui.heading("PROJECTION DESIGNER (Perception Layer)");
        ui.label("Map nodes to physical controls following HIG v3.0");

        // Iterate through persisted widgets
        for ((node_id, param_name), widget) in &mut layout.widgets {
            let mut pos_x = f32::from_bits(widget.position_bits[0]);
            let mut pos_y = f32::from_bits(widget.position_bits[1]);
            
            let pos = egui::pos2(pos_x, pos_y);
            let widget_rect = egui::Rect::from_center_size(pos, egui::vec2(64.0, 64.0));
            
            let id = egui::Id::new(format!("{}_{}", node_id, param_name));
            let response = ui.interact(widget_rect, id, egui::Sense::drag());
            
            if response.dragged() {
                let delta = response.drag_delta();
                pos_x += delta.x;
                pos_y += delta.y;
                // Snap to Grid (Enforcement)
                pos_x = (pos_x / grid_step).round() * grid_step;
                pos_y = (pos_y / grid_step).round() * grid_step;
                
                widget.position_bits[0] = pos_x.to_bits();
                widget.position_bits[1] = pos_y.to_bits();
            }

            // Draw Widget with Semantic Meaning
            match widget.widget_type {
                WidgetType::ModulationRing | WidgetType::Knob => {
                    crate::widgets::modulation_ring(ui, 0.5, 0.0, param_name);
                }
                WidgetType::Meter => {
                    crate::widgets::level_meter(ui, &crate::widgets::MeterState { peak: 0.7, rms: 0.4 });
                }
                _ => {
                    ui.painter().rect_stroke(
                        widget_rect, 
                        2.0, 
                        egui::Stroke::new(1.0, egui::Color32::from_hex(colors::CALM_LIME).unwrap())
                    );
                }
            }
        }
    }

    /// §SBG: Semantic Binding Generator
    /// Create a suggested widget based on parameter unit.
    pub fn suggest_widget(&self, unit: SignalUnit) -> WidgetType {
        match unit {
            SignalUnit::Hertz | SignalUnit::Normalized | SignalUnit::Bipolar => WidgetType::ModulationRing,
            SignalUnit::Decibel => WidgetType::Meter,
            _ => WidgetType::Knob,
        }
    }
}
