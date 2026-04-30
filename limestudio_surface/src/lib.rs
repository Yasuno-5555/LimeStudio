pub mod adapter;
pub mod authority;
pub mod color;
pub mod host_attach;
pub mod model;
pub mod motion;
pub mod profiler;
pub mod render;
pub mod runtime;
pub mod scene;
pub mod ui_ir;
pub mod widgets;

use crate::color::Color;
use crate::model::stable_id::SurfaceId;
use crate::ui_ir::{
    ArcKind, DisplaySignal, FrameStyle, IndicatorKind, SurfacePrimitive, TemporalStrategy,
};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use taffy::prelude::*;
use taffy::style::Dimension;

pub struct TransitionState {
    pub motion: crate::motion::MotionState,
    pub target: f32,
    pub strategy: TemporalStrategy,
}

pub struct LinkAnalysisState {
    pub baseline_velocity: f32,
    pub last_value: f32,
    pub is_top_k: bool,
    pub last_rank: u32,
    pub echo_timer: f32,
}

/// The Core Surface Engine (V7 Semantic Architecture)
pub struct SurfaceEngine {
    pub scene: scene::flat_scene::SurfaceScene,
    pub input: runtime::interaction_kernel::InteractionKernel,
    pub camera: scene::camera::InfiniteCamera,
    pub profiler: profiler::FrameProfiler,
    pub kernel: crate::ui_ir::SurfaceKernel,

    // V3-V7 Architecture: Asynchronous Perception
    pub primitive_stream: Arc<Mutex<Vec<SurfacePrimitive>>>,
    pub transition_store: HashMap<SurfaceId, TransitionState>,
    pub selections: std::collections::HashSet<SurfaceId>,
    pub causality_states: HashMap<SurfaceId, LinkAnalysisState>,
    pub last_frame_time: Instant,

    // Authority Layer
    pub visible_compiler: authority::visible_compiler::VisibleCompilerRegistry,
    pub authority_drawer: widgets::authority_drawer::AuthorityDrawer,
    pub time_travel: authority::time_travel::TimeTravelEngine,
    pub causal_replay: authority::causal_replay::CausalReplayEngine,
    pub trust_ledger: model::provenance::TrustLedger,

    // Layout Engine (Taffy)
    pub taffy: Taffy,

    // Geometry Cache for Hit Testing
    pub node_geometry: Vec<(SurfaceId, model::geometry::Rect)>,
    pub port_geometry: Vec<(SurfaceId, model::geometry::Circle)>,
    pub widget_geometry: Vec<(
        SurfaceId,
        model::geometry::Rect,
        crate::ui_ir::InteractionClass,
    )>,
    pub pending_intents: Vec<crate::runtime::interaction_kernel::InteractionIntent>,
    pub theme: model::theme::SurfaceTheme,
    pub debug_inspector: bool,
}

impl Default for SurfaceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceEngine {
    pub fn new() -> Self {
        let mut visible_compiler = authority::visible_compiler::VisibleCompilerRegistry::new();
        let mock_id = SurfaceId::generate(); // We'd need a way to reference actual nodes
        visible_compiler.provenance.insert(
            mock_id,
            authority::visible_compiler::CodeFragment {
                source: "fn dsp_process(in: f32) -> f32 {\n    in * 0.5 // Accountable Logic\n}"
                    .to_string(),
                language: "rust".to_string(),
            },
        );

        Self {
            scene: scene::flat_scene::SurfaceScene::new(),
            input: runtime::interaction_kernel::InteractionKernel::new(),
            camera: scene::camera::InfiniteCamera::new(glam::Vec2::new(1280.0, 720.0)),
            profiler: profiler::FrameProfiler::new(),
            primitive_stream: Arc::new(Mutex::new(Vec::new())),
            transition_store: HashMap::new(),
            selections: HashSet::new(),
            causality_states: HashMap::new(),
            last_frame_time: Instant::now(),
            visible_compiler,

            authority_drawer: widgets::authority_drawer::AuthorityDrawer::new(
                SurfaceId::generate(),
                glam::Vec2::new(1280.0, 720.0),
            ),
            time_travel: authority::time_travel::TimeTravelEngine::new(64),
            causal_replay: authority::causal_replay::CausalReplayEngine::new(3.0),
            trust_ledger: model::provenance::TrustLedger::new(),
            kernel: crate::ui_ir::SurfaceKernel::new(),
            taffy: Taffy::new(),
            node_geometry: Vec::new(),
            port_geometry: Vec::new(),
            widget_geometry: Vec::new(),
            pending_intents: Vec::new(),
            theme: model::theme::SurfaceTheme::default(),
            debug_inspector: false,
        }
    }

