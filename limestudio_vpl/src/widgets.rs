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

use limestudio_core::confidence::{ConfidenceInfo, ConfidenceState};

/// Trust UI Widget: Confidence Badge
/// "Shape over Color. Color is auxiliary."
pub fn confidence_badge(ui: &mut egui::Ui, info: &ConfidenceInfo) {
    let size = egui::vec2(12.0, 12.0);
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::hover());
    
    let color = match info.state {
        ConfidenceState::Safe => egui::Color32::from_hex(colors::CALM_LIME).unwrap(),
        ConfidenceState::Warning => egui::Color32::from_hex(colors::MUTED_AMBER).unwrap(),
        ConfidenceState::Dangerous => egui::Color32::from_rgb(200, 50, 50),
        ConfidenceState::Unknown => egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap(),
    };
    
    let painter = ui.painter();
    
    match info.state {
        ConfidenceState::Safe => {
            // Solid square
            painter.rect_filled(rect.shrink(2.0), 0.0, color);
        }
        ConfidenceState::Warning => {
            // Chamfered corner (Approximated by a polygon or rounded rect with small radius)
            let r = rect.shrink(2.0);
            painter.rect_filled(r, 2.0, color);
        }
        ConfidenceState::Dangerous => {
            // Broken edge (Approximated by a crossed box or jagged lines)
            let r = rect.shrink(2.0);
            painter.line_segment([r.left_top(), r.right_bottom()], egui::Stroke::new(2.0, color));
            painter.line_segment([r.right_top(), r.left_bottom()], egui::Stroke::new(2.0, color));
        }
        ConfidenceState::Unknown => {
            // Hollow frame
            painter.rect_stroke(rect.shrink(2.0), 0.0, egui::Stroke::new(1.0, color));
        }
    }

    // Hover Expand (Level 2)
    response.on_hover_ui(|ui| {
        ui.set_max_width(200.0);
        ui.vertical(|ui| {
            ui.heading(format!("Confidence: {}%", info.score));
            ui.separator();
            
            let v = &info.vector;
            ui.monospace(format!("RT Safety:    {}", v.rt_safety));
            ui.monospace(format!("Optimization: {}", v.optimization));
            ui.monospace(format!("Determinism:  {}", v.determinism));
            ui.monospace(format!("Stability:    {}", v.modulation_stability));
            ui.monospace(format!("Latency:      {}", v.latency_risk));
            
            if !info.details.is_empty() {
                ui.separator();
                for detail in &info.details {
                    ui.label(format!("• {}", detail));
                }
            }
        });
    });
}

#[derive(Debug, Clone, Copy)]
pub struct MeterState {
    pub peak: f32,
    pub rms: f32,
}

/// Trust UI Widget: Level Meter
/// "Information Density. High resolution feedback."
pub fn level_meter(ui: &mut egui::Ui, state: &MeterState) {
    let size = egui::vec2(16.0, 60.0);
    let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
    
    let painter = ui.painter();
    let bg_color = egui::Color32::from_hex(colors::SURFACE).unwrap();
    painter.rect_filled(rect, 2.0, bg_color);
    
    // Draw grid lines (-6dB, -12dB, -24dB)
    let db_to_y = |db: f32| {
        let linear = 10.0f32.powf(db / 20.0);
        rect.bottom() - linear * rect.height()
    };
    
    let grid_stroke = egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap());
    for &db in &[-6.0, -12.0, -24.0] {
        let y = db_to_y(db);
        painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], grid_stroke);
    }
    
    // Peak color using Oklab interpolation
    // 0.0 -> Calm Lime, 1.0 -> Muted Amber (at 0dB)
    let color_bytes = crate::style::oklab::mix_oklab(colors::CALM_LIME, colors::MUTED_AMBER, state.peak.clamp(0.0, 1.0));
    let color = egui::Color32::from_rgb(color_bytes[0], color_bytes[1], color_bytes[2]);
    
    // RMS Bar (Matte & Solid)
    let rms_height = (state.rms.clamp(0.0, 1.0) * rect.height()).max(1.0);
    let rms_rect = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 2.0, rect.bottom() - rms_height),
        egui::pos2(rect.right() - 2.0, rect.bottom())
    );
    painter.rect_filled(rms_rect, 1.0, color.gamma_multiply(0.6)); // Muted for RMS
    
    // Peak Line
    let peak_y = rect.bottom() - (state.peak.clamp(0.0, 1.0) * rect.height());
    painter.line_segment(
        [egui::pos2(rect.left() + 1.0, peak_y), egui::pos2(rect.right() - 1.0, peak_y)],
        egui::Stroke::new(2.0, color)
    );
    
    // Clipped indicator
    if state.peak > 1.0 {
        painter.rect_filled(
            egui::Rect::from_center_size(egui::pos2(rect.center().x, rect.top() + 4.0), egui::vec2(8.0, 4.0)),
            1.0,
            egui::Color32::from_hex(colors::MUTED_AMBER).unwrap()
        );
    }
}

