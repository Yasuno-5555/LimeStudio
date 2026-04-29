use crate::model::geometry::{Circle, Rect};
use crate::model::hit_test::{HitResult, HitTester};
use crate::model::stable_id::SurfaceId;
use crate::runtime::input::{MouseButton, SurfaceEvent};
use crate::scene::camera::InfiniteCamera;
use glam::Vec2;

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub ids: Vec<SurfaceId>,
}

#[derive(Debug, Clone)]
pub enum InteractionIntent {
    Connect {
        from: SurfaceId,
        to: SurfaceId,
    },
    SeekHistory {
        progress: f32,
    },
    Select {
        ids: Vec<SurfaceId>,
    },
    MoveNode {
        id: SurfaceId,
        delta: Vec2,
    },
    UpdateParameter {
        node_id: SurfaceId,
        parameter: String,
        value: f32,
    },
    ContextClick {
        id: SurfaceId,
        position: glam::Vec2,
    },
    CompileCode {
        node_id: SurfaceId,
        source: String,
    },
    Commit,
    Cancel,
    DeleteSelected,
}

#[derive(Debug, Clone)]
pub enum DragSession {
    None,
    /// Dragging a node (or multiple nodes).
    MovingNode {
        id: SurfaceId,
        /// Offset from the node center where it was grabbed.
        grab_offset: Vec2,
    },
    /// Dragging to create a selection box.
    BoxSelecting {
        origin: Vec2,  // World space
        current: Vec2, // World space
    },
    /// Dragging to create a connection.
    Connecting {
        from_port: SurfaceId,
        current_pos: Vec2, // World space
    },
    /// Dragging a knob to change a value.
    KnobDragging {
        id: SurfaceId,
        parameter: String,
        origin_y: f32,
        base_value: f32,
    },
}

pub struct InteractionKernel {
    pub session: DragSession,
    pub selection: SelectionState,
    pub focused_id: Option<SurfaceId>,
    pub last_world_pos: Vec2,
}

impl Default for InteractionKernel {
    fn default() -> Self {
        Self::new()
    }
}

impl InteractionKernel {
    pub fn new() -> Self {
        Self {
            session: DragSession::None,
            selection: SelectionState::default(),
            focused_id: None,
            last_world_pos: Vec2::ZERO,
        }
    }

    /// Entry point for all pointer events.
    pub fn handle_event(
        &mut self,
        event: &SurfaceEvent,
        camera: &InfiniteCamera,
        nodes: &[(SurfaceId, Rect)],
        ports: &[(SurfaceId, Circle)],
        widgets: &[(SurfaceId, Rect, crate::ui_ir::InteractionClass)],
    ) -> Vec<InteractionIntent> {
        match event {
            SurfaceEvent::PointerDown {
                position,
                button: MouseButton::Left,
                modifiers,
            } => {
                let world_pos = camera.screen_to_world(*position);
                self.last_world_pos = world_pos;

                // 0. Widgets (Highest Priority) - Bubbling Support
                let hits = HitTester::hit_test_all_nodes_v2(widgets, world_pos);
                for id in hits {
                    let (_, rect, class) = widgets.iter().find(|(rid, _, _)| rid == &id).unwrap();
                    let local_pos = world_pos - rect.min();

                    match class {
                        crate::ui_ir::InteractionClass::Knob => {
                            self.session = DragSession::KnobDragging {
                                id,
                                parameter: "unnamed".to_string(),
                                origin_y: world_pos.y,
                                base_value: 0.5,
                            };
                            return vec![];
                        }
                        crate::ui_ir::InteractionClass::Slider => {
                            // TODO: Add slider dragging session
                            return vec![];
                        }
                        crate::ui_ir::InteractionClass::Button => {
                            // For now, simple clicks are handled as Seek if it's a history bar,
                            // but here we should probably emit a Click intent.
                            if id.0 .0.to_string().contains("compile") {
                                return vec![InteractionIntent::CompileCode {
                                    node_id: id,
                                    source: "// Edited Code...".to_string(),
                                }];
                            }
                            let progress = (local_pos.x / rect.size.x).clamp(0.0, 1.0);
                            return vec![InteractionIntent::SeekHistory { progress }];
                        }
                        crate::ui_ir::InteractionClass::Draggable => {
                            // Generic draggable (e.g. Scroll)
                            continue; // Bubbling: let child or other sibling handle it if more specific
                        }
                        _ => {}
                    }
                }

                // 1. Ports -> 2. Nodes -> 3. Empty Canvas
                let hit_result =
                    if let Some(port_id) = HitTester::hit_test_ports(ports, world_pos, 12.0) {
                        HitResult::Port(port_id)
                    } else if let Some(node_id) = HitTester::hit_test_nodes(nodes, world_pos) {
                        HitResult::Node(node_id)
                    } else {
                        HitResult::None
                    };

                self.handle_down(world_pos, hit_result, modifiers.shift)
            }
            SurfaceEvent::PointerDown {
                position,
                button: MouseButton::Right,
                ..
            } => {
                let world_pos = camera.screen_to_world(*position);
                let hits = HitTester::hit_test_all_nodes_v2(widgets, world_pos);
                for id in hits {
                    return vec![InteractionIntent::ContextClick {
                        id,
                        position: world_pos,
                    }];
                }
                vec![]
            }
            SurfaceEvent::PointerMove { position, .. } => {
                let world_pos = camera.screen_to_world(*position);
                self.handle_move(world_pos)
            }
            SurfaceEvent::PointerUp { .. } => self.handle_up(nodes, ports),
            SurfaceEvent::KeyInput {
                key,
                pressed: true,
                modifiers,
            } => self.handle_key(*key, modifiers, widgets),
            _ => vec![],
        }
    }

