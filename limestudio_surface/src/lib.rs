pub mod authority;
pub mod model;
pub mod render;
pub mod adapter;
pub mod scene;
pub mod runtime;
pub mod motion;
pub mod color;
pub mod widgets;
pub mod profiler;
pub mod host_attach;
pub mod ui_ir;

use crate::model::stable_id::SurfaceId;
use crate::ui_ir::{SurfacePrimitive, TemporalStrategy, FrameStyle, ArcKind, IndicatorKind};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use crate::color::Color;
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
    pub input: runtime::input::InteractionState,
    pub profiler: profiler::FrameProfiler,
    
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
}

impl Default for SurfaceEngine {
    fn default() -> Self { Self::new() }
}

impl SurfaceEngine {
    pub fn new() -> Self {
        Self {
            scene: scene::flat_scene::SurfaceScene::new(),
            input: runtime::input::InteractionState::new(3.0),
            profiler: profiler::FrameProfiler::new(),
            primitive_stream: Arc::new(Mutex::new(Vec::new())),
            transition_store: HashMap::new(),
            selections: std::collections::HashSet::new(),
            causality_states: HashMap::new(),
            last_frame_time: Instant::now(),
            visible_compiler: authority::visible_compiler::VisibleCompilerRegistry::new(),
            authority_drawer: widgets::authority_drawer::AuthorityDrawer::new(SurfaceId(dirtydata_core::types::StableId(ulid::Ulid::nil())), glam::Vec2::new(1280.0, 720.0)),
            time_travel: authority::time_travel::TimeTravelEngine::new(64),
            causal_replay: authority::causal_replay::CausalReplayEngine::new(3.0),
            trust_ledger: model::provenance::TrustLedger::new(),
            taffy: Taffy::new(),
        }
    }

    /// Taffyを用いた本等なレイアウト解決。
    pub fn sync_ui(&mut self, tree: &crate::ui_ir::SurfaceWidget) {
        self.taffy.clear();
        
        let root_style = Style {
            size: Size { width: Dimension::Percent(1.0), height: Dimension::Percent(1.0) },
            flex_direction: FlexDirection::Column,
            ..Default::default()
        };
        
        let root_node = self.build_taffy_recursive(tree);
        let container = self.taffy.new_with_children(root_style, &[root_node]).unwrap();

        // 1280x720 を論理的な基準としてレイアウト計算
        self.taffy.compute_layout(container, Size { width: AvailableSpace::Definite(1280.0), height: AvailableSpace::Definite(720.0) }).unwrap();

        let mut primitives = Vec::new();
        self.generate_primitives_from_layout(tree, root_node, &mut primitives, glam::Vec2::ZERO);

        *self.primitive_stream.lock().unwrap() = primitives;
    }

