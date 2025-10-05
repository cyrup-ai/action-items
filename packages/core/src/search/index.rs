use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::*;

use crate::error::Result;
use crate::search::item::{SearchItem, SearchItemType};

#[derive(Resource)]
pub struct SearchIndex {
    items: HashMap<String, SearchItem>,
    indexed_at: std::time::SystemTime,
}

impl SearchIndex {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            indexed_at: std::time::SystemTime::now(),
        }
    }

    pub fn add_item(&mut self, item: SearchItem) {
        self.items.insert(item.id.clone(), item);
    }

    pub fn remove_item(&mut self, id: &str) -> Option<SearchItem> {
        self.items.remove(id)
    }

    pub fn search(&self, query: &str) -> Vec<SearchItem> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<SearchItem> = self
            .items
            .values()
            .filter_map(|item| {
                let mut score = 0.0;

                // Exact title match gets highest score
                if item.title.to_lowercase() == query_lower {
                    score += 100.0;
                } else if item.title.to_lowercase().starts_with(&query_lower) {
                    score += 75.0;
                } else if item.title.to_lowercase().contains(&query_lower) {
                    score += 50.0;
                }

                // Description match
                if item.description.to_lowercase().contains(&query_lower) {
                    score += 25.0;
                }

                // Keywords match
                for keyword in &item.keywords {
                    if keyword.to_lowercase().contains(&query_lower) {
                        score += 30.0;
                    }
                }

                // Path match for files
                if let Some(path) = &item.path
                    && let Some(filename) = path.file_name()
                    && filename
                        .to_string_lossy()
                        .to_lowercase()
                        .contains(&query_lower)
                {
                    score += 40.0;
                }

                if score > 0.0 {
                    let mut result = item.clone();
                    result.score = score;
                    Some(result)
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(10); // Limit to top 10 results
        results
    }

    pub fn get_item(&self, id: &str) -> Option<&SearchItem> {
        self.items.get(id)
    }

    pub fn rebuild_index(&mut self) -> Result<()> {
        self.items.clear();

        // Try to index applications, but don't fail the whole operation
        if let Err(e) = crate::search::platforms::index_applications(self) {
            error!("Failed to index applications: {}", e);
        }

        // Try to index common directories, but don't fail the whole operation
        if let Err(e) = self.index_common_directories() {
            error!("Failed to index common directories: {}", e);
        }

        // Always index system commands (cannot fail)
        self.index_system_commands();

        self.indexed_at = std::time::SystemTime::now();

        // Log final index size
        info!("Search index rebuilt with {} items", self.items.len());

        Ok(())
    }

    pub fn index_common_directories(&mut self) -> Result<()> {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();

        let common_dirs = [
            (format!("{home}/Desktop"), "Desktop"),
            (format!("{home}/Documents"), "Documents"),
            (format!("{home}/Downloads"), "Downloads"),
            (format!("{home}/Pictures"), "Pictures"),
            (format!("{home}/Music"), "Music"),
            (format!("{home}/Videos"), "Videos"),
        ];

        for (path_str, name) in common_dirs {
            let path = PathBuf::from(&path_str);
            if path.exists() {
                let item = SearchItem::new(
                    format!("dir_{name}"),
                    name.to_string(),
                    format!("Directory: {path_str}"),
                    SearchItemType::Directory,
                )
                .with_path(path);

                self.add_item(item);
            }
        }
        Ok(())
    }

    pub fn index_system_commands(&mut self) {
        let commands = [
            ("terminal", "Terminal", "Open terminal"),
            ("logout", "Logout", "Logout current user"),
            ("shutdown", "Shutdown", "Shutdown computer"),
            ("restart", "Restart", "Restart computer"),
            ("sleep", "Sleep", "Put computer to sleep"),
        ];

        for (cmd, name, desc) in commands {
            let executable = match cmd {
                "terminal" => {
                    #[cfg(target_os = "macos")]
                    {
                        "Terminal"
                    }
                    #[cfg(target_os = "linux")]
                    {
                        "gnome-terminal"
                    }
                    #[cfg(target_os = "windows")]
                    {
                        "cmd"
                    }
                },
                "logout" => {
                    #[cfg(target_os = "macos")]
                    {
                        "osascript -e 'tell application \"System Events\" to log out'"
                    }
                    #[cfg(target_os = "linux")]
                    {
                        "gnome-session-quit --logout"
                    }
                    #[cfg(target_os = "windows")]
                    {
                        "shutdown /l"
                    }
                },
                "shutdown" => {
                    #[cfg(target_os = "macos")]
                    {
                        "sudo shutdown -h now"
                    }
                    #[cfg(target_os = "linux")]
                    {
                        "shutdown -h now"
                    }
                    #[cfg(target_os = "windows")]
                    {
                        "shutdown /s /t 0"
                    }
                },
                "restart" => {
                    #[cfg(target_os = "macos")]
                    {
                        "sudo shutdown -r now"
                    }
                    #[cfg(target_os = "linux")]
                    {
                        "shutdown -r now"
                    }
                    #[cfg(target_os = "windows")]
                    {
                        "shutdown /r /t 0"
                    }
                },
                "sleep" => {
                    #[cfg(target_os = "macos")]
                    {
                        "pmset sleepnow"
                    }
                    #[cfg(target_os = "linux")]
                    {
                        "systemctl suspend"
                    }
                    #[cfg(target_os = "windows")]
                    {
                        "rundll32.exe powrprof.dll,SetSuspendState 0,1,0"
                    }
                },
                _ => cmd,
            };

            let item = SearchItem::new(
                format!("cmd_{cmd}"),
                name.to_string(),
                desc.to_string(),
                SearchItemType::Command,
            )
            .with_executable(executable.to_string());

            self.add_item(item);
        }
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}
