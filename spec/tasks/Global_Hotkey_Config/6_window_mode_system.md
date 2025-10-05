# Global Hotkey Config - Window Mode Configuration System

## Task: Implement Window Mode Selection and UI Adaptation

### File: `ui/src/systems/window_mode.rs` (new file)

Create comprehensive window mode system with visual selection cards, real-time UI adaptation, and preview generation.

### Implementation Requirements

#### Window Mode Selection System
- File: `ui/src/systems/window_mode.rs` (new file, line 1-123)
- Implement Default vs Compact mode switching with instant UI adaptation
- Bevy Example Reference: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Lines 24-45 show interaction state management for mode selection
- Real-time UI layout adaptation based on selected window mode
- Integration with existing UI systems for responsive design changes

#### Visual Mode Selection Cards
- File: `ui/src/components/mode_selector_cards.rs` (new file, line 1-89)
- Interactive selection cards with rounded corner wireframes
- Purple gradient card for Default mode, gray minimalist card for Compact mode
- Bevy Example Reference: [`ui/ui_texture_atlas.rs`](../../../docs/bevy/examples/ui/ui_texture_atlas.rs) - Texture management for card backgrounds and previews
- Click interaction with visual selection highlighting

#### UI Layout Adaptation System
```rust
pub fn adapt_ui_layout(
    window_mode: Res<WindowModeConfig>,
    mut ui_query: Query<&mut Style, With<ResponsiveUI>>,
    mut visibility_query: Query<&mut Visibility, With<ConditionalVisibility>>,
) {
    // Implementation for dynamic UI adaptation
}
```

#### Compact Mode Favorites Control
- File: `ui/src/systems/compact_mode_features.rs` (new file, line 1-56)  
- "Show favorites in compact mode" checkbox functionality
- Dynamic favorites list visibility based on mode and preference
- Balances minimalism with quick access functionality

#### Preview Generation System
- File: `ui/src/systems/mode_previews.rs` (new file, line 1-78)
- Generate accurate interface previews for mode selection cards
- Bevy Example Reference: [`ui/ui.rs`](../../../docs/bevy/examples/ui/ui.rs) - UI hierarchy management for preview rendering
- Real-time preview updates reflecting current interface state
- Efficient preview rendering without impacting main UI performance

### Architecture Notes
- Component-based responsive design with WindowMode markers
- Event-driven layout adaptation with instant visual feedback
- Zero-allocation mode switching using Bevy's change detection
- Integration with existing UI component systems
- Persistent mode preference storage and restoration

### Integration Points
- `ui/src/ui/components.rs` - UI component responsive behavior (lines 234-312)
- `ui/src/ui/systems.rs` - Layout system integration (lines 67-145)
- `app/src/window/` - Window size and behavior management
- Core settings persistence for mode preference storage

### Event System Integration
```rust
#[derive(Event)]
pub enum WindowModeEvent {
    ModeSelected(WindowMode),
    PreviewRequested(WindowMode),
    FavoritesVisibilityToggled(bool),
    LayoutAdaptationComplete(WindowMode),
}
```

#### Responsive UI Components
- File: `ui/src/components/responsive_ui.rs` (new file, line 1-67)
- ResponsiveUI component marker for mode-adaptive elements
- ConditionalVisibility component for compact mode element hiding
- Smooth transitions between mode layouts

### Text Size Integration
- File: `ui/src/systems/text_size_control.rs` (new file, line 1-45)
- Text size toggle button group ("Aa" buttons for small/large)
- Global text scaling across entire interface
- Integration with accessibility features and system text scaling

### Bevy Example References
- **UI Interaction**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Card selection patterns (lines 24-45)
- **Texture Management**: [`ui/ui_texture_atlas.rs`](../../../docs/bevy/examples/ui/ui_texture_atlas.rs) - Preview card rendering
- **Layout Management**: [`ui/ui.rs`](../../../docs/bevy/examples/ui/ui.rs) - Responsive UI hierarchy
- **Component Queries**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Query patterns for UI adaptation

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.