    fn build_taffy_recursive(&mut self, widget: &crate::ui_ir::SurfaceWidget) -> Node {
        use crate::ui_ir::SurfaceWidget::*;
        match widget {
            Column { children } => {
                let child_nodes: Vec<_> = children.iter().map(|c| self.build_taffy_recursive(c)).collect();
                self.taffy.new_with_children(Style {
                    flex_direction: FlexDirection::Column,
                    size: Size { width: Dimension::Percent(1.0), height: Dimension::Auto },
                    ..Default::default()
                }, &child_nodes).unwrap()
            }
            Row { children } => {
                let child_nodes: Vec<_> = children.iter().map(|c| self.build_taffy_recursive(c)).collect();
                self.taffy.new_with_children(Style {
                    flex_direction: FlexDirection::Row,
                    size: Size { width: Dimension::Percent(1.0), height: Dimension::Auto },
                    ..Default::default()
                }, &child_nodes).unwrap()
            }
            Knob { .. } | Slider { .. } => {
                self.taffy.new_leaf(Style {
                    size: Size { width: Dimension::Points(80.0), height: Dimension::Points(80.0) },
                    margin: Rect { left: LengthPercentageAuto::Points(8.0), right: LengthPercentageAuto::Points(8.0), top: LengthPercentageAuto::Points(8.0), bottom: LengthPercentageAuto::Points(8.0) },
                    ..Default::default()
                }).unwrap()
            }
            LevelMeter { .. } => {
                self.taffy.new_leaf(Style {
                    size: Size { width: Dimension::Points(32.0), height: Dimension::Points(128.0) },
                    ..Default::default()
                }).unwrap()
            }
            XYPad { .. } => {
                self.taffy.new_leaf(Style {
                    size: Size { width: Dimension::Points(160.0), height: Dimension::Points(160.0) },
                    margin: Rect { left: LengthPercentageAuto::Points(8.0), right: LengthPercentageAuto::Points(8.0), top: LengthPercentageAuto::Points(8.0), bottom: LengthPercentageAuto::Points(8.0) },
                    ..Default::default()
                }).unwrap()
            }
            Box { children, .. } => {
                let children_nodes: Vec<_> = children.iter().map(|c| self.build_taffy_recursive(c)).collect();
                self.taffy.new_with_children(
                    Style {
                        display: Display::Flex,
                        flex_direction: FlexDirection::Column,
                        padding: Rect::points(8.0),
                        ..Default::default()
                    },
                    &children_nodes,
                ).unwrap()
            }
            Button { .. } => {
                self.taffy.new_leaf(Style {
                    size: Size { width: Dimension::Auto, height: Dimension::Points(32.0) },
                    min_size: Size { width: Dimension::Points(80.0), height: Dimension::Points(32.0) },
                    margin: Rect::points(4.0),
                    ..Default::default()
                }).unwrap()
            }
            Custom { style, .. } => {
                self.taffy.new_leaf(*style.clone()).unwrap()
            }
            _ => self.taffy.new_leaf(Style::default()).unwrap(),
        }
    }

