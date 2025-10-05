# AI Menu 2 Specification

## Overview
AI Menu 2 represents an advanced configuration view of the AI settings interface, focusing on developer-oriented features like AI Commands, Tool integrations, Model Context Protocol (MCP), and API key management.

## Layout Architecture
- **Base Layout**: Identical to AI_Menu.md with tabbed navigation and split-pane design
- **Left Pane**: Unchanged branding and primary toggle section
- **Right Pane**: Advanced AI configuration sections with developer-focused controls

## Right Pane Configuration Sections

### AI Commands Section
- **Title**: "AI Commands"
- **Description**: "Custom instructions for common tasks such as improving your writing or code"
- **Layout**: Two-column layout with label left, controls right
- **Components**:
  - **Model Label**: "AI Commands Model"
  - **Model Dropdown**: "Gemini 2.5 Pro" (current selection)
    - **Provider Icon**: Google "G" icon displayed left of text
    - **Style**: Dark background dropdown with provider branding
  - **Model Description**: "Default model for custom AI Commands. You can still select a different model for individual AI Commands."
    - **Text Color**: Medium gray
    - **Position**: Below dropdown, spans full width
- **Functionality**: Allows selection of default AI model for custom command execution

### Tools Configuration
- **Title**: "Tools"
- **Layout**: Two-column layout with controls on right side
- **Components**:
  - **Show Tool Call Info Checkbox**: "Show Tool Call Info"
    - **Current State**: UNCHECKED (empty checkbox)
    - **Info Icon**: Circular "i" button for contextual help
  - **Reset Tool Confirmations Button**: "Reset Tool Confirmations"
    - **Style**: Standard button with darker background
    - **Info Icon**: Circular "i" button for details about reset functionality
- **Purpose**: Manages AI tool interaction behavior and user permission preferences

### Model Context Protocol (MCP) Section
- **Title**: "Model Context Protocol"
- **Description**: "Connect external tools and data sources using the Model Context Protocol (MCP)"
- **Layout**: Two-column layout with label left, controls right
- **Components**:
  - **Manage Servers Button**: "Manage Servers"
    - **Style**: Standard button with darker background
    - **Function**: Opens MCP server configuration interface
  - **Server Idle Time Setting**:
    - **Label**: "Server Idle Time"
    - **Dropdown**: "5 Minutes" (current selection)
    - **Style**: Dark background dropdown
    - **Info Icon**: Circular "i" button for idle timeout details
  - **Tool Call Automation**:
    - **Checkbox**: "Automatically confirm all tool calls"
    - **Current State**: CHECKED (visible checkmark)
    - **Warning Icon**: Yellow/orange triangle warning indicator
      - **Position**: Right side of checkbox line
      - **Purpose**: Indicates potential security implications
- **Functionality**: Enables integration with external data sources and tools through standardized protocol

### Custom API Keys Section
- **Title**: "Custom API Keys"
- **Description**: "Add API keys to use AI at your own cost."
- **Info Icon**: Circular "i" button for API key configuration details
- **Layout**: Two-column layout with label left, content right
- **Components**:
  - **Provider Information List**:
    - **Bullet Point 1**: "• Anthropic, Google, OpenAI: Requests are routed via Raycast servers"
    - **Bullet Point 2**: "• OpenRouter: Requests are routed directly to the provider's servers"
  - **Text Styling**: 
    - **Bullet Points**: White/light gray text with standard bullet formatting
    - **Provider Names**: Bold or emphasized styling within bullet points
- **Functionality**: Explains routing behavior for different AI provider API keys

## Visual Design Specifications

### Layout Consistency
- **Left Pane**: Identical to AI_Menu.md (branding, toggle, footer actions)
- **Right Pane**: Advanced configuration sections with consistent spacing
- **Section Separation**: Clear visual separation between configuration groups
- **Two-Column Layout**: Consistent label-left, control-right pattern throughout

