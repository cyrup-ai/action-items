//! Helper functions for action execution and search operations

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use serde_json;
use tracing::{info, warn};

use crate::events::*;
use crate::resources::*;

// REMOVED: Duplicate function execute_action_with_comprehensive_management
// The correct implementation is in src/systems/actions/execution.rs

/// Execute action operation using proper ECS Task Component patterns with AsyncComputeTaskPool
pub async fn create_action_execution_task(
    action_def: ActionDefinition,
    parameters: serde_json::Value,
) -> CommandQueue {
    let mut command_queue = CommandQueue::default();
    let action_id = action_def.id.clone();
    let action_type = action_def.action_type.clone();
    let operation_start = std::time::Instant::now();

    // Execute operation based on action type using proper async patterns
    let execution_result = match action_def.action_type.as_str() {
        "open_file" => execute_file_open_operation(action_def, parameters).await,
        "open_application" => execute_application_open_operation(action_def, parameters).await,
        "run_command" => execute_command_operation(action_def, parameters).await,
        _ => Err(format!(
            "Unsupported action type: {}. Supported types: open_file, open_application, \
             run_command",
            action_def.action_type
        )),
    };

    // Use CommandQueue to update ECS World from async context
    command_queue.push(move |world: &mut World| match execution_result {
        Ok(success_result) => {
            info!("Action operation completed successfully: {}", action_type);
            world.send_event(ActionExecuteCompleted {
                action_id: action_id.clone(),
                requester: "system".to_string(),
                success: true,
                result: Some(success_result),
                error_message: None,
                execution_time: operation_start.elapsed(),
            });
        },
        Err(error_msg) => {
            warn!(
                "Action operation failed: {} - Error: {}",
                action_type, error_msg
            );
            world.send_event(ActionExecuteCompleted {
                action_id: action_id.clone(),
                requester: "system".to_string(),
                success: false,
                result: None,
                error_message: Some(error_msg),
                execution_time: operation_start.elapsed(),
            });
        },
    });

    command_queue
}

/// Execute file open operation with comprehensive error handling and platform detection
pub async fn execute_file_open_operation(
    action_def: ActionDefinition,
    parameters: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if let Some(path) = parameters.get("path").and_then(|p| p.as_str()) {
        let path_buf = std::path::PathBuf::from(path);
        if !path_buf.exists() {
            return Err(format!("File does not exist: {}", path));
        }

        #[cfg(target_os = "macos")]
        let result = tokio::process::Command::new("open")
            .arg(path)
            .output()
            .await;

        #[cfg(target_os = "linux")]
        let result = tokio::process::Command::new("xdg-open")
            .arg(path)
            .output()
            .await;

        #[cfg(target_os = "windows")]
        let result = tokio::process::Command::new("cmd")
            .args(&["/C", "start", "", path])
            .output()
            .await;

        match result {
            Ok(output) if output.status.success() => Ok(serde_json::json!({
                "action": action_def.id,
                "result": "file_opened",
                "path": path,
                "platform": std::env::consts::OS
            })),
            Ok(output) => Err(format!(
                "Failed to open file: {}",
                String::from_utf8_lossy(&output.stderr)
            )),
            Err(e) => Err(format!("Failed to execute open command: {}", e)),
        }
    } else {
        Err("Missing required 'path' parameter".to_string())
    }
}

/// Execute application open operation with platform-specific command detection
pub async fn execute_application_open_operation(
    action_def: ActionDefinition,
    parameters: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if let Some(app_name) = parameters.get("application").and_then(|a| a.as_str()) {
        #[cfg(target_os = "macos")]
        let result = tokio::process::Command::new("open")
            .args(["-a", app_name])
            .output()
            .await;

        #[cfg(target_os = "linux")]
        let result = tokio::process::Command::new("gtk-launch")
            .arg(app_name)
            .output()
            .await;

        #[cfg(target_os = "windows")]
        let result = tokio::process::Command::new("cmd")
            .args(&["/C", "start", "", app_name])
            .output()
            .await;

        match result {
            Ok(output) if output.status.success() => Ok(serde_json::json!({
                "action": action_def.id,
                "result": "application_opened",
                "application": app_name,
                "platform": std::env::consts::OS
            })),
            Ok(output) => Err(format!(
                "Failed to open application: {}",
                String::from_utf8_lossy(&output.stderr)
            )),
            Err(e) => Err(format!("Failed to execute application launch: {}", e)),
        }
    } else {
        Err("Missing required 'application' parameter".to_string())
    }
}

