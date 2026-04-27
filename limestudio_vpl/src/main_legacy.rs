mod style;
mod widgets;
mod overlay;
mod canvas;
mod designer;
mod ritual;

use eframe::egui;
use eframe::egui_wgpu;
use limestudio_core::transaction::{TransactionLayer, Author};
use limestudio_core::pipeline::{PipelineFactory, EngineToUiPipeline};
use limestudio_core::{Intent, ViewCache, UiIndex, topology};
use dirtydata_core::ir::Graph;
use std::sync::Arc;
use std::collections::HashMap;
use glam::vec2;
use crate::style::colors;
use crate::canvas::{CanvasRenderer, CanvasState};
use crate::designer::DesignerState;
use crate::ritual::ShipRitual;
use limestudio_surface::render::sdf::SdfInstance;
use limestudio_surface::scene::camera::InfiniteCamera;
use limestudio_surface::runtime::interaction_kernel::InteractionKernel;
use limestudio_surface::runtime::input::{SurfaceEvent, MouseButton, Modifiers};

#[derive(PartialEq)]
enum AppMode {
    Patching,
    Designer,
    Ritual,
}

struct VplApp {
    transaction: TransactionLayer,
    engine_responses: EngineToUiPipeline,
    selected_node: Option<UiIndex>,
    hovered_node: Option<UiIndex>,
    canvas_renderer: CanvasRenderer,
    camera: InfiniteCamera,
    kernel: InteractionKernel,
    mode: AppMode,
    designer: DesignerState,
    ritual: ShipRitual,
}

impl VplApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // 1. Initialize Reality (Graph) and Perception (ViewCache)
        let graph = Arc::new(Graph::new());
        let view = ViewCache::new();
        
        // 2. Create RT-Safe Pipeline
        let (ui_to_eng, eng_to_ui, _eng_tx, _ui_rx) = PipelineFactory::create_pair(1024);
        
        // 3. Initialize Transaction Layer
        let transaction = TransactionLayer::new(graph, view, ui_to_eng);
        
        Self {
            transaction,
            engine_responses: eng_to_ui,
            selected_node: None,
            hovered_node: None,
            canvas_renderer: CanvasRenderer::new(),
            camera: InfiniteCamera::new(glam::Vec2::new(1024.0, 768.0)), // Placeholder size
            kernel: InteractionKernel::new(),
            mode: AppMode::Patching,
            designer: DesignerState::new(),
            ritual: ShipRitual::new(),
        }
    }
}

impl eframe::App for VplApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        setup_custom_style(ctx);
        
        // 1. Handle Kernel Responses (Reconciliation)
        self.transaction.handle_responses(&mut self.engine_responses);

        // 2. Top Bar
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("LimeStudio");
                ui.separator();
                
                ui.selectable_value(&mut self.mode, AppMode::Patching, "1. PATCHING");
                ui.selectable_value(&mut self.mode, AppMode::Designer, "2. DESIGNER");
                ui.selectable_value(&mut self.mode, AppMode::Ritual, "3. SHIP");
                
                ui.separator();
                if ui.button("Undo").clicked() {
                    let _ = self.transaction.undo();
                }
                if ui.button("Redo").clicked() {
                    let _ = self.transaction.redo();
                }
                
                ui.separator();
                if ui.button("SAVE TOML").clicked() {
                    self.save_project_toml();
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let revision = self.transaction.graph().revision.0;
                    ui.label(format!("REVISION: {}", revision));
                });
            });
        });

        // 3. Components Panel (Side Bar)
        egui::SidePanel::left("nodes").show(ctx, |ui| {
            ui.set_width(200.0);
            ui.heading("Components");
            ui.separator();
            
            for kind in ["Source", "Processor", "Sink", "Circuit"] {
                if ui.button(kind).clicked() {
                    let _ = self.transaction.dispatch_intent(Intent::AddNode {
                        kind: kind.to_string(),
                        position: [100.0, 100.0],
                    }, Author::User);
                }
            }

            ui.separator();
            ui.heading("Active Nodes");
            let view = self.transaction.view_cache().clone();
            for (&kernel_id, &pos) in &view.node_positions {
                if let Some(ui_idx) = view.id_map.get_ui_index(kernel_id) {
                    let label = format!("Node {} ({:?})", ui_idx, kernel_id.0);
                    if ui.selectable_label(self.selected_node == Some(ui_idx), label).clicked() {
                        self.selected_node = Some(ui_idx);
                    }
                }
            }
        });

        // 4. Main Workspace
        egui::CentralPanel::default().show(ctx, |ui| {
            let rect = ui.available_rect_before_wrap();
            
            match self.mode {
                AppMode::Patching => {
                    self.draw_patching_canvas(ui, rect);
                }
                AppMode::Designer => {
                    self.designer.draw(ui, rect);
                }
                AppMode::Ritual => {
                    self.ritual.draw(ui);
                }
            }
        });
    }
}

