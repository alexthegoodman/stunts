use super::utilities::get_ground_truth_dir;
use chrono::{DateTime, Local};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ProjectInfo {
    pub name: String,
    pub created: DateTime<Local>,
    pub modified: DateTime<Local>,
}

pub fn get_projects() -> Result<Vec<ProjectInfo>, Box<dyn std::error::Error>> {
    let sync_dir = get_ground_truth_dir().expect("Couldn't get CommonOS directory");
    let projects_dir = sync_dir.join("stunts/projects");

    let mut projects = Vec::new();

    for entry in fs::read_dir(&projects_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip if not a directory
        if !path.is_dir() {
            continue;
        }

        let metadata = fs::metadata(&path)?;

        // Get creation time
        let created = metadata
            .created()
            .unwrap_or(SystemTime::now())
            .duration_since(UNIX_EPOCH)?;
        let created: DateTime<Local> = DateTime::from(SystemTime::UNIX_EPOCH + created);

        // Get modification time
        let modified = metadata
            .modified()
            .unwrap_or(SystemTime::now())
            .duration_since(UNIX_EPOCH)?;
        let modified: DateTime<Local> = DateTime::from(SystemTime::UNIX_EPOCH + modified);

        projects.push(ProjectInfo {
            name: path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string(),
            created,
            modified,
        });
    }

    // Sort by modification date (newest first)
    projects.sort_by(|a, b| b.modified.cmp(&a.modified));

    Ok(projects)
}