    fn generate_primitives_from_layout(
        &self,
        widget: &crate::ui_ir::SurfaceWidget,
        node: Node,
        primitives: &mut Vec<SurfacePrimitive>,
        parent_pos: glam::Vec2,
    ) {
        let layout = self.taffy.layout(node).unwrap();
        // The Law of Lime (8px Grid & Sub-pixel Rounding)
        let pos = parent_pos + glam::vec2(layout.location.x, layout.location.y);
        let rounded_pos = (pos / 8.0).round() * 8.0;
        let size = glam::vec2(layout.size.width, layout.size.height);

        use crate::ui_ir::SurfaceWidget::*;
        match widget {
            Column { children } => {
                let node_children = self.taffy.children(node).unwrap();
                for (i, child) in children.iter().enumerate() {
                    if i < node_children.len() {
                        self.generate_primitives_from_layout(child, node_children[i], primitives, pos);
                    }
                }
            }
            Row { children } => {
                let node_children = self.taffy.children(node).unwrap();
                for (i, child) in children.iter().enumerate() {
                    if i < node_children.len() {
                        self.generate_primitives_from_layout(child, node_children[i], primitives, pos);
                    }
                }
            }
            Knob { id, label: _, signal } => {
                let value = match signal {
                    dirtydata_core::types::DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.15, 0.15, 0.15, 1.0],
                    temporal: TemporalStrategy::Standard(0.06),
                });
                primitives.push(SurfacePrimitive::Arc {
                    id: *id,
                    center: [rounded_pos.x + size.x * 0.5, rounded_pos.y + size.y * 0.5],
                    radius: size.x * 0.4,
                    thickness: 4.0,
                    start_angle: -135.0,
                    end_angle: -135.0 + (value * 270.0),
                    kind: ArcKind::Value,
                    temporal: TemporalStrategy::Standard(0.06),
                });
            }
            Slider { id, label: _, signal, is_vertical: _ } => {
                let value = match signal {
                    dirtydata_core::types::DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                primitives.push(SurfacePrimitive::Indicator {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    kind: IndicatorKind::Led,
                    value,
                    color: Color::ACCENT_LIME.to_array(),
                    temporal: TemporalStrategy::Standard(0.06),
                });
            }
            LevelMeter { id, signal } => {
                let (left, right) = match signal {
                    dirtydata_core::types::DisplaySignal::Meter { value, peak } => (*value, *peak),
                    _ => (0.0, 0.0),
                };
                let id_stable = SurfaceId::from_seed(id);
                primitives.push(SurfacePrimitive::Indicator {
                    id: id_stable,
                    rect: [rounded_pos.x, rounded_pos.y, size.x * 0.4, size.y],
                    kind: IndicatorKind::Led,
                    value: left,
                    color: [0.0, 1.0, 0.2, 1.0],
                    temporal: TemporalStrategy::Fluid(0.15),
                });
                primitives.push(SurfacePrimitive::Indicator {
                    id: id_stable,
                    rect: [rounded_pos.x + size.x * 0.6, rounded_pos.y, size.x * 0.4, size.y],
                    kind: IndicatorKind::Led,
                    value: right,
                    color: [0.0, 1.0, 0.2, 1.0],
                    temporal: TemporalStrategy::Fluid(0.15),
                });
            }
            XYPad { id, label: _, x_signal, y_signal } => {
                let x = match x_signal {
                    dirtydata_core::types::DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                let y = match y_signal {
                    dirtydata_core::types::DisplaySignal::Linear(v) => *v,
                    _ => 0.0,
                };
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: [0.15, 0.15, 0.15, 1.0],
                    temporal: TemporalStrategy::Standard(0.06),
                });
                // Pad Crosshair
                primitives.push(SurfacePrimitive::Indicator {
                    id: *id,
                    rect: [rounded_pos.x + x * size.x - 4.0, rounded_pos.y + y * size.y - 4.0, 8.0, 8.0],
                    kind: IndicatorKind::Led,
                    value: 1.0,
                    color: Color::ACCENT_LIME.to_array(),
                    temporal: TemporalStrategy::Standard(0.02),
                });
            }
            Box { children: _, style } => {
                primitives.push(SurfacePrimitive::Frame {
                    id: SurfaceId::generate(),
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: *style,
                    color: [0.2, 0.2, 0.2, 0.8],
                    temporal: TemporalStrategy::Standard(0.06),
                });
            }
            Button { id, label: _, is_active } => {
                primitives.push(SurfacePrimitive::Frame {
                    id: *id,
                    rect: [rounded_pos.x, rounded_pos.y, size.x, size.y],
                    style: FrameStyle::Standard,
                    color: if *is_active { [0.4, 0.4, 0.4, 1.0] } else { [0.3, 0.3, 0.3, 1.0] },
                    temporal: TemporalStrategy::Standard(0.04),
                });
                // TODO: Render text label
            }
            Custom { primitives: custom_primitives, .. } => {
                for cp in custom_primitives {
                    let mut cp_cloned = cp.clone();
                    Self::offset_primitive(&mut cp_cloned, rounded_pos);
                    primitives.push(cp_cloned);
                }
            }
            _ => {}
        }
    }

    pub fn generate_instances(&mut self) -> Vec<render::sdf::SdfInstance> {
        let primitives = self.primitive_stream.lock().unwrap().clone();
        let mut instances = Vec::new();
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // V7 Recursive Semantic Rendering
        self.generate_instances_recursive(&primitives, &mut instances, glam::Mat4::IDENTITY, dt, now);

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
                SurfacePrimitive::Frame { id: _, rect, style, color, .. } => {
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(rect[0] + size.x, rect[1] + size.y, 0.0));
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
                SurfacePrimitive::Arc { id, center, radius, thickness, start_angle, end_angle, kind, .. } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(*end_angle),
                        target: *end_angle,
                        strategy: TemporalStrategy::Standard(0.06),
                    });
                    let current_end = Self::update_transition(state, *end_angle, dt);
                    
                    let half_angle = (current_end - start_angle).to_radians() * 0.5;
                    let sc = glam::Vec2::new(half_angle.sin(), half_angle.cos());
                    let p = transform.transform_point3(glam::vec3(center[0], center[1], 0.0));
                    let scale = transform.x_axis.truncate().length();

                    let color = match kind {
                        ArcKind::Value => Color::ACCENT_LIME,
                        ArcKind::Modulation => Color::MOD_RANGE,
                        ArcKind::Progress => Color::ACCENT_BLUE,
                    };

                    instances.push(render::sdf::SdfInstance {
                        position: glam::Vec2::new(p.x, p.y),
                        size: glam::Vec2::new(*radius + *thickness, *radius + *thickness) * scale,
                        color: color.to_glam_vec4(), 
                        shape_type: 2, 
                        _pad: 0,
                        modulation_depth: 0.0,
                        modulation_current: 0.0,
                        params: glam::Vec4::new(sc.x, sc.y, *radius * scale, *thickness * 0.5 * scale),
                        params2: glam::Vec4::ZERO,
                    });
                }
                SurfacePrimitive::Indicator { id, rect, kind, value, color, temporal } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(*value),
                        target: *value,
                        strategy: *temporal,
                    });
                    
                    let current_val = Self::update_transition(state, *value, dt);
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(rect[0] + size.x, rect[1] + size.y, 0.0));

                    let shape_type = if matches!(kind, IndicatorKind::Led) { 11 } else { 6 };
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
                SurfacePrimitive::ProvenanceBadge { id, rect, level, temporal } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(1.0),
                        target: 1.0,
                        strategy: *temporal,
                    });
                    
                    let activity = Self::update_transition(state, 1.0, dt);
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(rect[0] + size.x, rect[1] + size.y, 0.0));

                    // Map ProvenanceLevel to PD_INDICATOR kinds
                    let (kind, color) = match level {
                        crate::ui_ir::ProvenanceLevel::Verified => (IndicatorKind::Led, Color::ACCENT_LIME),
                        crate::ui_ir::ProvenanceLevel::Inferred => (IndicatorKind::Toggle, Color::ACCENT_BLUE),
                        crate::ui_ir::ProvenanceLevel::Stale => (IndicatorKind::Radio, Color::SYNTAX_COMMENT),
                        crate::ui_ir::ProvenanceLevel::External => (IndicatorKind::Bang, Color::ERROR_RED),
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
                SurfacePrimitive::CausalityLink { source_id: _, target_id: _, voice_id: _, path, intensity, confidence, relevance: _, activity, color } => {
                    if path.len() < 2 { continue; }
                    
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
                SurfacePrimitive::FocusRing { id, rect, color, temporal } => {
                    let state = self.transition_store.entry(*id).or_insert(TransitionState {
                        motion: crate::motion::MotionState::new(0.0),
                        target: 1.0,
                        strategy: *temporal,
                    });
                    
                    let activity = Self::update_transition(state, 1.0, dt);
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(rect[0] + size.x, rect[1] + size.y, 0.0));

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
                SurfacePrimitive::ContradictionMarker { id: _, rect, severity, description: _ } => {
                    let size = glam::Vec2::new(rect[2] * 0.5, rect[3] * 0.5);
                    let p = transform.transform_point3(glam::vec3(rect[0] + size.x, rect[1] + size.y, 0.0));
                    
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

    fn update_transition(state: &mut TransitionState, target: f32, dt: f32) -> f32 {
        use crate::motion::constants::*;
        match state.strategy {
            TemporalStrategy::Instant => {
                state.motion.value = target;
                state.motion.velocity = 0.0;
                target
            }
            _ => {
                let stiffness = match state.strategy {
                    TemporalStrategy::Fast(_) => STIFFNESS_FAST,
                    TemporalStrategy::Standard(_) => STIFFNESS_NORMAL,
                    TemporalStrategy::Fluid(_) => STIFFNESS_SLOW,
                    _ => STIFFNESS_NORMAL,
                };
                state.motion.update(target, dt, stiffness)
            }
        }
    }

    fn offset_primitive(prim: &mut crate::ui_ir::SurfacePrimitive, offset: glam::Vec2) {
        use crate::ui_ir::SurfacePrimitive::*;
        match prim {
            Frame { rect, .. } | Indicator { rect, .. } | ConstraintBox { rect, .. } | FocusRing { rect, .. } | ProvenanceBadge { rect, .. } | ContradictionMarker { rect, .. } => {
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
