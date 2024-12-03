use std::{fs, path::PathBuf, sync::MutexGuard};

use directories::{BaseDirs, UserDirs};

use super::saved_state::SavedState;

pub fn get_ground_truth_dir() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| {
        let common_os = user_dirs
            .document_dir()
            .expect("Couldn't find Documents directory")
            .join("StuntsGroundTruth");
        fs::create_dir_all(&common_os)
            .ok()
            .expect("Couldn't check or create StuntsGroundTruth directory");
        common_os
    })
}

pub fn load_ground_truth_state() -> Result<SavedState, Box<dyn std::error::Error>> {
    let sync_dir = get_ground_truth_dir().expect("Couldn't get StuntsGroundTruth directory");
    // let project_dir = sync_dir.join("midpoint/projects").join(project_id);
    let json_path = sync_dir.join("motion_path_data.json");

    if !json_path.exists() {
        // TODO: create json file if it doesn't exist
        let json = SavedState {
            sequences: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&json).expect("Couldn't serialize saved state");

        fs::write(&json_path, json).expect("Couldn't write saved state");
    }

    // Read and parse the JSON file
    let json_content = fs::read_to_string(json_path)?;
    let state: SavedState = serde_json::from_str(&json_content)?;

    Ok(state)
}

pub fn save_saved_state(saved_state: MutexGuard<SavedState>) {
    let owned = saved_state.to_owned();
    save_saved_state_raw(owned);
}

pub fn save_saved_state_raw(saved_state: SavedState) {
    let json = serde_json::to_string_pretty(&saved_state).expect("Couldn't serialize saved state");
    let sync_dir = get_ground_truth_dir().expect("Couldn't get StuntsGroundTruth directory");
    let save_path = sync_dir.join("motion_path_data.json");

    println!("Saving saved state... {}", save_path.display());

    fs::write(&save_path, json).expect("Couldn't write saved state");

    drop(saved_state);

    println!("Saved!");
}
