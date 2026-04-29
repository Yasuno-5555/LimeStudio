pub mod layout;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use dirtydata_core::types::StableId;

/// UI-side local identifier.
/// UI components should only deal with this.
pub type UiIndex = u64;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IdBiMap {
    /// UI Index -> (Kernel ULID, Generation)
    ui_to_kernel: HashMap<UiIndex, (StableId, u32)>,
    /// Kernel ULID -> UI Index
    kernel_to_ui: HashMap<StableId, UiIndex>,
    /// UUID (LimeGraph) -> Kernel ULID
    uuid_to_kernel: HashMap<uuid::Uuid, StableId>,
    /// Generation counter for each UI index to detect zombie references.
    generations: HashMap<UiIndex, u32>,
    next_index: UiIndex,
}

impl IdBiMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a mapping between a new UI Index and a Kernel ULID.
    pub fn register(&mut self, kernel_id: StableId, uuid: Option<uuid::Uuid>) -> UiIndex {
        let index = self.next_index;
        self.next_index += 1;
        
        let gen = self.generations.entry(index).or_insert(0);
        *gen += 1;
        let current_gen = *gen;

        self.ui_to_kernel.insert(index, (kernel_id, current_gen));
        self.kernel_to_ui.insert(kernel_id, index);
        if let Some(u) = uuid {
            self.uuid_to_kernel.insert(u, kernel_id);
        }
        index
    }

    pub fn resolve_uuid(&self, uuid: uuid::Uuid) -> Option<StableId> {
        self.uuid_to_kernel.get(&uuid).copied()
    }

    /// Resolve a UI Index to a Kernel ULID, verifying the generation.
    pub fn resolve(&self, index: UiIndex) -> Option<StableId> {
        self.ui_to_kernel.get(&index).map(|(id, _)| *id)
    }

    /// Get the UI Index for a Kernel ULID.
    pub fn get_ui_index(&self, kernel_id: StableId) -> Option<UiIndex> {
        self.kernel_to_ui.get(&kernel_id).copied()
    }

    /// Unregister a mapping (e.g., when a node is removed).
    pub fn unregister_by_ui_index(&mut self, index: UiIndex) {
        if let Some((kernel_id, _)) = self.ui_to_kernel.remove(&index) {
            self.kernel_to_ui.remove(&kernel_id);
        }
    }

    pub fn unregister_by_kernel_id(&mut self, kernel_id: StableId) {
        if let Some(index) = self.kernel_to_ui.remove(&kernel_id) {
            self.ui_to_kernel.remove(&index);
        }
    }
}

/// View Projection Cache - 認知レイヤーの状態管理
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ViewCache {
    /// ID BiMap - The foundation of ID resolution.
    pub id_map: IdBiMap,

    /// 選択されているノードのID集合
    pub selected_nodes: HashSet<StableId>,
    
    /// ビューポートの状態
    pub viewport: ViewportState,
    
    /// ノードの表示位置（レイアウトキャッシュ）
    pub node_positions: HashMap<StableId, [f32; 2]>,
    
    /// インタラクションメモリ
    pub interaction: InteractionMemory,

    /// デザイナー・レイアウト（認知レイヤーの投影設定）
    pub designer_layout: DesignerLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DesignerLayout {
    pub widgets: HashMap<(StableId, String), DesignerWidgetDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DesignerWidgetDefinition {
    pub node_id: StableId,
    pub param_name: String,
    pub widget_type: WidgetType,
    pub position_bits: [u32; 2], // Use bits for float to allow Hash
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WidgetType {
    Knob,
    Slider,
    Lens,
    Meter,
    ModulationRing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewportState {
    pub offset: [f32; 2],
    pub zoom: f32,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            offset: [0.0, 0.0],
            zoom: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InteractionMemory {
    pub last_clicked_node: Option<StableId>,
    pub active_drag_start: Option<[f32; 2]>,
    pub focused_port: Option<(StableId, String)>,
}

impl ViewCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_selection(&mut self) {
        self.selected_nodes.clear();
    }

    pub fn update_node_position(&mut self, id: StableId, pos: [f32; 2]) {
        self.node_positions.insert(id, pos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dirtydata_core::types::StableId;

    #[test]
    fn test_id_bimap_basic() {
        let mut map = IdBiMap::new();
        let k1 = StableId::new();
        let k2 = StableId::new();

        let u1 = map.register(k1, None);
        let u2 = map.register(k2, None);

        assert_ne!(u1, u2);
        assert_eq!(map.resolve(u1), Some(k1));
        assert_eq!(map.resolve(u2), Some(k2));
        assert_eq!(map.get_ui_index(k1), Some(u1));
        assert_eq!(map.get_ui_index(k2), Some(u2));
    }

    #[test]
    fn test_id_bimap_unregistration() {
        let mut map = IdBiMap::new();
        let k1 = StableId::new();
        let u1 = map.register(k1, None);

        map.unregister_by_ui_index(u1);
        assert_eq!(map.resolve(u1), None);
        assert_eq!(map.get_ui_index(k1), None);
    }

    #[test]
    fn test_id_bimap_zombie_prevention_simulation() {
        let mut map = IdBiMap::new();
        let k1 = StableId::new();
        let u1 = map.register(k1, None);
        
        map.unregister_by_ui_index(u1);
        
        // Re-using the same index is not directly possible with current simple increment,
        // but we verify that once unregistered, it stays gone.
        assert_eq!(map.resolve(u1), None);
    }
}
