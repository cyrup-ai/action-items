# Global Hotkey Config - UI Components Architecture

## Task: Implement Settings Interface Components and Layout

### File: `ui/src/components/settings_interface.rs` (new file)

Create comprehensive settings interface components with consistent design patterns and interactive controls.

### Implementation Requirements

#### Settings Page Layout Component
- File: `ui/src/components/settings_interface.rs` (new file, line 1-145)
- Vertical configuration sections with consistent spacing and grouping
- Bevy Example Reference: [`ui/flex_layout.rs`](../../../docs/bevy/examples/ui/flex_layout.rs) - Flexible layout systems
- Two-column layout pattern (labels left, controls right) for all settings sections
- Progressive disclosure for advanced options

#### Interactive Control Components
- File: `ui/src/components/settings_controls.rs` (new file, line 1-123)
- Checkbox toggles for boolean settings (Launch at login, Menu bar visibility)
- Dropdown components for theme selection and system preferences  
- Bevy Example Reference: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Interactive control patterns
- Toggle button groups for text size selection
- Modal triggers for hotkey recording interface

#### Hotkey Display Component
```rust
#[derive(Component)]
pub struct HotkeyDisplayButton {
    pub current_hotkey: Option<KeyCombination>,
    pub recording_state: bool,
    pub display_text: String,
}
```
- File: `ui/src/components/hotkey_display.rs` (new file, line 1-67)
- Interactive hotkey display showing "⌘ Space" format
- Click-to-record functionality with visual feedback
- Real-time update display during recording sessions

#### Theme Selection Dropdown
- File: `ui/src/components/theme_dropdown.rs` (new file, line 1-89)
- Theme dropdown with icon integration (moon/sun indicators)
- "Follow system appearance" checkbox integration
- "Open Theme Studio" button with external integration
- Bevy Example Reference: [`ui/ui.rs`](../../../docs/bevy/examples/ui/ui.rs) - Dropdown menu patterns

#### Settings Section Grouping
- File: `ui/src/components/settings_sections.rs` (new file, line 1-78)
- Startup Settings, Global Hotkey, System Integration, Display sections
- Consistent section headers and visual separation
- Info icon integration for contextual help

### Architecture Notes
- Component-based architecture with reusable settings controls
- Event-driven interaction with immediate visual feedback
- Consistent design system following established UI patterns
- Integration with existing UI theme and component systems
- Accessible keyboard navigation and screen reader support

### Integration Points
- `ui/src/ui/components.rs` - Integration with existing UI component system (lines 89-167)
- `ui/src/ui/theme.rs` - Theme system integration for consistent styling (lines 45-123)
- `ui/src/ui/accessibility.rs` - Accessibility feature integration (lines 23-78)
- Settings state management with real-time persistence

### Event System Integration
```rust
#[derive(Event)]
pub enum SettingsUIEvent {
    CheckboxToggled(String, bool),
    DropdownChanged(String, String),
    ButtonClicked(String),
    HotkeyRecordingRequested,
    ThemeStudioRequested,
    SettingChanged(String, SettingValue),
}
```

#### Accessibility Components
- File: `ui/src/components/accessible_controls.rs` (new file, line 1-56)
- Proper ARIA labels and semantic markup for all controls
- Keyboard navigation support with logical tab order
- Screen reader announcements for setting changes
- Focus indicators and high contrast mode support

#### Validation and Feedback Components  
- File: `ui/src/components/settings_feedback.rs` (new file, line 1-45)
- Real-time validation feedback for setting changes
- Error message display for invalid configurations
- Success confirmations for applied settings
- Loading states during system integration operations

### Visual Design Implementation
- Consistent dark theme styling matching application design
- Proper spacing and alignment following design specifications
- Interactive states (hover, pressed, focused) for all controls
- Visual hierarchy with typography and color distinctions

### Bevy Example References
- **Layout System**: [`ui/flex_layout.rs`](../../../docs/bevy/examples/ui/flex_layout.rs) - Flexible settings layout
- **Interactive Controls**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Button and control interactions
- **Dropdown Patterns**: [`ui/ui.rs`](../../../docs/bevy/examples/ui/ui.rs) - Dropdown and selection components
- **Text Rendering**: [`ui/text.rs`](../../../docs/bevy/examples/ui/text.rs) - Text styling and typography
- **Accessibility**: [`ui/ui_texture_atlas.rs`](../../../docs/bevy/examples/ui/ui_texture_atlas.rs) - Icon management for controls

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.