    fn handle_key(
        &mut self,
        key: crate::runtime::input::Key,
        modifiers: &crate::runtime::input::Modifiers,
        widgets: &[(SurfaceId, Rect, crate::ui_ir::InteractionClass)],
    ) -> Vec<InteractionIntent> {
        use crate::runtime::input::Key;
        let mut intents = Vec::new();

        match key {
            Key::Tab => {
                if widgets.is_empty() {
                    return vec![];
                }

                // HIG 5.3: Tab navigation
                let current_idx = self
                    .focused_id
                    .and_then(|id| widgets.iter().position(|(wid, _, _)| wid == &id));

                let next_idx = if modifiers.shift {
                    // Shift+Tab: Previous
                    current_idx
                        .map(|i| if i == 0 { widgets.len() - 1 } else { i - 1 })
                        .unwrap_or(widgets.len() - 1)
                } else {
                    // Tab: Next
                    current_idx.map(|i| (i + 1) % widgets.len()).unwrap_or(0)
                };

                self.focused_id = Some(widgets[next_idx].0);
            }
            Key::Enter => {
                intents.push(InteractionIntent::Commit);
            }
            Key::Escape => {
                intents.push(InteractionIntent::Cancel);
                self.focused_id = None;
            }
            Key::Delete | Key::Backspace => {
                intents.push(InteractionIntent::DeleteSelected);
            }
            _ => {}
        }

        intents
    }

    fn handle_down(
        &mut self,
        world_pos: Vec2,
        hit_result: HitResult,
        is_shift: bool,
    ) -> Vec<InteractionIntent> {
        let mut intents = Vec::new();
        match hit_result {
            HitResult::Node(id) => {
                if !is_shift {
                    if !self.selection.ids.contains(&id) {
                        self.selection.ids = vec![id];
                        intents.push(InteractionIntent::Select {
                            ids: self.selection.ids.clone(),
                        });
                    }
                } else if let Some(pos) = self.selection.ids.iter().position(|&x| x == id) {
                    self.selection.ids.remove(pos);
                    intents.push(InteractionIntent::Select {
                        ids: self.selection.ids.clone(),
                    });
                } else {
                    self.selection.ids.push(id);
                    intents.push(InteractionIntent::Select {
                        ids: self.selection.ids.clone(),
                    });
                }

                self.session = DragSession::MovingNode {
                    id,
                    grab_offset: Vec2::ZERO,
                };
            }
            HitResult::Port(id) => {
                self.session = DragSession::Connecting {
                    from_port: id,
                    current_pos: world_pos,
                };
            }
            HitResult::None => {
                if !is_shift {
                    self.selection.ids.clear();
                    intents.push(InteractionIntent::Select { ids: vec![] });
                }
                self.session = DragSession::BoxSelecting {
                    origin: world_pos,
                    current: world_pos,
                };
            }
            _ => {}
        }
        intents
    }

    fn handle_move(&mut self, world_pos: Vec2) -> Vec<InteractionIntent> {
        let mut intents = Vec::new();
        let delta = world_pos - self.last_world_pos;
        self.last_world_pos = world_pos;

        match &mut self.session {
            DragSession::MovingNode { id, .. } => {
                intents.push(InteractionIntent::MoveNode { id: *id, delta });
            }
            DragSession::BoxSelecting { current, .. } => {
                *current = world_pos;
            }
            DragSession::Connecting { current_pos, .. } => {
                *current_pos = world_pos;
            }
            DragSession::KnobDragging {
                id,
                parameter,
                origin_y,
                base_value,
            } => {
                let delta_y = *origin_y - world_pos.y;
                let value = (*base_value + delta_y * 0.005).clamp(0.0, 1.0);
                intents.push(InteractionIntent::UpdateParameter {
                    node_id: *id, // Hack: using knob id as node id for now
                    parameter: parameter.clone(),
                    value,
                });
            }
            _ => {}
        }

        intents
    }

    fn handle_up(
        &mut self,
        nodes: &[(SurfaceId, Rect)],
        ports: &[(SurfaceId, Circle)],
    ) -> Vec<InteractionIntent> {
        let mut intents = Vec::new();
        match &self.session {
            DragSession::BoxSelecting { origin, current } => {
                let selection_rect = Rect::from_points(*origin, *current);
                for (id, rect) in nodes {
                    if selection_rect.intersects(*rect) && !self.selection.ids.contains(id) {
                        self.selection.ids.push(*id);
                    }
                }
                intents.push(InteractionIntent::Select {
                    ids: self.selection.ids.clone(),
                });
            }
            DragSession::Connecting {
                from_port,
                current_pos,
            } => {
                if let Some(to_port) = HitTester::hit_test_ports(ports, *current_pos, 16.0) {
                    if *from_port != to_port {
                        intents.push(InteractionIntent::Connect {
                            from: *from_port,
                            to: to_port,
                        });
                    }
                }
            }
            _ => {}
        }
        self.session = DragSession::None;
        intents
    }
}
