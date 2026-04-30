use crate::ui_ir::{SurfaceId, SurfaceKernel, SurfacePrimitive};
use crate::widgets::trait_def::{ReconcilerContext, Widget};

pub struct Reconciler {
    pub kernel: SurfaceKernel,
}

impl Reconciler {
    pub fn new() -> Self {
        Self {
            kernel: SurfaceKernel::new(),
        }
    }

    pub fn reconcile_tree(&mut self, root: &dyn Widget) -> Vec<SurfacePrimitive> {
        self.kernel.generation += 1;

        let mut cx = ReconcilerContext {
            kernel: &mut self.kernel,
            parent_id: SurfaceId::generate(),
        };

        let primitives = root.reconcile(&mut cx);

        // Update History Store for persistence
        self.update_history(&primitives);

        primitives
    }

    fn update_history(&mut self, primitives: &[SurfacePrimitive]) {
        for prim in primitives {
            let id = match prim {
                SurfacePrimitive::Frame { id, .. } => Some(*id),
                SurfacePrimitive::Indicator { id, .. } => Some(*id),
                SurfacePrimitive::Path { id, .. } => Some(*id),
                SurfacePrimitive::Organic { id, .. } => Some(*id),
                _ => None,
            };

            if let Some(id) = id {
                // Only track if it's already in history_store (meaning something wants to track it)
                if let Some(history) = self.kernel.history_store.get_mut(&id) {
                    history.push_front(prim.clone());
                    if history.len() > 64 {
                        // Max history depth
                        history.pop_back();
                    }
                }
            }
        }
    }
}
