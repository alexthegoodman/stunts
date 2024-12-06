use serde::{Deserialize, Serialize};
use stunts_engine::polygon::SavedPolygonConfig;

use super::animations::AnimationData;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Sequence {
    pub id: String,
    pub active_polygons: Vec<SavedPolygonConfig>, // used for dimensions, etc
    pub polygon_motion_paths: Vec<AnimationData>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub sequences: Vec<Sequence>,
}
