use std::fs;

use crate::error::Result;
use crate::search::index::SearchIndex;
use crate::search::item::{SearchItem, SearchItemType};

pub fn index_applications(search_index: &mut SearchIndex) -> Result<()> {
    let desktop_paths = [
        "/usr/share/applications",
        "/usr/local/share/applications",
        &format!(
            "{}/.local/share/applications",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    for desktop_path in desktop_paths {
        if let Ok(entries) = fs::read_dir(desktop_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            if let (Some(app_name), Some(exec)) = (
                                extract_desktop_field(&content, "Name"),
                                extract_desktop_field(&content, "Exec"),
                            ) {
                                let description = extract_desktop_field(&content, "Comment")
                                    .unwrap_or_else(|| format!("Application: {}", app_name));

                                let item = SearchItem::new(
                                    format!("app_{}", app_name),
                                    app_name,
                                    description,
                                    SearchItemType::Application,
                                )
                                .with_executable(exec);

                                search_index.add_item(item);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_desktop_field(content: &str, field: &str) -> Option<String> {
    for line in content.lines() {
        if line.starts_with(&format!("{}=", field)) {
            return Some(line.split('=').skip(1).collect::<Vec<_>>().join("="));
        }
    }
    None
}
