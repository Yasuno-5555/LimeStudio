//! Surface UI IR - Minimal structure for reactive reconciliation.
//! "Observation is a pure function. Reality is the state."

use serde::{Serialize, Deserialize};

pub use crate::model::stable_id::SurfaceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DisplaySignal {
    Static(f32),
    Linear(f32),
    Meter { value: f32, peak: f32 },
    Reactive { source: SurfaceId, factor: f32 },
    Forensic { metric: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceAccessibilityData {
    pub label: String,
    pub role: SemanticRole,
    pub description: Option<String>,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SemanticRole {
    None,
    Button,
    Slider,
    Knob,
    NumberBox,
    TextInput,
    Heading,
    Status,
    Alert,
    Terminal,
    Node,
    Canvas,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfaceWidget {
    DataTable {
        id: SurfaceId,
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
    },
    TreeView {
        id: SurfaceId,
        nodes: Vec<TreeNode>,
    },
    Column {
        children: Vec<SurfaceWidget>
    },
    Terminal {
        id: SurfaceId,
        history: Vec<String>,
        current_input: String,
    },
    Row {
        children: Vec<SurfaceWidget>
    },
    Box {
        children: Vec<SurfaceWidget>,
        style: FrameStyle,
    },
    Button {
        id: SurfaceId,
        label: String,
        is_active: bool,
    },
    Label {
        text: String,
        is_secondary: bool,
    },
    Knob {
        id: SurfaceId,
        label: String,
        signal: DisplaySignal,
    },
    Slider {
        id: SurfaceId,
        label: String,
        signal: DisplaySignal,
        is_vertical: bool,
    },
    LevelMeter {
        id: String,
        signal: DisplaySignal,
    },
    Waveform {
        id: String,
        data: Vec<f32>,
    },
    Spectrum {
        id: String,
        data: Vec<f32>,
    },
    CodeView {

        code: String,
        language: String,
    },
    XYPad {
        id: SurfaceId,
        label: String,
        x_signal: DisplaySignal,
        y_signal: DisplaySignal,
    },
    /// Custom: User-defined widget with explicit layout and primitives.
    /// This is the escape hatch for "Flutter-like" extensibility.
    Custom {
        id: SurfaceId,
        style: Box<taffy::style::Style>,
        primitives: Vec<SurfacePrimitive>,
    },
    /// Low-level Semantic Primitive Stream
    PrimitiveStream {
        primitives: Vec<SurfacePrimitive>,
    },
    ForensicMonitor {
        id: SurfaceId,
        data: TelemetryData,
    },
    Timeline {
        id: SurfaceId,
        snapshots: Vec<String>,
        current_idx: usize,
    },
    /// Focused: Special wrapper or state for focusing.
    /// In Lime, focus is a first-class citizen.
    FocusProxy {
        id: SurfaceId,
        child: Box<SurfaceWidget>,
        is_focused: bool,
    },
    /// Accessibility Wrapper: Purely for providing A11y context
    Accessibility {
        data: SurfaceAccessibilityData,
        child: Box<SurfaceWidget>,
    },
}

impl SurfaceWidget {
    pub fn id(&self) -> Option<&SurfaceId> {
        match self {
            Self::DataTable { id, .. } => Some(id),
            Self::TreeView { id, .. } => Some(id),
            Self::Terminal { id, .. } => Some(id),
            Self::Button { id, .. } => Some(id),
            Self::Knob { id, .. } => Some(id),
            Self::Slider { id, .. } => Some(id),
            Self::XYPad { id, .. } => Some(id),
            Self::Custom { id, .. } => Some(id),
            Self::ForensicMonitor { id, .. } => Some(id),
            Self::Timeline { id, .. } => Some(id),
            _ => None,
        }
    }
}



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SurfacePrimitive {
    /// Frame: Standardized UI regions
    Frame {
        id: SurfaceId,
        rect: [f32; 4],
        style: FrameStyle,
        color: [f32; 4],
        temporal: TemporalStrategy,
    },
    /// Arc: Value representation
    Arc {
        id: SurfaceId,
        center: [f32; 2],
        radius: f32,
        thickness: f32,
        start_angle: f32,
        end_angle: f32,
        kind: ArcKind,
        temporal: TemporalStrategy,
    },
    /// Indicator: Momentary or Binary status
    Indicator {
        id: SurfaceId,
        rect: [f32; 4],
        kind: IndicatorKind,
        value: f32,
        color: [f32; 4],
        temporal: TemporalStrategy,
    },
    /// Connector: Typed IO Point (The Law)
    Connector {
        id: SurfaceId,
        pos: [f32; 2],
        signal_type: SignalType,
        state: ConnectorState,
    },
    /// Curve: Semantic Vector Path (Quadratic Bezier based)
    Curve {
        id: SurfaceId,
        control_points: Vec<[f32; 2]>,
        kind: CurveKind,
        thickness: f32,
        color: [f32; 4],
        temporal: TemporalStrategy,
    },
    /// CausalityLink: Visualizing "Why" (Trust UI)
    CausalityLink {
        source_id: SurfaceId,
        target_id: SurfaceId,
        voice_id: Option<u32>,
        path: Vec<[f32; 2]>,
        intensity: f32,
        confidence: f32,
        relevance: f32,
        activity: f32,
        color: [f32; 4],
    },
    /// ClipMask: Spatial cutting (Doctrine of Overlap)
    ClipMask {
        id: SurfaceId,
        rect: [f32; 4],
        law: OverlapLaw,
        children: Vec<SurfacePrimitive>,
    },
    /// PolyShape: Filled area (EQ curves, Waveforms)
    PolyShape {
        id: SurfaceId,
        points: Vec<[f32; 2]>,
        law: OverlapLaw,
        color: [f32; 4],
    },
    /// DensityMap: Raster stream (Spectrogram, Oscilloscope)
    DensityMap {
        id: SurfaceId,
        rect: [f32; 4],
        policy: ProjectionPolicy,
        // Data is passed via a specialized side-channel in implementation
    },
    /// TransformNode: Affine transformation group
    TransformNode {
        id: SurfaceId,
        translation: [f32; 2],
        rotation: f32,
        scale: f32,
        children: Vec<SurfacePrimitive>,
    },
    /// GradientMap: Data-driven semantic color flow
    GradientMap {
        id: SurfaceId,
        rect: [f32; 4],
        stops: Vec<([f32; 4], f32)>, // (Color, Position)
        law: OverlapLaw,
    },
    /// GlyphRun: Shaped text from CPU
    GlyphRun {
        placements: Vec<GlyphPlacement>,
        color: [f32; 4],
    },
    /// Text: High-level text rendering
    Text {
        id: SurfaceId,
        rect: [f32; 4],
        text: String,
        font_size: f32,
        color: [f32; 4],
    },

    /// ConstraintBox: Layout semantic primitive (The Place)
    ConstraintBox {
        id: SurfaceId,
        rect: [f32; 4],
        policy: ConstraintPolicy,
    },
    /// FocusRing: Accessibility primitive (The Civilization)
    FocusRing {
        id: SurfaceId,
        rect: [f32; 4],
        color: [f32; 4],
        temporal: TemporalStrategy,
    },
    /// ProvenanceBadge: Trust primitive (Verified/Inferred/Stale/External)
    ProvenanceBadge {
        id: SurfaceId,
        rect: [f32; 4],
        level: ProvenanceLevel,
        temporal: TemporalStrategy,
    },
    /// SnapshotMarker: Forensic primitive (Evidence pins on the timeline)
    SnapshotMarker {
        id: SurfaceId,
        pos: [f32; 2],
        kind: SnapshotKind,
    },
    /// ContradictionMarker: Visualizing inconsistencies (The Discord)
    ContradictionMarker {
        id: SurfaceId,
        rect: [f32; 4],
        severity: ContradictionSeverity,
        description: String,
    },
    /// AuthorityDiff: Before/After comparison of changes
    AuthorityDiff {
        id: SurfaceId,
        rect: [f32; 4],
        before_hash: [u8; 32],
        after_hash: [u8; 32],
        changes: Vec<DiffChange>,
    },
    /// SdfTexture: RESTRICTED. Only if semantic reduction is impossible.
    /// Used for: Waveform cache, spectrogram raster, brand marks, MSDF icon atlas.
    SdfTexture {
        id: SurfaceId,
        rect: [f32; 4],
        texture_id: u32,
        color: [f32; 4],
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContradictionSeverity {
    /// Divergence: Minor difference in perception.
    Divergence,
    /// Inconsistency: Logical conflict.
    Inconsistency,
    /// Hostile: Active reality corruption.
    Hostile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeNode {
    pub id: SurfaceId,
    pub label: String,
    pub children: Vec<TreeNode>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffChange {
    Property { name: String, old: String, new: String },
    Causal { source: SurfaceId, target: SurfaceId, action: CausalAction },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CausalAction {
    Rewired,
    Broken,
    Prioritized,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConstraintPolicy {
    /// Strict 8px grid alignment
    Grid8,
    /// Semantic center (Anchor to logic)
    SemanticCenter,
    /// Adaptive boundary
    Adaptive,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProvenanceLevel {
    /// Verified: System-guaranteed truth.
    Verified,
    /// Inferred: Predicted or extrapolated.
    Inferred,
    /// Stale: Outdated or potentially invalid.
    Stale,
    /// External: Foreign or unverified source.
    External,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SnapshotKind {
    Evidence,
    Checkpoint,
    Trace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OverlapLaw {
    /// Law of Total Response: Individual components are lines, union is solid.
    TotalResponse,
    /// Semantic Occlusion: Front hides back with an SDF-negative margin.
    SemanticOcclusion,
    /// Boolean Intersection: Overlap creates a third solid state (Lightness shift).
    BooleanIntersection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ProjectionPolicy {
    /// Instant: One slice per frame.
    Instant,
    /// HistoricalWindow: Batch processing of all slices since last frame.
    HistoricalWindow,
    /// ForensicFreeze: Locked snapshot.
    ForensicFreeze,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TemporalStrategy {
    /// Instant: Snap to latest truth (Playhead, Trigger)
    Instant,
    /// Fast: Stiff 600, Damping 50 (Button 반응など)
    Fast,
    /// Standard: Stiff 300, Damping 35 (一般的な遷移)
    Standard,
    /// Slow: Stiff 150, Damping 25 (背景の変化など)
    Slow,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FrameStyle {

    Standard, // Pd Object
    Message,  // Pd Message (Flag)
    Number,   // Pd Number (Chamfered)
    Field,    // Edit mode
    None,     // Comment
    /// AuthorityGlass: PRIVILEGED. Authority Drawer / Forensic Panels ONLY.
    AuthorityGlass, 
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ArcKind {
    Value,
    Modulation,
    Progress,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IndicatorKind {
    Bang,
    Toggle,
    Radio,
    Led,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SignalType {
    Control,
    Audio,
    Event,
    Trigger,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConnectorState {
    Default,
    Hover,
    Active,
    Illegal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CurveKind {
    Envelope,
    Cable,
    Automation,
    /// Semantic Flow: Moving dash, Phase pulse, Density modulation (NO GLOW).
    Flow {
        direction: f32, // 1.0 forward, -1.0 backward
        phase: f32,
        density: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlyphPlacement {
    pub glyph_id: u32,
    pub pos: [f32; 2],
    pub scale: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub cpu_micros: f32,
    pub peak_cpu_micros: f32,
    pub has_clipped: bool,
    pub has_nan: bool,
    pub active_voices: usize,
}

