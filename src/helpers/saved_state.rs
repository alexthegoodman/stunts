use serde::{Deserialize, Serialize};

use super::animations::AnimationData;

#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub motion_paths: Vec<AnimationData>,
}
