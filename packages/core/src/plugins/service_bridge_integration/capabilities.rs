use std::collections::HashMap;

use ecs_service_bridge::components::Capability;
use ecs_service_bridge::types::ServiceError;

use crate::discovery::DiscoveredPlugin;

/// Extract capabilities from discovered plugin metadata
pub fn extract_plugin_capabilities(
    plugin: &DiscoveredPlugin,
) -> Result<Vec<Capability>, ServiceError> {
    let mut capabilities = Vec::new();

    match plugin {
        DiscoveredPlugin::Native(wrapper) => {
            extract_native_capabilities(wrapper, &mut capabilities);
        },
        DiscoveredPlugin::Extism(wrapper) => {
            extract_extism_capabilities(wrapper, &mut capabilities);
        },
        DiscoveredPlugin::Raycast(wrapper) => {
            extract_raycast_capabilities(wrapper, &mut capabilities);
        },
        DiscoveredPlugin::Deno(wrapper) => {
            extract_deno_capabilities(wrapper, &mut capabilities);
        },
    }

    Ok(capabilities)
}

/// Extract capabilities from Deno plugin wrapper
fn extract_deno_capabilities(
    wrapper: &crate::runtime::plugin_wrapper::DenoPluginWrapper,
    capabilities: &mut Vec<Capability>,
) {
    let metadata = wrapper.metadata();

    // Add basic Deno runtime capability
    let mut deno_metadata = HashMap::new();
    deno_metadata.insert("plugin_type".to_string(), "deno".to_string());
    deno_metadata.insert("runtime".to_string(), "deno".to_string());
    deno_metadata.insert("permission".to_string(), "script_execution".to_string());

    capabilities.push(Capability {
        name: "deno_runtime".to_string(),
        version: "1.0".to_string(),
        description: "Deno JavaScript/TypeScript runtime capability".to_string(),
        metadata: deno_metadata,
    });

    // Add web API capabilities if available
    if metadata.manifest.capabilities.network_access {
        let mut web_metadata = HashMap::new();
        web_metadata.insert("plugin_type".to_string(), "deno".to_string());
        web_metadata.insert("api_type".to_string(), "web".to_string());
        web_metadata.insert("permission".to_string(), "network_access".to_string());

        capabilities.push(Capability {
            name: "web_apis".to_string(),
            version: "1.0".to_string(),
            description: "Web API access capability".to_string(),
            metadata: web_metadata,
        });
    }

    // Add file system capabilities if available
    if metadata.manifest.capabilities.file_system_access {
        let mut fs_metadata = HashMap::new();
        fs_metadata.insert("plugin_type".to_string(), "deno".to_string());
        fs_metadata.insert("access_type".to_string(), "file_system".to_string());
        fs_metadata.insert("permission".to_string(), "file_system_access".to_string());

        capabilities.push(Capability {
            name: "file_system".to_string(),
            version: "1.0".to_string(),
            description: "File system access capability".to_string(),
            metadata: fs_metadata,
        });
    }

    // Add network access capabilities if available
    if metadata.manifest.capabilities.network_access {
        let mut network_metadata = HashMap::new();
        network_metadata.insert("plugin_type".to_string(), "deno".to_string());
        network_metadata.insert("access_type".to_string(), "network".to_string());
        network_metadata.insert("permission".to_string(), "network_access".to_string());

        capabilities.push(Capability {
            name: "network".to_string(),
            version: "1.0".to_string(),
            description: "Network access capability".to_string(),
            metadata: network_metadata,
        });
    }

    // Add system commands capabilities if available
    if metadata.manifest.capabilities.system_commands {
        let mut sys_metadata = HashMap::new();
        sys_metadata.insert("plugin_type".to_string(), "deno".to_string());
        sys_metadata.insert("access_type".to_string(), "system_commands".to_string());
        sys_metadata.insert("permission".to_string(), "system_commands".to_string());

        capabilities.push(Capability {
            name: "system_commands".to_string(),
            version: "1.0".to_string(),
            description: "System commands execution capability".to_string(),
            metadata: sys_metadata,
        });
    }

    // Add clipboard capabilities if available
    if metadata.manifest.capabilities.clipboard_access {
        let mut clipboard_metadata = HashMap::new();
        clipboard_metadata.insert("plugin_type".to_string(), "deno".to_string());
        clipboard_metadata.insert("access_type".to_string(), "clipboard".to_string());
        clipboard_metadata.insert("permission".to_string(), "clipboard_access".to_string());

        capabilities.push(Capability {
            name: "clipboard".to_string(),
            version: "1.0".to_string(),
            description: "Clipboard access capability".to_string(),
            metadata: clipboard_metadata,
        });
    }

    // Add notifications capabilities if available
    if metadata.manifest.capabilities.notifications {
        let mut notifications_metadata = HashMap::new();
        notifications_metadata.insert("plugin_type".to_string(), "deno".to_string());
        notifications_metadata.insert("access_type".to_string(), "notifications".to_string());
        notifications_metadata.insert("permission".to_string(), "notifications".to_string());

        capabilities.push(Capability {
            name: "notifications".to_string(),
            version: "1.0".to_string(),
            description: "System notifications capability".to_string(),
            metadata: notifications_metadata,
        });
    }
}

