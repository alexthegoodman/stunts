use std::{fs, path::PathBuf, sync::MutexGuard};

use directories::{BaseDirs, UserDirs};
use floem::reactive::RwSignal;
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use stunts_engine::animations::AnimationData;
use stunts_engine::animations::AnimationProperty;
use stunts_engine::animations::BackgroundFill;
use stunts_engine::animations::EasingType;
use stunts_engine::animations::KeyType;
use stunts_engine::animations::KeyframeValue;
use stunts_engine::animations::ObjectType;
use stunts_engine::animations::Sequence;
use stunts_engine::animations::UIKeyframe;
use stunts_engine::editor::wgpu_to_human;
use stunts_engine::editor::PathType;
use stunts_engine::polygon::SavedPoint;
use stunts_engine::polygon::SavedPolygonConfig;
use stunts_engine::polygon::SavedStroke;
use stunts_engine::timelines::SavedTimelineStateConfig;
use uuid::Uuid;

use super::saved_state::ProjectData;
use super::saved_state::ProjectsDataFile;
use super::saved_state::SavedState;

#[cfg(feature = "production")]
pub const API_URL: &str = "https://madebycommon.com";

#[cfg(not(feature = "production"))]
pub const API_URL: &str = "http://localhost:3000";

#[cfg(feature = "production")]
pub const AUTH_TOKEN_NAME: &str = "prod_auth_token.json";

#[cfg(not(feature = "production"))]
pub const AUTH_TOKEN_NAME: &str = "auth_token.json";

pub fn get_ground_truth_dir() -> Option<PathBuf> {
    UserDirs::new().map(|user_dirs| {
        let common_os = user_dirs
            .document_dir()
            .expect("Couldn't find Documents directory")
            .join("Stunts");
        fs::create_dir_all(&common_os)
            .ok()
            .expect("Couldn't check or create Stunts directory");
        common_os
    })
}

// TODO: put images and videos and exports in separate project folders
pub fn get_images_dir() -> PathBuf {
    let main_dir = get_ground_truth_dir().expect("Couldn't check or create Stunts directory");
    let images_dir = main_dir.join("images");

    fs::create_dir_all(&images_dir)
        .ok()
        .expect("Couldn't check or create Stunts images directory");

    images_dir
}

pub fn get_videos_dir() -> PathBuf {
    let main_dir = get_ground_truth_dir().expect("Couldn't check or create Stunts directory");
    let videos_dir = main_dir.join("videos");

    fs::create_dir_all(&videos_dir)
        .ok()
        .expect("Couldn't check or create Stunts videos directory");

    videos_dir
}

pub fn get_exports_dir() -> PathBuf {
    let main_dir = get_ground_truth_dir().expect("Couldn't check or create Stunts directory");
    let exports_dir = main_dir.join("exports");

    fs::create_dir_all(&exports_dir)
        .ok()
        .expect("Couldn't check or create Stunts exports directory");

    exports_dir
}

pub fn get_captures_dir() -> PathBuf {
    let main_dir = get_ground_truth_dir().expect("Couldn't check or create Stunts directory");
    let captures_dir = main_dir.join("captures");

    fs::create_dir_all(&captures_dir)
        .ok()
        .expect("Couldn't check or create Stunts captures directory");

    captures_dir
}

// pub fn load_ground_truth_state() -> Result<SavedState, Box<dyn std::error::Error>> {
//     let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
//     // let project_dir = sync_dir.join("midpoint/projects").join(project_id);
//     let json_path = sync_dir.join("motion_path_data.json");

//     if !json_path.exists() {
//         // TODO: create json file if it doesn't exist
//         let json = SavedState {
//             sequences: Vec::new(),
//         };

//         let json = serde_json::to_string_pretty(&json).expect("Couldn't serialize saved state");

//         fs::write(&json_path, json).expect("Couldn't write saved state");
//     }

