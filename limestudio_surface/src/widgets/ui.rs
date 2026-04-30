use crate::ui_ir::{SurfaceId, SurfacePrimitive};
use crate::widgets::trait_def::{ReconcilerContext, Widget};

pub struct Ui<'a, 'b> {
    pub cx: &'a mut ReconcilerContext<'b>,
    pub primitives: Vec<SurfacePrimitive>,
}

impl<'a, 'b> Ui<'a, 'b> {
    pub fn new(cx: &'a mut ReconcilerContext<'b>) -> Self {
        Self {
            cx,
            primitives: Vec::new(),
        }
    }

    pub fn add(&mut self, widget: impl Widget) {
        let mut sub_cx = ReconcilerContext {
            kernel: self.cx.kernel,
            parent_id: widget.id(),
        };
        let mut prims = widget.reconcile(&mut sub_cx);
        self.primitives.append(&mut prims);
    }

    pub fn aura(&mut self, id: SurfaceId, center: [f32; 2], radius: f32) {
        // Immediate-mode helper that emits a primitive directly
        self.primitives.push(SurfacePrimitive::Organic {
            id,
            kind: crate::ui_ir::OrganicKind::Aura {
                center,
                radius,
                pulsation: 0.0, // Should be driven by state or signal
                harmonics: 1,
            },
            brush: crate::ui_ir::BespokeBrush::Solid([0.0, 1.0, 0.8, 0.5]),
            temporal: crate::ui_ir::TemporalStrategy::Standard,
        });
    }
}
