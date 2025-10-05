use std::path::{Path, PathBuf};

use log::info;

use crate::Result;

/// Find existing library or build the plugin project
pub fn find_or_build_plugin_library(dir: &Path) -> Result<PathBuf> {
    let target_dir = dir.join("target").join("release");

    // Determine expected library name based on platform
    let (prefix, extension) = if cfg!(target_os = "windows") {
        ("", "dll")
    } else if cfg!(target_os = "macos") {
        ("lib", "dylib")
    } else {
        ("lib", "so")
    };

    // Try to find existing library
    if target_dir.exists() {
        for entry in std::fs::read_dir(&target_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|s| s.to_str())
                && file_name.starts_with(prefix)
                && path.extension().and_then(|s| s.to_str()) == Some(extension)
                && !file_name.contains(".d")
                && !file_name.contains(".rlib")
                && !needs_rebuild(dir, &path)?
            {
                info!("Using existing plugin library: {}", path.display());
                return Ok(path);
            }
        }
    }

    // Build the plugin
    info!("Building plugin at {}", dir.display());
    let output = std::process::Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--manifest-path")
        .arg(dir.join("Cargo.toml"))
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(crate::Error::PluginLoadError(format!(
            "Failed to build plugin at {}: {}",
            dir.display(),
            stderr
        )));
    }

    // Find the built library
    for entry in std::fs::read_dir(&target_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str())
            && file_name.starts_with(prefix)
            && path.extension().and_then(|s| s.to_str()) == Some(extension)
            && !file_name.contains(".d")
            && !file_name.contains(".rlib")
        {
            info!("Built plugin library: {}", path.display());
            return Ok(path);
        }
    }

    Err(crate::Error::PluginLoadError(format!(
        "Could not find built library for plugin at {}",
        dir.display()
    )))
}

/// Check if the plugin needs to be rebuilt
fn needs_rebuild(src_dir: &Path, lib_path: &Path) -> Result<bool> {
    let lib_modified = std::fs::metadata(lib_path)?.modified()?;

    // Check Cargo.toml
    let cargo_toml = src_dir.join("Cargo.toml");
    if cargo_toml.exists() {
        let cargo_modified = std::fs::metadata(&cargo_toml)?.modified()?;
        if cargo_modified > lib_modified {
            return Ok(true);
        }
    }

    // Check src directory
    let src_path = src_dir.join("src");
    if src_path.exists() && check_dir_modified(&src_path, lib_modified)? {
        return Ok(true);
    }

    Ok(false)
}

/// Recursively check if any file in directory is newer than the given time
fn check_dir_modified(dir: &Path, compare_time: std::time::SystemTime) -> Result<bool> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = std::fs::metadata(&path)?;

        if metadata.modified()? > compare_time {
            return Ok(true);
        }

        if path.is_dir() && check_dir_modified(&path, compare_time)? {
            return Ok(true);
        }
    }
    Ok(false)
}
