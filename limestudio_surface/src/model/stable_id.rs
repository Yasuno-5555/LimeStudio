use dirtydata_core::types::StableId;
use serde::{Serialize, Deserialize};

/// A wrapper or utility for StableId in the surface domain.
/// This helps in tracking nodes, ports, and cables between UI frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SurfaceId(pub StableId);

impl From<StableId> for SurfaceId {
    fn from(id: StableId) -> Self {
        Self(id)
    }
}
impl SurfaceId {
    pub fn generate() -> Self {
        Self(dirtydata_core::types::StableId::new())
    }

    pub fn from_seed(seed: &str) -> Self {
        Self(dirtydata_core::types::StableId::from_seed(seed))
    }
}

impl From<SurfaceId> for StableId {
    fn from(id: SurfaceId) -> Self {
        id.0
    }
}