/// Extract capabilities from Native plugin wrapper
fn extract_native_capabilities(
    wrapper: &crate::plugins::native::wrapper::NativePluginWrapper,
    capabilities: &mut Vec<Capability>,
) {
    let metadata = wrapper.metadata();

    // Add basic native runtime capability
    let mut native_metadata = HashMap::new();
    native_metadata.insert("plugin_type".to_string(), "native".to_string());
    native_metadata.insert("runtime".to_string(), "native".to_string());
    native_metadata.insert("permission".to_string(), "native_execution".to_string());

    capabilities.push(Capability {
        name: "native_runtime".to_string(),
        version: "1.0".to_string(),
        description: "Native binary runtime capability".to_string(),
        metadata: native_metadata,
    });

    // Add permission-based capabilities from the permissions struct
    let permissions = &metadata.manifest.permissions;

    if permissions.read_clipboard || permissions.write_clipboard {
        let mut clipboard_metadata = HashMap::new();
        clipboard_metadata.insert("plugin_type".to_string(), "native".to_string());
        clipboard_metadata.insert("permission_type".to_string(), "clipboard".to_string());
        clipboard_metadata.insert("runtime".to_string(), "native".to_string());

        capabilities.push(Capability {
            name: "native_clipboard".to_string(),
            version: "1.0".to_string(),
            description: "Native clipboard access capability".to_string(),
            metadata: clipboard_metadata,
        });
    }

    if !permissions.read_files.is_empty() || !permissions.write_files.is_empty() {
        let mut file_metadata = HashMap::new();
        file_metadata.insert("plugin_type".to_string(), "native".to_string());
        file_metadata.insert("permission_type".to_string(), "file_access".to_string());
        file_metadata.insert("runtime".to_string(), "native".to_string());

        capabilities.push(Capability {
            name: "native_file_access".to_string(),
            version: "1.0".to_string(),
            description: "Native file system access capability".to_string(),
            metadata: file_metadata,
        });
    }

    if !permissions.execute_commands.is_empty() {
        let mut exec_metadata = HashMap::new();
        exec_metadata.insert("plugin_type".to_string(), "native".to_string());
        exec_metadata.insert(
            "permission_type".to_string(),
            "command_execution".to_string(),
        );
        exec_metadata.insert("runtime".to_string(), "native".to_string());

        capabilities.push(Capability {
            name: "native_command_execution".to_string(),
            version: "1.0".to_string(),
            description: "Native command execution capability".to_string(),
            metadata: exec_metadata,
        });
    }
}

