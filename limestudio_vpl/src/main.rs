mod style;
mod widgets;

use eframe::egui;
use limestudio_core::graph::{AudioGraph, GraphNode, NodeId};
use limestudio_core::registry::{NodeRegistry};
use limestudio_core::compile::compile_graph;
use limestudio_core::validate::validate_graph;
use limestudio_core::hostile::validate_hostile;
use crate::style::colors;

struct VplApp {
    graph: AudioGraph,
    registry: NodeRegistry,
    engine: Option<limestudio_core::engine::DspEngine>,
    selected_node: Option<NodeId>,
    show_hostile_report: bool,
}

impl Default for VplApp {
    fn default() -> Self {
        Self {
            graph: AudioGraph::new(),
            registry: NodeRegistry::new(),
            engine: None,
            selected_node: None,
            show_hostile_report: false,
        }
    }
}

impl eframe::App for VplApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        setup_custom_style(ctx);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("LimeStudio");
                ui.separator();
                if ui.button("Hostile Validation").clicked() {
                    self.show_hostile_report = !self.show_hostile_report;
                }
                
                // Show Latency in top bar (Trust UI)
                match validate_graph(&self.graph) {
                    Ok(order) => {
                        let program = compile_graph(&self.graph, &order).program;
                        let latency = 0; // TODO: Calculate from IR
                        widgets::safety_monitor(ui, latency, false);
                    }
                    Err(_) => {
                        ui.colored_label(egui::Color32::from_hex(colors::MUTED_AMBER).unwrap(), "GRAPH ERROR");
                    }
                }
            });
        });

        egui::SidePanel::left("nodes").show(ctx, |ui| {
            ui.set_width(200.0);
            ui.heading("Components");
            ui.separator();
            
            for def in &self.registry.definitions {
                if ui.button(&def.name).clicked() {
                    self.graph.add_node(def.template.clone());
                }
            }

            ui.separator();
            ui.heading("Active Nodes");
            for (idx, node) in self.graph.nodes.iter().enumerate() {
                let id = NodeId(idx);
                let response = ui.selectable_label(self.selected_node == Some(id), format!("{:?}", id));
                if response.clicked() {
                    self.selected_node = Some(id);
                }
                
                // Hover Card (Level 2)
                response.on_hover_ui(|ui| {
                    match validate_graph(&self.graph) {
                        Ok(order) => {
                            let result = compile_graph(&self.graph, &order);
                            let info = result.confidence.get(&id);
                            widgets::node_hover_card(ui, node, info);
                        }
                        Err(_) => {
                            widgets::node_hover_card(ui, node, None);
                        }
                    }
                });
            }
        });

        if self.show_hostile_report {
            egui::SidePanel::right("hostile").show(ctx, |ui| {
                ui.set_width(300.0);
                ui.heading("Hostile Report");
                ui.separator();
                
                match validate_graph(&self.graph) {
                    Ok(order) => {
                        let program = compile_graph(&self.graph, &order).program;
                        let report = validate_hostile(&self.graph, &program);
                        for check in &report.checks {
                            let color = match check.severity {
                                limestudio_core::hostile::CheckSeverity::Critical | 
                                limestudio_core::hostile::CheckSeverity::Error => colors::MUTED_AMBER,
                                _ => colors::TEXT_PRIMARY,
                            };
                            ui.colored_label(egui::Color32::from_hex(color).unwrap(), format!("- {}", check.name));
                        }
                    }
                    Err(e) => {
                        ui.label(format!("Structural Error: {:?}", e));
                    }
                }
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Workspace");
            
            // Example Trust Widget
            if let Some(id) = self.selected_node {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Node {:?}", id));
                        
                        // Recompile if needed
                        if self.engine.is_none() {
                            if let Ok(engine) = limestudio_core::engine::DspEngine::new(&self.graph) {
                                self.engine = Some(engine);
                            }
                        }

                        // Confidence Badge (Level 1)
                        match validate_graph(&self.graph) {
                            Ok(order) => {
                                let result = compile_graph(&self.graph, &order);
                                if let Some(info) = result.confidence.get(&id) {
                                    widgets::confidence_badge(ui, info);
                                }
                            }
                            Err(_) => {}
                        }
                    });
                    
                    if let Some(engine) = &mut self.engine {
                        // Simulate audio processing for UI feedback
                        let input = vec![0.0f32; 64];
                        let mut output = vec![0.0f32; 64];
                        engine.selected_node_for_scope = Some(id);
                        engine.process_block(&[&input], &mut [&mut output]);

                        ui.horizontal(|ui| {
                            widgets::modulation_ring(ui, 0.5, 0.1, "CUTOFF");
                            
                            // Level Meter (Real Data)
                            let peak = engine.node_peaks.get(id.0).cloned().unwrap_or(0.0);
                            let rms = engine.node_rms.get(id.0).cloned().unwrap_or(0.0);
                            widgets::level_meter(ui, &widgets::MeterState { peak, rms });
                            
                            // Oscilloscope (Real Data)
                            widgets::oscilloscope(ui, &widgets::ScopeState { 
                                buffer: engine.scope_buffer.clone(), 
                                trigger_index: 0 
                            });
                        });
                    }

                    // Live Scripting UI
                    if let Some(node) = self.graph.nodes.get_mut(id.0) {
                        if let GraphNode::Script { source, .. } = node {
                            ui.add_space(8.0);
                            if widgets::script_editor(ui, source).changed() {
                                self.engine = None; // Force re-compile and engine swap
                            }
                        }
                    }
                });
            }
        });
    }
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    
    // Set colors from the design system
    style.visuals.panel_fill = egui::Color32::from_hex(colors::MAIN_BG).unwrap();
    style.visuals.window_fill = egui::Color32::from_hex(colors::SURFACE).unwrap();
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_hex(colors::SURFACE).unwrap();
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_hex(colors::TEXT_PRIMARY).unwrap());
    
    style.visuals.widgets.inactive.bg_fill = egui::Color32::from_hex(colors::SURFACE).unwrap();
    style.visuals.widgets.hovered.bg_fill = egui::Color32::from_hex(colors::GRID).unwrap();
    style.visuals.widgets.active.bg_fill = egui::Color32::from_hex(colors::CALM_LIME).unwrap();
    
    // Matte & Solid: No shadows, sharp or slightly rounded corners
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.popup_shadow = egui::epaint::Shadow::NONE;
    style.visuals.window_rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.noninteractive.rounding = egui::Rounding::same(4.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(4.0);
    
    ctx.set_style(style);
}

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "LimeStudio VPL",
        options,
        Box::new(|_cc| Ok(Box::new(VplApp::default()))),
    ).map_err(|e| anyhow::anyhow!(format!("{:?}", e)))
}