//     // Read and parse the JSON file
//     let json_content = fs::read_to_string(json_path)?;
//     let state: SavedState = serde_json::from_str(&json_content)?;

//     Ok(state)
// }

pub fn load_projects_datafile() -> Result<ProjectsDataFile, Box<dyn std::error::Error>> {
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let json_path = sync_dir.join("projects.json");

    if !json_path.exists() {
        // create json file if it doesn't exist
        let json = ProjectsDataFile {
            projects: Vec::new(),
        };

        let json = serde_json::to_string_pretty(&json).expect("Couldn't serialize saved state");

        fs::write(&json_path, json).expect("Couldn't write saved state");
    }

    // Read and parse the JSON file
    let json_content = fs::read_to_string(json_path)?;
    let state: ProjectsDataFile = serde_json::from_str(&json_content)?;

    Ok(state)
}

pub fn save_projects_datafile(projects_datafile: ProjectsDataFile) {
    let json =
        serde_json::to_string_pretty(&projects_datafile).expect("Couldn't serialize saved state");
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let save_path = sync_dir.join("projects.json");

    println!("Saving datafile... {}", save_path.display());

    fs::write(&save_path, json).expect("Couldn't write saved state");

    drop(projects_datafile);

    println!("Saved datafile!");
}

pub fn load_project_state(project_id: String) -> Result<SavedState, Box<dyn std::error::Error>> {
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let project_dir = sync_dir.join("projects").join(project_id);
    let json_path = project_dir.join("project_data.json");

    if !json_path.exists() {
        // create json file if it doesn't exist
        let project_id = Uuid::new_v4().to_string();

        let json = SavedState {
            id: project_id,
            // name: "New Project".to_string(),
            sequences: Vec::new(),
            timeline_state: SavedTimelineStateConfig {
                timeline_sequences: Vec::new(),
            },
        };

        let json = serde_json::to_string_pretty(&json).expect("Couldn't serialize saved state");

        fs::write(&json_path, json).expect("Couldn't write saved state");
    }

    // Read and parse the JSON file
    let json_content = fs::read_to_string(json_path)?;
    let state: SavedState = serde_json::from_str(&json_content)?;

    Ok(state)
}

// Add this function to handle project creation
pub fn create_project_state(name: String) -> Result<SavedState, Box<dyn std::error::Error>> {
    let project_id = Uuid::new_v4().to_string();

    // Create project directory and save initial state
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let project_dir = sync_dir.join("projects").join(&project_id);
    fs::create_dir_all(&project_dir)?;

    // Create initial saved state
    let initial_state = SavedState {
        id: project_id.clone(),
        // name: name.clone(),
        sequences: Vec::new(),
        timeline_state: SavedTimelineStateConfig {
            timeline_sequences: Vec::new(),
        },
    };

    let json = serde_json::to_string_pretty(&initial_state)?;
    fs::write(project_dir.join("project_data.json"), json)?;

    // auto-add to ProjectsDataFile?
    // this will also create the datafile if it doesn't already exist
    let mut datafile = load_projects_datafile().expect("Couldn't load datafile");

    datafile.projects.push(ProjectData {
        project_id: project_id,
        project_name: name.clone(),
    });

    save_projects_datafile(datafile);

    Ok(initial_state)
}

pub fn save_saved_state(saved_state: MutexGuard<SavedState>) {
    let owned = saved_state.to_owned();
    save_saved_state_raw(owned);
}