    /// Taffyを用いた本等なレイアウト解決。
    pub fn sync_ui(&mut self, tree: &crate::ui_ir::SurfaceWidget) {
        self.taffy.clear();
        self.node_geometry.clear();
        self.port_geometry.clear();
        self.widget_geometry.clear();

        let main_view = self.build_taffy_recursive(tree);
        let mut root_children = vec![main_view];

        // 2. Inject Authority Drawer if a node is selected
        let mut drawer_widget = None;
        if let Some(selected_id) = self.selections.iter().next() {
            let snapshot = self.time_travel.get_current_state();
            // Convert SurfaceId (wraps StableId) to StableId for lookup
            let stable_id = dirtydata_core::types::StableId(selected_id.0 .0);
            let node_state = snapshot.and_then(|s| s.node_states.get(&stable_id));
            let code_fragment = self.visible_compiler.get_fragment(*selected_id);

            let dw = self.authority_drawer.build_widget(
                *selected_id,
                "Selected Node",
                node_state,
                code_fragment,
            );

            let drawer_node = self.build_taffy_recursive(&dw);
            root_children.push(drawer_node);
            drawer_widget = Some(dw);
        }

        let root_style = Style {
            size: Size {
                width: Dimension::Points(1280.0),
                height: Dimension::Points(720.0),
            },
            flex_direction: FlexDirection::Row, // Side-by-side
            ..Default::default()
        };
        let container = self
            .taffy
            .new_with_children(root_style, &root_children)
            .unwrap();

        // 1280x720 を論理的な基準としてレイアウト計算
        self.taffy
            .compute_layout(
                container,
                Size {
                    width: AvailableSpace::Definite(1280.0),
                    height: AvailableSpace::Definite(720.0),
                },
            )
            .unwrap();

        self.widget_geometry.clear();
        self.node_geometry.clear();
        self.port_geometry.clear();

        let mut primitives = Vec::new();
        self.generate_primitives_from_layout(tree, main_view, &mut primitives, glam::Vec2::ZERO, 0);

        if let Some(dw) = drawer_widget {
            let node_children = self.taffy.children(container).unwrap();
            if node_children.len() > 1 {
                self.generate_primitives_from_layout(
                    &dw,
                    node_children[1],
                    &mut primitives,
                    glam::Vec2::ZERO,
                    100, // Drawer is always on top
                );
            }
        }

        // 5. Debug Inspector Overlay
        if self.debug_inspector {
            for (_, rect, _) in &self.widget_geometry {
                primitives.push(SurfacePrimitive::Frame {
                    id: SurfaceId::generate(),
                    rect: [rect.min().x, rect.min().y, rect.size.x, rect.size.y],
                    style: crate::ui_ir::FrameStyle::None,
                    color: [1.0, 0.0, 1.0, 0.2], // Semi-transparent magenta
                    temporal: TemporalStrategy::Instant,
                });
            }
        }

        *self.primitive_stream.lock().unwrap() = primitives;
    }

    pub fn handle_intents(
        &mut self,
        intents: Vec<crate::runtime::interaction_kernel::InteractionIntent>,
    ) {
        for intent in intents {
            // Internal processing
            match &intent {
                crate::runtime::interaction_kernel::InteractionIntent::Select { ids } => {
                    self.selections = ids.iter().cloned().collect();
                }
                _ => {}
            }
            // Push to public queue for host/plugin to observe
            self.pending_intents.push(intent);
        }
    }

    pub fn take_intents(&mut self) -> Vec<crate::runtime::interaction_kernel::InteractionIntent> {
        std::mem::take(&mut self.pending_intents)
    }