### Control Specifications
- **Dropdowns**: 
  - Dark background (#2a2a2a or similar)
  - Provider icons positioned left of text (Google "G", etc.)
  - Down arrow indicator on right
  - Full width spanning right column
- **Checkboxes**:
  - Standard square checkboxes with checkmarks when selected
  - "Show Tool Call Info": Currently unchecked (empty box)
  - "Automatically confirm all tool calls": Currently checked (visible checkmark)
- **Buttons**:
  - "Reset Tool Confirmations": Standard button with darker background
  - "Manage Servers": Standard button styling consistent with other buttons

### Icon System
- **Info Icons**: Circular "i" buttons for contextual help throughout interface
- **Warning Icons**: Yellow/orange triangle warning indicators for security concerns
- **Provider Icons**: Brand-specific icons (Google "G") within dropdowns
- **Consistent Sizing**: All icons maintain consistent dimensions and positioning

### Typography & Text Hierarchy
- **Section Titles**: Bold, white/light gray text for main section headers
- **Field Labels**: Regular weight, white/light gray for individual setting labels
- **Descriptions**: Medium gray text for explanatory content below main descriptions
- **Bullet Points**: Standard bullet formatting with emphasized provider names
- **Control Text**: White text within dropdowns and buttons

### State Indicators
- **Checked Checkboxes**: Visible checkmarks in selected state
- **Unchecked Checkboxes**: Empty square boxes in unselected state
- **Warning States**: Triangle warning icons for potentially risky settings
- **Info States**: Circular info icons for help and additional context

### Color & Contrast
- **Consistent Theme**: Matches AI_Menu.md dark theme throughout
- **Warning Colors**: Yellow/orange for warning triangle icons
- **Text Contrast**: High contrast white/light gray text on dark backgrounds
- **Control Backgrounds**: Darker grays for interactive elements

## Functional Requirements

### AI Commands Management
- **Model Selection**: Per-command model override capability
- **Custom Instructions**: Ability to define task-specific AI behaviors
- **Template System**: Predefined templates for common development tasks
- **Performance Optimization**: Model selection based on task complexity

### Tool Integration System
- **Permission Management**: Fine-grained control over tool access and execution
- **Confirmation System**: User control over automatic vs manual tool approval
- **Tool Discovery**: Dynamic loading and registration of available tools
- **Security Framework**: Safe execution environment for AI tool operations

### MCP Server Management
- **Server Discovery**: Automatic detection of available MCP servers
- **Connection Management**: Reliable connection handling with retry logic
- **Protocol Validation**: Compliance checking for MCP specification
- **Resource Monitoring**: Real-time server status and performance metrics

### API Key Management
- **Secure Storage**: Encrypted storage of user-provided API keys
- **Provider Validation**: Automatic verification of API key authenticity
- **Usage Tracking**: Monitoring of API usage and cost implications
- **Routing Logic**: Intelligent request routing based on provider capabilities

## Security Requirements

### API Key Security
- **Encryption**: All API keys encrypted at rest using system keychain
- **Access Control**: API keys only accessible to authenticated user sessions
- **Audit Logging**: Comprehensive logging of API key usage and modifications
- **Secure Transmission**: TLS encryption for all API communications

### Tool Execution Safety
- **Sandboxing**: Tool execution in isolated environment
- **Permission System**: Explicit user consent for tool access to system resources
- **Validation**: Input/output validation for all tool interactions
- **Rollback**: Ability to undo tool actions where applicable

### MCP Security
- **Protocol Validation**: Strict adherence to MCP security specifications
- **Server Authentication**: Mutual authentication between client and MCP servers
- **Data Isolation**: Separation of data between different MCP connections
- **Access Logging**: Detailed logs of all MCP server interactions

## Bevy Implementation Examples

### Advanced Dropdowns
- Reference: `./docs/bevy/examples/ui/ui.rs` - Complex dropdown with provider icons
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Dynamic icon loading for providers

### Button Collections
- Reference: `./docs/bevy/examples/ui/button.rs` - Multiple button states and interactions
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Button group layouts

### Checkbox Groups
- Reference: `./docs/bevy/examples/ui/ui.rs` - Checkbox styling and state management
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Checkbox interaction handling

### Warning Icons
- Reference: `./docs/bevy/examples/ui/text_debug.rs` - Icon positioning and styling
- Reference: `./docs/bevy/examples/animation/animated_fox.rs` - Warning icon animations

### Server Management Interface
- Reference: `./docs/bevy/examples/games/loading_screen.rs` - Server connection status displays
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Asynchronous server operations

### API Key Input Fields
- Reference: `./docs/bevy/examples/ui/text_input.rs` - Secure text input handling
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Input validation and masking

### Provider Icon System
- Reference: `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs` - Dynamic provider icon loading
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Icon atlas management

## State Management Requirements

### Configuration Persistence
- **Settings Storage**: All configuration persisted to application preferences
- **Migration Support**: Automatic migration between configuration versions
- **Backup/Restore**: User ability to export/import configuration settings
- **Validation**: Real-time validation of configuration changes

### Real-time Updates
- **Server Status**: Live updates of MCP server connectivity status
- **API Validation**: Real-time validation of API key functionality
- **Tool Availability**: Dynamic updates of available tools and capabilities
- **Provider Status**: Real-time status of AI provider availability

### Error Handling
- **Connection Failures**: Graceful handling of server connectivity issues
- **API Errors**: User-friendly error messages for API key problems
- **Tool Failures**: Safe error handling for tool execution failures
- **Recovery Mechanisms**: Automatic retry logic with exponential backoff

## Performance Considerations

### Resource Management
- **Lazy Loading**: On-demand loading of provider configurations
- **Connection Pooling**: Efficient reuse of API connections
- **Caching**: Intelligent caching of provider capabilities and responses
- **Memory Management**: Automatic cleanup of unused resources

### Scalability
- **Concurrent Connections**: Support for multiple simultaneous API calls
- **Rate Limiting**: Built-in rate limiting to prevent provider throttling
- **Load Balancing**: Intelligent distribution of requests across providers
- **Failover**: Automatic failover to alternative providers when available

## Bevy Implementation Details

### Advanced Component Architecture

```rust
use bevy::{prelude::*, utils::HashMap};

// AI Commands section components
#[derive(Component, Reflect)]
pub struct AiCommandsSection {
    pub selected_model: String,
    pub default_model: String,
}

#[derive(Component, Reflect)]
pub struct ModelDropdownWithProvider {
    pub current_model: String,
    pub provider_name: String,
    pub provider_icon: Handle<Image>,
    pub is_expanded: bool,
    pub available_models: Vec<ModelWithProvider>,
}

#[derive(Clone, Reflect)]
pub struct ModelWithProvider {
    pub model_id: String,
    pub display_name: String,
    pub provider: String,
    pub icon_path: String,
    pub capabilities: ModelCapabilities,
}

// Tools section components
#[derive(Component, Reflect)]
pub struct ToolsSection {
    pub show_tool_call_info: bool,
    pub tool_confirmations_enabled: bool,
}

#[derive(Component, Reflect)]
pub struct ToolCallInfoCheckbox {
    pub is_checked: bool,
}

#[derive(Component, Reflect)]
pub struct ResetToolConfirmationsButton;

// MCP (Model Context Protocol) components
#[derive(Component, Reflect)]
pub struct McpSection {
    pub server_idle_timeout: u32,
    pub auto_confirm_tools: bool,
    pub connected_servers: Vec<McpServer>,
}

#[derive(Component, Reflect)]
pub struct McpServerButton;

#[derive(Component, Reflect)]
pub struct McpIdleTimeDropdown {
    pub selected_minutes: u32,
    pub is_expanded: bool,
}

#[derive(Component, Reflect)]
pub struct AutoConfirmToolsCheckbox {
    pub is_checked: bool,
    pub show_warning: bool,
}

#[derive(Component, Reflect)]
pub struct SecurityWarningIcon;

// API Keys section components
#[derive(Component, Reflect)]
pub struct ApiKeysSection {
    pub configured_providers: Vec<String>,
    pub routing_info: ApiKeyRoutingInfo,
}

#[derive(Clone, Reflect)]
pub struct ApiKeyRoutingInfo {
    pub raycast_routed: Vec<String>, // Anthropic, Google, OpenAI
    pub direct_routed: Vec<String>,  // OpenRouter
}

#[derive(Clone, Reflect)]
pub struct McpServer {
    pub name: String,
    pub url: String,
    pub status: McpServerStatus,
    pub capabilities: Vec<String>,
    pub last_ping: Option<f64>,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum McpServerStatus {
    Connected,
    Disconnected,
    Error(String),
    Connecting,
}
```

### Advanced Resource Management

```rust
// Extended AI menu state for advanced features
#[derive(Resource, Reflect)]
pub struct AdvancedAiMenuState {
    pub ai_commands_model: String,
    pub tools_config: ToolsConfiguration,
    pub mcp_config: McpConfiguration,
    pub api_keys: ApiKeyConfiguration,
}

#[derive(Clone, Reflect)]
pub struct ToolsConfiguration {
    pub show_tool_call_info: bool,
    pub tool_confirmations: HashMap<String, bool>,
    pub trusted_tools: Vec<String>,
}

#[derive(Clone, Reflect)]
pub struct McpConfiguration {
    pub server_idle_timeout_minutes: u32,
    pub auto_confirm_all_tools: bool,
    pub managed_servers: Vec<McpServerConfig>,
    pub protocol_version: String,
}

#[derive(Clone, Reflect)]
pub struct McpServerConfig {
    pub server_id: String,
    pub name: String,
    pub endpoint: String,
    pub authentication: Option<McpAuth>,
    pub enabled: bool,
}

#[derive(Clone, Reflect)]
pub struct McpAuth {
    pub auth_type: String,
    pub credentials: String, // Encrypted
}

#[derive(Clone, Reflect)]
pub struct ApiKeyConfiguration {
    pub provider_keys: HashMap<String, EncryptedApiKey>,
    pub routing_preferences: HashMap<String, RoutingMode>,
    pub usage_limits: HashMap<String, UsageLimit>,
}

#[derive(Clone, Reflect)]
pub struct EncryptedApiKey {
    pub encrypted_key: String,
    pub provider: String,
    pub is_valid: bool,
    pub last_validated: Option<f64>,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum RoutingMode {
    RaycastProxy,
    DirectProvider,
    Automatic,
}

#[derive(Clone, Reflect)]
pub struct UsageLimit {
    pub max_requests_per_hour: Option<u32>,
    pub max_cost_per_month: Option<f64>,
    pub current_usage: UsageStats,
}

#[derive(Clone, Reflect, Default)]
pub struct UsageStats {
    pub requests_this_hour: u32,
    pub cost_this_month: f64,
    pub last_reset: f64,
}
```

### Advanced Event System

```rust
// Extended events for advanced AI menu features
#[derive(Event, Reflect)]
pub enum AdvancedAiMenuEvent {
    // AI Commands events
    CommandModelChanged(String),
    CommandCreated(AiCommand),
    CommandUpdated(String, AiCommand),
    CommandDeleted(String),
    
    // Tools events
    ToolCallInfoToggled(bool),
    ToolConfirmationsReset,
    ToolPermissionChanged(String, bool),
    
    // MCP events
    McpServerManagementOpened,
    McpIdleTimeChanged(u32),
    McpAutoConfirmToggled(bool),
    McpServerConnected(String),
    McpServerDisconnected(String),
    McpServerError(String, String),
    
    // API Key events
    ApiKeyAdded(String, String), // provider, encrypted_key
    ApiKeyRemoved(String),
    ApiKeyValidated(String, bool),
    RoutingModeChanged(String, RoutingMode),
}

#[derive(Event, Reflect)]
pub struct McpServerStatusChanged {
    pub server_id: String,
    pub old_status: McpServerStatus,
    pub new_status: McpServerStatus,
}

#[derive(Event, Reflect)]
pub struct ApiKeyValidationEvent {
    pub provider: String,
    pub is_valid: bool,
    pub error_message: Option<String>,
}

#[derive(Event, Reflect)]
pub struct SecurityWarningEvent {
    pub warning_type: SecurityWarningType,
    pub context: String,
}

#[derive(Clone, Reflect)]
pub enum SecurityWarningType {
    AutoConfirmAllTools,
    UntrustedMcpServer,
    InvalidApiKey,
    ExcessiveApiUsage,
}

#[derive(Clone, Reflect)]
pub struct AiCommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub model_override: Option<String>,
    pub tools_enabled: bool,
}
```

### Advanced System Architecture

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AdvancedAiMenuSystems {
    Input,
    Validation,
    NetworkOperations,
    StateUpdate,
    SecurityChecks,
    Animation,
    Rendering,
}

impl Plugin for AdvancedAiMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<AdvancedAiMenuState>()
            .init_resource::<McpClientPool>()
            .init_resource::<ApiKeyValidator>()
            .init_resource::<SecurityManager>()
            
            // Events
            .add_event::<AdvancedAiMenuEvent>()
            .add_event::<McpServerStatusChanged>()
            .add_event::<ApiKeyValidationEvent>()
            .add_event::<SecurityWarningEvent>()
            
            // System ordering with async operations support
            .configure_sets(Update, (
                AdvancedAiMenuSystems::Input,
                AdvancedAiMenuSystems::Validation,
                AdvancedAiMenuSystems::NetworkOperations,
                AdvancedAiMenuSystems::StateUpdate,
                AdvancedAiMenuSystems::SecurityChecks,
                AdvancedAiMenuSystems::Animation,
                AdvancedAiMenuSystems::Rendering,
            ).chain())
            
            // Input systems
            .add_systems(Update, (
                handle_ai_commands_interactions,
                handle_tools_interactions,
                handle_mcp_interactions,
                handle_api_key_interactions,
            ).in_set(AdvancedAiMenuSystems::Input))
            
            // Validation systems
            .add_systems(Update, (
                validate_api_keys,
                validate_mcp_configurations,
                validate_tool_permissions,
            ).in_set(AdvancedAiMenuSystems::Validation))
            
            // Network operations (using AsyncComputeTaskPool)
            .add_systems(Update, (
                manage_mcp_connections,
                test_api_key_validity,
                sync_server_status,
            ).in_set(AdvancedAiMenuSystems::NetworkOperations))
            
            // State management
            .add_systems(Update, (
                update_advanced_menu_state,
                persist_configuration_changes,
                handle_configuration_migration,
            ).in_set(AdvancedAiMenuSystems::StateUpdate))
            
            // Security systems
            .add_systems(Update, (
                monitor_security_warnings,
                audit_api_usage,
                validate_tool_permissions,
            ).in_set(AdvancedAiMenuSystems::SecurityChecks))
            
            // Animations
            .add_systems(Update, (
                animate_warning_icons,
                animate_server_status_indicators,
                animate_dropdown_expansions,
            ).in_set(AdvancedAiMenuSystems::Animation))
            
            // Rendering updates
            .add_systems(Update, (
                update_provider_icons,
                update_server_status_displays,
                update_security_warning_visibility,
                apply_advanced_styling,
            ).in_set(AdvancedAiMenuSystems::Rendering));
    }
}
```

### Flex-Based Advanced Layout

```rust
fn setup_advanced_right_pane(
    parent: &mut ChildSpawnerCommands, 
    asset_server: &AssetServer,
    menu_state: &AdvancedAiMenuState,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_height: Val::Px(800.0), // Constrain height
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(24.0),
            overflow: Overflow::clip_y(), // Handle overflow properly
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
    )).with_children(|content_parent| {
        
        // AI Commands Section with constrained layout
        spawn_ai_commands_section(content_parent, asset_server, &menu_state.ai_commands_model);
        
        // Tools Section with proper flex constraints
        spawn_tools_section(content_parent, asset_server, &menu_state.tools_config);
        
        // MCP Section with server management
        spawn_mcp_section(content_parent, asset_server, &menu_state.mcp_config);
        
        // API Keys Section with routing information
        spawn_api_keys_section(content_parent, asset_server, &menu_state.api_keys);
    });
}

