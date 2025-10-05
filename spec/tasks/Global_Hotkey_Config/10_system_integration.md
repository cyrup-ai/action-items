# Global Hotkey Config - System Integration

## Task: Implement OS-Level Integration and Platform Services

### File: `ui/src/platform/system_integration.rs` (new file)

Create comprehensive system integration layer for startup management, menu bar control, and OS-specific functionality.

### Implementation Requirements

#### Startup Integration System
- File: `ui/src/platform/startup_integration.rs` (new file, line 1-134)
- macOS login items integration using LaunchServices framework
- Windows startup registry management and service integration
- Linux autostart desktop entry management
- Safe registration/deregistration with proper error handling

#### Menu Bar Integration
- File: `ui/src/platform/menu_bar.rs` (new file, line 1-89)
- macOS status bar icon management with NSStatusBar integration
- System menu integration with application controls
- Dynamic show/hide based on user preference
- Bevy Example Reference: [`app/return_after_run.rs`](../../../docs/bevy/examples/app/return_after_run.rs) - System service integration patterns

#### Platform Abstraction Layer
```rust
pub trait SystemIntegration {
    fn register_startup(&self) -> Result<(), SystemError>;
    fn unregister_startup(&self) -> Result<(), SystemError>;
    fn set_menu_bar_visibility(&self, visible: bool) -> Result<(), SystemError>;
    fn get_system_appearance(&self) -> SystemAppearance;
}
```

#### Global Hotkey Registration Platform Layer
- File: `ui/src/platform/hotkey_platform.rs` (new file, line 1-123)
- macOS Carbon/Cocoa hotkey registration integration
- Windows RegisterHotKey API integration with proper cleanup
- Linux X11/Wayland global hotkey support
- Cross-platform abstraction with unified error handling

#### System Appearance Detection
- File: `ui/src/platform/appearance_monitor.rs` (new file, line 1-78)
- macOS NSAppearance change notification handling
- Windows theme change registry monitoring
- Linux dconf/gsettings integration for theme detection
- Real-time appearance change events with debouncing

### Architecture Notes
- Platform-specific implementations with unified trait interface
- Event-driven system change notifications
- Safe system API integration with proper privilege handling
- Graceful degradation when system features unavailable
- Resource cleanup on application termination

### Integration Points
- `core/src/` - System service status integration with core application state
- `app/src/` - Application lifecycle integration with system services
- Event bus integration for system change notifications
- Settings persistence for system integration preferences

### Platform-Specific Implementation Files
- File: `ui/src/platform/macos/mod.rs` (new file, line 1-156)
- File: `ui/src/platform/windows/mod.rs` (new file, line 1-134) 
- File: `ui/src/platform/linux/mod.rs` (new file, line 1-145)

#### Error Handling and Recovery
```rust
#[derive(Debug, thiserror::Error)]
pub enum SystemIntegrationError {
    #[error("Permission denied for system integration")]
    PermissionDenied,
    #[error("System API unavailable: {0}")]
    ApiUnavailable(String),
    #[error("Registration failed: {0}")]
    RegistrationFailed(String),
}
```

#### System Service Management
- File: `ui/src/platform/service_manager.rs` (new file, line 1-67)
- Background service management for system integration
- Health monitoring and automatic recovery
- Resource usage monitoring and optimization
- Clean shutdown and resource cleanup

### Security and Permissions
- Minimal privilege requirements for system integration
- Proper handling of system permission requests
- Secure storage of system integration state
- Audit logging for system-level operations

### Bevy Example References
- **System Integration**: [`app/return_after_run.rs`](../../../docs/bevy/examples/app/return_after_run.rs) - System service patterns
- **Resource Management**: [`asset/asset_loading.rs`](../../../docs/bevy/examples/asset/asset_loading.rs) - Resource lifecycle management
- **Error Handling**: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Async error handling patterns
- **Event System**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - System event integration

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.