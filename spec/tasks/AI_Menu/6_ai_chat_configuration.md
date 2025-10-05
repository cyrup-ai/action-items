# AI Menu - AI Chat Configuration System

## Implementation Task: Dedicated AI Chat Window with Hotkey and Provider Management

### Architecture Overview
Implement the AI Chat configuration system that enables dedicated chat window management, hotkey assignment, provider selection, and conversation timeout controls.

### Core Components

#### AI Chat Configuration Component
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AIChatConfiguration {
    pub chat_hotkey: Option<KeyCombination>,
    pub timeout_minutes: u32,
    pub selected_provider: String,
    pub text_size: TextSizeOption,
    pub auto_clear_enabled: bool,
}

#[derive(Reflect, Default)]
pub struct KeyCombination {
    pub modifiers: ModifierKeys,
    pub key: KeyCode,
    pub display_string: String,
}

#[derive(Reflect, Default)]
pub struct ModifierKeys {
    pub cmd: bool,
    pub ctrl: bool,
    pub opt: bool,
    pub shift: bool,
}

#[derive(Reflect, Default)]
pub enum TextSizeOption {
    Small,
    #[default]
    Medium,
    Large,
}
```

#### Chat Window Management System
- **Window Creation**: Dedicated chat window spawning system
- **Hotkey Registration**: Global hotkey system integration
- **Provider Integration**: Real-time provider selection and authentication
- **Session Management**: Chat timeout and conversation cleanup

### Bevy Implementation References

#### Hotkey Input System
- **Keyboard Input**: `docs/bevy/examples/input/keyboard_input_events.rs`
  - Key combination capture and validation
  - Modifier key detection and state management
  - Global hotkey registration with system

#### Window Management
- **Multiple Windows**: `docs/bevy/examples/window/multiple_windows.rs`
  - Dedicated chat window creation and management
  - Window positioning and sizing coordination
  - Cross-window communication and state synchronization

#### UI Input Fields
- **Text Input**: `docs/bevy/examples/input/text_input.rs`
  - Hotkey display field with keyboard symbols
  - Interactive hotkey recording functionality
  - Clear button for hotkey removal

#### Provider Selection System
- **Button Components**: `docs/bevy/examples/ui/button.rs`
  - Provider dropdown with icon integration
  - Selection state management and visual feedback
  - Dynamic provider list population

### Chat Window Configuration Layout

#### Hotkey Assignment Section
- **Label**: "AI Chat" section header with description
- **Hotkey Field**: Interactive hotkey display and recording
  - **Current Value**: "⌘ ⌃ ⌥ L" (Command + Control + Option + L)
  - **Background**: Dark input field styling (#2a2a2a)
  - **Clear Button**: "X" button for hotkey removal
  - **Recording State**: Visual feedback during hotkey capture

#### Conversation Management
- **Start New Chat Label**: Configuration for chat session lifecycle
- **Timeout Dropdown**: "After 30 minutes" (configurable)
  - **Options**: 5, 15, 30, 60 minutes, Never
  - **Style**: Standard dark dropdown with down arrow
  - **Info Button**: Circular "i" for timeout behavior explanation

#### Provider Configuration Section  
- **Label**: "New Chat Settings" with provider selection
- **Provider Dropdown**: "CYRUP (openai)" with provider icon
  - **Icon**: Circular provider branding (CYRUP logo)
  - **Style**: Dark background with provider icon left of text
  - **Authentication**: Real-time authentication status display

#### Text Preferences Section
- **Label**: "Text Size" for chat interface customization
- **Size Buttons**: Two horizontal buttons for Small ("Aa") and Large ("Aa")
- **Selection State**: Visual indication of currently active size
- **Layout**: Side-by-side button arrangement with toggle behavior

### Data Integration Points

#### Chat Session Management
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct ChatSessionManager {
    pub active_sessions: HashMap<String, ChatSession>,
    pub timeout_config: TimeoutConfiguration,
    pub provider_connections: HashMap<String, ProviderStatus>,
}

#[derive(Reflect)]
pub struct ChatSession {
    pub session_id: String,
    pub provider: String,
    pub last_activity: SystemTime,
    pub conversation_history: Vec<ChatMessage>,
    pub window_handle: Option<Entity>,
}

#[derive(Event)]
pub struct ChatWindowRequested {
    pub hotkey_source: bool,
    pub provider_preference: Option<String>,
}
```

#### Hotkey System Integration
- **Global Registration**: Register chat hotkey with system hotkey manager
- **Conflict Detection**: Validate hotkey doesn't conflict with existing shortcuts
- **Recording Interface**: Interactive hotkey capture with visual feedback
- **Persistence**: Save and restore hotkey assignments across sessions

### Interactive Behavior Implementation

#### Hotkey Recording System
- **Recording Activation**: Click hotkey field to enter recording mode
- **Key Capture**: Detect modifier + key combinations during recording
- **Visual Feedback**: Show "Recording..." state with animated indicator
- **Validation**: Verify hotkey is valid and not conflicting
- **Clear Functionality**: Remove hotkey assignment with X button

