# Main Menu - Visual Interface and UI Rendering System

## Task: Implement Main Launcher Visual Components and Rendering Architecture

### File: `ui/src/components/main_launcher.rs` (new file)

Create comprehensive visual interface system with scrollable lists, icon management, and responsive design.

### Implementation Requirements

#### Main Launcher Layout Component
- File: `ui/src/components/main_launcher.rs` (new file, line 1-134)
- Implement complete launcher layout with search bar, action list, bottom controls
- Bevy Example Reference: [`ui/flex_layout.rs`](../../../docs/bevy/examples/ui/flex_layout.rs) - Flexible layout systems for launcher structure
- Dark theme integration with consistent color scheme and typography
- Responsive design adapting to different window sizes and modes

#### Scrollable Action List System
- File: `ui/src/components/scrollable_list.rs` (new file, line 1-156)
- Implement efficient scrolling for large action lists with virtual rendering
- Smooth 60fps scrolling performance without layout thrashing
- Bevy Example Reference: [`ui/overflow.rs`](../../../docs/bevy/examples/ui/overflow.rs) - Scrolling behavior patterns
- Integration with keyboard navigation for selected item visibility

#### Icon Management and Rendering
- File: `ui/src/components/icon_system.rs` (new file, line 1-89)
- Dynamic icon loading and caching for action sources (red, yellow, teal, blue icons)
- High-resolution icon rendering with consistent sizing (16x16 or 20x20px)
- Bevy Example Reference: [`ui/ui_texture_atlas.rs`](../../../docs/bevy/examples/ui/ui_texture_atlas.rs) - Texture atlas management
- Icon atlas optimization for memory efficiency and fast rendering

#### Action Item Visual Component
```rust
#[derive(Component)]
pub struct ActionItemVisual {
    pub title_text: Entity,
    pub description_text: Entity,
    pub icon_entity: Entity,
    pub command_tag: Entity,
    pub hover_background: Entity,
    pub selection_state: SelectionState,
}
```

#### Search Bar Interface Component
- File: `ui/src/components/search_interface.rs` (new file, line 1-78)
- Search input field with placeholder text and real-time filtering
- "Ask AI | Tab" button integration with proper styling
- Visual focus states and accessibility support
- Integration with text input system and keyboard handling

#### Bottom Action Bar Component
- File: `ui/src/components/action_bar.rs` (new file, line 1-67)
- "Open Command" and "Actions ⌘K" buttons with keyboard shortcut indicators
- Navigation controls with visual feedback
- Responsive button layout adapting to content and window size

### Architecture Notes
- Component-based UI architecture with reusable visual elements
- Efficient rendering with change detection for minimal redraws
- Integration with existing theme system for consistent styling
- Accessibility support with proper focus management and ARIA labels
- Performance optimization with virtual rendering for large lists

### Integration Points
- `ui/src/ui/theme.rs` - Theme system integration for color and styling (lines 45-123)
- `ui/src/ui/components.rs` - Integration with existing UI component system (lines 89-167)
- `ui/src/ui/accessibility.rs` - Accessibility feature integration (lines 23-78)
- `ui/src/launcher/` - Data model integration with visual representation

### Visual State Management
- File: `ui/src/systems/visual_state.rs` (new file, line 1-95)
- Selection highlighting with smooth visual transitions
- Hover effects with subtle background changes and scale animations
- Loading states during search operations and action execution
- Error states with appropriate visual feedback

#### Interactive Visual Effects
- File: `ui/src/systems/visual_effects.rs` (new file, line 1-56)
- Hover state animations with background highlights
- Selection transitions with smooth color changes
- Micro-animations for button interactions and state changes
- Performance-optimized animations that don't impact responsiveness

### Typography and Text Rendering
- File: `ui/src/components/text_system.rs` (new file, line 1-67)
- Consistent typography hierarchy for titles, descriptions, tags
- Text size integration with global text scaling from General Menu
- Font loading and caching for optimal rendering performance
- Text color management for theme consistency

### Bevy Example References
- **Layout System**: [`ui/flex_layout.rs`](../../../docs/bevy/examples/ui/flex_layout.rs) - Main launcher layout
- **Scrolling**: [`ui/overflow.rs`](../../../docs/bevy/examples/ui/overflow.rs) - Action list scrolling
- **Icons**: [`ui/ui_texture_atlas.rs`](../../../docs/bevy/examples/ui/ui_texture_atlas.rs) - Icon management
- **Text Rendering**: [`ui/text.rs`](../../../docs/bevy/examples/ui/text.rs) - Typography and text styling
- **Interactive Elements**: [`ui/button.rs`](../../../docs/bevy/examples/ui/button.rs) - Button and interaction states

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.