use serde::{Deserialize, Serialize};
use stunts_engine::{animations::Sequence, polygon::SavedPolygonConfig};

use crate::views::sequence_timeline::SavedTimelineStateConfig;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub id: String,
    pub name: String,
    pub sequences: Vec<Sequence>,
    pub timeline_sequences: SavedTimelineStateConfig,
}