/// Execute shell command operation with comprehensive security validation
pub async fn execute_command_operation(
    action_def: ActionDefinition,
    parameters: serde_json::Value,
) -> Result<serde_json::Value, String> {
    if let Some(command) = parameters.get("command").and_then(|c| c.as_str()) {
        // Comprehensive security validation with production-grade patterns
        if let Err(security_error) = validate_command_security(command) {
            return Err(format!("Security validation failed: {}", security_error));
        }

        let result = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .await;

        match result {
            Ok(output) => Ok(serde_json::json!({
                "action": action_def.id,
                "result": "command_executed",
                "command": command,
                "stdout": String::from_utf8_lossy(&output.stdout),
                "stderr": String::from_utf8_lossy(&output.stderr),
                "exit_code": output.status.code().unwrap_or(-1),
                "success": output.status.success()
            })),
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    } else {
        Err("Missing required 'command' parameter".to_string())
    }
}

/// Production-grade command security validation system
/// Implements comprehensive security patterns to prevent malicious command execution
fn validate_command_security(command: &str) -> Result<(), String> {
    // Normalize command for analysis (lowercase, trimmed)
    let normalized = command.to_lowercase();
    let normalized = normalized.trim();

    // Category 1: Destructive file operations
    let destructive_patterns = [
        "rm -rf", "rm -fr", "rm -r", "rmdir /s", "del /f", "del /q", "del /s", "format ", "fdisk",
        "dd if=", "shred", "wipefs", "mkfs", "parted", "gpart",
    ];

    for pattern in &destructive_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Destructive file operation blocked: {}", pattern));
        }
    }

    // Category 2: System modification
    let system_patterns = [
        "sudo ",
        "su ",
        "doas ",
        "runas",
        "chmod 777",
        "chmod -r",
        "chown -r",
        "systemctl",
        "service ",
        "launchctl",
        "registry ",
        "regedit",
        "reg add",
        "reg delete",
    ];

    for pattern in &system_patterns {
        if normalized.contains(pattern) {
            return Err(format!("System modification blocked: {}", pattern));
        }
    }

    // Category 3: Network/security risks
    let network_patterns = [
        "wget ", "curl ", "nc ", "netcat", "telnet", "ssh ", "scp ", "rsync ", "ftp ", "iptables",
        "ufw ", "firewall", "netsh",
    ];

    for pattern in &network_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Network operation blocked: {}", pattern));
        }
    }

    // Category 4: Code execution risks
    let execution_patterns = [
        "eval ",
        "exec ",
        "python -c",
        "perl -e",
        "ruby -e",
        "node -e",
        "bash -c",
        "sh -c",
        "powershell ",
        "pwsh ",
        "cmd /c",
    ];

    for pattern in &execution_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Code execution blocked: {}", pattern));
        }
    }

    // Category 5: Process manipulation
    let process_patterns = [
        "kill ", "killall", "pkill", "taskkill", "ps aux", "top ", "htop", "proc", "lsof",
        "netstat", "ss ",
    ];

    for pattern in &process_patterns {
        if normalized.contains(pattern) {
            return Err(format!("Process manipulation blocked: {}", pattern));
        }
    }

    // Category 6: Path traversal and injection
    if normalized.contains("../") || normalized.contains("..\\") {
        return Err("Path traversal attack blocked".to_string());
    }

    if normalized.contains("$(") || normalized.contains("`") {
        return Err("Command injection blocked".to_string());
    }

    if normalized.contains("&&") || normalized.contains("||") || normalized.contains(";") {
        return Err("Command chaining blocked".to_string());
    }

    // Category 7: Environment manipulation
    if normalized.contains("export ") || normalized.contains("set ") || normalized.contains("env ")
    {
        return Err("Environment modification blocked".to_string());
    }

    // Category 8: Length and character validation
    if command.len() > 1000 {
        return Err("Command too long - potential buffer overflow".to_string());
    }

    // Check for suspicious repeated characters (potential DoS)
    let mut char_counts = std::collections::HashMap::new();
    for ch in command.chars() {
        *char_counts.entry(ch).or_insert(0) += 1;
        if char_counts[&ch] > 100 {
            return Err("Suspicious character repetition detected".to_string());
        }
    }

    // Category 9: Binary and encoded content
    if command.chars().any(|c| c as u32 > 127) {
        return Err("Non-ASCII characters detected - potential encoding attack".to_string());
    }

    Ok(())
}