/// Trust UI Widget: Hover Card
/// Contextual information for nodes, ports, and cables.
pub fn hover_card(ui: &mut egui::Ui, title: &str, subtitle: &str, content: impl FnOnce(&mut egui::Ui)) {
    ui.vertical(|ui| {
        ui.set_max_width(240.0);
        
        let bg_color = egui::Color32::from_hex(colors::SURFACE_GLASS).unwrap();
        let frame = egui::Frame::none()
            .fill(bg_color)
            .rounding(crate::style::dimen::CORNER_RADIUS)
            .stroke(egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap()))
            .inner_margin(crate::style::dimen::PADDING);
            
        frame.show(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(title).strong().color(egui::Color32::from_hex(colors::TEXT_PRIMARY).unwrap()));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(egui::RichText::new(subtitle).small().color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
                    });
                });
                
                ui.add_space(2.0);
                let separator_stroke = egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap());
                ui.painter().line_segment(
                    [ui.cursor().min, ui.cursor().min + egui::vec2(ui.available_width(), 0.0)],
                    separator_stroke
                );
                ui.add_space(6.0);
                
                content(ui);
            });
        });
    });
}

pub fn node_hover_card(ui: &mut egui::Ui, node: &limestudio_core::graph::GraphNode, info: Option<&ConfidenceInfo>) {
    let title = format!("{:?}", node); // Simplified for now
    let subtitle = "NODE";
    
    hover_card(ui, &title, subtitle, |ui| {
        ui.vertical(|ui| {
            // Inputs
            let in_ports = node.input_ports();
            if !in_ports.is_empty() {
                ui.label(egui::RichText::new("INPUTS").small().color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
                for (idx, port) in in_ports.iter().enumerate() {
                    ui.monospace(format!("  [{}] {}: {}", idx, port.name, port.port_type));
                }
                ui.add_space(4.0);
            }
            
            // Outputs
            let out_ports = node.output_ports();
            if !out_ports.is_empty() {
                ui.label(egui::RichText::new("OUTPUTS").small().color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
                for (idx, port) in out_ports.iter().enumerate() {
                    ui.monospace(format!("  [{}] {}: {}", idx, port.name, port.port_type));
                }
            }
            
            if let Some(conf) = info {
                ui.add_space(4.0);
                ui.label(egui::RichText::new("COMPILER TRUST").small().color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
                ui.monospace(format!("  Confidence: {}%", conf.score));
            }

            if let limestudio_core::graph::GraphNode::Container { inner_graph, .. } = node {
                ui.add_space(8.0);
                ui.label(egui::RichText::new("STRUCTURE PREVIEW").small().color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
                container_preview(ui, inner_graph);
                
                ui.add_space(4.0);
                if ui.button("Enter Subgraph").clicked() {
                    // Logic to change the current view to the inner graph
                }
            }
        });
    });
}

/// Trust UI Widget: Container Preview
/// "Ghostly representation of internal logic."
pub fn container_preview(ui: &mut egui::Ui, inner_graph: &limestudio_core::graph::AudioGraph) {
    let size = egui::vec2(100.0, 60.0);
    let (rect, _) = ui.allocate_at_least(size, egui::Sense::hover());
    let painter = ui.painter();
    
    painter.rect_filled(rect, 4.0, egui::Color32::from_hex(colors::GRID).unwrap().gamma_multiply(0.3));
    
    let count = inner_graph.nodes.len();
    if count == 0 { return; }
    
    // Simple layout for preview
    for i in 0..count {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let pos = rect.center() + egui::vec2(angle.cos(), angle.sin()) * 18.0;
        
        // Connect to center for visualization
        painter.line_segment([rect.center(), pos], egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap()));
        painter.circle_filled(pos, 3.0, egui::Color32::from_hex(colors::CALM_LIME).unwrap().gamma_multiply(0.7));
    }
}

#[derive(Debug, Clone)]
pub struct CurvePoint {
    pub pos: egui::Pos2, // Normalized 0.0 to 1.0
    pub handle_in: egui::Vec2,
    pub handle_out: egui::Vec2,
}

pub struct CurveState {
    pub points: Vec<CurvePoint>,
}

/// Trust UI Widget: Curve Editor
/// Bezier-based automation envelope editor.
pub fn curve_editor(ui: &mut egui::Ui, state: &mut CurveState) {
    let size = ui.available_width().at_least(200.0) * egui::vec2(1.0, 0.6);
    let (rect, response) = ui.allocate_at_least(size, egui::Sense::click_and_drag());
    
    let painter = ui.painter();
    let bg_color = egui::Color32::from_hex(colors::SURFACE).unwrap();
    let grid_color = egui::Color32::from_hex(colors::GRID).unwrap();
    
    painter.rect_filled(rect, 4.0, bg_color);
    
    // Draw Grid
    for i in 1..4 {
        let x = rect.left() + rect.width() * (i as f32 / 4.0);
        let y = rect.top() + rect.height() * (i as f32 / 4.0);
        painter.line_segment([egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())], egui::Stroke::new(1.0, grid_color));
        painter.line_segment([egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)], egui::Stroke::new(1.0, grid_color));
    }
    
    // Convert normalized to screen
    let to_screen = |p: egui::Pos2| {
        egui::pos2(
            rect.left() + p.x * rect.width(),
            rect.bottom() - p.y * rect.height()
        )
    };
    
    // Draw Curve
    if state.points.len() >= 2 {
        for i in 0..state.points.len() - 1 {
            let p1 = &state.points[i];
            let p2 = &state.points[i+1];
            
            let s1 = to_screen(p1.pos);
            let s2 = to_screen(p2.pos);
            let c1 = s1 + p1.handle_out * egui::vec2(rect.width(), -rect.height());
            let c2 = s2 + p2.handle_in * egui::vec2(rect.width(), -rect.height());
            
            let stroke = egui::Stroke::new(2.0, egui::Color32::from_hex(colors::CALM_LIME).unwrap());
            let curve = egui::epaint::CubicBezierShape::from_points_stroke(
                [s1, c1, c2, s2],
                false,
                egui::Color32::TRANSPARENT,
                stroke
            );
            painter.add(curve);
        }
    }
    
    // Draw Points
    for point in &state.points {
        let s = to_screen(point.pos);
        painter.circle_filled(s, 3.0, egui::Color32::from_hex(colors::TEXT_PRIMARY).unwrap());
    }
    
    // Interactive part: Simple point dragging (simplified)
    if response.dragged() {
        // Implementation of point dragging would go here
    }
}

/// Trust UI Widget: Mini-map
/// Overview of the entire node graph.
pub fn mini_map(ui: &mut egui::Ui, scene: &limestudio_surface::scene::SurfaceScene) {
    let size = egui::vec2(120.0, 100.0);
    let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
    
    let painter = ui.painter();
    painter.rect_filled(rect, 2.0, egui::Color32::from_hex(colors::SURFACE).unwrap());
    painter.rect_stroke(rect, 2.0, egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap()));
    
    // Simple bounding box of all nodes
    let mut min = egui::pos2(f32::INFINITY, f32::INFINITY);
    let mut max = egui::pos2(f32::NEG_INFINITY, f32::NEG_INFINITY);
    
    if scene.nodes.is_empty() { return; }
    
    for node in &scene.nodes {
        min.x = min.x.min(node.position.x);
        min.y = min.y.min(node.position.y);
        max.x = max.x.max(node.position.x);
        max.y = max.y.max(node.position.y);
    }
    
    let padding = 20.0;
    let world_rect = egui::Rect::from_min_max(min, max).expand(padding);
    
    let to_mini = |p: glam::Vec2| {
        let x = rect.left() + (p.x - world_rect.left()) / world_rect.width() * rect.width();
        let y = rect.top() + (p.y - world_rect.top()) / world_rect.height() * rect.height();
        egui::pos2(x, y)
    };
    
    // Draw Edges
    for edge in &scene.edges {
        if let (Some(from), Some(to)) = (scene.nodes.get(edge.from_node as usize), scene.nodes.get(edge.to_node as usize)) {
            let s = to_mini(from.position);
            let e = to_mini(to.position);
            painter.line_segment([s, e], egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap()));
        }
    }
    
    // Draw Nodes
    for node in &scene.nodes {
        let p = to_mini(node.position);
        painter.circle_filled(p, 2.0, egui::Color32::from_hex(colors::CALM_LIME).unwrap());
    }
}