impl VplApp {
    fn save_project_toml(&self) {
        let mut nodes = HashMap::new();
        let view = self.transaction.view_cache();
        let graph = self.transaction.graph();
        
        for (&kernel_id, &pos) in &view.node_positions {
            if let Some(node) = graph.nodes.get(&kernel_id) {
                let mut params = HashMap::new();
                for (k, v) in &node.config {
                    if let Some(f) = v.as_float() {
                        params.insert(k.clone(), f as f32);
                    }
                }
                nodes.insert(kernel_id.0.to_string(), topology::NodeConfig {
                    kind: node.config.get("name").and_then(|n| n.as_string()).map(|s| s.to_string()).unwrap_or_else(|| "unknown".into()),
                    position: [pos[0], pos[1]],
                    params,
                });
            }
        }
        
        let edges = graph.edges.iter().map(|(_id, e)| topology::EdgeConfig {
            from: e.source.node_id.0.to_string(),
            from_port: e.source.port_name.clone(),
            to: e.target.node_id.0.to_string(),
            to_port: e.target.port_name.clone(),
        }).collect();

        let topology = topology::ProjectTopology {
            name: "Lime Project".into(),
            nodes,
            edges,
        };
        
        let toml_str = topology.to_toml();
        println!("--- PROJECT TOML ---\n{}\n-------------------", toml_str);
    }

