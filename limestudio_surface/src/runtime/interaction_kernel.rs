use glam::Vec2;
use crate::model::stable_id::SurfaceId;
use crate::model::geometry::{Rect, Circle};
use crate::scene::camera::InfiniteCamera;
use crate::model::hit_test::{HitTester, HitResult};
use crate::runtime::input::{SurfaceEvent, MouseButton};

#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    pub ids: Vec<SurfaceId>,
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
        origin: Vec2, // World space
        current: Vec2, // World space
    },
    /// Dragging to create a connection.
    Connecting {
        from_port: SurfaceId,
        current_pos: Vec2, // World space
    },
}

pub struct InteractionKernel {
    pub session: DragSession,
    pub selection: SelectionState,
    pub last_world_pos: Vec2,
}

impl Default for InteractionKernel {
    fn default() -> Self { Self::new() }
}

impl InteractionKernel {
    pub fn new() -> Self {
        Self {
            session: DragSession::None,
            selection: SelectionState::default(),
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
    ) {
        match event {
            SurfaceEvent::PointerDown { position, button: MouseButton::Left, modifiers } => {
                let world_pos = camera.screen_to_world(*position);
                self.last_world_pos = world_pos;

                // Hit testing prioritized by Ports -> Nodes -> Empty Canvas
                let hit_result = if let Some(port_id) = HitTester::hit_test_ports(ports, world_pos, 12.0) {
                    HitResult::Port(port_id)
                } else if let Some(node_id) = HitTester::hit_test_nodes(nodes, world_pos) {
                    HitResult::Node(node_id)
                } else {
                    HitResult::None
                };

                // TODO: Get Shift/Ctrl modifiers from SurfaceEvent if added
                self.handle_down(world_pos, hit_result, modifiers.shift);
            }
            SurfaceEvent::PointerMove { position, .. } => {
                let world_pos = camera.screen_to_world(*position);
                self.handle_move(world_pos);
            }
            SurfaceEvent::PointerUp { .. } => {
                self.handle_up(nodes);
            }
            _ => {}
        }
    }

    fn handle_down(&mut self, world_pos: Vec2, hit_result: HitResult, is_shift: bool) {
        match hit_result {
            HitResult::Node(id) => {
                if !is_shift {
                    if !self.selection.ids.contains(&id) {
                        self.selection.ids = vec![id];
                    }
                } else if let Some(pos) = self.selection.ids.iter().position(|&x| x == id) {
                    self.selection.ids.remove(pos);
                } else {
                    self.selection.ids.push(id);
                }
                
                // Grab offset would ideally use the node's current center.
                // For now, we store ZERO and the caller can calculate it.
                self.session = DragSession::MovingNode { id, grab_offset: Vec2::ZERO };
            }
            HitResult::Port(id) => {
                self.session = DragSession::Connecting { from_port: id, current_pos: world_pos };
            }
            HitResult::None => {
                if !is_shift {
                    self.selection.ids.clear();
                }
                self.session = DragSession::BoxSelecting { origin: world_pos, current: world_pos };
            }
            _ => {}
        }
    }

    fn handle_move(&mut self, world_pos: Vec2) {
        self.last_world_pos = world_pos;
        match &mut self.session {
            DragSession::MovingNode { .. } => {
                // Dragging logic is reported via the session state.
            }
            DragSession::BoxSelecting { current, .. } => {
                *current = world_pos;
            }
            DragSession::Connecting { current_pos, .. } => {
                *current_pos = world_pos;
            }
            _ => {}
        }
    }

    fn handle_up(&mut self, nodes: &[(SurfaceId, Rect)]) {
        if let DragSession::BoxSelecting { origin, current } = self.session {
            let selection_rect = Rect::from_points(origin, current);
            for (id, rect) in nodes {
                if selection_rect.intersects(*rect) && !self.selection.ids.contains(id) {
                    self.selection.ids.push(*id);
                }
            }
        }
        self.session = DragSession::None;
    }
}