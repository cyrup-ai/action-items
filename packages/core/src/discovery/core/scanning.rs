use std::collections::HashSet;
use std::path::{Path, PathBuf};

use super::detection::is_plugin_file;
use super::types::{DiscoveredPlugin, DiscoveryConfig};
use super::wrapper_creation::{
    create_plugin_wrapper_from_file, create_plugin_wrapper_from_rust_project,
};
use crate::Result;

/// Scans a directory for plugin files and creates wrappers
pub fn scan_directory_for_plugin_wrappers(
    dir: &Path,
    config: &DiscoveryConfig,
    visited: &mut HashSet<PathBuf>,
) -> Result<Vec<DiscoveredPlugin>> {
    let mut wrappers = Vec::new();

    // Prevent infinite loops with symlinks
    let canonical_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    if visited.contains(&canonical_dir) {
        return Ok(wrappers);
    }
    visited.insert(canonical_dir);

    let entries = std::fs::read_dir(dir)?;

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_file() && is_plugin_file(&path) {
            if let Ok(wrapper) = create_plugin_wrapper_from_file(&path) {
                wrappers.push(wrapper);
            }
        } else if path.is_dir() && path.join("Cargo.toml").exists() {
            // This is a Rust plugin project - build and create wrapper
            if let Ok(wrapper) = create_plugin_wrapper_from_rust_project(&path) {
                wrappers.push(wrapper);
            }
        } else if path.is_dir() && config.recursive_scan {
            // Check depth limit
            let depth = path.components().count() - dir.components().count();
            if depth < config.max_depth
                && let Ok(mut sub_wrappers) =
                    scan_directory_for_plugin_wrappers(&path, config, visited)
            {
                wrappers.append(&mut sub_wrappers);
            }
        }
    }

    Ok(wrappers)
}