pub struct ScopeState {
    pub buffer: Vec<f32>,
    pub trigger_index: usize,
}

/// Trust UI Widget: Oscilloscope
/// "Infinite smoothness. Signal reality."
pub fn oscilloscope(ui: &mut egui::Ui, state: &ScopeState) {
    let size = ui.available_width().at_least(200.0) * egui::vec2(1.0, 0.5);
    let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
    
    let painter = ui.painter();
    painter.rect_filled(rect, 4.0, egui::Color32::from_hex(colors::SURFACE).unwrap());
    
    // Draw Center Line
    let center_y = rect.center().y;
    painter.line_segment(
        [egui::pos2(rect.left(), center_y), egui::pos2(rect.right(), center_y)],
        egui::Stroke::new(1.0, egui::Color32::from_hex(colors::GRID).unwrap())
    );
    
    if state.buffer.is_empty() { return; }
    
    // High-quality path for the waveform
    let mut points = Vec::new();
    let step = rect.width() / (state.buffer.len() as f32).max(1.0);
    
    for (i, &sample) in state.buffer.iter().enumerate() {
        let x = rect.left() + i as f32 * step;
        let y = center_y - sample * (rect.height() * 0.45);
        points.push(egui::pos2(x, y));
    }
    
    // Draw the waveform as a smooth path
    let stroke = egui::Stroke::new(1.5, egui::Color32::from_hex(colors::CALM_LIME).unwrap());
    painter.add(egui::Shape::line(points, stroke));
    
    // Gradient fill under the curve (Subtle)
    // In egui, we can do this with a Mesh or several triangles, 
    // but for "Matte & Solid" philosophy, we keep it simple or use a very subtle alpha.
}

/// Trust UI Widget: Script Editor
/// Live coding editor for Script nodes.
pub fn script_editor(ui: &mut egui::Ui, source: &mut String) -> egui::Response {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new("SCRIPT SOURCE").small().color(egui::Color32::from_hex(colors::TEXT_SECONDARY).unwrap()));
        
        let mut style = (*ui.style()).clone();
        style.visuals.widgets.inactive.bg_fill = egui::Color32::from_hex(colors::MAIN_BG).unwrap();
        
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Min), |ui| {
            ui.set_style(style);
            egui::TextEdit::multiline(source)
                .font(egui::FontId::monospace(12.0))
                .code_editor()
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                .show(ui)
                .response
        }).inner
    }).inner
}
