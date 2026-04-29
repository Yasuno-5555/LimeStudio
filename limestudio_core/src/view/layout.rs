//! Node Layout Logic — The Geometry of Trust.

pub struct NodeLayout;

impl NodeLayout {
    pub const DEFAULT_WIDTH: f32 = 120.0;
    pub const DEFAULT_HEIGHT: f32 = 48.0;
    pub const PORT_RADIUS: f32 = 4.0;
    pub const GRID_STEP: f32 = 8.0;
    pub const PADDING: f32 = 8.0; // 1 unit

    /// Snap any float to the nearest 8px grid point.
    #[inline]
    pub fn snap(v: f32) -> f32 {
        (v / Self::GRID_STEP).round() * Self::GRID_STEP
    }

    /// Calculate the bounding box for a node.
    pub fn calculate_node_rect(name: &str, _num_inputs: usize, _num_outputs: usize) -> [f32; 4] {
        // Dynamic width based on name length, snapped to 8px grid
        let char_width = 8.0;
        let base_width = (name.len() as f32 * char_width) + (Self::PADDING * 2.0);
        let width = Self::snap(base_width).max(Self::DEFAULT_WIDTH);

        [0.0, 0.0, width, Self::DEFAULT_HEIGHT]
    }

    /// Get the absolute position of a port.
    pub fn get_port_position(
        node_pos: [f32; 2],
        node_rect: [f32; 4],
        port_index: usize,
        num_ports: usize,
        is_input: bool,
    ) -> [f32; 2] {
        let width = node_rect[2];
        let height = node_rect[3];

        let x_offset = if num_ports == 1 {
            width / 2.0
        } else {
            (port_index as f32 / (num_ports - 1) as f32) * width
        };

        let y_offset = if is_input { 0.0 } else { height };

        [node_pos[0] + x_offset, node_pos[1] + y_offset]
    }
}
