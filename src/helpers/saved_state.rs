use serde::{Deserialize, Serialize};

use super::animations::AnimationData;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct Sequence {
    pub id: String,
    pub motion_paths: Vec<AnimationData>,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub sequences: Vec<Sequence>,
}