pub fn save_saved_state_raw(saved_state: SavedState) {
    let json = serde_json::to_string_pretty(&saved_state).expect("Couldn't serialize saved state");
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let project_dir = sync_dir.join("projects").join(saved_state.id.clone());
    let save_path = project_dir.join("project_data.json");

    println!("Saving saved state... {}", save_path.display());

    fs::write(&save_path, json).expect("Couldn't write saved state");

    drop(saved_state);

    println!("Saved!");
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthToken {
    pub token: String,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub expiry: Option<chrono::DateTime<chrono::Utc>>,
}

// #[derive(Clone)]
// pub struct AuthState {
//     pub token: Option<AuthToken>,
//     pub is_authenticated: bool,
// }

#[derive(Debug, Clone, Deserialize)]
pub struct Plan {
    pub id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionDetails {
    pub subscription_status: String,
    pub current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    pub plan: Option<Plan>,
    pub cancel_at_period_end: bool,
}

// Extend AuthState to include subscription details
#[derive(Clone)]
pub struct AuthState {
    pub token: Option<AuthToken>,
    pub is_authenticated: bool,
    pub subscription: Option<SubscriptionDetails>,
}

impl AuthState {
    pub fn can_create_projects(&self) -> bool {
        if !self.is_authenticated {
            return false;
        }

        match &self.subscription {
            Some(sub) => matches!(sub.subscription_status.as_str(), "ACTIVE" | "TRIALING"),
            None => false,
        }
    }
}

// Function to fetch subscription details
pub async fn fetch_subscription_details(
    token: &str,
) -> Result<SubscriptionDetails, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
        .get(API_URL.to_owned() + &"/api/subscription/details")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    if response.status().is_success() {
        let details = response.json::<SubscriptionDetails>().await?;
        Ok(details)
    } else {
        Err(response.text().await?.into())
    }
}

// // Function to check subscription status
// pub fn check_subscription(auth_state: RwSignal<AuthState>) {
//     if let Some(token) = auth_state.get().token.as_ref() {
//         match fetch_subscription_details(&token.token) {
//             Ok(subscription) => {
//                 let mut current_state = auth_state.get();
//                 current_state.subscription = Some(subscription);
//                 auth_state.set(current_state);
//             }
//             Err(e) => {
//                 println!("Failed to fetch subscription details: {}", e);
//                 // Optionally handle error in UI
//             }
//         }
//     }
// }

// Function to get the auth token file path
pub fn get_auth_token_path() -> PathBuf {
    get_ground_truth_dir()
        .expect("Couldn't get Stunts directory")
        .join(AUTH_TOKEN_NAME)
}

// Load saved auth token if it exists
pub fn load_auth_token() -> Option<AuthToken> {
    let token_path = get_auth_token_path();
    if token_path.exists() {
        if let Ok(content) = fs::read_to_string(token_path) {
            if let Ok(token) = serde_json::from_str::<AuthToken>(&content) {
                // Check if token is expired
                if let Some(expiry) = token.expiry {
                    if expiry > chrono::Utc::now() {
                        return Some(token);
                    }
                }
            }
        }
    }
    None
}

// Save auth token to disk
pub fn save_auth_token(token: &AuthToken) -> Result<(), Box<dyn std::error::Error>> {
    let token_path = get_auth_token_path();
    let json = serde_json::to_string_pretty(token)?;
    fs::write(token_path, json)?;
    Ok(())
}

// Clear saved auth token
pub fn clear_auth_token() -> Result<(), Box<dyn std::error::Error>> {
    let token_path = get_auth_token_path();
    if token_path.exists() {
        fs::remove_file(token_path)?;
    }
    Ok(())
}

// for reimporting ml data
use std::collections::HashMap;
use std::time::Duration;

pub fn parse_animation_data(content: &str) -> Result<Vec<Sequence>, Box<dyn std::error::Error>> {
    let sequences: Vec<&str> = content.split("---").collect();

    let mut result = Vec::new();

    for sequence_data in sequences {
        if sequence_data.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = sequence_data.split("!!!").collect();
        if parts.len() != 2 {
            return Err("Invalid sequence format".into());
        }

        // Create a mapping of original indices to UUIDs
        let mut polygon_id_map = HashMap::new();

        let active_polygons = parse_active_polygons(parts[0], &mut polygon_id_map)?;
        let motion_paths = parse_motion_paths(parts[1], &polygon_id_map)?;

        let sequence = Sequence {
            id: Uuid::new_v4().to_string(),
            name: "Imported Seq".to_string(),
            background_fill: Some(BackgroundFill::Color([
                wgpu_to_human(0.8) as i32,
                wgpu_to_human(0.8) as i32,
                wgpu_to_human(0.8) as i32,
                1,
            ])),
            duration_ms: 20000,
            active_polygons,
            polygon_motion_paths: motion_paths,
            active_text_items: Vec::new(),
            active_image_items: Vec::new(),
            active_video_items: Vec::new(),
        };

        result.push(sequence);
    }

    Ok(result)
}

fn parse_active_polygons(
    data: &str,
    polygon_id_map: &mut HashMap<String, String>,
) -> Result<Vec<SavedPolygonConfig>, Box<dyn std::error::Error>> {
    let mut polygons = Vec::new();

    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() != 6 {
            return Err("Invalid polygon data format".into());
        }

        let original_id = parts[0].to_string();
        let uuid = Uuid::new_v4().to_string();
        polygon_id_map.insert(original_id.clone(), uuid.clone());

        let polygon = SavedPolygonConfig {
            id: uuid,
            name: format!("Polygon {}", original_id),
            fill: [255, 255, 255, 255], // Default white
            dimensions: (parts[2].parse::<i32>()?, parts[3].parse::<i32>()?),
            position: SavedPoint {
                x: parts[4].parse::<i32>()?,
                y: parts[5].parse::<i32>()?,
            },
            border_radius: 0,
            stroke: SavedStroke {
                thickness: 1,
                fill: [0, 0, 0, 255], // Default black
            },
            layer: -2,
        };

        polygons.push(polygon);
    }

    Ok(polygons)
}

