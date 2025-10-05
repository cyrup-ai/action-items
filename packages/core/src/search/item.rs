use std::path::PathBuf;
use std::process::Command;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchItemType {
    Application,
    File,
    Directory,
    Command,
    Plugin,
    ActionItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub path: Option<PathBuf>,
    pub icon_path: Option<PathBuf>,
    pub item_type: SearchItemType,
    pub executable: Option<String>,
    pub score: f32,
    pub keywords: Vec<String>,
}

impl SearchItem {
    pub fn new(id: String, title: String, description: String, item_type: SearchItemType) -> Self {
        Self {
            id,
            title,
            description,
            path: None,
            icon_path: None,
            item_type,
            executable: None,
            score: 0.0,
            keywords: Vec::new(),
        }
    }

    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }

    pub fn with_executable(mut self, executable: String) -> Self {
        self.executable = Some(executable);
        self
    }

    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    pub fn execute(&self) -> Result<()> {
        match &self.item_type {
            SearchItemType::Application => {
                if let Some(exec) = &self.executable {
                    #[cfg(target_os = "macos")]
                    {
                        Command::new("open")
                            .arg("-a")
                            .arg(exec)
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    #[cfg(target_os = "linux")]
                    {
                        Command::new(exec)
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    #[cfg(target_os = "windows")]
                    {
                        Command::new("cmd")
                            .args(["/C", "start", exec])
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    Ok(())
                } else {
                    Err(Error::ExecutionError("No executable specified".to_string()))
                }
            },
            SearchItemType::File | SearchItemType::Directory => {
                if let Some(path) = &self.path {
                    #[cfg(target_os = "macos")]
                    {
                        Command::new("open")
                            .arg(path)
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    #[cfg(target_os = "linux")]
                    {
                        Command::new("xdg-open")
                            .arg(path)
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    #[cfg(target_os = "windows")]
                    {
                        Command::new("explorer")
                            .arg(path)
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    Ok(())
                } else {
                    Err(Error::ExecutionError("No path specified".to_string()))
                }
            },
            SearchItemType::Command => {
                if let Some(cmd) = &self.executable {
                    #[cfg(unix)]
                    {
                        Command::new("sh")
                            .arg("-c")
                            .arg(cmd)
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    #[cfg(windows)]
                    {
                        Command::new("cmd")
                            .args(["/C", cmd])
                            .spawn()
                            .map_err(|e| Error::ExecutionError(e.to_string()))?;
                    }
                    Ok(())
                } else {
                    Err(Error::ExecutionError("No command specified".to_string()))
                }
            },
            SearchItemType::Plugin => {
                // Plugin execution handled separately
                Ok(())
            },
            SearchItemType::ActionItem => {
                // Action item execution handled separately
                Ok(())
            },
        }
    }
}
