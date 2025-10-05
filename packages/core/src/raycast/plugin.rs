use std::sync::Arc;

use action_items_common::plugin_interface::{ActionItem, ActionType, Icon, ItemAction};
use action_items_native::native::NativePlugin;
use action_items_native::{Error as NativeError, PluginContext, PluginManifest};
use bevy::tasks::{AsyncComputeTaskPool, Task};
use log::info;
use serde_json::{Value, json};

use crate::raycast::adapter::RaycastAdapter;
use crate::raycast::loader::RaycastExtension;

/// Wrapper that makes a Raycast extension behave like a native plugin
pub struct RaycastPlugin {
    extension: RaycastExtension,
    adapter: Arc<RaycastAdapter>,
    manifest: PluginManifest,
}

impl RaycastPlugin {
    pub fn new(extension: RaycastExtension, adapter: Arc<RaycastAdapter>) -> Self {
        let manifest = adapter.to_plugin_manifest(&extension);
        Self {
            extension,
            adapter,
            manifest,
        }
    }
}

impl NativePlugin for RaycastPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn initialize(
        &mut self,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), NativeError>> {
        let _extension = self.extension.clone();

        task_pool.spawn(async move {
            // Initialize Deno runtime for this extension
            // For now, just return success
            Ok(())
        })
    }

    fn search(
        &self,
        query: String,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<ActionItem>, NativeError>> {
        let extension = self.extension.clone();
        let _adapter = self.adapter.clone();

        task_pool.spawn(async move {
            // Execute the actual Raycast extension's search command
            let action_id = format!("raycast-{}-action-{}", extension.id, query);

            // Execute Raycast CLI command to search the extension
            let raycast_result = std::process::Command::new("raycast")
                .arg("search")
                .arg(&extension.id)
                .arg(&query)
                .output();

            let (title, subtitle) = match raycast_result {
                Ok(output) if output.status.success() => {
                    // Parse Raycast JSON output
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    match serde_json::from_str::<serde_json::Value>(&output_str) {
                        Ok(json) => {
                            let title = json
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&query)
                                .to_string();
                            let subtitle = json
                                .get("subtitle")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                                .or_else(|| Some(format!("Raycast: {}", extension.description)));
                            (title, subtitle)
                        },
                        Err(_) => {
                            // Fallback if JSON parsing fails
                            (
                                format!("{} - {}", extension.title, query),
                                Some(format!("Raycast extension: {}", extension.description)),
                            )
                        },
                    }
                },
                Ok(_) | Err(_) => {
                    // Fallback if Raycast command fails
                    log::warn!(
                        "Raycast command failed for extension {}, falling back to mock result",
                        extension.id
                    );
                    (
                        format!("{} - {}", extension.title, query),
                        Some(format!("Raycast extension: {}", extension.description)),
                    )
                },
            };
            let result = ActionItem {
                id: format!("raycast-{}-{}", extension.id, query),
                title,
                subtitle,
                description: None,
                tags: vec![],
                icon: extension
                    .icon
                    .as_ref()
                    .map(|icon_path| Icon::File(icon_path.into())),
                actions: vec![ItemAction {
                    id: action_id.clone(),
                    title: "Open".to_string(),
                    icon: None,
                    shortcut: None,
                    action_type: ActionType::OpenUrl(format!(
                        "raycast://extensions/{}/commands/{}",
                        extension.id, query
                    )),
                }],
                item_badges: Vec::new(),
                metadata: Some(json!({
                    "extension_id": extension.id,
                    "query": query,
                })),
                score: 90.0,
                created_at: Some(chrono::Utc::now()),
                updated_at: Some(chrono::Utc::now()),
            };

            Ok(vec![result])
        })
    }

    fn execute_command(
        &mut self,
        command_id: String,
        _context: PluginContext,
        _args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, NativeError>> {
        let extension = self.extension.clone();

        task_pool.spawn(async move {
            // Execute the Raycast command by loading the TypeScript/JavaScript
            info!(
                "Executing Raycast command {} for extension {}",
                command_id, extension.id
            );

            // Find the command in the extension
            let command = extension.commands.iter().find(|cmd| cmd.name == command_id);

            if let Some(cmd) = command {
                // Load the main entry point for this extension
                let main_path = extension.path.join("src").join("index.ts");
                if main_path.exists() {
                    // For now, simulate successful execution
                    // In a full implementation, this would:
                    // 1. Load the TypeScript file
                    // 2. Initialize Deno runtime with @raycast/api bindings
                    // 3. Execute the command handler
                    // 4. Return the results
                    info!("Would execute {} from {}", cmd.title, main_path.display());
                    Ok(Some(serde_json::Value::String(format!(
                        "Executed {}",
                        cmd.title
                    ))))
                } else {
                    Err(NativeError::ExecutionError(format!(
                        "Extension entry point not found: {}",
                        main_path.display()
                    )))
                }
            } else {
                Err(NativeError::ExecutionError(format!(
                    "Command {} not found in extension {}",
                    command_id, extension.id
                )))
            }
        })
    }

    fn execute_action(
        &mut self,
        action_id: String,
        _context: PluginContext,
        _args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, NativeError>> {
        let extension = self.extension.clone();

        task_pool.spawn(async move {
            // Execute the Raycast action
            info!(
                "Executing Raycast action {} for extension {}",
                action_id, extension.id
            );

            // Load the main entry point for this extension
            let main_path = extension.path.join("src").join("index.ts");
            if main_path.exists() {
                // For now, simulate successful execution
                // In a full implementation, this would:
                // 1. Load the TypeScript file
                // 2. Find the specific action handler
                // 3. Initialize Deno runtime with @raycast/api bindings
                // 4. Execute the action with provided context and args
                // 5. Return the results
                info!(
                    "Would execute action {} from {}",
                    action_id,
                    main_path.display()
                );
                Ok(Some(serde_json::Value::String(format!(
                    "Executed action {action_id}"
                ))))
            } else {
                Err(NativeError::ExecutionError(format!(
                    "Extension entry point not found: {}",
                    main_path.display()
                )))
            }
        })
    }

    fn background_refresh(
        &mut self,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), NativeError>> {
        task_pool.spawn(async move {
            // Raycast extensions don't have a refresh concept
            Ok(())
        })
    }

    fn cleanup(&mut self, task_pool: &AsyncComputeTaskPool) -> Task<Result<(), NativeError>> {
        task_pool.spawn(async move {
            // Clean up Deno runtime if needed
            Ok(())
        })
    }
}