fn spawn_ai_commands_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    selected_model: &str,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(120.0),
            max_height: Val::Px(200.0), // Constrain section height
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(16.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        AiCommandsSection {
            selected_model: selected_model.to_string(),
            default_model: "Gemini 2.5 Pro".to_string(),
        },
    )).with_children(|section_parent| {
        
        // Section title
        section_parent.spawn((
            Text::new("AI Commands"),
            TextFont {
                font: asset_server.load("fonts/Inter-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        
        // Section description
        section_parent.spawn((
            Text::new("Custom instructions for common tasks such as improving your writing or code"),
            TextFont {
                font: asset_server.load("fonts/Inter-Regular.ttf"),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        // Two-column layout for controls
        section_parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                max_height: Val::Px(80.0), // Constrain control area
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                ..default()
            },
        )).with_children(|row_parent| {
            
            // Left label
            row_parent.spawn((
                Text::new("AI Commands Model"),
                TextFont {
                    font: asset_server.load("fonts/Inter-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                Node {
                    width: Val::Percent(35.0),
                    max_width: Val::Px(200.0), // Constrain label width
                    flex_grow: 0.0,
                    ..default()
                },
            ));
            
            // Right dropdown with provider icon
            spawn_model_dropdown_with_provider(
                row_parent, 
                asset_server, 
                selected_model,
                "Google",
                "icons/google_g.png"
            );
        });
    });
}

fn spawn_mcp_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    mcp_config: &McpConfiguration,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            min_height: Val::Px(180.0),
            max_height: Val::Px(280.0), // Constrain MCP section height
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0,
            row_gap: Val::Px(16.0),
            padding: UiRect::all(Val::Px(16.0)),
            overflow: Overflow::clip_y(),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        McpSection {
            server_idle_timeout: mcp_config.server_idle_timeout_minutes,
            auto_confirm_tools: mcp_config.auto_confirm_all_tools,
            connected_servers: mcp_config.managed_servers.iter()
                .map(|config| McpServer {
                    name: config.name.clone(),
                    url: config.endpoint.clone(),
                    status: if config.enabled { 
                        McpServerStatus::Connected 
                    } else { 
                        McpServerStatus::Disconnected 
                    },
                    capabilities: vec![], // Would be populated from actual server
                    last_ping: None,
                })
                .collect(),
        },
    )).with_children(|mcp_parent| {
        
        // Title and description
        mcp_parent.spawn((
            Text::new("Model Context Protocol"),
            TextFont {
                font: asset_server.load("fonts/Inter-Bold.ttf"),
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ));
        
        mcp_parent.spawn((
            Text::new("Connect external tools and data sources using the Model Context Protocol (MCP)"),
            TextFont {
                font: asset_server.load("fonts/Inter-Regular.ttf"),
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
        
        // Manage Servers button row
        spawn_two_column_row(mcp_parent, asset_server, "Manage Servers", |right_parent| {
            right_parent.spawn((
                Button,
                Node {
                    width: Val::Px(140.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                McpServerButton,
            )).with_children(|btn_parent| {
                btn_parent.spawn((
                    Text::new("Manage Servers"),
                    TextFont {
                        font: asset_server.load("fonts/Inter-Regular.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        });
        
        // Server Idle Time dropdown
        spawn_two_column_row(mcp_parent, asset_server, "Server Idle Time", |right_parent| {
            spawn_idle_time_dropdown(right_parent, asset_server, mcp_config.server_idle_timeout_minutes);
        });
        
        // Auto-confirm tools checkbox with warning
        spawn_auto_confirm_checkbox_with_warning(mcp_parent, asset_server, mcp_config.auto_confirm_all_tools);
    });
}
```

### Advanced Animation and Security Systems

```rust
fn animate_warning_icons(
    time: Res<Time>,
    mut warning_query: Query<(&mut Transform, &SecurityWarningIcon), With<SecurityWarningIcon>>,
) {
    for (mut transform, _) in warning_query.iter_mut() {
        // Subtle pulsing animation for warning icons
        let pulse = (time.elapsed_secs() * 2.0).sin() * 0.1 + 1.0;
        transform.scale = Vec3::splat(pulse);
    }
}

fn monitor_security_warnings(
    mcp_query: Query<&McpSection, Changed<McpSection>>,
    tools_query: Query<&ToolsSection, Changed<ToolsSection>>,
    mut security_events: EventWriter<SecurityWarningEvent>,
    mut commands: Commands,
) {
    // Monitor MCP auto-confirm setting
    for mcp_section in mcp_query.iter() {
        if mcp_section.auto_confirm_tools {
            security_events.write(SecurityWarningEvent {
                warning_type: SecurityWarningType::AutoConfirmAllTools,
                context: "Auto-confirming all tool calls poses security risks".to_string(),
            });
        }
    }
    
    // Monitor tools configuration
    for tools_section in tools_query.iter() {
        if !tools_section.tool_confirmations_enabled {
            security_events.write(SecurityWarningEvent {
                warning_type: SecurityWarningType::UntrustedMcpServer,
                context: "Tool confirmations disabled".to_string(),
            });
        }
    }
}

// Async API key validation using Bevy's task system
fn test_api_key_validity(
    mut api_validation_events: EventWriter<ApiKeyValidationEvent>,
    api_keys: Res<AdvancedAiMenuState>,
    task_pool: Res<AsyncComputeTaskPool>,
    mut commands: Commands,
) {
    for (provider, key_info) in &api_keys.api_keys.provider_keys {
        if key_info.last_validated.is_none() || 
           key_info.last_validated.unwrap() < (time::SystemTime::now()
               .duration_since(time::UNIX_EPOCH)
               .unwrap()
               .as_secs() as f64 - 3600.0) // 1 hour cache
        {
            let provider_clone = provider.clone();
            let key_clone = key_info.encrypted_key.clone();
            
            let task = task_pool.spawn(async move {
                // Simulate API key validation (would be actual HTTP request)
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Return validation result
                ApiKeyValidationEvent {
                    provider: provider_clone,
                    is_valid: !key_clone.is_empty(), // Simplified validation
                    error_message: None,
                }
            });
            
            commands.spawn(AsyncApiValidationTask(task));
        }
    }
}

#[derive(Component)]
struct AsyncApiValidationTask(Task<ApiKeyValidationEvent>);

fn handle_api_validation_results(
    mut commands: Commands,
    mut validation_tasks: Query<(Entity, &mut AsyncApiValidationTask)>,
    mut validation_events: EventWriter<ApiKeyValidationEvent>,
) {
    for (entity, mut task) in validation_tasks.iter_mut() {
        if let Some(result) = block_on(future::poll_once(&mut task.0)) {
            validation_events.write(result);
            commands.entity(entity).despawn();
        }
    }
}
```

### Testing Strategy for Advanced Features

```rust
#[cfg(test)]
mod advanced_tests {
    use super::*;
    
    #[test]
    fn test_mcp_auto_confirm_security_warning() {
        let mut app = setup_test_app();
        
        // Create MCP section with auto-confirm enabled
        let mcp_entity = app.world_mut().spawn((
            McpSection {
                server_idle_timeout: 5,
                auto_confirm_tools: true,
                connected_servers: vec![],
            },
        )).id();
        
        app.update();
        
        // Verify security warning was generated
        let security_events: Vec<_> = app.world()
            .resource::<Events<SecurityWarningEvent>>()
            .get_reader()
            .read(app.world().resource::<Events<SecurityWarningEvent>>())
            .collect();
        
        assert!(!security_events.is_empty());
        assert!(matches!(
            security_events[0].warning_type, 
            SecurityWarningType::AutoConfirmAllTools
        ));
    }
    
    #[test]
    fn test_api_key_validation_flow() {
        let mut app = setup_test_app();
        
        // Add API key configuration
        let mut api_keys = HashMap::new();
        api_keys.insert("openai".to_string(), EncryptedApiKey {
            encrypted_key: "test_key".to_string(),
            provider: "OpenAI".to_string(),
            is_valid: false,
            last_validated: None,
        });
        
        app.world_mut().insert_resource(AdvancedAiMenuState {
            ai_commands_model: "GPT-4".to_string(),
            tools_config: ToolsConfiguration::default(),
            mcp_config: McpConfiguration::default(),
            api_keys: ApiKeyConfiguration {
                provider_keys: api_keys,
                routing_preferences: HashMap::new(),
                usage_limits: HashMap::new(),
            },
        });
        
        // Run several update cycles to allow async validation
        for _ in 0..10 {
            app.update();
        }
        
        // Verify validation events were generated
        let validation_events: Vec<_> = app.world()
            .resource::<Events<ApiKeyValidationEvent>>()
            .get_reader()
            .read(app.world().resource::<Events<ApiKeyValidationEvent>>())
            .collect();
        
        assert!(!validation_events.is_empty());
        assert_eq!(validation_events[0].provider, "openai");
    }
}