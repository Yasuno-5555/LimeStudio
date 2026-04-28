use glam::Vec2;
use crate::model::geometry::{Rect, Circle, Segment};
use crate::model::stable_id::SurfaceId;

/// Result of a hit test operation.
#[derive(Debug, Clone, Copy)]
pub enum HitResult {
    None,
    Node(SurfaceId),
    Port(SurfaceId),
    Cable(SurfaceId),
    Widget(SurfaceId, Vec2), // Local position within widget
}


pub struct HitTester;

impl HitTester {
    /// Find a node at the given world position.
    pub fn hit_test_nodes(nodes: &[(SurfaceId, Rect)], point: Vec2) -> Option<SurfaceId> {
        // Search in reverse to hit the "top" nodes first
        for (id, rect) in nodes.iter().rev() {
            if rect.contains(point) {
                return Some(*id);
            }
        }
        None
    }

    /// Find a port at the given world position with a specific snap radius.
    pub fn hit_test_ports(ports: &[(SurfaceId, Circle)], point: Vec2, snap_radius: f32) -> Option<SurfaceId> {
        let mut best_id = None;
        let mut min_dist = snap_radius;

        for (id, circle) in ports {
            let dist = circle.center.distance(point);
            if dist < min_dist {
                min_dist = dist;
                best_id = Some(*id);
            }
        }
        best_id
    }

    /// Find a cable at the given world position.
    pub fn hit_test_cables(cables: &[(SurfaceId, Segment)], point: Vec2, threshold: f32) -> Option<SurfaceId> {
        for (id, segment) in cables {
            if segment.distance_to_point(point) < threshold {
                return Some(*id);
            }
        }
        None
    }

    /// Snap a world position to the 8px grid.
    pub fn snap_to_grid(point: Vec2) -> Vec2 {
        let grid = 8.0;
        Vec2::new(
            (point.x / grid).round() * grid,
            (point.y / grid).round() * grid,
        )
    }
}
