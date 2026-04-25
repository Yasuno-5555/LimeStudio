use eframe::egui;
use limestudio_core::graph::{AudioGraph, GraphNode, NodeId};
use limestudio_core::registry::{NodeRegistry, NodeCategory};
use limestudio_core::compile::compile_graph;
use limestudio_core::validate::validate_graph;

struct VplApp {
    graph: AudioGraph,
    registry: NodeRegistry,
    selected_node: Option<NodeId>,
    error_message: Option<String>,
}

impl Default for VplApp {
    fn default() -> Self {
        Self {
            graph: AudioGraph::new(),
            registry: NodeRegistry::new(),
            selected_node: None,
            error_message: None,
        }
    }
}

impl eframe::App for VplApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::left("nodes").show(ctx, |ui| {
            ui.heading("Nodes");
            ui.menu_button("Add Node...", |ui| {
                for def in &self.registry.definitions {
                    if ui.button(&def.name).clicked() {
                        let id = self.graph.add_node(def.template.clone());
                        println!("Added node: {} with id {:?}", def.name, id);
                        ui.close_menu();
                    }
                }
            });

            ui.separator();
            
            for (idx, node) in self.graph.nodes.iter().enumerate() {
                let id = NodeId(idx);
                let label = format!("{:?}: {:?}", id, node);
                if ui.selectable_label(self.selected_node == Some(id), label).clicked() {
                    self.selected_node = Some(id);
                }
            }
        });

        egui::SidePanel::right("ir").show(ctx, |ui| {
            ui.heading("Compiled IR Inspector");
            ui.separator();
            
            match validate_graph(&self.graph) {
                Ok(order) => {
                    let program = compile_graph(&self.graph, &order);
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for op in &program.ops {
                            ui.label(format!("{:?}", op));
                        }
                    });
                }
                Err(e) => {
                    ui.colored_label(egui::Color32::RED, format!("Validation Error: {:?}", e));
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("LimeStudio VPL - Ugly and True");
            ui.label("Direct AST Editor (No lines yet, coming in Step 3)");
            
            if let Some(id) = self.selected_node {
                ui.separator();
                ui.heading(format!("Node {:?}", id));
                
                let node = &mut self.graph.nodes[id.0];
                match node {
                    GraphNode::Script { source, .. } => {
                        ui.label("Rhai Script Source:");
                        ui.text_edit_multiline(source);
                    }
                    _ => {
                        ui.label("Static parameters editing not implemented yet.");
                    }
                }

                if ui.button("Remove (Disabled)").clicked() {
                }
            }
        });
    }
}

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "LimeStudio VPL",
        options,
        Box::new(|_cc| Ok(Box::new(VplApp::default()))),
    ).map_err(|e| anyhow::anyhow!(format!("{:?}", e)))
}
