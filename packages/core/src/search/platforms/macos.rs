use std::fs;

use crate::error::Result;
use crate::search::index::SearchIndex;
use crate::search::item::{SearchItem, SearchItemType};

pub fn index_applications(search_index: &mut SearchIndex) -> Result<()> {
    let app_paths = [
        "/Applications",
        "/System/Applications",
        &format!(
            "{}/.Applications",
            std::env::var("HOME").unwrap_or_default()
        ),
    ];

    for app_path in app_paths {
        if let Ok(entries) = fs::read_dir(app_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str()
                    && name.ends_with(".app")
                {
                    let app_name = name.strip_suffix(".app").unwrap_or(name);
                    let item = SearchItem::new(
                        format!("app_{app_name}"),
                        app_name.to_string(),
                        format!("Application: {app_name}"),
                        SearchItemType::Application,
                    )
                    .with_path(entry.path())
                    .with_executable(app_name.to_string());

                    search_index.add_item(item);
                }
            }
        }
    }
    Ok(())
}
