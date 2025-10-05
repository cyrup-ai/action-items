# Global Hotkey Config - Hotkey Capture and Recording System

## Task: Implement Global Hotkey Recording and System Integration

### File: `ui/src/systems/hotkey_recording.rs` (new file)

Create comprehensive hotkey capture system with real-time recording, conflict detection, and OS integration.

### Implementation Requirements

#### Hotkey Recording System
- File: `ui/src/systems/hotkey_recording.rs` (new file, line 1-156)
- Implement `hotkey_recording_system` using Bevy input resources
- Bevy Example Reference: [`input/keyboard_input.rs`](../../../docs/bevy/examples/input/keyboard_input.rs) - Lines 12-23 show `just_pressed()` pattern for key detection
- Real-time key combination capture with `Res<ButtonInput<KeyCode>>`
- Integration with HotkeyRecordingModal component for visual feedback

#### Modifier Key Detection System
```rust
pub fn capture_key_combination(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut recording_state: ResMut<GlobalHotkeyConfig>,
    mut modal_query: Query<&mut HotkeyRecordingModal>,
) {
    // Implementation following keyboard_modifiers.rs pattern
}
```
- Bevy Example Reference: [`input/keyboard_modifiers.rs`](../../../docs/bevy/examples/input/keyboard_modifiers.rs) - Lines 12-17 demonstrate modifier detection with `any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])`

#### Recording Modal Interface System
- File: `ui/src/systems/recording_modal.rs` (new file, line 1-89)
- Implement modal overlay with "Recording..." indicator and real-time key display
- Bevy Example Reference: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Lines 24-45 show interaction state management patterns
- Visual feedback system showing captured keys as "⌘ Space", "⌃ ⌥ L", etc.
- Automatic modal dismissal on successful capture or Escape key

#### Conflict Detection System
- File: `ui/src/systems/hotkey_conflicts.rs` (new file, line 1-67)
- System-level conflict detection with macOS/Windows system shortcuts
- Integration with existing application hotkeys database
- Real-time validation and user feedback for conflicting combinations
- Fallback suggestions for conflicted hotkeys

#### OS Integration Layer
- File: `ui/src/platform/hotkey_registration.rs` (new file, line 1-134)
- Platform-specific global hotkey registration (macOS Carbon/Cocoa, Windows, Linux)
- Safe registration/unregistration with proper cleanup
- Error handling for registration failures and permission issues

### Architecture Notes
- Use Bevy's input systems for cross-platform key capture
- Event-driven architecture for recording state changes
- Zero-allocation key combination processing
- Integration with existing window focus and modal systems
- Atomic hotkey registration with rollback on conflicts

### Integration Points
- `app/src/window/` - Window focus management integration
- `core/src/events.rs` - Global hotkey trigger events (integrate at lines 45-67)
- `ui/src/ui/systems.rs` - Modal rendering integration
- System APIs: Carbon (macOS), RegisterHotKey (Windows), X11 (Linux)

### Event System Integration
```rust
#[derive(Event)]
pub enum HotkeyRecordingEvent {
    StartRecording,
    KeyCombinationCaptured(KeyCombination),
    RecordingCancelled,
    ConflictDetected(ConflictInfo),
    RegistrationComplete(KeyCombination),
}
```

### Bevy Example References
- **Primary**: [`input/keyboard_input.rs`](../../../docs/bevy/examples/input/keyboard_input.rs) - Key detection patterns (lines 12-23)
- **Modifier Detection**: [`input/keyboard_modifiers.rs`](../../../docs/bevy/examples/input/keyboard_modifiers.rs) - Modifier combinations (lines 12-17)
- **UI Interaction**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Modal state management (lines 24-45)
- **Event Handling**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Event system patterns

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.