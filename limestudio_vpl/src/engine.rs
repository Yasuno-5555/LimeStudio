use dirtydata_core::types::{DataType, PortDirection};
use glam::Vec2;
use limestudio_core::pipeline::EngineToUiPipeline;
use limestudio_core::transaction::TransactionLayer;
use limestudio_core::view::layout::NodeLayout;
use limestudio_core::UiIndex;
use limestudio_surface::model::geometry::{Circle, Rect};
use limestudio_surface::runtime::input::SurfaceEvent;
use limestudio_surface::runtime::interaction_kernel::InteractionKernel;
use limestudio_surface::scene::camera::InfiniteCamera;
use limestudio_surface::ui_ir::*;
use std::sync::Arc;

use limestudio_core::node_discovery::{NodeRegistry, NodeTier};

#[derive(PartialEq, Clone, Copy)]
pub enum AppMode {
    Patching,
    Designer,
    Ritual,
}

pub struct VplEngine {
    pub transaction: TransactionLayer,
    pub engine_responses: EngineToUiPipeline,
    pub selected_node: Option<UiIndex>,
    pub camera: InfiniteCamera,
    pub kernel: InteractionKernel,
    pub shared_state: Option<Arc<dirtydata_runtime::SharedState>>,
    pub mode: AppMode,
    pub registry: Arc<NodeRegistry>,
    pub designer: crate::designer::DesignerState,
    pub causality_monitor: limestudio_core::causality::CausalityMonitor,
    pub node_telemetry: std::collections::HashMap<SurfaceId, f32>,
}

impl VplEngine {
    pub fn new(transaction: TransactionLayer, engine_responses: EngineToUiPipeline) -> Self {
        let registry = Arc::new(NodeRegistry::new());
        Self {
            transaction,
            engine_responses,
            selected_node: None,
            camera: InfiniteCamera::new(Vec2::new(1024.0, 768.0)),
            kernel: InteractionKernel::new(),
            shared_state: None,
            mode: AppMode::Patching,
            registry: registry.clone(),
            designer: crate::designer::DesignerState::new(registry),
            causality_monitor: limestudio_core::causality::CausalityMonitor::new(),
            node_telemetry: std::collections::HashMap::new(),
        }
    }

    pub fn set_shared_state(&mut self, state: Arc<dirtydata_runtime::SharedState>) {
        self.shared_state = Some(state);
    }

    fn wrap_with_focus_and_a11y(
        &self,
        widget: SurfaceWidget,
        label: &str,
        role: SemanticRole,
    ) -> SurfaceWidget {
        let id = widget
            .id()
            .cloned()
            .unwrap_or_else(|| SurfaceId::generate());
        let is_focused = self.kernel.focused_id == Some(id);

        let focused_widget = SurfaceWidget::FocusProxy {
            id,
            child: Box::new(widget),
            is_focused,
        };

        SurfaceWidget::Accessibility {
            data: SurfaceAccessibilityData {
                label: label.to_string(),
                role,
                description: None,
                hint: None,
            },
            child: Box::new(focused_widget),
        }
    }

