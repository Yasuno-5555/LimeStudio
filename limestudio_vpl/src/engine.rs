use std::sync::Arc;
use glam::Vec2;
use limestudio_core::transaction::TransactionLayer;
use limestudio_core::pipeline::EngineToUiPipeline;
use limestudio_core::UiIndex;
use limestudio_surface::ui_ir::{SurfaceWidget, SurfaceId, FrameStyle};
use limestudio_surface::scene::camera::InfiniteCamera;
use limestudio_surface::runtime::interaction_kernel::InteractionKernel;
use limestudio_surface::runtime::input::SurfaceEvent;

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
        }
    }

    pub fn set_shared_state(&mut self, state: Arc<dirtydata_runtime::SharedState>) {
        self.shared_state = Some(state);
    }

    pub fn build_ui(&mut self) -> SurfaceWidget {
        // 1. Handle Kernel Responses
        self.transaction.handle_responses(&mut self.engine_responses);

        // 2. Build UI Tree
        use SurfaceWidget::*;

        let top_bar = Box {
            style: FrameStyle::Standard,
            children: vec![
                Row {
                    children: vec![
                        Label { text: "LimeStudio".to_string(), is_secondary: false },
                        Button { id: SurfaceId::generate(), label: "PATCHING".to_string(), is_active: self.mode == AppMode::Patching },
                        Button { id: SurfaceId::generate(), label: "DESIGNER".to_string(), is_active: self.mode == AppMode::Designer },
                        Button { id: SurfaceId::generate(), label: "SHIP".to_string(), is_active: self.mode == AppMode::Ritual },
                    ]
                }
            ]
        };

        // --- Tiered & Categorized Sidebar ---
        let mut side_bar_children = vec![
            Label { text: "COMPONENTS".to_string(), is_secondary: false },
        ];

        // Group by Category
        let mut categories = std::collections::BTreeSet::new();
        for b in self.registry.blueprints.values() {
            categories.insert(&b.category);
        }

        for cat in categories {
            side_bar_children.push(Label { text: format!("> {}", cat.to_uppercase()), is_secondary: true });
            for b in self.registry.find_by_category(cat) {
                let tier_label = match b.tier {
                    NodeTier::Core => " [S]",
                    NodeTier::Advanced => " [A]",
                    NodeTier::Experimental => " [B]",
                    NodeTier::Forbidden => " [!]",
                };
                side_bar_children.push(Button { 
                    id: SurfaceId::generate(), 
                    label: format!("{}{}", b.display_name, tier_label), 
                    is_active: false 
                });
            }
        }

        let side_bar = Box {
            style: FrameStyle::Standard,
            children: side_bar_children,
        };

        let central_panel = self.build_canvas();

        Row {
            children: vec![
                side_bar,
                Column {
                    children: vec![
                        top_bar,
                        central_panel,
                    ]
                }
            ]
        }
    }

    fn build_canvas(&self) -> SurfaceWidget {
        use limestudio_surface::ui_ir::{SurfacePrimitive, CurveKind, TemporalStrategy};

        let project = self.transaction.project();
        let graph = &project.ui;
        let mut primitives = Vec::new();

        // 1. Cables
        for edge in &graph.edges {
            if let (Some(from_node), Some(to_node)) = (graph.nodes.get(&edge.from), graph.nodes.get(&edge.to)) {
                if let (Some(from_ui), Some(to_ui)) = (&from_node.ui, &to_node.ui) {
                    primitives.push(SurfacePrimitive::Curve {
                        id: SurfaceId::generate(),
                        control_points: vec![from_ui.position, to_ui.position],
                        kind: CurveKind::Cable,
                        thickness: 2.0,
                        color: [0.5, 0.5, 0.5, 1.0],
                        temporal: TemporalStrategy::Standard(0.06),
                    });
                }
            }
        }

        // 2. Nodes
        for node in graph.nodes.values() {
            let Some(ui) = &node.ui else { continue };
            let pos = ui.position;
            let surface_id = SurfaceId::from_seed(&node.id.to_string());
            let is_selected = self.kernel.selection.ids.contains(&surface_id);
            
            let cpu_load = 0.0; // Placeholder for now
            let is_hot = cpu_load > 15.0;
            let border_color = if is_hot { [1.0, 0.2, 0.2, 1.0] } else if is_selected { [0.6, 1.0, 0.4, 1.0] } else { ui.color };

            primitives.push(SurfacePrimitive::Frame {
                id: SurfaceId::from_seed(&node.id.to_string()),
                rect: [pos[0], pos[1], 120.0, 48.0],
                style: FrameStyle::Standard,
                color: border_color,
                temporal: TemporalStrategy::Standard(0.06),
            });
        }

        // 3. Causality Links (Polyphonic)
        for trace in &self.causality_monitor.poly_traces {
            for node_id in &trace.impact_nodes {
                if let Some(&pos) = project.view.node_positions.get(node_id) {
                    primitives.push(limestudio_surface::ui_ir::SurfacePrimitive::CausalityLink {
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

        SurfaceWidget::PrimitiveStream {
            primitives
        }
    }

    pub fn handle_event(&mut self, _event: SurfaceEvent) {
        // Forward to kernel
        // self.kernel.handle_event(&event, &self.camera, &nodes, &ports);
    }
}
