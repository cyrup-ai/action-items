# Main Menu - Keyboard Navigation and Action Execution System

## Task: Implement Keyboard Navigation and Command Execution Framework

### File: `ui/src/systems/keyboard_navigation.rs` (new file)

Create comprehensive keyboard navigation system with action execution, contextual menu integration, and AI activation.

### Implementation Requirements

#### Keyboard Navigation System
- File: `ui/src/systems/keyboard_navigation.rs` (new file, line 1-134)
- Implement arrow key navigation through action list with visual selection
- Bevy Example Reference: [`input/keyboard_input.rs`](../../../docs/bevy/examples/input/keyboard_input.rs) - Lines 12-23 show keyboard input detection patterns
- Real-time selection highlighting with smooth transitions between items
- Integration with search results and favorites list navigation

#### Action Execution Framework
- File: `ui/src/systems/action_execution.rs` (new file, line 1-156)
- Implement `execute_action_system` for Enter key command execution
- Safe command execution with sandboxed environment integration
- Integration with existing Deno runtime for extensible action processing
- Bevy Example Reference: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Asynchronous action execution patterns

#### Contextual Menu Integration
```rust
pub fn contextual_menu_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut menu_events: EventWriter<ContextualMenuEvent>,
    search_state: Res<SearchState>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyK) && 
       keyboard_input.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]) {
        // Open contextual actions menu (⌘K)
    }
}
```

#### AI Assistant Activation System
- File: `ui/src/systems/ai_activation.rs` (new file, line 1-78)
- Tab key activation for "Ask AI" functionality with context preservation
- Natural language query processing and intent recognition  
- Integration with existing AI system from AI_Menu specifications
- Context-aware suggestions based on current search and selected items

#### Quick Task Creation System
- File: `ui/src/systems/quick_task_creation.rs` (new file, line 1-89)
- Create tasks directly from search text when no results found
- Integration with existing ActionItems database through Deno runtime
- Smart task categorization and priority assignment based on search context

### Architecture Notes
- Event-driven architecture with LauncherEvent for all interactions
- Zero-allocation navigation state updates using Bevy change detection
- Integration with existing window focus and modal management
- Cross-platform keyboard handling with macOS/Windows/Linux support
- Atomic action execution with proper error handling and rollback

### Integration Points
- `core/src/runtime/` - Deno runtime integration for action execution (lines 67-134)
- `app/src/events/handlers.rs` - Event handling integration (lines 45-123)
- `ui/src/ui/systems.rs` - UI state coordination and visual updates (lines 89-167)
- `core/src/plugins/` - Plugin action execution coordination

### Event System Integration
```rust
#[derive(Event)]
pub enum LauncherEvent {
    NavigateUp,
    NavigateDown,
    ExecuteAction(String),
    ActivateAI,
    OpenActionsMenu,
    CreateQuickTask { text: String },
    EscapePressed,
    FocusSearch,
}
```

#### Navigation State Management
- File: `ui/src/systems/navigation_state.rs` (new file, line 1-67)
- Selected index tracking with bounds checking
- Visual feedback for currently selected item
- Smooth scrolling for large result lists
- Integration with search result updates

#### Escape Key Behavior
- File: `ui/src/systems/escape_handling.rs` (new file, line 1-45)
- Configurable Escape key behavior from Advanced Menu settings
- Clear search vs close launcher functionality
- Navigation hierarchy handling for nested interfaces

### Bevy Example References
- **Keyboard Input**: [`input/keyboard_input.rs`](../../../docs/bevy/examples/input/keyboard_input.rs) - Key detection (lines 12-23)
- **Async Actions**: [`async_tasks/async_compute.rs`](../../../docs/bevy/examples/async_tasks/async_compute.rs) - Action execution
- **Event System**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Navigation event patterns
- **System Coordination**: [`ecs/system_param.rs`](../../../docs/bevy/examples/ecs/system_param.rs) - Multi-system coordination

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.