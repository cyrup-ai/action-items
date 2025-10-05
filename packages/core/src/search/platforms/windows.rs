use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use jwalk::WalkDir;
use rayon::prelude::*;

use crate::error::Result;
use crate::search::index::SearchIndex;
use crate::search::item::{SearchItem, SearchItemType};

pub fn index_applications(search_index: &mut SearchIndex) -> Result<()> {
    let mut indexed_apps = HashSet::new();

    index_start_menu_applications(search_index, &mut indexed_apps)?;
    index_program_files_applications(search_index, &mut indexed_apps)?;
    index_registry_applications(search_index, &mut indexed_apps)?;

    Ok(())
}

fn index_start_menu_applications(
    search_index: &mut SearchIndex,
    indexed_apps: &mut HashSet<String>,
) -> Result<()> {
    let start_menu_paths = [
        format!(
            "{}/Microsoft/Windows/Start Menu/Programs",
            std::env::var("APPDATA").unwrap_or_default()
        ),
        format!(
            "{}/Microsoft/Windows/Start Menu/Programs",
            std::env::var("PROGRAMDATA").unwrap_or_default()
        ),
    ];

    for start_menu_path in start_menu_paths {
        if let Ok(entries) = scan_directory_recursive(&start_menu_path, 10) {
            for entry in entries {
                if let Some(name) = entry.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".lnk") {
                        let app_name = name.strip_suffix(".lnk").unwrap_or(name);
                        if !indexed_apps.contains(app_name) {
                            indexed_apps.insert(app_name.to_string());

                            let item = SearchItem::new(
                                format!("app_{}", app_name),
                                app_name.to_string(),
                                format!("Application: {}", app_name),
                                SearchItemType::Application,
                            )
                            .with_path(entry.clone())
                            .with_executable(format!("explorer.exe \"{}\"", entry.display()));

                            search_index.add_item(item);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn index_program_files_applications(
    search_index: &mut SearchIndex,
    indexed_apps: &mut HashSet<String>,
) -> Result<()> {
    let program_paths = [
        std::env::var("PROGRAMFILES").unwrap_or_default(),
        std::env::var("PROGRAMFILES(X86)").unwrap_or_default(),
        format!(
            "{}/WindowsApps",
            std::env::var("PROGRAMFILES").unwrap_or_default()
        ),
    ];

    for program_path in program_paths {
        if !program_path.is_empty() {
            if let Ok(entries) = fs::read_dir(&program_path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        if let Some(dir_name) = entry.file_name().to_str() {
                            if !indexed_apps.contains(dir_name) {
                                if let Ok(exe_files) = find_executables_in_dir(&entry.path()) {
                                    if !exe_files.is_empty() {
                                        indexed_apps.insert(dir_name.to_string());

                                        let main_exe = &exe_files[0];
                                        let exe_name = main_exe
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or(dir_name);

                                        let item = SearchItem::new(
                                            format!("app_{}", exe_name),
                                            exe_name.to_string(),
                                            format!("Application: {}", dir_name),
                                            SearchItemType::Application,
                                        )
                                        .with_path(main_exe.clone())
                                        .with_executable(format!("\"{}\"", main_exe.display()));

                                        search_index.add_item(item);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn index_registry_applications(
    search_index: &mut SearchIndex,
    indexed_apps: &mut HashSet<String>,
) -> Result<()> {
    let system_apps = [
        ("notepad.exe", "Notepad", "Text editor"),
        ("calc.exe", "Calculator", "Calculator"),
        ("mspaint.exe", "Paint", "Image editor"),
        ("cmd.exe", "Command Prompt", "Command line interface"),
        ("powershell.exe", "PowerShell", "PowerShell command line"),
        ("explorer.exe", "File Explorer", "File manager"),
        ("taskmgr.exe", "Task Manager", "System task manager"),
        ("control.exe", "Control Panel", "System control panel"),
        ("regedit.exe", "Registry Editor", "Windows registry editor"),
        (
            "msconfig.exe",
            "System Configuration",
            "System configuration utility",
        ),
    ];

    for (exec, name, desc) in system_apps {
        if !indexed_apps.contains(name) {
            indexed_apps.insert(name.to_string());

            let item = SearchItem::new(
                format!("app_{}", name),
                name.to_string(),
                desc.to_string(),
                SearchItemType::Application,
            )
            .with_executable(exec.to_string());

            search_index.add_item(item);
        }
    }
    Ok(())
}

fn scan_directory_recursive(dir_path: &str, max_depth: usize) -> Result<Vec<PathBuf>> {
    let path = PathBuf::from(dir_path);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let results: Vec<PathBuf> = WalkDir::new(path)
        .max_depth(max_depth)
        .skip_hidden(true)
        .follow_links(true)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path())
        .collect();

    Ok(results)
}

fn find_executables_in_dir(dir_path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut executables = Vec::new();

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            if let Some(extension) = entry.path().extension() {
                if extension.to_string_lossy().to_lowercase() == "exe" {
                    executables.push(entry.path());
                }
            }
        }
    }

    Ok(executables)
}
