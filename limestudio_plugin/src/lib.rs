pub use dirtydata_core;
pub use limestudio_macro::plugin;
pub use nih_plug;
pub use rtrb;
pub mod core;
pub mod dsl;
pub mod observation;
pub mod safety;

pub use limestudio_surface::ui_ir::SurfaceWidget as WidgetIR;
pub use limestudio_surface::widgets::trait_def::Widget;

pub use crate::core::{LimeAdapter, LimeProcessor};

// UI Components - Strictly Structure Only (Pure Projection)
pub mod ui {
    pub use limestudio_surface::ui_ir::{DisplaySignal, SurfaceId, SurfaceWidget as WidgetIR};
    pub use limestudio_surface::widgets::trait_def::Widget;
    use nih_plug::prelude::*;

    /// A reference to a parameter in the UI projection.
    pub struct UiParam<'a> {
        pub id: &'static str,
        pub param: &'a FloatParam,
    }

    pub struct Knob<'a> {
        ui_param: UiParam<'a>,
        label: String,
    }

    impl<'a> Knob<'a> {
        pub fn new(ui_param: UiParam<'a>) -> Self {
            Self {
                ui_param,
                label: String::new(),
            }
        }

        pub fn label(mut self, label: &str) -> Self {
            self.label = label.to_string();
            self
        }

        pub fn build(self) -> WidgetIR {
            WidgetIR::Knob {
                id: SurfaceId::from_seed(self.ui_param.id),
                label: self.label,
                signal: DisplaySignal::Linear(self.ui_param.param.unmodulated_plain_value()),
            }
        }
    }

    impl<'a> Widget for Knob<'a> {
        fn build(&self) -> WidgetIR {
            WidgetIR::Knob {
                id: SurfaceId::from_seed(self.ui_param.id),
                label: self.label.clone(),
                signal: DisplaySignal::Linear(self.ui_param.param.unmodulated_plain_value()),
            }
        }
    }

    impl<'a> std::fmt::Debug for Knob<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Knob").field("label", &self.label).finish()
        }
    }

    pub struct Slider<'a> {
        ui_param: UiParam<'a>,
        label: String,
        is_vertical: bool,
    }

    impl<'a> Slider<'a> {
        pub fn new(ui_param: UiParam<'a>) -> Self {
            Self {
                ui_param,
                label: String::new(),
                is_vertical: true,
            }
        }

        pub fn label(mut self, label: &str) -> Self {
            self.label = label.to_string();
            self
        }

        pub fn horizontal(mut self) -> Self {
            self.is_vertical = false;
            self
        }

        pub fn build(self) -> WidgetIR {
            WidgetIR::Slider {
                id: SurfaceId::from_seed(self.ui_param.id),
                label: self.label,
                signal: DisplaySignal::Linear(self.ui_param.param.unmodulated_plain_value()),
                is_vertical: self.is_vertical,
            }
        }
    }

    impl<'a> Widget for Slider<'a> {
        fn build(&self) -> WidgetIR {
            WidgetIR::Slider {
                id: SurfaceId::from_seed(self.ui_param.id),
                label: self.label.clone(),
                signal: DisplaySignal::Linear(self.ui_param.param.unmodulated_plain_value()),
                is_vertical: self.is_vertical,
            }
        }
    }

    impl<'a> std::fmt::Debug for Slider<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Slider")
                .field("label", &self.label)
                .field("is_vertical", &self.is_vertical)
                .finish()
        }
    }

    pub struct Toggle<'a> {
        ui_param: UiParam<'a>,
        label: String,
    }

    impl<'a> Toggle<'a> {
        pub fn new(ui_param: UiParam<'a>) -> Self {
            Self {
                ui_param,
                label: String::new(),
            }
        }
        pub fn label(mut self, label: &str) -> Self {
            self.label = label.to_string();
            self
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Button {
                id: SurfaceId::from_seed(self.ui_param.id),
                label: self.label,
                is_active: self.ui_param.param.unmodulated_plain_value() > 0.5,
            }
        }
    }

    impl<'a> Widget for Toggle<'a> {
        fn build(&self) -> WidgetIR {
            WidgetIR::Button {
                id: SurfaceId::from_seed(self.ui_param.id),
                label: self.label.clone(),
                is_active: self.ui_param.param.unmodulated_plain_value() > 0.5,
            }
        }
    }

    impl<'a> std::fmt::Debug for Toggle<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Toggle")
                .field("label", &self.label)
                .finish()
        }
    }

    pub struct NumberBox<'a> {
        ui_param: UiParam<'a>,
        label: String,
    }

    impl<'a> NumberBox<'a> {
        pub fn new(ui_param: UiParam<'a>) -> Self {
            Self {
                ui_param,
                label: String::new(),
            }
        }
        pub fn label(mut self, label: &str) -> Self {
            self.label = label.to_string();
            self
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: format!(
                    "{}: {:.2}",
                    self.label,
                    self.ui_param.param.unmodulated_plain_value()
                ),
                is_secondary: false,
            }
        }
    }

    impl<'a> Widget for NumberBox<'a> {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: format!(
                    "{}: {:.2}",
                    self.label,
                    self.ui_param.param.unmodulated_plain_value()
                ),
                is_secondary: false,
            }
        }
    }

    impl<'a> std::fmt::Debug for NumberBox<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("NumberBox")
                .field("label", &self.label)
                .finish()
        }
    }

    pub struct Button {
        _id: String,
        label: String,
    }

    impl Button {
        pub fn new(id: &str) -> Self {
            Self {
                _id: id.to_string(),
                label: id.to_string(),
            }
        }
        pub fn label(mut self, label: &str) -> Self {
            self.label = label.to_string();
            self
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("[ {} ]", self.label),
                is_secondary: false,
            }
        }
    }

    impl Widget for Button {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("[ {} ]", self.label),
                is_secondary: false,
            }
        }
    }

    impl std::fmt::Debug for Button {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Button")
                .field("label", &self.label)
                .finish()
        }
    }

    pub struct Label {
        text: String,
        is_secondary: bool,
    }

    impl Label {
        pub fn new(text: &str) -> Self {
            Self {
                text: text.to_string(),
                is_secondary: false,
            }
        }
        pub fn secondary(mut self) -> Self {
            self.is_secondary = true;
            self
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: self.text,
                is_secondary: self.is_secondary,
            }
        }
    }

    impl Widget for Label {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: self.text.clone(),
                is_secondary: self.is_secondary,
            }
        }
    }

    impl std::fmt::Debug for Label {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Label").field("text", &self.text).finish()
        }
    }

    pub struct Badge {
        text: String,
    }

    impl Badge {
        pub fn new(text: &str) -> Self {
            Self {
                text: text.to_string(),
            }
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("({})", self.text),
                is_secondary: true,
            }
        }
    }

    impl Widget for Badge {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("({})", self.text),
                is_secondary: true,
            }
        }
    }

    impl std::fmt::Debug for Badge {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Badge").field("text", &self.text).finish()
        }
    }

    pub struct ListView {
        _id: String,
        items: Vec<String>,
        selected_index: Option<usize>,
    }

    impl ListView {
        pub fn new(id: &str) -> Self {
            Self {
                _id: id.to_string(),
                items: Vec::new(),
                selected_index: None,
            }
        }
        pub fn items(mut self, items: Vec<String>) -> Self {
            self.items = items;
            self
        }
        pub fn selected(mut self, index: usize) -> Self {
            self.selected_index = Some(index);
            self
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("List: {:?}", self.items),
                is_secondary: false,
            }
        }
    }

    impl Widget for ListView {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("List: {:?}", self.items),
                is_secondary: false,
            }
        }
    }

    impl std::fmt::Debug for ListView {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ListView")
                .field("items", &self.items)
                .finish()
        }
    }

    pub struct Envelope {
        id: String,
    }

    impl Envelope {
        pub fn new(id: &str) -> Self {
            Self { id: id.to_string() }
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("Envelope: {}", self.id),
                is_secondary: false,
            }
        }
    }

    impl Widget for Envelope {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("Envelope: {}", self.id),
                is_secondary: false,
            }
        }
    }

    impl std::fmt::Debug for Envelope {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Envelope").field("id", &self.id).finish()
        }
    }

    pub struct Lens {
        id: String,
        kind: String,
    }

    impl Lens {
        pub fn new(id: &str, kind: &str) -> Self {
            Self {
                id: id.to_string(),
                kind: kind.to_string(),
            }
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("Lens: {} ({})", self.id, self.kind),
                is_secondary: true,
            }
        }
    }

    impl Widget for Lens {
        fn build(&self) -> WidgetIR {
            WidgetIR::Label {
                text: format!("Lens: {} ({})", self.id, self.kind),
                is_secondary: true,
            }
        }
    }

    impl std::fmt::Debug for Lens {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Lens")
                .field("id", &self.id)
                .field("kind", &self.kind)
                .finish()
        }
    }

    pub struct LevelMeter {
        id: String,
        signal: DisplaySignal,
    }

    impl LevelMeter {
        pub fn new(id: &str, value: f32, peak: f32) -> Self {
            Self {
                id: id.to_string(),
                signal: DisplaySignal::Meter { value, peak },
            }
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::LevelMeter {
                id: self.id,
                signal: self.signal,
            }
        }
    }

    impl Widget for LevelMeter {
        fn build(&self) -> WidgetIR {
            WidgetIR::LevelMeter {
                id: self.id.clone(),
                signal: self.signal.clone(),
            }
        }
    }

    impl std::fmt::Debug for LevelMeter {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("LevelMeter").field("id", &self.id).finish()
        }
    }

    pub struct Waveform {
        id: String,
        data: Vec<f32>,
    }

    impl Waveform {
        pub fn new(id: &str, data: Vec<f32>) -> Self {
            Self {
                id: id.to_string(),
                data,
            }
        }
        pub fn build(self) -> WidgetIR {
            WidgetIR::Waveform {
                id: self.id,
                data: self.data.clone(),
            }
        }
    }

    impl Widget for Waveform {
        fn build(&self) -> WidgetIR {
            WidgetIR::Waveform {
                id: self.id.clone(),
                data: self.data.clone(),
            }
        }
    }

    impl std::fmt::Debug for Waveform {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Waveform").field("id", &self.id).finish()
        }
    }

    // Layout Helpers (Flutter-like)

    pub struct Padding {
        pub amount: f32,
        pub child: Box<dyn Widget>,
    }

    impl Padding {
        pub fn new(amount: f32, child: impl Widget + 'static) -> Self {
            Self {
                amount,
                child: Box::new(child),
            }
        }
    }

    impl Widget for Padding {
        fn build(&self) -> WidgetIR {
            WidgetIR::Box {
                children: vec![self.child.build()],
                style: limestudio_surface::ui_ir::FrameStyle::None,
            }
        }
    }

    impl std::fmt::Debug for Padding {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Padding")
                .field("amount", &self.amount)
                .finish()
        }
    }

    pub struct Spacer {
        pub flex: f32,
    }

    impl Spacer {
        pub fn new(flex: f32) -> Self {
            Self { flex }
        }
    }

    impl Widget for Spacer {
        fn build(&self) -> WidgetIR {
            // Spacer is just an empty box with flex
            WidgetIR::Box {
                children: Vec::new(),
                style: limestudio_surface::ui_ir::FrameStyle::None,
            }
        }
    }

    impl std::fmt::Debug for Spacer {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Spacer").field("flex", &self.flex).finish()
        }
    }

    pub struct Column<'a> {
        pub children: Vec<Box<dyn Widget + 'a>>,
    }

    impl<'a> Widget for Column<'a> {
        fn build(&self) -> WidgetIR {
            WidgetIR::Column {
                children: self.children.iter().map(|c| c.build()).collect(),
            }
        }
    }

    impl<'a> std::fmt::Debug for Column<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Column").finish()
        }
    }

    pub struct Row<'a> {
        pub children: Vec<Box<dyn Widget + 'a>>,
    }

    impl<'a> Widget for Row<'a> {
        fn build(&self) -> WidgetIR {
            WidgetIR::Row {
                children: self.children.iter().map(|c| c.build()).collect(),
            }
        }
    }

    impl<'a> std::fmt::Debug for Row<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Row").finish()
        }
    }
}

pub mod crash;
pub mod editor;
pub mod interaction;

// Re-exports
pub use editor::ObservationState;
pub use ui::{
    Badge, Button, Envelope, Knob, Label, Lens, LevelMeter, ListView, NumberBox, Slider, Toggle,
    UiParam, Waveform,
};

#[macro_export]
macro_rules! vbox {
    ($($child:expr),* $(,)?) => {
        $crate::ui::Column {
            children: vec![$(Box::new($child) as Box<dyn $crate::Widget>),*]
        }
    };
}

#[macro_export]
macro_rules! hbox {
    ($($child:expr),* $(,)?) => {
        $crate::ui::Row {
            children: vec![$(Box::new($child) as Box<dyn $crate::Widget>),*]
        }
    };
}

#[macro_export]
macro_rules! view {
    ($widget:expr) => {
        Box::new($widget) as Box<dyn $crate::Widget>
    };
}
