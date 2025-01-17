use serde::{Deserialize, Serialize};
use stunts_engine::{
    animations::Sequence, polygon::SavedPolygonConfig, timelines::SavedTimelineStateConfig,
};

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct SavedState {
    pub id: String,
    // pub name: String,
    pub sequences: Vec<Sequence>,
    pub timeline_state: SavedTimelineStateConfig,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ProjectData {
    pub project_id: String,
    pub project_name: String,
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct ProjectsDataFile {
    pub projects: Vec<ProjectData>,
}
