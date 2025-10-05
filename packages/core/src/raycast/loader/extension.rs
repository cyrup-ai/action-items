//! Raycast extension data structures and parsing

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastExtension {
    pub id: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub categories: Vec<String>,
    pub icon: Option<String>,
    pub path: PathBuf,
    pub commands: Vec<RaycastCommand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaycastCommand {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub mode: String,
}

impl RaycastExtension {
    /// Load a Raycast extension from its directory
    pub fn from_path(path: &Path) -> Result<Self> {
        let package_json_path = path.join("package.json");
        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| Error::IoError(format!("Failed to read package.json: {e}")))?;

        let package: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| Error::SerializationError(format!("Failed to parse package.json: {e}")))?;

        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| Error::SystemError("Invalid extension path".to_string()))?
            .to_string();

        let commands = package
            .get("commands")
            .and_then(|c| c.as_array())
            .map(|cmds| {
                cmds.iter()
                    .filter_map(|cmd| {
                        Some(RaycastCommand {
                            name: cmd.get("name")?.as_str()?.to_string(),
                            title: cmd.get("title")?.as_str()?.to_string(),
                            description: cmd
                                .get("description")
                                .and_then(|d| d.as_str())
                                .map(|s| s.to_string()),
                            mode: cmd
                                .get("mode")
                                .and_then(|m| m.as_str())
                                .unwrap_or("view")
                                .to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(Self {
            id: id.clone(),
            name: package
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or(&id)
                .to_string(),
            title: package
                .get("title")
                .and_then(|t| t.as_str())
                .unwrap_or(&id)
                .to_string(),
            description: package
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string(),
            author: package
                .get("author")
                .and_then(|a| a.as_str())
                .unwrap_or("Unknown")
                .to_string(),
            categories: package
                .get("categories")
                .and_then(|c| c.as_array())
                .map(|cats| {
                    cats.iter()
                        .filter_map(|c| c.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            icon: package
                .get("icon")
                .and_then(|i| i.as_str())
                .map(|s| s.to_string()),
            path: path.to_path_buf(),
            commands,
        })
    }
}