    pub fn build_ui(&mut self) -> SurfaceWidget {
        // 1. Handle Kernel Responses
        self.transaction
            .handle_responses(&mut self.engine_responses);
        while let Some(res) = self.engine_responses.try_recv() {
            match res {
                limestudio_core::pipeline::EngineResponse::Telemetry {
                    cpu_load: _,
                    causality_events,
                    node_cpu,
                } => {
                    for ev in causality_events {
                        self.causality_monitor.poly_traces.push(ev);
                        if self.causality_monitor.poly_traces.len() > 1024 {
                            self.causality_monitor.poly_traces.remove(0);
                        }
                    }
                    for (id, load) in node_cpu {
                        self.node_telemetry
                            .insert(SurfaceId::from_seed(&id.to_string()), load);
                    }
                }
                _ => {}
            }
        }

        // 2. Build UI Tree
        let top_bar = SurfaceWidget::Box {
            style: FrameStyle::Standard,
            children: vec![SurfaceWidget::Row {
                children: vec![
                    SurfaceWidget::Label {
                        text: "LimeStudio".to_string(),
                        is_secondary: false,
                    },
                    self.wrap_with_focus_and_a11y(
                        SurfaceWidget::Button {
                            id: SurfaceId::from_seed("top_bar_patching"),
                            label: "PATCHING".to_string(),
                            is_active: self.mode == AppMode::Patching,
                        },
                        "Switch to Patching Mode",
                        SemanticRole::Button,
                    ),
                    self.wrap_with_focus_and_a11y(
                        SurfaceWidget::Button {
                            id: SurfaceId::from_seed("top_bar_designer"),
                            label: "DESIGNER".to_string(),
                            is_active: self.mode == AppMode::Designer,
                        },
                        "Switch to Designer Mode",
                        SemanticRole::Button,
                    ),
                    self.wrap_with_focus_and_a11y(
                        SurfaceWidget::Button {
                            id: SurfaceId::from_seed("top_bar_ship"),
                            label: "SHIP".to_string(),
                            is_active: self.mode == AppMode::Ritual,
                        },
                        "Switch to Ship Mode",
                        SemanticRole::Button,
                    ),
                ],
            }],
        };

        // --- Tiered & Categorized Sidebar ---
        let mut side_bar_children = vec![SurfaceWidget::Label {
            text: "COMPONENTS".to_string(),
            is_secondary: false,
        }];

        // Group by Category
        let mut categories = std::collections::BTreeSet::new();
        for b in self.registry.blueprints.values() {
            categories.insert(&b.category);
        }

        for cat in categories {
            side_bar_children.push(SurfaceWidget::Label {
                text: format!("> {}", cat.to_uppercase()),
                is_secondary: true,
            });
            for b in self.registry.find_by_category(cat) {
                let tier_label = match b.tier {
                    NodeTier::Core => " [S]",
                    NodeTier::Advanced => " [A]",
                    NodeTier::Experimental => " [B]",
                    NodeTier::Forbidden => " [!]",
                };
                side_bar_children.push(self.wrap_with_focus_and_a11y(
                    SurfaceWidget::Button {
                        id: SurfaceId::from_seed(&format!("sidebar_comp_{}", b.id_name)),
                        label: format!("{}{}", b.display_name, tier_label),
                        is_active: false,
                    },
                    &format!("Create {} component", b.display_name),
                    SemanticRole::Button,
                ));
            }
        }

        let side_bar = SurfaceWidget::Box {
            style: FrameStyle::Standard,
            children: side_bar_children,
        };

        let central_panel = SurfaceWidget::Accessibility {
            data: SurfaceAccessibilityData {
                label: "Node Graph Canvas".to_string(),
                role: SemanticRole::Canvas,
                description: None,
                hint: None,
            },
            child: Box::new(self.build_canvas()),
        };

        let inspector = self.build_inspector();

        SurfaceWidget::Row {
            children: vec![
                side_bar,
                SurfaceWidget::Column {
                    children: vec![top_bar, central_panel],
                },
                inspector,
            ],
        }
    }

    fn build_inspector(&self) -> SurfaceWidget {
        let Some(selected) = self.selected_node else {
            return SurfaceWidget::Box {
                style: FrameStyle::Standard,
                children: vec![SurfaceWidget::Label {
                    text: "INSPECTOR: Select a node".to_string(),
                    is_secondary: true,
                }],
            };
        };

        let kernel_id = self.transaction.view_cache().id_map.resolve(selected);
        let Some(kid) = kernel_id else {
            return SurfaceWidget::Box {
                style: FrameStyle::Standard,
                children: vec![SurfaceWidget::Label {
                    text: "Error: Could not resolve node ID".to_string(),
                    is_secondary: true,
                }],
            };
        };

        let lineage = self.transaction.get_node_lineage(kid);

        let mut history_items = Vec::new();
        let load = self
            .node_telemetry
            .get(&SurfaceId::from_seed(&kid.to_string()))
            .unwrap_or(&0.0);
        history_items.push(SurfaceWidget::Accessibility {
            data: SurfaceAccessibilityData {
                label: format!("Node Telemetry: {:.2}% CPU", load * 100.0),
                role: SemanticRole::Status,
                description: None,
                hint: None,
            },
            child: Box::new(SurfaceWidget::Column {
                children: vec![
                    SurfaceWidget::Label {
                        text: format!("IDENTITY: {}", kid),
                        is_secondary: false,
                    },
                    SurfaceWidget::Label {
                        text: format!("CPU LOAD: {:.2}%", load * 100.0),
                        is_secondary: false,
                    },
                ],
            }),
        });
        history_items.push(SurfaceWidget::Label {
            text: "HISTORY:".to_string(),
            is_secondary: true,
        });

        for meta in lineage {
            history_items.push(SurfaceWidget::Label {
                text: format!("[{}] {}: {:?}", meta.timestamp, meta.author, meta.intent),
                is_secondary: true,
            });
        }

        SurfaceWidget::Box {
            style: FrameStyle::Standard,
            children: history_items,
        }
    }

