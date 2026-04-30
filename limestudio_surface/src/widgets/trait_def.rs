use crate::ui_ir::{SurfaceId, SurfacePrimitive};
use taffy::style::Style;

/// The base trait for all Lime Surface widgets (Sentient Edition).
/// "Everything is an Intention. Reconciliation is the realization."
pub trait Widget: std::fmt::Debug {
    /// The unique identity of the widget for reconciliation.
    fn id(&self) -> crate::ui_ir::SurfaceId {
        crate::ui_ir::SurfaceId::generate()
    }

    /// Reconcile the widget with the current state and produce primitives.
    fn reconcile(&self, _cx: &mut ReconcilerContext) -> Vec<crate::ui_ir::SurfacePrimitive> {
        self.build()
            .id()
            .map(|_id| {
                // Default reconciliation logic: just use the old build() output
                // This is a bridge between the old and new worlds.
                Vec::new()
            })
            .unwrap_or_default()
    }

    /// Legacy build method for backward compatibility.
    fn build(&self) -> crate::ui_ir::SurfaceWidget;
}

pub struct ReconcilerContext<'a> {
    pub kernel: &'a mut crate::ui_ir::SurfaceKernel,
    pub parent_id: crate::ui_ir::SurfaceId,
}

impl<'a> ReconcilerContext<'a> {
    pub fn use_state<S: Any + Send + Sync + Default + Clone>(
        &mut self,
        id: crate::ui_ir::SurfaceId,
    ) -> S {
        self.kernel
            .state_store
            .entry(id)
            .or_insert_with(|| Arc::new(S::default()))
            .downcast_ref::<S>()
            .cloned()
            .unwrap()
    }
}

use std::any::Any;
use std::sync::Arc;

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
    fn id(&self) -> SurfaceId {
        self.id
    }

    fn reconcile(&self, _cx: &mut ReconcilerContext) -> Vec<crate::ui_ir::SurfacePrimitive> {
        (self.build_primitives)()
    }

    fn build(&self) -> crate::ui_ir::SurfaceWidget {
        crate::ui_ir::SurfaceWidget::Custom {
            id: self.id,
            style: Box::new((self.style)()),
            primitives: (self.build_primitives)(),
            interaction: crate::ui_ir::InteractionClass::None,
        }
    }
}
