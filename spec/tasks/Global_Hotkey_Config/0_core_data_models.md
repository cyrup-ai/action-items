# Global Hotkey Config - Core Data Models and State Management

## Task: Implement Core Data Structures for Global Settings Configuration

### File: `ui/src/settings/global_config/mod.rs` (new file)

Create comprehensive data models for global application configuration with zero-allocation patterns and blazing-fast state management.

### Implementation Requirements

#### Global Hotkey Configuration Resource
- File: `ui/src/settings/global_config/hotkey.rs` (new file, line 1-89)
- Implement `GlobalHotkeyConfig` resource with current hotkey, recording state, conflicts
- Bevy Example Reference: [`input/keyboard_modifiers.rs`](../../../docs/bevy/examples/input/keyboard_modifiers.rs) - Lines 12-17 show modifier key detection patterns
- Integration with macOS/Windows/Linux global hotkey APIs
- Conflict detection data structures for system shortcut validation

#### Key Combination Data Structure
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct KeyCombination {
    pub modifiers: ModifierKeys,
    pub key: KeyCode,
    pub display_string: String,
    pub system_representation: String,
}
```

#### Theme Management Resource
- File: `ui/src/settings/global_config/theme.rs` (new file, line 1-67)
- Implement `ThemeConfig` resource for Dark/Light/System theme management
- Bevy Example Reference: [`asset/asset_loading.rs`](../../../docs/bevy/examples/asset/asset_loading.rs) - Asset management patterns for theme resources
- System appearance change monitoring integration
- Custom theme support data structures

#### Window Mode Configuration
- File: `ui/src/settings/global_config/window_mode.rs` (new file, line 1-45)
- Implement `WindowModeConfig` resource for Default/Compact mode management
- Preview generation system for visual mode selection
- Mode-specific UI adaptation data structures

#### Hotkey Recording Modal Component
- File: `ui/src/settings/global_config/recording_modal.rs` (new file, line 1-56)
- Implement `HotkeyRecordingModal` component for modal recording interface
- Bevy Example Reference: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Lines 24-45 show interaction state management
- Real-time key capture display and visual feedback system

### Architecture Notes
- Use Bevy's `Reflect` trait for all configuration structures
- Implement `Resource` trait for global settings access across systems
- Zero-allocation serialization with `serde` for settings persistence
- Atomic state updates with Bevy's change detection system
- Integration with existing `app/src/preferences/` module for settings persistence

### Integration Points
- `core/src/` - Core settings persistence system (integrate with existing settings at core/src/lib.rs lines 156-189)
- `app/src/preferences/` - Existing preference management integration
- System APIs for global hotkey registration and theme detection
- Event system integration for real-time configuration updates

### Bevy Example References
- **Primary**: [`input/keyboard_modifiers.rs`](../../../docs/bevy/examples/input/keyboard_modifiers.rs) - Modifier key combination patterns
- **Supporting**: [`reflection/reflection.rs`](../../../docs/bevy/examples/reflection/reflection.rs) - Settings serialization patterns
- **UI**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Interactive component state management

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.