    fn build_canvas(&self) -> SurfaceWidget {
        let project = self.transaction.project();
        let graph = &project.graph;
        let mut primitives = Vec::new();

        // 1. Cables
        for edge in graph.topology.edges.values() {
            if let (Some(from_node), Some(to_node)) = (
                graph.topology.nodes.get(&edge.source.node_id),
                graph.topology.nodes.get(&edge.target.node_id),
            ) {
                let from_pos_opt = project.view.node_positions.get(&edge.source.node_id);
                let to_pos_opt = project.view.node_positions.get(&edge.target.node_id);

                if let (Some(&f_pos), Some(&t_pos)) = (from_pos_opt, to_pos_opt) {
                    let from_port_idx = from_node
                        .ports
                        .iter()
                        .position(|p| p.name == edge.source.port_name);
                    let to_port_idx = to_node
                        .ports
                        .iter()
                        .position(|p| p.name == edge.target.port_name);

                    if let (Some(f_idx), Some(t_idx)) = (from_port_idx, to_port_idx) {
                        let fp = from_node.ports.get(f_idx).unwrap();
                        let tp = to_node.ports.get(t_idx).unwrap();

                        let is_illegal = match (&fp.data_type, &tp.data_type) {
                            (DataType::Audio { channels: s }, DataType::Audio { channels: d }) => {
                                s != d
                            }
                            (DataType::Control, DataType::Control) => false,
                            (DataType::Midi, DataType::Midi) => false,
                            (DataType::Spectral { bins: s }, DataType::Spectral { bins: d }) => {
                                s != d
                            }
                            _ => true,
                        };

                        let rect_size_f =
                            NodeLayout::calculate_node_rect("node", from_node.ports.len(), 0);
                        let rect_size_t =
                            NodeLayout::calculate_node_rect("node", to_node.ports.len(), 0);

                        let from_port_pos = NodeLayout::get_port_position(
                            f_pos,
                            [f_pos[0], f_pos[1], rect_size_f[2], rect_size_f[3]],
                            f_idx,
                            from_node.ports.len(),
                            false,
                        );
                        let to_port_pos = NodeLayout::get_port_position(
                            t_pos,
                            [t_pos[0], t_pos[1], rect_size_t[2], rect_size_t[3]],
                            t_idx,
                            to_node.ports.len(),
                            true,
                        );

                        primitives.push(SurfacePrimitive::Curve {
                            id: SurfaceId::from_seed(&format!("cable_{}", edge.id)),
                            control_points: vec![from_port_pos, to_port_pos],
                            kind: CurveKind::Cable,
                            thickness: if is_illegal { 4.0 } else { 2.0 },
                            color: if is_illegal {
                                [1.0, 0.2, 0.2, 1.0]
                            } else {
                                [0.5, 0.5, 0.5, 1.0]
                            },
                            temporal: TemporalStrategy::Standard,
                        });
                    }
                }
            }
        }

        // 2. Nodes
        for node in project.ui.nodes.values() {
            let Some(ui) = &node.ui else { continue };
            let pos = [
                NodeLayout::snap(ui.position[0]),
                NodeLayout::snap(ui.position[1]),
            ];
            let surface_id = SurfaceId::from_seed(&node.id.to_string());
            let is_selected = self.kernel.selection.ids.contains(&surface_id);

            let cpu_load = *self.node_telemetry.get(&surface_id).unwrap_or(&0.0);
            let is_hot = cpu_load > 0.15; // 15%
            let border_color = if is_hot {
                [1.0, 0.2, 0.2, 1.0]
            } else if is_selected {
                [0.6, 1.0, 0.4, 1.0]
            } else {
                ui.color
            };

            // Resolve Kernel Node for layout info
            let kernel_id = project.view.id_map.resolve_uuid(node.id);
            let (node_name, num_ports, ports) = if let Some(kid) = kernel_id {
                if let Some(knode) = project.graph.topology.nodes.get(&kid) {
                    (
                        knode
                            .config
                            .get("name")
                            .and_then(|v| v.as_string())
                            .map(|s| s.clone())
                            .unwrap_or_else(|| "node".to_string()),
                        knode.ports.len(),
                        Some(&knode.ports),
                    )
                } else {
                    ("node".to_string(), 0, None)
                }
            } else {
                ("node".to_string(), 0, None)
            };

            let rect_size = NodeLayout::calculate_node_rect(&node_name, num_ports, 0);
            let node_rect = [pos[0], pos[1], rect_size[2], rect_size[3]];

            primitives.push(SurfacePrimitive::Frame {
                id: SurfaceId::from_seed(&node.id.to_string()),
                rect: node_rect,
                style: FrameStyle::Standard,
                color: border_color,
                temporal: TemporalStrategy::Standard,
            });

            if self.kernel.focused_id == Some(surface_id) {
                primitives.push(SurfacePrimitive::FocusRing {
                    id: SurfaceId::from_seed(&format!("focus_{}", node.id)),
                    rect: node_rect,
                    color: [0.6, 1.0, 0.4, 1.0], // Lime accent
                    temporal: TemporalStrategy::Fast,
                });
            }

            // 2b. Ports
            if let Some(ports) = ports {
                for (i, port) in ports.iter().enumerate() {
                    let is_input = port.direction == dirtydata_core::types::PortDirection::Input;
                    let port_pos =
                        NodeLayout::get_port_position(pos, node_rect, i, ports.len(), is_input);

                    primitives.push(SurfacePrimitive::Connector {
                        id: SurfaceId::from_seed(&format!("port_{}_{}", node.id, port.name)),
                        pos: port_pos,
                        signal_type: limestudio_surface::ui_ir::SignalType::Audio,
                        state: limestudio_surface::ui_ir::ConnectorState::Default,
                    });
                }
            }
        }

        // 3. Causality Links (Polyphonic)
        for trace in &self.causality_monitor.poly_traces {
            for node_id in &trace.impact_nodes {
                if let Some(&pos) = project.view.node_positions.get(node_id) {
                    primitives.push(SurfacePrimitive::CausalityLink {
                        source_id: SurfaceId::from_seed(&trace.event_source),
                        target_id: (*node_id).into(),
                        voice_id: Some(trace.voice_id),
                        path: vec![[0.0, 0.0], pos],
                        intensity: 1.0,
                        confidence: 1.0,
                        relevance: 1.0,
                        activity: 1.0,
                        color: [0.0, 1.0, 1.0, 0.6],
                    });
                }
            }
        }

        SurfaceWidget::PrimitiveStream { primitives }
    }

