use std::{fs, path::PathBuf, sync::MutexGuard};

use directories::{BaseDirs, UserDirs};
use floem::reactive::RwSignal;
use floem::reactive::SignalGet;
use floem::reactive::SignalUpdate;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uuid::Uuid;

use super::saved_state::SavedState;

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

pub fn load_project_state(project_id: String) -> Result<SavedState, Box<dyn std::error::Error>> {
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let project_dir = sync_dir.join("projects").join(project_id);
    let json_path = project_dir.join("project_data.json");

    if !json_path.exists() {
        // TODO: create json file if it doesn't exist
        let project_id = Uuid::new_v4().to_string();

        let json = SavedState {
            id: project_id,
            name: "New Project".to_string(),
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

// Add this function to handle project creation
pub fn create_project_state(name: String) -> Result<SavedState, Box<dyn std::error::Error>> {
    let project_id = Uuid::new_v4().to_string();

    // Create project directory and save initial state
    let sync_dir = get_ground_truth_dir().expect("Couldn't get Stunts directory");
    let project_dir = sync_dir.join("projects").join(&project_id);
    fs::create_dir_all(&project_dir)?;

    // Create initial saved state
    let initial_state = SavedState {
        id: project_id,
        name: name.clone(),
        sequences: Vec::new(),
    };

    let json = serde_json::to_string_pretty(&initial_state)?;
    fs::write(project_dir.join("project_data.json"), json)?;

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
pub fn fetch_subscription_details(
    token: &str,
) -> Result<SubscriptionDetails, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
        .get("http://localhost:3000/api/subscriptions/details")
        .header("Authorization", format!("Bearer {}", token))
        .send()?;

    if response.status().is_success() {
        let details = response.json::<SubscriptionDetails>()?;
        Ok(details)
    } else {
        Err(response.text()?.into())
    }
}

// Function to check subscription status
pub fn check_subscription(auth_state: RwSignal<AuthState>) {
    if let Some(token) = auth_state.get().token.as_ref() {
        match fetch_subscription_details(&token.token) {
            Ok(subscription) => {
                let mut current_state = auth_state.get();
                current_state.subscription = Some(subscription);
                auth_state.set(current_state);
            }
            Err(e) => {
                println!("Failed to fetch subscription details: {}", e);
                // Optionally handle error in UI
            }
        }
    }
}

// Function to get the auth token file path
pub fn get_auth_token_path() -> PathBuf {
    get_ground_truth_dir()
        .expect("Couldn't get Stunts directory")
        .join("auth_token.json")
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