#### Provider Selection Interface
- **Dynamic Loading**: Populate provider list from available AI services
- **Authentication Status**: Real-time display of provider connection status
- **Icon Management**: Load and cache provider-specific branding icons
- **Selection Persistence**: Remember provider choice for new chat sessions

#### Timeout Management
- **Dropdown Options**: Predefined timeout intervals with custom option
- **Session Cleanup**: Automatic chat session cleanup based on timeout
- **Warning System**: Optional warnings before session timeout
- **Manual Override**: Allow users to extend active sessions

#### Text Size Controls
- **Toggle Behavior**: Exclusive selection between Small and Large options
- **Preview System**: Real-time preview of text size changes
- **Persistence**: Save text size preference for all chat sessions
- **Accessibility Integration**: Respect system accessibility text settings

### Visual Implementation Details

#### Hotkey Display Field
- **Background**: Dark theme (#2a2a2a) with subtle border
- **Text Style**: Monospace font for keyboard symbols
- **Symbol Rendering**: Proper display of ⌘ ⌃ ⌥ modifier symbols
- **Clear Button**: Small "X" on right side with hover state
- **Recording State**: Animated border or background during capture

#### Provider Dropdown Styling
- **Icon Integration**: 16x16px provider icons with proper alignment
- **Text Layout**: Provider name with service in parentheses
- **Dropdown Arrow**: Down-facing chevron indicating expandable state
- **Selection Highlight**: Visual distinction for selected provider
- **Loading States**: Spinner or placeholder during provider loading

#### Text Size Button Design
- **Button Styling**: Two distinct buttons with toggle selection
- **Typography**: Different font sizes for "Aa" to demonstrate size
- **Selection State**: Active button highlighted with different background
- **Hover Effects**: Subtle hover states for interactive feedback

### Performance Considerations

#### Hotkey System Performance
- **Zero-Allocation Recording**: Efficient hotkey capture without heap allocations
- **Global Hook Efficiency**: Minimal system impact for global hotkey monitoring
- **Conflict Resolution**: Fast validation against existing hotkey assignments
- **Registration Cleanup**: Proper cleanup of system hotkey registrations

#### Chat Window Management
- **Lazy Window Creation**: Create chat windows only when requested
- **Memory Management**: Efficient cleanup of closed chat sessions
- **Provider Connection Pooling**: Reuse provider connections across sessions
- **Background Session Cleanup**: Efficient timeout-based session cleanup

### Integration Points

#### Global Hotkey System Coordination
- Interface with system-level global hotkey management
- Coordinate with application hotkey registry to prevent conflicts
- Handle hotkey registration failures gracefully
- Provide fallback interaction methods when hotkeys unavailable

#### Provider Authentication Integration
- Monitor provider authentication status in real-time
- Handle authentication failures and re-authentication flows
- Sync provider selection with global AI configuration
- Validate provider capabilities for chat functionality

#### Window Management Coordination
- Integrate with application window management system
- Handle multi-monitor setups for chat window positioning
- Coordinate with main application focus and visibility
- Manage chat window lifecycle and cleanup

### Testing Requirements

#### Functional Testing
- Verify hotkey recording captures all modifier combinations correctly
- Test chat window creation via hotkey and manual triggers
- Validate provider selection updates authentication status
- Confirm timeout configuration affects session cleanup

#### Integration Testing  
- Test global hotkey registration and conflict detection
- Verify chat window coordination with main application window
- Validate provider authentication flow integration
- Test text size changes affect active and new chat sessions

#### Performance Testing
- Verify hotkey system has minimal performance impact
- Test chat session cleanup performance with many active sessions
- Validate provider icon loading doesn't block UI thread
- Confirm efficient memory usage during extended chat sessions

### Implementation Files
- `ai_menu/chat_config.rs` - Chat configuration components and data structures
- `ai_menu/hotkey_recording.rs` - Interactive hotkey capture and management
- `ai_menu/chat_sessions.rs` - Chat session lifecycle and timeout management
- `window/chat_window.rs` - Dedicated chat window creation and management
- `providers/chat_integration.rs` - Provider-specific chat functionality

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all hotkey detection loops  
- **Blazing-fast performance** - efficient window and session management
- **Production quality** - complete, robust chat system implementation## Bevy Implementation Details

### AI Chat Component Architecture

```rust
#[derive(Component, Reflect)]
pub struct ChatWindow {
    pub session_id: String,
    pub provider_entity: Entity,
    pub window_handle: Entity,
    pub last_activity: SystemTime,
    pub message_history: VecDeque<ChatMessage>,
    pub text_size: TextSizeOption,
}

#[derive(Component, Reflect)]
pub struct HotkeyRecorder {
    pub recording: bool,
    pub captured_keys: Vec<KeyCode>,
    pub captured_modifiers: ModifierKeys,
    pub display_entity: Entity,
    pub validation_status: HotkeyValidation,
}

#[derive(Component, Reflect)]
pub struct ChatTimeoutManager {
    pub timeout_duration: Duration,
    pub cleanup_timer: Timer,
    pub session_entities: Vec<Entity>,
    pub warning_sent: bool,
}
```

### Multi-Window Management System

```rust
fn create_chat_window(
    mut commands: Commands,
    mut chat_events: EventReader<ChatWindowRequested>,
    chat_config: Res<AIChatConfiguration>,
    provider_registry: Res<AiProviderRegistry>,
) {
    for event in chat_events.read() {
        // Spawn dedicated chat window
        let window_entity = commands.spawn((
            Window {
                title: "AI Chat".to_string(),
                resolution: (600.0, 800.0).into(),
                position: WindowPosition::Automatic,
                mode: WindowMode::Windowed,
                visible: true,
                ..default()
            },
            ChatWindowMarker,
        )).id();
        
        // Create chat session component
        let session_entity = commands.spawn((
            ChatWindow {
                session_id: generate_session_id(),
                provider_entity: get_provider_entity(&chat_config.selected_provider),
                window_handle: window_entity,
                last_activity: SystemTime::now(),
                message_history: VecDeque::new(),
                text_size: chat_config.text_size.clone(),
            },
            ActiveChatSession,
        )).id();
    }
}
```

### Global Hotkey Integration

```rust
#[derive(Component)]
pub struct GlobalHotkeyTask(Task<HotkeyRegistrationResult>);

fn handle_hotkey_registration(
    mut commands: Commands,
    mut hotkey_config: Query<(&mut AIChatConfiguration, &HotkeyRecorder), Changed<HotkeyRecorder>>,
    mut registration_tasks: Query<(Entity, &mut GlobalHotkeyTask)>,
) {
    let thread_pool = AsyncComputeTaskPool::get();
    
    // Process new hotkey registrations
    for (mut config, recorder) in &mut hotkey_config {
        if !recorder.recording && recorder.validation_status.is_valid {
            let key_combo = KeyCombination {
                modifiers: recorder.captured_modifiers.clone(),
                key: recorder.captured_keys.last().copied().unwrap_or(KeyCode::KeyL),
                display_string: format_hotkey_display(&recorder.captured_modifiers, recorder.captured_keys.last()),
            };
            
            let task = thread_pool.spawn(async move {
                register_global_hotkey(key_combo).await
            });
            
            commands.spawn(GlobalHotkeyTask(task));
        }
    }
    
    // Poll completed registrations
    for (entity, mut task) in &mut registration_tasks {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            match result {
                Ok(hotkey_handle) => {
                    info!("Successfully registered AI Chat hotkey");
                    // Store hotkey handle for cleanup
                    commands.entity(entity)
                        .remove::<GlobalHotkeyTask>()
                        .insert(RegisteredHotkey(hotkey_handle));
                },
                Err(e) => {
                    error!("Failed to register AI Chat hotkey: {}", e);
                    commands.entity(entity).remove::<GlobalHotkeyTask>();
                }
            }
        }
    }
}
```

### Chat Session Management

```rust
fn manage_chat_timeouts(
    mut commands: Commands,
    time: Res<Time>,
    mut timeout_manager: Query<&mut ChatTimeoutManager>,
    mut chat_sessions: Query<(Entity, &mut ChatWindow), With<ActiveChatSession>>,
    mut session_events: EventWriter<ChatSessionEvent>,
) {
    for mut manager in &mut timeout_manager {
        manager.cleanup_timer.tick(time.delta());
        
        if manager.cleanup_timer.finished() {
            let current_time = SystemTime::now();
            
            for (entity, mut session) in &mut chat_sessions {
                if let Ok(inactive_duration) = current_time.duration_since(session.last_activity) {
                    if inactive_duration >= manager.timeout_duration {
                        // Send warning before cleanup
                        if !manager.warning_sent && inactive_duration >= manager.timeout_duration * 3/4 {
                            session_events.send(ChatSessionEvent::TimeoutWarning(entity));
                            manager.warning_sent = true;
                        } else {
                            // Clean up expired session
                            session_events.send(ChatSessionEvent::SessionExpired(entity));
                            commands.entity(entity).despawn_recursive();
                        }
                    }
                }
            }
        }
    }
}
```

### Flex-Based Chat UI Layout

```rust
fn setup_chat_configuration_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(16.0),
            flex_grow: 0.0,
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
        ChatConfigurationUI,
    )).with_children(|parent| {
        // Hotkey configuration section
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
        )).with_children(|row| {
            // Hotkey input field with recording capability
            row.spawn((
                Node {
                    width: Val::Percent(60.0),
                    height: Val::Px(36.0),
                    padding: UiRect::horizontal(Val::Px(12.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 1.0)),
                BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0)),
                HotkeyInputField,
                Interaction::default(),
            ));
        });
    });
}
```