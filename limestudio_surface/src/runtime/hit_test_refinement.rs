use glam::Vec2;
use std::collections::HashMap;
use crate::model::stable_id::SurfaceId;
use crate::model::geometry::{Rect, Circle};

/// §HTR: High-Precision Hit Testing Refinement
/// 
/// O(N) の線形探索を脱却し、グリッドベースの空間分割を用いて
/// ノード密集地帯でも 60fps を維持したまま正確な選択を可能にする。

pub struct SpatialGrid {
    cell_size: f32,
    /// Grid cells mapping to a list of Node IDs.
    cells: HashMap<(i32, i32), Vec<SurfaceId>>,
    /// Fast lookup for node geometries.
    geometries: HashMap<SurfaceId, Rect>,
}

impl SpatialGrid {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
            geometries: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
        self.geometries.clear();
    }

    pub fn insert_node(&mut self, id: SurfaceId, rect: Rect) {
        self.geometries.insert(id, rect);
        
        let min_x = (rect.min.x / self.cell_size).floor() as i32;
        let max_x = (rect.max.x / self.cell_size).floor() as i32;
        let min_y = (rect.min.y / self.cell_size).floor() as i32;
        let max_y = (rect.max.y / self.cell_size).floor() as i32;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                self.cells.entry((x, y)).or_default().push(id);
            }
        }
    }

    pub fn hit_test(&self, point: Vec2) -> Option<SurfaceId> {
        let x = (point.x / self.cell_size).floor() as i32;
        let y = (point.y / self.cell_size).floor() as i32;

        if let Some(candidates) = self.cells.get(&(x, y)) {
            // セル内の候補を背面（挿入順の逆）からチェック
            for &id in candidates.iter().rev() {
                if let Some(rect) = self.geometries.get(&id) {
                    if rect.contains(point) {
                        return Some(id);
                    }
                }
            }
        }
        None
    }
}

/// §HPS: Precise Port Snapping
/// ポート選択は極小領域のため、近傍4セルを検索対象にする。
pub struct PortSnapper {
    grid: SpatialGrid,
    ports: HashMap<SurfaceId, Circle>,
}

impl PortSnapper {
    pub fn new(cell_size: f32) -> Self {
        Self {
            grid: SpatialGrid::new(cell_size),
            ports: HashMap::new(),
        }
    }

    pub fn insert_port(&mut self, id: SurfaceId, circle: Circle) {
        self.ports.insert(id, circle);
        // ポートの中心点を囲む矩形として登録
        let rect = Rect::from_center_size(circle.center, Vec2::splat(circle.radius * 2.0));
        self.grid.insert_node(id, rect);
    }

    pub fn find_best_port(&self, point: Vec2, snap_radius: f32) -> Option<SurfaceId> {
        let x = (point.x / self.grid.cell_size).floor() as i32;
        let y = (point.y / self.grid.cell_size).floor() as i32;

        let mut best_id = None;
        let mut min_dist = snap_radius;

        // ポートは小さいので、隣接セルもチェック
        for dx in -1..=1 {
            for dy in -1..=1 {
                if let Some(candidates) = self.grid.cells.get(&(x + dx, y + dy)) {
                    for &id in candidates {
                        if let Some(port) = self.ports.get(&id) {
                            let dist = port.center.distance(point);
                            if dist < min_dist {
                                min_dist = dist;
                                best_id = Some(id);
                            }
                        }
                    }
                }
            }
        }
        best_id
    }
}