fn parse_motion_paths(
    data: &str,
    polygon_id_map: &HashMap<String, String>,
) -> Result<Vec<AnimationData>, Box<dyn std::error::Error>> {
    let mut motion_paths = Vec::new();
    let mut current_polygon_keyframes: HashMap<String, Vec<(f32, [i32; 2])>> = HashMap::new();

    // First, group keyframes by polygon ID
    for line in data.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() != 6 {
            return Err("Invalid motion path data format".into());
        }

        let original_id = parts[0].to_string();
        let uuid = polygon_id_map
            .get(&original_id)
            .ok_or_else(|| format!("No UUID mapping found for polygon {}", original_id))?;
        let time = parts[1].parse::<f32>()?;
        let position = [parts[4].parse::<i32>()?, parts[5].parse::<i32>()?];

        current_polygon_keyframes
            .entry(uuid.clone())
            .or_default()
            .push((time, position));
    }

    // Convert grouped keyframes into AnimationData structures
    for (polygon_id, keyframes) in current_polygon_keyframes {
        let mut position_property = AnimationProperty {
            name: "Position".to_string(),
            property_path: "position".to_string(),
            children: Vec::new(),
            keyframes: keyframes
                .into_iter()
                .map(|(time, pos)| UIKeyframe {
                    id: Uuid::new_v4().to_string(),
                    time: Duration::from_secs_f32(time),
                    value: KeyframeValue::Position(pos),
                    easing: EasingType::Linear,
                    path_type: PathType::Linear,
                    key_type: KeyType::Frame,
                })
                .collect(),
            depth: 0,
        };

        // Find the maximum time to set as duration
        let max_time = position_property
            .keyframes
            .iter()
            .map(|k| k.time)
            .max()
            .unwrap_or(Duration::from_secs(0));

        let animation_data = AnimationData {
            id: Uuid::new_v4().to_string(),
            object_type: ObjectType::Polygon,
            polygon_id: polygon_id.clone(),
            duration: max_time,
            start_time_ms: 0,
            position: [0, 0],
            properties: vec![position_property],
        };

        motion_paths.push(animation_data);
    }

    Ok(motion_paths)
}
