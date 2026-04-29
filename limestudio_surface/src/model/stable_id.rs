use dirtydata_core::types::StableId;
use serde::{Deserialize, Serialize};

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
        // Deterministic ID from seed using BLAKE3 hash
        let h = blake3::hash(seed.as_bytes());
        let bytes = &h.as_bytes()[0..16];
        Self(dirtydata_core::types::StableId(ulid::Ulid::from_bytes(
            bytes.try_into().unwrap(),
        )))
    }
}

impl From<SurfaceId> for StableId {
    fn from(id: SurfaceId) -> Self {
        id.0
    }
}