    fn build_taffy_recursive(&mut self, widget: &crate::ui_ir::SurfaceWidget) -> Node {
        use crate::ui_ir::SurfaceWidget::*;
        match widget {
            Column { children } => {
                let child_nodes: Vec<_> = children
                    .iter()
                    .map(|c| self.build_taffy_recursive(c))
                    .collect();
                self.taffy
                    .new_with_children(
                        Style {
                            flex_direction: FlexDirection::Column,
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Auto,
                            },
                            ..Default::default()
                        },
                        &child_nodes,
                    )
                    .unwrap()
            }
            Row { children } => {
                let child_nodes: Vec<_> = children
                    .iter()
                    .map(|c| self.build_taffy_recursive(c))
                    .collect();
                self.taffy
                    .new_with_children(
                        Style {
                            flex_direction: FlexDirection::Row,
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Auto,
                            },
                            ..Default::default()
                        },
                        &child_nodes,
                    )
                    .unwrap()
            }
            FocusProxy { child, .. } => self.build_taffy_recursive(child),
            Accessibility { child, .. } => self.build_taffy_recursive(child),
            Knob { .. } | Slider { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Points(80.0),
                        height: Dimension::Points(80.0),
                    },
                    margin: Rect {
                        left: LengthPercentageAuto::Points(8.0),
                        right: LengthPercentageAuto::Points(8.0),
                        top: LengthPercentageAuto::Points(8.0),
                        bottom: LengthPercentageAuto::Points(8.0),
                    },
                    ..Default::default()
                })
                .unwrap(),
            LevelMeter { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Points(32.0),
                        height: Dimension::Points(128.0),
                    },
                    ..Default::default()
                })
                .unwrap(),
            XYPad { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Points(160.0),
                        height: Dimension::Points(160.0),
                    },
                    margin: Rect {
                        left: LengthPercentageAuto::Points(8.0),
                        right: LengthPercentageAuto::Points(8.0),
                        top: LengthPercentageAuto::Points(8.0),
                        bottom: LengthPercentageAuto::Points(8.0),
                    },
                    ..Default::default()
                })
                .unwrap(),
            Box {
                children,
                layout_style,
                ..
            } => {
                let children_nodes: Vec<_> = children
                    .iter()
                    .map(|c| self.build_taffy_recursive(c))
                    .collect();
                self.taffy
                    .new_with_children(*layout_style.clone(), &children_nodes)
                    .unwrap()
            }
            Button { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Auto,
                        height: Dimension::Points(32.0),
                    },
                    min_size: Size {
                        width: Dimension::Points(80.0),
                        height: Dimension::Points(32.0),
                    },
                    margin: Rect::points(4.0),
                    ..Default::default()
                })
                .unwrap(),
            Custom { style, .. } => self.taffy.new_leaf(*style.clone()).unwrap(),
            Label { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Auto,
                        height: Dimension::Points(24.0),
                    },
                    margin: Rect {
                        left: LengthPercentageAuto::Points(8.0),
                        right: LengthPercentageAuto::Points(8.0),
                        top: LengthPercentageAuto::Points(4.0),
                        bottom: LengthPercentageAuto::Points(4.0),
                    },
                    ..Default::default()
                })
                .unwrap(),
            PrimitiveStream { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Auto,
                    },
                    ..Default::default()
                })
                .unwrap(),
            Timeline { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Points(40.0),
                    },
                    margin: Rect::points(8.0),
                    ..Default::default()
                })
                .unwrap(),
            CodeView { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Points(300.0),
                    },
                    margin: Rect::points(4.0),
                    ..Default::default()
                })
                .unwrap(),
            Waveform { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Points(100.0),
                    },
                    margin: Rect::points(8.0),
                    ..Default::default()
                })
                .unwrap(),
            Spectrum { .. } => self
                .taffy
                .new_leaf(Style {
                    size: Size {
                        width: Dimension::Percent(1.0),
                        height: Dimension::Points(120.0),
                    },
                    margin: Rect::points(8.0),
                    ..Default::default()
                })
                .unwrap(),
            Scroll { child, .. } => {
                let child_node = self.build_taffy_recursive(child);
                self.taffy
                    .new_with_children(
                        Style {
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Percent(1.0),
                            },
                            ..Default::default()
                        },
                        &[child_node],
                    )
                    .unwrap()
            }
            Layer { child, .. } | Memo { child, .. } => {
                let child_node = self.build_taffy_recursive(child);
                self.taffy
                    .new_with_children(
                        Style {
                            display: Display::Flex,
                            size: Size {
                                width: Dimension::Percent(1.0),
                                height: Dimension::Percent(1.0),
                            },
                            ..Default::default()
                        },
                        &[child_node],
                    )
                    .unwrap()
            }
            _ => self.taffy.new_leaf(Style::default()).unwrap(),
        }
    }

    fn generate_primitives_from_layout(
        &mut self,
        widget: &crate::ui_ir::SurfaceWidget,
        node: Node,
        primitives: &mut Vec<SurfacePrimitive>,
        parent_pos: glam::Vec2,
        level: i32,
    ) {
        let layout = self.taffy.layout(node).unwrap();
        // The Law of Lime (8px Grid & Sub-pixel Rounding)
        let pos = parent_pos + glam::vec2(layout.location.x, layout.location.y);
        let rounded_pos = (pos / 8.0).round() * 8.0;
        let size = glam::vec2(layout.size.width, layout.size.height);
        let rect = crate::model::geometry::Rect::new(rounded_pos, size);

        use crate::ui_ir::SurfaceWidget::*;

        // Populate Widget Geometry for generic interaction
        self.widget_geometry.push((
            *widget.id().unwrap_or(&SurfaceId::generate()),
            rect,
            widget.interaction_class(),
        ));

        match widget {
            FocusProxy {
                id,
                child,
                is_focused,
                ..
            } => {
                let node_child = self.taffy.children(node).unwrap()[0];
                self.generate_primitives_from_layout(child, node_child, primitives, pos, level);
                if *is_focused {
                    primitives.push(SurfacePrimitive::FocusRing {
                        id: SurfaceId::from_seed(&format!("focus_proxy_{}", id.0 .0)),
                        rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                        color: [0.6, 1.0, 0.4, 1.0], // Lime accent
                        temporal: TemporalStrategy::Fast,
                    });
                }
            }
            Accessibility { child, .. } => {
                let node_child = self.taffy.children(node).unwrap()[0];
                self.generate_primitives_from_layout(child, node_child, primitives, pos, level);
            }

            Scroll {
                child, scroll_pos, ..
            } => {
                let node_child = self.taffy.children(node).unwrap()[0];
                let mut child_primitives = Vec::new();
                self.generate_primitives_from_layout(
                    child,
                    node_child,
                    &mut child_primitives,
                    pos - glam::Vec2::new(scroll_pos[0], scroll_pos[1]),
                    level,
                );

                primitives.push(SurfacePrimitive::ClipMask {
                    id: SurfaceId::generate(),
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    law: crate::ui_ir::OverlapLaw::Inside,
                    children: child_primitives,
                });
            }
            Layer {
                level: layer_level,
                child,
                ..
            } => {
                let node_child = self.taffy.children(node).unwrap()[0];
                self.generate_primitives_from_layout(
                    child,
                    node_child,
                    primitives,
                    pos,
                    level + layer_level,
                );
            }
            Memo { child, .. } => {
                let node_child = self.taffy.children(node).unwrap()[0];
                self.generate_primitives_from_layout(child, node_child, primitives, pos, level);
            }

            Knob { id, label, signal } => {
                let value = match signal {
                    DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.15, 0.15, 0.15, 1.0],
                    temporal: TemporalStrategy::Standard,
                });
                primitives.push(SurfacePrimitive::Arc {
                    id: *id,
                    center: [rounded_pos.x + size.x * 0.5, rounded_pos.y + size.y * 0.5],
                    radius: size.x * 0.4,
                    thickness: 4.0,
                    start_angle: -135.0,
                    end_angle: -135.0 + (value * 270.0),
                    kind: ArcKind::Value,
                    temporal: TemporalStrategy::Standard,
                });
                primitives.push(SurfacePrimitive::Text {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y + size.y + 4.0, size.x, 20.0],
                    text: label.clone(),
                    font_size: 12.0,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
            }
            Slider {
                id,
                label,
                signal,
                is_vertical: _,
            } => {
                let value = match signal {
                    DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                primitives.push(SurfacePrimitive::Indicator {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    kind: IndicatorKind::Led,
                    value,
                    color: Color::ACCENT_LIME.to_array(),
                    temporal: TemporalStrategy::Standard,
                });
                primitives.push(SurfacePrimitive::Text {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y + size.y + 4.0, size.x, 20.0],
                    text: label.clone(),
                    font_size: 12.0,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
            }
            LevelMeter { id, signal } => {
                let (left, right) = match signal {
                    DisplaySignal::Meter { value, peak } => (*value, *peak),
                    _ => (0.0, 0.0),
                };
                let id_stable = SurfaceId::from_seed(id);
                primitives.push(SurfacePrimitive::Indicator {
                    id: id_stable,
                    rect: [rounded_pos.x, rounded_pos.y, size.x * 0.4, size.y],
                    kind: IndicatorKind::Led,
                    value: left,
                    color: [0.0, 1.0, 0.2, 1.0],
                    temporal: TemporalStrategy::Slow,
                });
                primitives.push(SurfacePrimitive::Indicator {
                    id: id_stable,
                    rect: [
                        rounded_pos.x + size.x * 0.6,
                        rounded_pos.y,
                        size.x * 0.4,
                        size.y,
                    ],
                    kind: IndicatorKind::Led,
                    value: right,
                    color: [0.0, 1.0, 0.2, 1.0],
                    temporal: TemporalStrategy::Slow,
                });
            }
            XYPad {
                id,
                label,
                x_signal,
                y_signal,
            } => {
                let x = match x_signal {
                    DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                let y = match y_signal {
                    DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.15, 0.15, 0.15, 1.0],
                    temporal: TemporalStrategy::Standard,
                });
                // Pad Crosshair
                primitives.push(SurfacePrimitive::Indicator {
                    id: *id,
                    rect: [
                        rounded_pos.x + x * size.x - 4.0,
                        rounded_pos.y + y * size.y - 4.0,
                        8.0,
                        8.0,
                    ],
                    kind: IndicatorKind::Led,
                    value: 1.0,
                    color: Color::ACCENT_LIME.to_array(),
                    temporal: TemporalStrategy::Standard,
                });
                primitives.push(SurfacePrimitive::Text {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y + size.y + 4.0, size.x, 20.0],
                    text: label.clone(),
                    font_size: 12.0,
                    color: [0.8, 0.8, 0.8, 1.0],
                });
            }
            Box {
                children,
                style,
                layout_style: _,
            } => {
                primitives.push(SurfacePrimitive::Frame {
                    id: SurfaceId::generate(),
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: *style,
                    color: if *style == FrameStyle::AuthorityGlass {
                        [0.05, 0.05, 0.05, 0.9]
                    } else {
                        [0.2, 0.2, 0.2, 0.8]
                    },
                    temporal: TemporalStrategy::Instant,
                });

                let node_children = self.taffy.children(node).unwrap();
                for (i, child) in children.iter().enumerate() {
                    if i < node_children.len() {
                        self.generate_primitives_from_layout(
                            child,
                            node_children[i],
                            primitives,
                            pos,
                            level,
                        );
                    }
                }
            }

            Button {
                id,
                label,
                is_active,
            } => {
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: if *is_active {
                        [0.4, 0.4, 0.4, 1.0]
                    } else {
                        [0.3, 0.3, 0.3, 1.0]
                    },
                    temporal: TemporalStrategy::Standard,
                });
                primitives.push(SurfacePrimitive::Text {
                    id: *id,
                    rect: [
                        rounded_pos.x + 8.0,
                        rounded_pos.y + 4.0,
                        size.x - 16.0,
                        size.y - 8.0,
                    ],
                    text: label.clone(),
                    font_size: 14.0,
                    color: [0.9, 0.9, 0.9, 1.0],
                });
            }
            Label { text, is_secondary } => {
                primitives.push(SurfacePrimitive::Text {
                    id: SurfaceId::generate(),
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    text: text.clone(),
                    font_size: if *is_secondary { 12.0 } else { 16.0 },
                    color: if *is_secondary {
                        [0.6, 0.6, 0.6, 1.0]
                    } else {
                        [0.9, 0.9, 0.9, 1.0]
                    },
                });
            }
            Custom {
                primitives: custom_primitives,
                ..
            } => {
                for cp in custom_primitives {
                    let mut cp_cloned = cp.clone();
                    Self::offset_primitive(&mut cp_cloned, rounded_pos);
                    primitives.push(cp_cloned);
                }
            }
            CodeView { code, .. } => {
                primitives.push(SurfacePrimitive::Frame {
                    id: SurfaceId::generate(),
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.05, 0.05, 0.05, 1.0],
                    temporal: TemporalStrategy::Instant,
                });
                primitives.push(SurfacePrimitive::Text {
                    id: SurfaceId::generate(),
                    rect: [
                        rounded_pos.x + 8.0,
                        rounded_pos.y + 8.0,
                        size.x - 16.0,
                        size.y - 16.0,
                    ],
                    text: code.clone(),
                    font_size: 13.0,
                    color: [0.7, 0.9, 0.7, 1.0], // Hack: greenish code
                });
            }
            Waveform { id, data } => {
                let id_stable = SurfaceId::from_seed(id);
                // 1. Background
                primitives.push(SurfacePrimitive::Frame {
                    id: id_stable,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.02, 0.02, 0.02, 1.0],
                    temporal: TemporalStrategy::Instant,
                });

                // 2. Data Segments
                if data.len() > 1 {
                    let step = size.x / (data.len() - 1) as f32;
                    let center_y = rounded_pos.y + size.y * 0.5;
                    let scale_y = size.y * 0.4;

                    for i in 0..(data.len() - 1) {
                        primitives.push(SurfacePrimitive::Curve {
                            id: id_stable,
                            control_points: vec![
                                [
                                    rounded_pos.x + i as f32 * step,
                                    center_y - data[i] * scale_y,
                                ],
                                [
                                    rounded_pos.x + (i + 1) as f32 * step,
                                    center_y - data[i + 1] * scale_y,
                                ],
                            ],
                            kind: crate::ui_ir::CurveKind::Cable, // Hack: use cable kind for now
                            thickness: 1.5,
                            color: [0.0, 1.0, 1.0, 0.8], // Cyan
                            temporal: TemporalStrategy::Slow,
                        });
                    }
                }
            }
            Spectrum { id, data } => {
                let id_stable = SurfaceId::from_seed(id);
                // 1. Background
                primitives.push(SurfacePrimitive::Frame {
                    id: id_stable,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.01, 0.01, 0.01, 1.0],
                    temporal: TemporalStrategy::Instant,
                });

                // 2. Spectrum Bars/Line
                if data.len() > 1 {
                    let step = size.x / (data.len() - 1) as f32;
                    let bottom_y = rounded_pos.y + size.y;
                    let scale_y = size.y * 0.9;

                    for i in 0..(data.len() - 1) {
                        primitives.push(SurfacePrimitive::Curve {
                            id: id_stable,
                            control_points: vec![
                                [
                                    rounded_pos.x + i as f32 * step,
                                    bottom_y - data[i] * scale_y,
                                ],
                                [
                                    rounded_pos.x + (i + 1) as f32 * step,
                                    bottom_y - data[i + 1] * scale_y,
                                ],
                            ],
                            kind: crate::ui_ir::CurveKind::Cable,
                            thickness: 2.0,
                            color: [1.0, 0.2, 0.5, 0.9], // Pink/Magenta for spectrum
                            temporal: TemporalStrategy::Slow,
                        });
                    }
                }
            }

            ForensicMonitor { id, data } => {
                let id_stable = *id;
                // 1. Background (Authority Glass)
                primitives.push(SurfacePrimitive::Frame {
                    id: id_stable,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::AuthorityGlass,
                    color: [0.1, 0.1, 0.1, 0.8],
                    temporal: TemporalStrategy::Instant,
                });

                // 2. CPU Meter (Authority Lime)
                let cpu_norm = (data.cpu_micros / 1000.0).min(1.0);
                primitives.push(SurfacePrimitive::Indicator {
                    id: id_stable,
                    rect: [rounded_pos.x + 8.0, rounded_pos.y + 8.0, size.x - 16.0, 4.0],
                    kind: IndicatorKind::Led,
                    value: cpu_norm,
                    color: [0.75, 1.0, 0.0, 1.0], // Lime
                    temporal: TemporalStrategy::Fast,
                });

                // 3. Status Indicators (CLIP & NaN)
                if data.has_clipped {
                    primitives.push(SurfacePrimitive::Indicator {
                        id: id_stable,
                        rect: [rounded_pos.x + 8.0, rounded_pos.y + 16.0, 12.0, 12.0],
                        kind: IndicatorKind::Led,
                        value: 1.0,
                        color: [1.0, 0.1, 0.1, 1.0], // Red
                        temporal: TemporalStrategy::Instant,
                    });
                }
                if data.has_nan {
                    primitives.push(SurfacePrimitive::Indicator {
                        id: id_stable,
                        rect: [rounded_pos.x + 24.0, rounded_pos.y + 16.0, 12.0, 12.0],
                        kind: IndicatorKind::Led,
                        value: 1.0,
                        color: [1.0, 0.5, 0.0, 1.0], // Orange
                        temporal: TemporalStrategy::Instant,
                    });
                }
            }
            PrimitiveStream {
                primitives: stream_primitives,
            } => {
                for p in stream_primitives {
                    let mut p_cloned = p.clone();
                    Self::offset_primitive(&mut p_cloned, rounded_pos);
                    primitives.push(p_cloned);
                }
            }
            Timeline {
                id,
                snapshots,
                current_idx,
            } => {
                // 1. Background Bar
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y + 16.0, size.x, 8.0],
                    style: FrameStyle::Standard,
                    color: [0.1, 0.1, 0.1, 1.0],
                    temporal: TemporalStrategy::Instant,
                });

                // 2. Markers for each snapshot
                let count = snapshots.len();
                if count > 1 {
                    for i in 0..count {
                        let progress = i as f32 / (count - 1) as f32;
                        let is_current = i == *current_idx;
                        primitives.push(SurfacePrimitive::Indicator {
                            id: *id,
                            rect: [
                                rounded_pos.x + progress * size.x - 4.0,
                                rounded_pos.y + 12.0,
                                8.0,
                                16.0,
                            ],
                            kind: IndicatorKind::Led,
                            value: if is_current { 1.0 } else { 0.3 },
                            color: if is_current {
                                Color::ACCENT_LIME.to_array()
                            } else {
                                [0.5, 0.5, 0.5, 1.0]
                            },
                            temporal: TemporalStrategy::Standard,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    pub fn generate_instances(&mut self) -> Vec<render::sdf::SdfInstance> {
        let mut primitives = self.primitive_stream.lock().unwrap().clone();

        // Inject Active Interaction Primitives
        match &self.input.session {
            crate::runtime::interaction_kernel::DragSession::Connecting {
                from_port,
                current_pos,
            } => {
                let start_pos = self
                    .port_geometry
                    .iter()
                    .find(|(id, _)| id == from_port)
                    .map(|(_, circle)| circle.center)
                    .unwrap_or(glam::Vec2::ZERO);

                primitives.push(SurfacePrimitive::Curve {
                    id: SurfaceId::generate(),
                    control_points: vec![
                        [start_pos.x, start_pos.y],
                        [current_pos.x, current_pos.y],
                    ],
                    kind: crate::ui_ir::CurveKind::Cable,
                    thickness: 3.0,
                    color: [1.0, 1.0, 1.0, 0.5],
                    temporal: TemporalStrategy::Instant,
                });
            }

            _ => {}
        }

        let mut instances = Vec::new();

        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // V7 Recursive Semantic Rendering
        self.generate_instances_recursive(
            &primitives,
            &mut instances,
            glam::Mat4::IDENTITY,
            dt,
            now,
        );

        instances
    }

    fn generate_instances_recursive(
        &mut self,
        primitives: &[SurfacePrimitive],
        instances: &mut Vec<render::sdf::SdfInstance>,
        transform: glam::Mat4,
        dt: f32,
        _now: Instant,
    ) {
        for prim in primitives {
            match prim {
                SurfacePrimitive::Frame {
                    id: _,
                    rect,
                    style,
                    color,
                    ..
                } => {
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(
                        rect[0] + size.x,
                        rect[1] + size.y,
                        0.0,
                    ));
                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: size * transform.x_axis.truncate().length(),
                        color: glam::Vec4::from_array(*color),
                        shape_type: 5,
                        modulation_depth: 0.0,
                        modulation_current: 0.0,
                        _pad: 0,
                        params: glam::Vec4::new(*style as u32 as f32, 1.0, 0.0, 0.0),
                        params2: glam::Vec4::ZERO,
                    });
                }
                SurfacePrimitive::Arc {
                    id,
                    center,
                    radius,
                    thickness,
                    start_angle,
                    end_angle,
                    kind,
                    ..
                } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(*end_angle),
                        target: *end_angle,
                        strategy: TemporalStrategy::Standard,
                    });
                    let current_end = {
                        let (stiffness, damping) = match state.strategy {
                            crate::ui_ir::TemporalStrategy::Fast => (
                                crate::motion::constants::STIFFNESS_FAST,
                                crate::motion::constants::DAMPING_FAST,
                            ),
                            crate::ui_ir::TemporalStrategy::Standard => (
                                crate::motion::constants::STIFFNESS_STANDARD,
                                crate::motion::constants::DAMPING_STANDARD,
                            ),
                            crate::ui_ir::TemporalStrategy::Slow => (
                                crate::motion::constants::STIFFNESS_SLOW,
                                crate::motion::constants::DAMPING_SLOW,
                            ),
                            crate::ui_ir::TemporalStrategy::Instant => (1000000.0, 1000.0),
                        };
                        state.motion.update(*end_angle, dt, stiffness, damping)
                    };

                    let half_angle = (current_end - start_angle).to_radians() * 0.5;
                    let sc = glam::Vec2::new(half_angle.sin(), half_angle.cos());
                    let p = transform.transform_point3(glam::vec3(center[0], center[1], 0.0));
                    let scale = transform.x_axis.truncate().length();

                    let color = match kind {
                        ArcKind::Value => Color::ACCENT_LIME,
                        ArcKind::Modulation => Color::MOD_RANGE,
                        ArcKind::Progress => Color::ACCENT_BLUE,
                    };

                    let size_val = *radius + *thickness;
                    let ra = *radius / size_val;
                    let rb = (*thickness * 0.5) / size_val;

                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: glam::Vec2::splat(size_val * scale),
                        color: color.to_glam_vec4(),
                        shape_type: 2,
                        _pad: 0,
                        modulation_depth: 0.0,
                        modulation_current: 0.0,
                        params: glam::Vec4::new(sc.x, sc.y, ra, rb),
                        params2: glam::Vec4::ZERO,
                    });
                }
                SurfacePrimitive::Indicator {
                    id,
                    rect,
                    kind,
                    value,
                    color,
                    temporal,
                } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(*value),
                        target: *value,
                        strategy: *temporal,
                    });

                    let current_val = {
                        let (stiffness, damping) = match state.strategy {
                            crate::ui_ir::TemporalStrategy::Fast => (
                                crate::motion::constants::STIFFNESS_FAST,
                                crate::motion::constants::DAMPING_FAST,
                            ),
                            crate::ui_ir::TemporalStrategy::Standard => (
                                crate::motion::constants::STIFFNESS_STANDARD,
                                crate::motion::constants::DAMPING_STANDARD,
                            ),
                            crate::ui_ir::TemporalStrategy::Slow => (
                                crate::motion::constants::STIFFNESS_SLOW,
                                crate::motion::constants::DAMPING_SLOW,
                            ),
                            crate::ui_ir::TemporalStrategy::Instant => (1000000.0, 1000.0),
                        };
                        state.motion.update(*value, dt, stiffness, damping)
                    };
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(
                        rect[0] + size.x,
                        rect[1] + size.y,
                        0.0,
                    ));

                    let shape_type = if matches!(kind, IndicatorKind::Led) {
                        11
                    } else {
                        6
                    };
                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: size * transform.x_axis.truncate().length(),
                        color: glam::Vec4::from_array(*color),
                        shape_type,
                        modulation_depth: 0.0,
                        modulation_current: current_val,
                        _pad: 0,
                        params: if shape_type == 11 {
                            glam::Vec4::new(current_val, 0.0, 0.0, 0.0)
                        } else {
                            glam::Vec4::new(*kind as u32 as f32, current_val, 0.0, 0.0)
                        },
                        params2: glam::Vec4::ZERO,
                    });
                }
                SurfacePrimitive::ProvenanceBadge {
                    id,
                    rect,
                    level,
                    temporal,
                } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(1.0),
                        target: 1.0,
                        strategy: *temporal,
                    });

                    let activity = {
                        let (stiffness, damping) = match state.strategy {
                            crate::ui_ir::TemporalStrategy::Fast => (
                                crate::motion::constants::STIFFNESS_FAST,
                                crate::motion::constants::DAMPING_FAST,
                            ),
                            crate::ui_ir::TemporalStrategy::Standard => (
                                crate::motion::constants::STIFFNESS_STANDARD,
                                crate::motion::constants::DAMPING_STANDARD,
                            ),
                            crate::ui_ir::TemporalStrategy::Slow => (
                                crate::motion::constants::STIFFNESS_SLOW,
                                crate::motion::constants::DAMPING_SLOW,
                            ),
                            crate::ui_ir::TemporalStrategy::Instant => (1000000.0, 1000.0),
                        };
                        state.motion.update(1.0, dt, stiffness, damping)
                    };
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(
                        rect[0] + size.x,
                        rect[1] + size.y,
                        0.0,
                    ));

                    // Map ProvenanceLevel to PD_INDICATOR kinds
                    let (kind, color) = match level {
                        crate::ui_ir::ProvenanceLevel::Verified => {
                            (IndicatorKind::Led, Color::ACCENT_LIME)
                        }
                        crate::ui_ir::ProvenanceLevel::Inferred => {
                            (IndicatorKind::Toggle, Color::ACCENT_BLUE)
                        }
                        crate::ui_ir::ProvenanceLevel::Stale => {
                            (IndicatorKind::Radio, Color::SYNTAX_COMMENT)
                        }
                        crate::ui_ir::ProvenanceLevel::External => {
                            (IndicatorKind::Bang, Color::ERROR_RED)
                        }
                    };

                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: size * transform.x_axis.truncate().length(),
                        color: color.to_glam_vec4(),
                        shape_type: 6, // PD_INDICATOR
                        modulation_depth: 0.0,
                        modulation_current: activity,
                        _pad: 0,
                        params: glam::Vec4::new(kind as u32 as f32, activity, 0.0, 0.0),
                        params2: glam::Vec4::ZERO,
                    });
                }
                SurfacePrimitive::CausalityLink {
                    source_id: _,
                    target_id: _,
                    voice_id: _,
                    path,
                    intensity,
                    confidence,
                    relevance: _,
                    activity,
                    color,
                } => {
                    if path.len() < 2 {
                        continue;
                    }

                    for window in path.windows(2) {
                        let p0 = glam::Vec2::from_array(window[0]);
                        let p1 = glam::Vec2::from_array(window[1]);
                        let center = (p0 + p1) * 0.5;
                        let diff = p1 - p0;
                        let length = diff.length();

                        let p = transform.transform_point3(glam::vec3(center.x, center.y, 0.0));

                        instances.push(render::sdf::SdfInstance {
                            position: glam::Vec2::new(p.x, p.y),
                            size: glam::Vec2::new(length * 0.5, 4.0), // thickness 4.0
                            color: glam::Vec4::from_array(*color),
                            shape_type: 10, // CAUSALITY_LINK
                            modulation_depth: *intensity,
                            modulation_current: *activity,
                            _pad: 0,
                            params: glam::Vec4::new(-length * 0.5, 0.0, length * 0.5, 0.0), // Local segment in UV space
                            params2: glam::Vec4::new(*confidence, *activity, 0.0, 0.0),
                        });
                    }
                }
                SurfacePrimitive::FocusRing {
                    id,
                    rect,
                    color,
                    temporal,
                } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(0.0),
                        target: 1.0,
                        strategy: *temporal,
                    });

                    let activity = {
                        let (stiffness, damping) = match state.strategy {
                            crate::ui_ir::TemporalStrategy::Fast => (
                                crate::motion::constants::STIFFNESS_FAST,
                                crate::motion::constants::DAMPING_FAST,
                            ),
                            crate::ui_ir::TemporalStrategy::Standard => (
                                crate::motion::constants::STIFFNESS_STANDARD,
                                crate::motion::constants::DAMPING_STANDARD,
                            ),
                            crate::ui_ir::TemporalStrategy::Slow => (
                                crate::motion::constants::STIFFNESS_SLOW,
                                crate::motion::constants::DAMPING_SLOW,
                            ),
                            crate::ui_ir::TemporalStrategy::Instant => (1000000.0, 1000.0),
                        };
                        state.motion.update(1.0, dt, stiffness, damping)
                    };
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(
                        rect[0] + size.x,
                        rect[1] + size.y,
                        0.0,
                    ));

                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: (size + 2.0) * transform.x_axis.truncate().length(),
                        color: glam::Vec4::from_array(*color),
                        shape_type: 5, // PD_FRAME
                        modulation_depth: 0.0,
                        modulation_current: activity,
                        _pad: 0,
                        params: glam::Vec4::new(0.0, 1.0, 0.0, 0.0), // Standard frame, stroke 1.0
                        params2: glam::Vec4::ZERO,
                    });
                }
                SurfacePrimitive::ContradictionMarker {
                    id: _,
                    rect,
                    severity,
                    description: _,
                } => {
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(
                        rect[0] + size.x,
                        rect[1] + size.y,
                        0.0,
                    ));

                    let color = match severity {
                        crate::ui_ir::ContradictionSeverity::Divergence => Color::SYNTAX_COMMENT,
                        crate::ui_ir::ContradictionSeverity::Inconsistency => Color::AMBER,
                        crate::ui_ir::ContradictionSeverity::Hostile => Color::ERROR_RED,
                    };

                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: size * transform.x_axis.truncate().length(),
                        color: color.to_glam_vec4(),
                        shape_type: 6, // PD_INDICATOR
                        modulation_depth: 0.0,
                        modulation_current: 1.0,
                        _pad: 0,
                        params: glam::Vec4::new(IndicatorKind::Bang as u32 as f32, 1.0, 0.0, 0.0),
                        params2: glam::Vec4::ZERO,
                    });
                }
                _ => {}
            }
        }
    }

    fn offset_primitive(prim: &mut crate::ui_ir::SurfacePrimitive, offset: glam::Vec2) {
        use crate::ui_ir::SurfacePrimitive::*;
        match prim {
            Frame { rect, .. }
            | Indicator { rect, .. }
            | ConstraintBox { rect, .. }
            | FocusRing { rect, .. }
            | ProvenanceBadge { rect, .. }
            | ContradictionMarker { rect, .. } => {
                rect[0] += offset.x;
                rect[1] += offset.y;
            }
            Arc { center, .. } | SnapshotMarker { pos: center, .. } => {
                center[0] += offset.x;
                center[1] += offset.y;
            }
            PolyShape { points, .. } => {
                for p in points {
                    p[0] += offset.x;
                    p[1] += offset.y;
                }
            }
            _ => {}
        }
    }
}