    pub fn handle_event(&mut self, event: SurfaceEvent) {
        let project = self.transaction.project();
        let graph = &project.ui;

        // 1. Collect Hit-testable candidates
        let mut nodes = Vec::new();
        let mut ports = Vec::new();
        let widgets = Vec::new();

        for node in graph.nodes.values() {
            let Some(ui) = &node.ui else { continue };
            let pos = [
                NodeLayout::snap(ui.position[0]),
                NodeLayout::snap(ui.position[1]),
            ];

            let kernel_id = project.view.id_map.resolve_uuid(node.id);
            let (node_name, num_ports) = if let Some(kid) = kernel_id {
                if let Some(knode) = project.graph.topology.nodes.get(&kid) {
                    (
                        knode
                            .config
                            .get("name")
                            .and_then(|v| v.as_string())
                            .map(|s| s.clone())
                            .unwrap_or_else(|| "node".to_string()),
                        knode.ports.len(),
                    )
                } else {
                    ("node".to_string(), 0)
                }
            } else {
                ("node".to_string(), 0)
            };

            let rect_size = NodeLayout::calculate_node_rect(&node_name, num_ports, 0);
            let surface_id = SurfaceId::from_seed(&node.id.to_string());

            let rect = Rect {
                center: Vec2::new(pos[0] + rect_size[2] * 0.5, pos[1] + rect_size[3] * 0.5),
                size: Vec2::new(rect_size[2], rect_size[3]),
            };

            nodes.push((surface_id, rect));

            // Ports as hit-testable
            if let Some(kid) = kernel_id {
                if let Some(knode) = project.graph.topology.nodes.get(&kid) {
                    for (i, port) in knode.ports.iter().enumerate() {
                        let is_input =
                            port.direction == dirtydata_core::types::PortDirection::Input;
                        let node_rect = [pos[0], pos[1], rect_size[2], rect_size[3]];
                        let port_pos = NodeLayout::get_port_position(
                            pos,
                            node_rect,
                            i,
                            knode.ports.len(),
                            is_input,
                        );
                        let port_id =
                            SurfaceId::from_seed(&format!("port_{}_{}", node.id, port.name));
                        ports.push((
                            port_id,
                            Circle {
                                center: Vec2::new(port_pos[0], port_pos[1]),
                                radius: 6.0,
                            },
                        ));
                    }
                }
            }
        }

        // 2. Dispatch to kernel
        let intents = self
            .kernel
            .handle_event(&event, &self.camera, &nodes, &ports, &widgets);

        // 3. Process Intents
        for intent in intents {
            match intent {
                limestudio_surface::runtime::interaction_kernel::InteractionIntent::Select {
                    ids,
                } => {
                    if let Some(_first) = ids.first() {
                        // TODO: Map back to UiIndex and update self.selected_node
                    }
                }
                _ => {}
            }
        }
    }
}