    fn draw_patching_canvas(&mut self, ui: &mut egui::Ui, rect: egui::Rect) {
        // 1. Update Camera
        self.camera.viewport_size = glam::Vec2::new(rect.width(), rect.height());

        // 2. Capture Events from egui
        let response = ui.interact(rect, ui.id(), egui::Sense::click_and_drag());
        
        // Forward pointer events to kernel
        if let Some(pointer_pos) = response.hover_pos() {
            let screen_pos = glam::Vec2::new(pointer_pos.x - rect.left(), pointer_pos.y - rect.top());
            
            if response.drag_started() {
                // Forward Down
                let view = self.transaction.view_cache();
                let nodes: Vec<_> = view.node_positions.iter().map(|(&id, &pos)| {
                    (id.into(), limestudio_surface::model::geometry::Rect::new(pos.into(), glam::Vec2::new(120.0, 48.0)))
                }).collect();
                
                self.kernel.handle_event(
                    &SurfaceEvent::PointerDown { 
                        position: screen_pos, 
                        button: MouseButton::Left,
                        modifiers: Modifiers::default(),
                    },
                    &self.camera,
                    &nodes,
                    &[], // TODO: Ports
                );
            } else if response.dragged() {
                // Forward Move
                self.kernel.handle_event(
                    &SurfaceEvent::PointerMove { 
                        position: screen_pos,
                        modifiers: Modifiers::default(),
                    },
                    &self.camera,
                    &[],
                    &[],
                );
            } else if response.drag_released() {
                // Forward Up
                self.kernel.handle_event(
                    &SurfaceEvent::PointerUp { 
                        position: screen_pos, 
                        button: MouseButton::Left,
                        modifiers: Modifiers::default(),
                    },
                    &self.camera,
                    &[],
                    &[],
                );
            }
        }

        // 3. Apply Kernel State to Transaction Layer
        match &self.kernel.session {
            limestudio_surface::runtime::interaction_kernel::DragSession::MovingNode { id, .. } => {
                let world_pos = self.kernel.last_world_pos;
                // Apply 8px Law
                let snapped = limestudio_surface::model::hit_test::HitTester::snap_to_grid(world_pos);
                
                if let Some(ui_idx) = self.transaction.view_cache().id_map.get_ui_index((*id).into()) {
                    let _ = self.transaction.dispatch_intent(limestudio_core::Intent::MoveNode {
                        node_id: ui_idx,
                        position: [snapped.x, snapped.y],
                    }, Author::User);
                }
            }
            _ => {}
        }

        // 4. Background Grid (The Law)
        let painter = ui.painter();
        let grid_step = 8.0 * self.camera.zoom;
        // ... (simplified grid drawing for now)
        
        // 5. Cables
        let view = self.transaction.view_cache().clone();
        let graph = self.transaction.graph();
        for (_id, edge) in &graph.edges {
            if let (Some(&from_pos), Some(&to_pos)) = (view.node_positions.get(&edge.source.node_id), view.node_positions.get(&edge.target.node_id)) {
                let start = egui::pos2(rect.left() + from_pos[0], rect.top() + from_pos[1]); // Needs camera transform
                let end = egui::pos2(rect.left() + to_pos[0], rect.top() + to_pos[1]);
                painter.line_segment([start, end], egui::Stroke::new(1.5, egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
            }
        }

        // 6. Nodes Rendering
        let mut sdf_instances = Vec::new();
        for (&kernel_id, &pos) in &view.node_positions {
            if let Some(ui_idx) = view.id_map.get_ui_index(kernel_id) {
                let is_selected = self.kernel.selection.ids.contains(&kernel_id.into());
                let screen_pos = self.camera.world_to_screen(pos.into());
                let node_rect = egui::Rect::from_center_size(
                    egui::pos2(rect.left() + screen_pos.x, rect.top() + screen_pos.y),
                    egui::vec2(120.0, 48.0) * self.camera.zoom
                );

                // Collect for Trust Canvas
                sdf_instances.push(SdfInstance {
                    position: pos.into(),
                    size: [120.0, 48.0].into(),
                    color: if is_selected { [0.6, 1.0, 0.4, 1.0] } else { [0.8, 0.8, 0.8, 1.0] }.into(),
                    shape_type: 0,
                    modulation_depth: 0.0,
                    modulation_current: 0.0,
                    _pad: 0,
                    params: glam::Vec4::ZERO,
                    params2: glam::Vec4::ZERO,
                });

                // Fallback UI
                let color = if is_selected { colors::CALM_LIME } else { colors::TEXT_PRIMARY };
                ui.painter().rect_stroke(node_rect, 4.0, egui::Stroke::new(2.0, egui::Color32::from_hex(color).unwrap()));
                ui.painter().text(node_rect.center(), egui::Align2::CENTER_CENTER, format!("Node {}", ui_idx), egui::FontId::monospace(14.0 * self.camera.zoom), egui::Color32::from_hex(colors::TEXT_PRIMARY).unwrap());
            }
        }

        // 4. Render Trust Canvas
        self.canvas_renderer.render(ui, rect, sdf_instances);
    }
}

fn setup_custom_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals.panel_fill = egui::Color32::from_hex(colors::MAIN_BG).unwrap();
    style.visuals.window_fill = egui::Color32::from_hex(colors::SURFACE).unwrap();
    style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_hex(colors::SURFACE).unwrap();
    style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, egui::Color32::from_hex(colors::TEXT_PRIMARY).unwrap());
    
    style.visuals.window_shadow = egui::epaint::Shadow::NONE;
    style.visuals.window_rounding = egui::Rounding::same(4.0);
    
    ctx.set_style(style);
}

fn main() -> anyhow::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "LimeStudio VPL",
        options,
        Box::new(|cc| Ok(Box::new(VplApp::new(cc)) as Box<dyn eframe::App>)),
    ).map_err(|e| anyhow::anyhow!("Eframe error: {:?}", e))
}
