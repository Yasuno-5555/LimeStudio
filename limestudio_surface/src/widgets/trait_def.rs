use crate::ui_ir::{SurfaceWidget, SurfacePrimitive, SurfaceId};
use taffy::style::Style;

/// The base trait for all Lime Surface widgets.
/// "Everything is a Widget."
pub trait Widget: std::fmt::Debug {
    /// Describe the layout and rendering by producing a SurfaceWidget IR.
    fn build(&self) -> SurfaceWidget;
}

/// Helper to wrap a Custom implementation into a Widget.
#[derive(Debug)]
pub struct CustomWidget<F, L> 
where 
    F: Fn() -> Vec<SurfacePrimitive> + std::fmt::Debug,
    L: Fn() -> Style + std::fmt::Debug,
{
    pub id: SurfaceId,
    pub build_primitives: F,
    pub style: L,
}

impl<F, L> Widget for CustomWidget<F, L>
where 
    F: Fn() -> Vec<SurfacePrimitive> + std::fmt::Debug,
    L: Fn() -> Style + std::fmt::Debug,
{
    fn build(&self) -> SurfaceWidget {
        SurfaceWidget::Custom {
            id: self.id,
            style: Box::new((self.style)()),
            primitives: (self.build_primitives)(),
        }
    }
}
