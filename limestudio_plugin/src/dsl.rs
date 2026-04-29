//! DSL for LimeStudio Plugin
//! Provides a fluent interface for building the DSP graph and UI.

pub use limestudio_core::graph::ParamSource as DslParam;
use limestudio_core::graph::{GraphBuilder, ParamSource, StableId};
use std::cell::RefCell;
use std::ops::Shr;

pub struct NodeHandle {
    pub id: StableId,
}

pub struct Chainable<'a> {
    pub current_node: StableId,
    pub builder: &'a RefCell<GraphBuilder>,
}

impl<'a> Chainable<'a> {
    pub fn chain(self, node: StableId) -> Self {
        self.builder.borrow_mut().connect(self.current_node, node);
        Self {
            current_node: node,
            builder: self.builder,
        }
    }
}

impl<'a> Shr<NodeHandle> for Chainable<'a> {
    type Output = Chainable<'a>;

    fn shr(self, rhs: NodeHandle) -> Self::Output {
        self.chain(rhs.id)
    }
}

// Support for node >> ctx.output where node is a NodeHandle
impl NodeHandle {
    pub fn into_chainable(self, builder: &RefCell<GraphBuilder>) -> Chainable<'_> {
        Chainable {
            current_node: self.id,
            builder,
        }
    }
}

// Global scope node builders for the dsp: block.
// These are used as markers for the macro or as placeholders.

#[allow(non_snake_case)]
pub fn Sine(_freq: ParamSource) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn Oscillator(_freq: ParamSource, _wave: &str) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn ADSR(
    _a: ParamSource,
    _d: ParamSource,
    _s: ParamSource,
    _r: ParamSource,
    _gate: ParamSource,
) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn Multiply(_a: NodeHandle, _b: NodeHandle) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn Filter(_freq: ParamSource, _q: ParamSource, _kind: &str) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn Delay(_time: ParamSource) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn Add(_a: NodeHandle, _b: NodeHandle) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn m2f(_p: ParamSource, _t: ParamSource) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}

#[allow(non_snake_case)]
pub fn SineVCO(_freq: NodeHandle) -> NodeHandle {
    NodeHandle {
        id: StableId::new(),
    }
}
