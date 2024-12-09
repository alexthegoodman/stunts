use serde::{Deserialize, Serialize};
use stunts_engine::{animations::Sequence, polygon::SavedPolygonConfig};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub sequences: Vec<Sequence>,
}