/// Extract capabilities from Extism plugin wrapper
fn extract_extism_capabilities(
    wrapper: &crate::plugins::extism::wrapper::ExtismPluginWrapper,
    capabilities: &mut Vec<Capability>,
) {
    let _metadata = wrapper.metadata();

    // Add basic Extism runtime capability
    let mut extism_metadata = HashMap::new();
    extism_metadata.insert("plugin_type".to_string(), "extism".to_string());
    extism_metadata.insert("runtime".to_string(), "wasm".to_string());
    extism_metadata.insert("permission".to_string(), "wasm_execution".to_string());

    capabilities.push(Capability {
        name: "extism_runtime".to_string(),
        version: "1.0".to_string(),
        description: "Extism WebAssembly runtime capability".to_string(),
        metadata: extism_metadata,
    });

    // Add basic WASM runtime capability for Extism plugins
    let mut wasm_metadata = HashMap::new();
    wasm_metadata.insert("plugin_type".to_string(), "extism".to_string());
    wasm_metadata.insert("runtime".to_string(), "wasm".to_string());
    wasm_metadata.insert("permission".to_string(), "wasm_execution".to_string());

    capabilities.push(Capability {
        name: "wasm_execution".to_string(),
        version: "1.0".to_string(),
        description: "WebAssembly execution capability".to_string(),
        metadata: wasm_metadata,
    });
}

/// Extract capabilities from Raycast plugin wrapper
fn extract_raycast_capabilities(
    wrapper: &crate::raycast::wrapper::RaycastPluginWrapper,
    capabilities: &mut Vec<Capability>,
) {
    let metadata = wrapper.metadata();

    // Add basic Raycast runtime capability
    let mut raycast_metadata = HashMap::new();
    raycast_metadata.insert("plugin_type".to_string(), "raycast".to_string());
    raycast_metadata.insert("runtime".to_string(), "raycast".to_string());
    raycast_metadata.insert("permission".to_string(), "raycast_execution".to_string());

    capabilities.push(Capability {
        name: "raycast_runtime".to_string(),
        version: "1.0".to_string(),
        description: "Raycast extension runtime capability".to_string(),
        metadata: raycast_metadata,
    });

    // Add basic Raycast runtime capability
    let mut raycast_metadata = HashMap::new();
    raycast_metadata.insert("plugin_type".to_string(), "raycast".to_string());
    raycast_metadata.insert("runtime".to_string(), "raycast".to_string());
    raycast_metadata.insert("permission".to_string(), "raycast_execution".to_string());

    capabilities.push(Capability {
        name: "raycast_execution".to_string(),
        version: "1.0".to_string(),
        description: "Raycast plugin execution capability".to_string(),
        metadata: raycast_metadata,
    });

    // Add capabilities based on Raycast commands
    for command in &metadata.commands {
        let mut command_metadata = HashMap::new();
        command_metadata.insert("plugin_type".to_string(), "raycast".to_string());
        command_metadata.insert("command_name".to_string(), command.clone());
        command_metadata.insert("command_title".to_string(), command.clone());
        command_metadata.insert("permission".to_string(), "command_execution".to_string());

        capabilities.push(Capability {
            name: format!("raycast_command_{}", command),
            version: "1.0".to_string(),
            description: format!("Raycast command: {}", command),
            metadata: command_metadata,
        });
    }

    // Add search capability if the plugin has search functionality
    if metadata
        .commands
        .iter()
        .any(|cmd| cmd == "search" || cmd.to_lowercase().contains("search"))
    {
        let mut search_metadata = HashMap::new();
        search_metadata.insert("plugin_type".to_string(), "raycast".to_string());
        search_metadata.insert("capability_type".to_string(), "search".to_string());
        search_metadata.insert("permission".to_string(), "raycast_search".to_string());

        capabilities.push(Capability {
            name: "raycast_search".to_string(),
            version: "1.0".to_string(),
            description: "Raycast search capability".to_string(),
            metadata: search_metadata,
        });
    }

    // Add preferences capability if available
    if !metadata.metadata.is_empty() {
        let mut prefs_metadata = HashMap::new();
        prefs_metadata.insert("plugin_type".to_string(), "raycast".to_string());
        prefs_metadata.insert("feature".to_string(), "preferences".to_string());
        prefs_metadata.insert("permission".to_string(), "preferences_access".to_string());

        capabilities.push(Capability {
            name: "preferences".to_string(),
            version: "1.0".to_string(),
            description: "User preferences capability".to_string(),
            metadata: prefs_metadata,
        });
    }
}
