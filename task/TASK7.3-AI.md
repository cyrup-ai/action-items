# TASK7.3: Settings Panel - AI

**Status**: Not Started  
**Estimated Time**: 5-7 hours  
**Priority**: High  
**Dependencies**: TASK7.0-INFRASTRUCTURE.md, TASK7.C-COMPONENTS.md

---

## Objective

Implement the AI settings panel with comprehensive AI configuration including Quick AI responses, AI Chat settings, AI Commands configuration, Model Context Protocol (MCP) integration, Ollama local model support, and experimental features. This panel provides complete control over AI functionality with model selection, hotkey configuration, tool management, and advanced experiments.

---

## Dependencies

**MUST complete first:**
1. ✅ TASK7.0-INFRASTRUCTURE.md - Settings modal, tabs, entity pre-allocation
2. ✅ TASK7.C-COMPONENTS.md - Toggle, Dropdown, TextInput, Button, HotkeyRecorder components

**Required systems:**
- `SettingsUIEntities` resource (from TASK7.0)
- `SettingControl` component (from TASK7.C)
- Form control spawning functions (from TASK7.C)
- Event handlers for database I/O (from TASK7.0)

---

## Screenshot References

![AI Menu 1](/Volumes/samsung_t9/action-items/spec/screenshots/AI_Menu.png)
![AI Menu 2](/Volumes/samsung_t9/action-items/spec/screenshots/AI_Menu_2.png)
![AI Menu 3](/Volumes/samsung_t9/action-items/spec/screenshots/AI_Menu_3.png)

**Visual Structure (3 screens worth of content, scrollable):**

```
┌─────────────────────────────────────────────────────────┐
│ ◉ Full control  🔒 No collection  🔐 Encrypted  ℹ️     │
│                                                         │
│    ┌───┐                                                │
│    │ ✨ │  Raycast AI                                   │
│    └───┘  Unlock the power of AI                       │
│           [──●──] Master toggle                        │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ Quick AI          Get instant AI responses             │
│                                                         │
│ Trigger          [Tab to Ask AI            ▼] ℹ️       │
│ ☑ Show Ask AI hint in root search                     │
│ Quick AI Model   [Sonar Reasoning Pro     ▼] ℹ️       │
│ ☑ Web Search                                           │
│ Default Action   [Paste Response to Active ▼]         │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ AI Chat          Dedicated chat window                 │
│                                                         │
│                  [  ^ ⌥ ⌘ L  ]              [×]        │
│ Start New Chat   [After 30 minutes        ▼] ℹ️       │
│ New Chat Settings[CYRUP (openai)          ▼]          │
│ Text Size        Aa  Aa                                │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ AI Commands      Custom instructions for tasks         │
│                                                         │
│ Commands Model   [Gemini 2.5 Pro          ▼]          │
│                  Default model for custom commands     │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ Tools            ☐ Show Tool Call Info         ℹ️      │
│                  [Reset Tool Confirmations]            │
│                                                         │
│ Model Context    Connect external tools (MCP)          │
│ Protocol         [Manage Servers]                      │
│                                                         │
│ Server Idle Time [5 Minutes               ▼] ℹ️       │
│ ☑ Automatically confirm all tool calls    ⚠️          │
│                                                         │
│ Custom API Keys  Add API keys for custom providers     │
│                  • Anthropic, Google, OpenAI routed    │
│                    via Raycast servers                 │
│                  • OpenRouter routed directly          │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ Ollama Host      [127.0.0.1:11434        ] ℹ️         │
│                  [Sync Models]                         │
│ Install Model    [Enter a model name     ⬇]           │
│                  5 models installed via Ollama         │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ Advanced                                                │
│                                                         │
│ Browser Ext.     Bring context from browser            │
│                  Last connection: 8/6/2025, 5:30 PM    │
│                                                         │
│ Experiments      New AI features in development        │
│                                                         │
│ Auto Models      [──●──] ℹ️                            │
│ Chat Branching   [──●──] ℹ️                            │
│ Custom Providers [────○] ℹ️                            │
│ MCP HTTP Servers [──●──] ℹ️                            │
│ Ollama Extensions[──●──] ℹ️                            │
│                                                         │
│ [Open AI Help Manual]  [Discover Raycast AI]          │
└─────────────────────────────────────────────────────────┘
```

---

## Database Schema

### Table: `ai_quick_settings`

```sql
DEFINE TABLE ai_quick_settings SCHEMALESS;

-- Quick AI Configuration
DEFINE FIELD trigger_type ON TABLE ai_quick_settings TYPE string DEFAULT "tab";
  -- Values: "tab", "hotkey", "slash_command"

DEFINE FIELD show_hint_in_search ON TABLE ai_quick_settings TYPE bool DEFAULT true;

DEFINE FIELD quick_ai_model ON TABLE ai_quick_settings TYPE string DEFAULT "sonar-reasoning-pro";
  -- Model identifier for quick responses

DEFINE FIELD web_search_enabled ON TABLE ai_quick_settings TYPE bool DEFAULT true;

DEFINE FIELD default_primary_action ON TABLE ai_quick_settings TYPE string DEFAULT "paste_to_active";
  -- Values: "paste_to_active", "copy_to_clipboard", "show_in_window"
```

### Table: `ai_chat_settings`

```sql
DEFINE TABLE ai_chat_settings SCHEMALESS;

-- Chat Window Configuration
DEFINE FIELD chat_hotkey ON TABLE ai_chat_settings TYPE string DEFAULT "Ctrl+Alt+Meta+L";
DEFINE FIELD chat_hotkey_enabled ON TABLE ai_chat_settings TYPE bool DEFAULT true;

DEFINE FIELD start_new_chat_after ON TABLE ai_chat_settings TYPE string DEFAULT "30_minutes";
  -- Values: "never", "5_minutes", "30_minutes", "1_hour", "1_day"

DEFINE FIELD new_chat_default_model ON TABLE ai_chat_settings TYPE string DEFAULT "cyrup_openai";

DEFINE FIELD chat_text_size ON TABLE ai_chat_settings TYPE string DEFAULT "medium";
  -- Values: "small", "medium", "large"
```

### Table: `ai_commands_settings`

```sql
DEFINE TABLE ai_commands_settings SCHEMALESS;

-- AI Commands Configuration
DEFINE FIELD commands_model ON TABLE ai_commands_settings TYPE string DEFAULT "gemini-2.5-pro";
DEFINE FIELD show_tool_call_info ON TABLE ai_commands_settings TYPE bool DEFAULT false;
DEFINE FIELD confirmed_tools ON TABLE ai_commands_settings TYPE array DEFAULT [];
```

### Table: `ai_mcp_settings`

```sql
DEFINE TABLE ai_mcp_settings SCHEMALESS;

-- Model Context Protocol Settings
DEFINE FIELD server_idle_time ON TABLE ai_mcp_settings TYPE string DEFAULT "5_minutes";
  -- Values: "1_minute", "5_minutes", "15_minutes", "30_minutes", "never"

DEFINE FIELD auto_confirm_tool_calls ON TABLE ai_mcp_settings TYPE bool DEFAULT false;

DEFINE FIELD mcp_servers ON TABLE ai_mcp_settings TYPE array DEFAULT [];
  -- Array of MCP server configurations
```

### Table: `ai_custom_api_keys`

```sql
DEFINE TABLE ai_custom_api_keys SCHEMALESS;

-- Custom API Keys (encrypted)
DEFINE FIELD provider ON TABLE ai_custom_api_keys TYPE string;
  -- Values: "anthropic", "openai", "google", "openrouter"

DEFINE FIELD api_key_encrypted ON TABLE ai_custom_api_keys TYPE string;
DEFINE FIELD api_key_last_4 ON TABLE ai_custom_api_keys TYPE string;
DEFINE FIELD added_at ON TABLE ai_custom_api_keys TYPE datetime;
DEFINE FIELD last_used_at ON TABLE ai_custom_api_keys TYPE datetime;

DEFINE INDEX idx_provider ON TABLE ai_custom_api_keys COLUMNS provider UNIQUE;
```

### Table: `ai_ollama_settings`

```sql
DEFINE TABLE ai_ollama_settings SCHEMALESS;

-- Ollama Local Model Configuration
DEFINE FIELD ollama_host ON TABLE ai_ollama_settings TYPE string DEFAULT "127.0.0.1:11434";
DEFINE FIELD installed_models ON TABLE ai_ollama_settings TYPE array DEFAULT [];
DEFINE FIELD last_sync_at ON TABLE ai_ollama_settings TYPE datetime;
```

### Table: `ai_browser_extension_settings`

```sql
DEFINE TABLE ai_browser_extension_settings SCHEMALESS;

-- Browser Extension Integration
DEFINE FIELD enabled ON TABLE ai_browser_extension_settings TYPE bool DEFAULT false;
DEFINE FIELD last_connection_at ON TABLE ai_browser_extension_settings TYPE datetime;
DEFINE FIELD connection_status ON TABLE ai_browser_extension_settings TYPE string DEFAULT "disconnected";
  -- Values: "disconnected", "connected", "error"
```

### Table: `ai_experiments`

```sql
DEFINE TABLE ai_experiments SCHEMALESS;

-- Experimental Features
DEFINE FIELD auto_models ON TABLE ai_experiments TYPE bool DEFAULT true;
DEFINE FIELD chat_branching ON TABLE ai_experiments TYPE bool DEFAULT true;
DEFINE FIELD custom_providers ON TABLE ai_experiments TYPE bool DEFAULT false;
DEFINE FIELD mcp_http_servers ON TABLE ai_experiments TYPE bool DEFAULT true;
DEFINE FIELD ollama_extensions ON TABLE ai_experiments TYPE bool DEFAULT true;
```

---

## Component Structure

### Components

```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Marker component for the AI panel root entity
#[derive(Component, Debug)]
pub struct AIPanel;

/// Marker for the master AI enable/disable toggle
#[derive(Component, Debug)]
pub struct AIMasterToggle;

/// Component for status badge entities
#[derive(Component, Debug, Clone)]
pub struct AIStatusBadge {
    pub badge_type: AIStatusBadgeType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIStatusBadgeType {
    FullControl,      // User has full control over AI
    NoCollection,     // No data collection
    Encrypted,        // Data is encrypted
}

impl AIStatusBadgeType {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::FullControl => "◉",
            Self::NoCollection => "🔒",
            Self::Encrypted => "🔐",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            Self::FullControl => "Full control",
            Self::NoCollection => "No collection",
            Self::Encrypted => "Encrypted",
        }
    }
}

/// Component for MCP server management button
#[derive(Component, Debug)]
pub struct MCPManageServersButton;

/// Component for tool confirmations reset button
#[derive(Component, Debug)]
pub struct ResetToolConfirmationsButton;

/// Component for Ollama sync button
#[derive(Component, Debug)]
pub struct OllamaSyncModelsButton;

/// Component for Ollama model install input
#[derive(Component, Debug)]
pub struct OllamaModelInstallInput {
    pub model_name: String,
}

/// Component for AI help buttons
#[derive(Component, Debug)]
pub struct AIHelpButton {
    pub button_type: AIHelpButtonType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AIHelpButtonType {
    OpenHelpManual,
    DiscoverRaycastAI,
}

/// Component for experimental feature toggles
#[derive(Component, Debug)]
pub struct ExperimentToggle {
    pub experiment: ExperimentType,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExperimentType {
    AutoModels,
    ChatBranching,
    CustomProviders,
    MCPHTTPServers,
    OllamaExtensions,
}

impl ExperimentType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AutoModels => "Auto Models",
            Self::ChatBranching => "Chat Branching",
            Self::CustomProviders => "Custom Providers",
            Self::MCPHTTPServers => "MCP HTTP Servers",
            Self::OllamaExtensions => "AI Extensions for Ollama Models",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            Self::AutoModels => "Automatically select the best model for each task",
            Self::ChatBranching => "Create conversation branches to explore different ideas",
            Self::CustomProviders => "Add custom AI provider integrations",
            Self::MCPHTTPServers => "Enable HTTP-based MCP server connections",
            Self::OllamaExtensions => "Allow Ollama models to use extensions",
        }
    }
}
```

### Resources

```rust
/// Resource tracking all entities in the AI panel
#[derive(Resource)]
pub struct AIPanelEntities {
    pub panel_root: Entity,
    
    // Status badges
    pub status_badges: Vec<Entity>,
    pub master_toggle: Entity,
    
    // Quick AI section
    pub quick_ai_trigger_dropdown: Entity,
    pub show_hint_toggle: Entity,
    pub quick_ai_model_dropdown: Entity,
    pub web_search_toggle: Entity,
    pub default_action_dropdown: Entity,
    
    // AI Chat section
    pub chat_hotkey_recorder: Entity,
    pub start_new_chat_dropdown: Entity,
    pub new_chat_model_dropdown: Entity,
    pub chat_text_size_small: Entity,
    pub chat_text_size_large: Entity,
    
    // AI Commands section
    pub commands_model_dropdown: Entity,
    pub show_tool_info_toggle: Entity,
    pub reset_confirmations_button: Entity,
    
    // MCP section
    pub mcp_manage_servers_button: Entity,
    pub mcp_idle_time_dropdown: Entity,
    pub mcp_auto_confirm_toggle: Entity,
    
    // Ollama section
    pub ollama_host_input: Entity,
    pub ollama_sync_button: Entity,
    pub ollama_install_input: Entity,
    pub ollama_status_text: Entity,
    
    // Browser extension section
    pub browser_ext_status_text: Entity,
    
    // Experiments section
    pub experiment_toggles: HashMap<ExperimentType, Entity>,
    
    // Help buttons
    pub help_manual_button: Entity,
    pub discover_ai_button: Entity,
}

/// Resource tracking available AI models
#[derive(Resource, Default)]
pub struct AIModelRegistry {
    pub quick_ai_models: Vec<AIModel>,
    pub chat_models: Vec<AIModel>,
    pub commands_models: Vec<AIModel>,
    pub ollama_models: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AIModel {
    pub id: String,
    pub display_name: String,
    pub provider: String, // "anthropic", "openai", "google", etc.
    pub supports_web_search: bool,
    pub supports_tools: bool,
}
```

### Events

```rust
/// Event sent when MCP servers button is clicked
#[derive(Event, Debug)]
pub struct MCPManageServersRequested;

/// Event sent when reset tool confirmations button is clicked
#[derive(Event, Debug)]
pub struct ResetToolConfirmationsRequested;

/// Event sent when Ollama sync is requested
#[derive(Event, Debug)]
pub struct OllamaSyncRequested;

/// Event sent when Ollama model install is requested
#[derive(Event, Debug)]
pub struct OllamaModelInstallRequested {
    pub model_name: String,
}

/// Event sent when AI help button is clicked
#[derive(Event, Debug)]
pub struct AIHelpRequested {
    pub help_type: AIHelpButtonType,
}
```

---

## Implementation Details

### System 1: Setup AI Panel Entities

**Purpose**: Pre-allocate all AI panel UI entities during initialization

```rust
pub fn setup_ai_panel(
    mut commands: Commands,
    settings_entities: Res<SettingsUIEntities>,
    asset_server: Res<AssetServer>,
) {
    let content_area = settings_entities.content_area;
    
    // Create panel root
    let panel_root = commands.spawn((
        AIPanel,
        UiLayout::window()
            .size((Rl(100.0), Rl(100.0)))
            .pos((Rl(0.0), Rl(0.0)))
            .pack(),
        Visibility::Hidden,
        Name::new("AIPanel"),
    )).id();
    
    commands.entity(content_area).add_child(panel_root);
    
    // ═══════════════════════════════════════════════════════════
    // STATUS BADGES + MASTER TOGGLE
    // ═══════════════════════════════════════════════════════════
    
    let status_bar = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(40.0)))
            .pos((Rl(50.0), Ab(10.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Name::new("StatusBar"),
    )).id();
    
    let mut status_badges = Vec::new();
    let badge_types = [
        AIStatusBadgeType::FullControl,
        AIStatusBadgeType::NoCollection,
        AIStatusBadgeType::Encrypted,
    ];
    
    let mut x_offset = 0.0;
    for badge_type in badge_types {
        let badge = commands.spawn((
            AIStatusBadge { badge_type },
            UiLayout::window()
                .size((Ab(150.0), Ab(30.0)))
                .pos((Ab(x_offset), Ab(5.0)))
                .pack(),
            Text::new(format!("{} {}", badge_type.icon(), badge_type.label())),
            UiTextSize::from(Em(0.9)),
            UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
            Name::new(format!("StatusBadge_{:?}", badge_type)),
        )).id();
        
        status_badges.push(badge);
        commands.entity(status_bar).add_child(badge);
        x_offset += 160.0;
    }
    
    commands.entity(panel_root).add_child(status_bar);
    
    // AI branding section with master toggle
    let branding_section = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(120.0)))
            .pos((Rl(50.0), Ab(60.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Name::new("BrandingSection"),
    )).id();
    
    // AI icon placeholder (would load actual image)
    let ai_icon = commands.spawn((
        UiLayout::window()
            .size((Ab(80.0), Ab(80.0)))
            .pos((Rl(50.0), Ab(0.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        UiColor::from(Color::srgba(0.8, 0.2, 0.3, 1.0)), // Placeholder color
        Text::new("✨"),
        UiTextSize::from(Em(3.0)),
        Name::new("AIIcon"),
    )).id();
    
    // "Raycast AI" title
    let title = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(25.0)))
            .pos((Rl(50.0), Ab(85.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("Action Items AI"),
        UiTextSize::from(Em(1.3)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        Name::new("AITitle"),
    )).id();
    
    // Description text
    let description = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(40.0)))
            .pos((Rl(50.0), Ab(115.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("Unlock the power of AI on your Mac.\nWrite smarter, code faster, and answer questions\nquicker with Action Items AI."),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
        Name::new("AIDescription"),
    )).id();
    
    // Master toggle
    let master_toggle = spawn_toggle_switch(
        &mut commands,
        SettingControl {
            table: "ai_settings".to_string(),
            field: "enabled".to_string(),
            control_type: ControlType::Toggle,
        },
        true, // Default enabled
    );
    
    let toggle_container = commands.spawn((
        AIMasterToggle,
        UiLayout::window()
            .size((Ab(44.0), Ab(24.0)))
            .pos((Rl(50.0), Ab(160.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
    )).id();
    
    commands.entity(toggle_container).add_child(master_toggle);
    
    commands.entity(branding_section).push_children(&[ai_icon, title, description, toggle_container]);
    commands.entity(panel_root).add_child(branding_section);
    
    let mut y_offset = 200.0;
    
    // ═══════════════════════════════════════════════════════════
    // QUICK AI SECTION
    // ═══════════════════════════════════════════════════════════
    
    let quick_ai_section = create_section(
        &mut commands,
        "Quick AI",
        "Get instant AI responses directly from the root search",
        y_offset,
    );
    y_offset += 40.0;
    
    // Trigger dropdown
    let (trigger_label, trigger_dropdown) = create_labeled_dropdown(
        &mut commands,
        "Trigger",
        SettingControl {
            table: "ai_quick_settings".to_string(),
            field: "trigger_type".to_string(),
            control_type: ControlType::Dropdown {
                options: vec!["Tab to Ask AI".to_string(), "Hotkey".to_string(), "Slash Command".to_string()],
            },
        },
        "Tab to Ask AI",
        y_offset,
    );
    y_offset += 50.0;
    
    // Show hint checkbox
    let show_hint_toggle = create_labeled_toggle(
        &mut commands,
        "Show Ask AI hint in root search",
        SettingControl {
            table: "ai_quick_settings".to_string(),
            field: "show_hint_in_search".to_string(),
            control_type: ControlType::Toggle,
        },
        true,
        y_offset,
    );
    y_offset += 45.0;
    
    // Quick AI Model dropdown
    let (model_label, quick_ai_model_dropdown) = create_labeled_dropdown(
        &mut commands,
        "Quick AI Model",
        SettingControl {
            table: "ai_quick_settings".to_string(),
            field: "quick_ai_model".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Sonar Reasoning Pro".to_string(),
                    "GPT-4".to_string(),
                    "Claude 3.5 Sonnet".to_string(),
                    "Gemini 2.5 Pro".to_string(),
                ],
            },
        },
        "Sonar Reasoning Pro",
        y_offset,
    );
    y_offset += 50.0;
    
    // Web Search toggle
    let web_search_toggle = create_labeled_toggle(
        &mut commands,
        "Web Search",
        SettingControl {
            table: "ai_quick_settings".to_string(),
            field: "web_search_enabled".to_string(),
            control_type: ControlType::Toggle,
        },
        true,
        y_offset,
    );
    y_offset += 45.0;
    
    // Default Primary Action dropdown
    let (action_label, default_action_dropdown) = create_labeled_dropdown(
        &mut commands,
        "Default Primary Action",
        SettingControl {
            table: "ai_quick_settings".to_string(),
            field: "default_primary_action".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Paste Response to Active App".to_string(),
                    "Copy to Clipboard".to_string(),
                    "Show in Window".to_string(),
                ],
            },
        },
        "Paste Response to Active App",
        y_offset,
    );
    y_offset += 70.0;
    
    commands.entity(quick_ai_section).push_children(&[
        trigger_label, trigger_dropdown,
        show_hint_toggle,
        model_label, quick_ai_model_dropdown,
        web_search_toggle,
        action_label, default_action_dropdown,
    ]);
    commands.entity(panel_root).add_child(quick_ai_section);
    
    // ═══════════════════════════════════════════════════════════
    // AI CHAT SECTION
    // ═══════════════════════════════════════════════════════════
    
    let chat_section = create_section(
        &mut commands,
        "AI Chat",
        "Dedicated chat window for longer conversations with AI",
        y_offset,
    );
    y_offset += 40.0;
    
    // Chat hotkey recorder
    let chat_hotkey_label = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Hotkey"),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let chat_hotkey_recorder = spawn_hotkey_recorder(
        &mut commands,
        SettingControl {
            table: "ai_chat_settings".to_string(),
            field: "chat_hotkey".to_string(),
            control_type: ControlType::HotkeyRecorder,
        },
        "Ctrl+Alt+Meta+L".to_string(),
    );
    
    commands.spawn((
        UiLayout::window()
            .size((Rl(50.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset - 5.0)))
            .pack(),
    )).add_child(chat_hotkey_recorder);
    
    y_offset += 50.0;
    
    // Start New Chat dropdown
    let (start_chat_label, start_new_chat_dropdown) = create_labeled_dropdown(
        &mut commands,
        "Start New Chat",
        SettingControl {
            table: "ai_chat_settings".to_string(),
            field: "start_new_chat_after".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "After 5 minutes".to_string(),
                    "After 30 minutes".to_string(),
                    "After 1 hour".to_string(),
                    "Never".to_string(),
                ],
            },
        },
        "After 30 minutes",
        y_offset,
    );
    y_offset += 50.0;
    
    // New Chat Settings dropdown
    let (chat_model_label, new_chat_model_dropdown) = create_labeled_dropdown(
        &mut commands,
        "New Chat Settings",
        SettingControl {
            table: "ai_chat_settings".to_string(),
            field: "new_chat_default_model".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "CYRUP (openai)".to_string(),
                    "Claude 3.5 Sonnet".to_string(),
                    "GPT-4".to_string(),
                    "Gemini 2.5 Pro".to_string(),
                ],
            },
        },
        "CYRUP (openai)",
        y_offset,
    );
    y_offset += 50.0;
    
    // Chat text size controls
    let text_size_label = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Text Size"),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let chat_text_size_small = commands.spawn((
        TextSizeControl { size: TextSizeOption::Small },
        UiLayout::window()
            .size((Ab(50.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset - 5.0)))
            .pack(),
        UiColor::from(Color::srgba(0.25, 0.25, 0.28, 1.0)),
        Text::new("Aa"),
        UiTextSize::from(Em(0.9)),
    )).id();
    
    let chat_text_size_large = commands.spawn((
        TextSizeControl { size: TextSizeOption::Large },
        UiLayout::window()
            .size((Ab(50.0), Ab(40.0)))
            .pos((Rl(40.0) + Ab(60.0), Ab(y_offset - 5.0)))
            .pack(),
        UiColor::from(Color::srgba(0.25, 0.25, 0.28, 1.0)),
        Text::new("Aa"),
        UiTextSize::from(Em(1.3)),
    )).id();
    
    y_offset += 70.0;
    
    commands.entity(chat_section).push_children(&[
        chat_hotkey_label,
        start_chat_label, start_new_chat_dropdown,
        chat_model_label, new_chat_model_dropdown,
        text_size_label, chat_text_size_small, chat_text_size_large,
    ]);
    commands.entity(panel_root).add_child(chat_section);
    
    // ═══════════════════════════════════════════════════════════
    // AI COMMANDS SECTION
    // ═══════════════════════════════════════════════════════════
    
    let commands_section = create_section(
        &mut commands,
        "AI Commands",
        "Custom instructions for common tasks such as\nimproving your writing or code",
        y_offset,
    );
    y_offset += 60.0;
    
    let (cmd_model_label, commands_model_dropdown) = create_labeled_dropdown(
        &mut commands,
        "AI Commands Model",
        SettingControl {
            table: "ai_commands_settings".to_string(),
            field: "commands_model".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Gemini 2.5 Pro".to_string(),
                    "Claude 3.5 Sonnet".to_string(),
                    "GPT-4".to_string(),
                ],
            },
        },
        "Gemini 2.5 Pro",
        y_offset,
    );
    y_offset += 50.0;
    
    let help_text = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        Text::new("Default model for custom AI Commands.\nYou can still select a different model for\nindividual AI Commands."),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 70.0;
    
    commands.entity(commands_section).push_children(&[
        cmd_model_label, commands_model_dropdown, help_text,
    ]);
    commands.entity(panel_root).add_child(commands_section);
    
    // ═══════════════════════════════════════════════════════════
    // TOOLS SECTION
    // ═══════════════════════════════════════════════════════════
    
    let tools_section_title = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Tools"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 35.0;
    
    let show_tool_info_toggle = create_labeled_toggle(
        &mut commands,
        "Show Tool Call Info",
        SettingControl {
            table: "ai_commands_settings".to_string(),
            field: "show_tool_call_info".to_string(),
            control_type: ControlType::Toggle,
        },
        false,
        y_offset,
    );
    y_offset += 45.0;
    
    let reset_confirmations_button = commands.spawn((
        ResetToolConfirmationsButton,
        UiLayout::window()
            .size((Ab(200.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Text::new("Reset Tool Confirmations"),
        Pickable::default(),
        Interaction::None,
    )).id();
    y_offset += 70.0;
    
    commands.entity(panel_root).push_children(&[
        tools_section_title,
        show_tool_info_toggle,
        reset_confirmations_button,
    ]);
    
    // ═══════════════════════════════════════════════════════════
    // MODEL CONTEXT PROTOCOL SECTION
    // ═══════════════════════════════════════════════════════════
    
    let mcp_section = create_section(
        &mut commands,
        "Model Context Protocol",
        "Connect external tools and data sources using\nthe Model Context Protocol (MCP)",
        y_offset,
    );
    y_offset += 60.0;
    
    let mcp_manage_servers_button = commands.spawn((
        MCPManageServersButton,
        UiLayout::window()
            .size((Ab(150.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Text::new("Manage Servers"),
        Pickable::default(),
        Interaction::None,
    )).id();
    y_offset += 55.0;
    
    let (idle_label, mcp_idle_time_dropdown) = create_labeled_dropdown(
        &mut commands,
        "Server Idle Time",
        SettingControl {
            table: "ai_mcp_settings".to_string(),
            field: "server_idle_time".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "1 Minute".to_string(),
                    "5 Minutes".to_string(),
                    "15 Minutes".to_string(),
                    "30 Minutes".to_string(),
                    "Never".to_string(),
                ],
            },
        },
        "5 Minutes",
        y_offset,
    );
    y_offset += 50.0;
    
    let mcp_auto_confirm_toggle = create_labeled_toggle(
        &mut commands,
        "Automatically confirm all tool calls",
        SettingControl {
            table: "ai_mcp_settings".to_string(),
            field: "auto_confirm_tool_calls".to_string(),
            control_type: ControlType::Toggle,
        },
        false,
        y_offset,
    );
    y_offset += 70.0;
    
    commands.entity(mcp_section).push_children(&[
        mcp_manage_servers_button,
        idle_label, mcp_idle_time_dropdown,
        mcp_auto_confirm_toggle,
    ]);
    commands.entity(panel_root).add_child(mcp_section);
    
    // ═══════════════════════════════════════════════════════════
    // CUSTOM API KEYS SECTION
    // ═══════════════════════════════════════════════════════════
    
    let api_keys_section_title = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Custom API Keys"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 35.0;
    
    let api_keys_description = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(60.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        Text::new("Add API keys to use AI at your own cost.\n• Anthropic, Google, OpenAI: Requests are\n  routed via Raycast servers\n• OpenRouter: Requests are routed directly to\n  the provider's servers"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
    )).id();
    y_offset += 100.0;
    
    commands.entity(panel_root).push_children(&[
        api_keys_section_title,
        api_keys_description,
    ]);
    
    // ═══════════════════════════════════════════════════════════
    // OLLAMA SECTION
    // ═══════════════════════════════════════════════════════════
    
    let ollama_section_separator = create_separator(&mut commands, y_offset);
    y_offset += 20.0;
    
    let (ollama_host_label, ollama_host_input) = create_labeled_text_input(
        &mut commands,
        "Ollama Host",
        SettingControl {
            table: "ai_ollama_settings".to_string(),
            field: "ollama_host".to_string(),
            control_type: ControlType::TextInput { validation: None },
        },
        "127.0.0.1:11434",
        y_offset,
    );
    y_offset += 50.0;
    
    let ollama_sync_button = commands.spawn((
        OllamaSyncModelsButton,
        UiLayout::window()
            .size((Ab(130.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Text::new("Sync Models"),
        Pickable::default(),
        Interaction::None,
    )).id();
    y_offset += 55.0;
    
    let (install_label, ollama_install_input) = create_labeled_text_input(
        &mut commands,
        "Install Ollama Model",
        SettingControl {
            table: "ai_ollama_settings".to_string(),
            field: "install_model_name".to_string(),
            control_type: ControlType::TextInput { validation: None },
        },
        "Enter a model name",
        y_offset,
    );
    y_offset += 50.0;
    
    let ollama_status_text = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        Text::new("5 models installed via Ollama"),
        UiTextSize::from(Em(0.9)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 70.0;
    
    commands.entity(panel_root).push_children(&[
        ollama_section_separator,
        ollama_host_label, ollama_host_input,
        ollama_sync_button,
        install_label, ollama_install_input,
        ollama_status_text,
    ]);
    
    // ═══════════════════════════════════════════════════════════
    // ADVANCED SECTION
    // ═══════════════════════════════════════════════════════════
    
    let advanced_separator = create_separator(&mut commands, y_offset);
    y_offset += 20.0;
    
    let advanced_title = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Advanced"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 50.0;
    
    // Browser Extension
    let browser_ext_label = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Browser Extension"),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let browser_ext_description = commands.spawn((
        UiLayout::window()
            .size((Rl(60.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        Text::new("Bring context from your browser tab into\nAction Items AI"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
    )).id();
    y_offset += 45.0;
    
    let browser_ext_status_text = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        Text::new("Last successful connection on 8/6/2025, 5:30 PM"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 70.0;
    
    commands.entity(panel_root).push_children(&[
        advanced_separator,
        advanced_title,
        browser_ext_label,
        browser_ext_description,
        browser_ext_status_text,
    ]);
    
    // ═══════════════════════════════════════════════════════════
    // EXPERIMENTS SECTION
    // ═══════════════════════════════════════════════════════════
    
    let experiments_title = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Experiments"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    y_offset += 35.0;
    
    let experiments_description = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset)))
            .pack(),
        Text::new("New AI features in development. Your feedback\nwill help us improve these experiments."),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
    )).id();
    y_offset += 55.0;
    
    commands.entity(panel_root).push_children(&[
        experiments_title,
        experiments_description,
    ]);
    
    // Experiment toggles
    let mut experiment_toggles = HashMap::new();
    let experiments = [
        (ExperimentType::AutoModels, true),
        (ExperimentType::ChatBranching, true),
        (ExperimentType::CustomProviders, false),
        (ExperimentType::MCPHTTPServers, true),
        (ExperimentType::OllamaExtensions, true),
    ];
    
    for (exp_type, default_value) in experiments {
        let toggle = create_labeled_toggle(
            &mut commands,
            exp_type.label(),
            SettingControl {
                table: "ai_experiments".to_string(),
                field: format!("{:?}", exp_type).to_lowercase(),
                control_type: ControlType::Toggle,
            },
            default_value,
            y_offset,
        );
        
        experiment_toggles.insert(exp_type, toggle);
        commands.entity(panel_root).add_child(toggle);
        
        y_offset += 45.0;
    }
    
    y_offset += 30.0;
    
    // ═══════════════════════════════════════════════════════════
    // HELP BUTTONS (bottom)
    // ═══════════════════════════════════════════════════════════
    
    let help_manual_button = commands.spawn((
        AIHelpButton { button_type: AIHelpButtonType::OpenHelpManual },
        UiLayout::window()
            .size((Ab(180.0), Ab(40.0)))
            .pos((Rl(20.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.25, 0.25, 0.28, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Text::new("Open AI Help Manual"),
        Pickable::default(),
        Interaction::None,
    )).id();
    
    let discover_ai_button = commands.spawn((
        AIHelpButton { button_type: AIHelpButtonType::DiscoverRaycastAI },
        UiLayout::window()
            .size((Ab(180.0), Ab(40.0)))
            .pos((Rl(60.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Text::new("Discover Action Items AI"),
        Pickable::default(),
        Interaction::None,
    )).id();
    
    commands.entity(panel_root).push_children(&[
        help_manual_button,
        discover_ai_button,
    ]);
    
    // Store entities in resource
    commands.insert_resource(AIPanelEntities {
        panel_root,
        status_badges,
        master_toggle: toggle_container,
        quick_ai_trigger_dropdown: trigger_dropdown,
        show_hint_toggle,
        quick_ai_model_dropdown,
        web_search_toggle,
        default_action_dropdown,
        chat_hotkey_recorder,
        start_new_chat_dropdown,
        new_chat_model_dropdown,
        chat_text_size_small,
        chat_text_size_large,
        commands_model_dropdown,
        show_tool_info_toggle,
        reset_confirmations_button,
        mcp_manage_servers_button,
        mcp_idle_time_dropdown,
        mcp_auto_confirm_toggle,
        ollama_host_input,
        ollama_sync_button,
        ollama_install_input,
        ollama_status_text,
        browser_ext_status_text,
        experiment_toggles,
        help_manual_button,
        discover_ai_button,
    });
    
    info!("✅ Pre-allocated AI panel UI entities");
}

// Helper functions

fn create_section(
    commands: &mut Commands,
    title: &str,
    description: &str,
    y_offset: f32,
) -> Entity {
    let section = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(100.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Name::new(format!("Section_{}", title)),
    )).id();
    
    let title_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(25.0)))
            .pos((Rl(0.0), Ab(0.0)))
            .pack(),
        Text::new(title),
        UiTextSize::from(Em(1.1)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let desc_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(40.0)))
            .pos((Rl(0.0), Ab(28.0)))
            .pack(),
        Text::new(description),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.7, 0.7, 0.75, 1.0)),
    )).id();
    
    commands.entity(section).push_children(&[title_entity, desc_entity]);
    section
}

fn create_separator(commands: &mut Commands, y_offset: f32) -> Entity {
    commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(1.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
    )).id()
}

fn create_labeled_dropdown(
    commands: &mut Commands,
    label: &str,
    control: SettingControl,
    default_value: &str,
    y_offset: f32,
) -> (Entity, Entity) {
    let label_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new(label),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let dropdown = spawn_dropdown_menu(commands, control, default_value.to_string());
    
    let dropdown_container = commands.spawn((
        UiLayout::window()
            .size((Rl(50.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset - 5.0)))
            .pack(),
    )).id();
    
    commands.entity(dropdown_container).add_child(dropdown);
    
    (label_entity, dropdown_container)
}

fn create_labeled_toggle(
    commands: &mut Commands,
    label: &str,
    control: SettingControl,
    default_value: bool,
    y_offset: f32,
) -> Entity {
    let container = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(35.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
    )).id();
    
    let label_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(70.0), Ab(30.0)))
            .pos((Rl(0.0), Ab(5.0)))
            .pack(),
        Text::new(label),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let toggle = spawn_toggle_switch(commands, control, default_value);
    
    let toggle_container = commands.spawn((
        UiLayout::window()
            .size((Ab(44.0), Ab(24.0)))
            .pos((Rl(95.0), Ab(10.0)))
            .anchor(Anchor::TopRight)
            .pack(),
    )).id();
    
    commands.entity(toggle_container).add_child(toggle);
    commands.entity(container).push_children(&[label_entity, toggle_container]);
    
    container
}

fn create_labeled_text_input(
    commands: &mut Commands,
    label: &str,
    control: SettingControl,
    placeholder: &str,
    y_offset: f32,
) -> (Entity, Entity) {
    let label_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new(label),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let input = spawn_text_input(commands, control, placeholder.to_string());
    
    let input_container = commands.spawn((
        UiLayout::window()
            .size((Rl(50.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset - 5.0)))
            .pack(),
    )).id();
    
    commands.entity(input_container).add_child(input);
    
    (label_entity, input_container)
}
```

**Note**: Due to the extensive size of this implementation, the remaining systems (load, save, event handlers) follow the same patterns as TASK7.1-GENERAL.md. The complete systems would include:

- `load_ai_settings()` - Load all AI settings from database when panel visible
- `populate_ai_controls_from_database()` - Update control states from database
- `save_ai_control_changes()` - Save changes to database
- `handle_mcp_manage_servers_button()` - Open MCP server management
- `handle_reset_tool_confirmations()` - Reset tool confirmation settings
- `handle_ollama_sync()` - Sync Ollama models
- `handle_ollama_install()` - Install new Ollama model
- `handle_ai_help_buttons()` - Open help resources
- `update_experiment_toggles()` - Manage experimental features

---

## Plugin Definition

```rust
pub struct AIPanelPlugin;

impl Plugin for AIPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AIModelRegistry>()
            .add_event::<MCPManageServersRequested>()
            .add_event::<ResetToolConfirmationsRequested>()
            .add_event::<OllamaSyncRequested>()
            .add_event::<OllamaModelInstallRequested>()
            .add_event::<AIHelpRequested>()
            .add_systems(Startup, setup_ai_panel)
            .add_systems(Update, (
                load_ai_settings,
                populate_ai_controls_from_database,
                save_ai_control_changes,
                handle_mcp_manage_servers_button,
                handle_reset_tool_confirmations,
                handle_ollama_sync,
                handle_ollama_install,
                handle_ai_help_buttons,
                update_experiment_toggles,
            ).chain());
    }
}
```

---

## Acceptance Criteria

1. ✅ All UI sections render correctly (Quick AI, Chat, Commands, MCP, Ollama, Experiments)
2. ✅ Settings load from database when panel visible
3. ✅ All controls save to appropriate database tables
4. ✅ Master toggle enables/disables all AI features
5. ✅ Model dropdowns populate from `AIModelRegistry`
6. ✅ Hotkey recorder works for AI Chat
7. ✅ MCP manage servers button triggers action
8. ✅ Ollama sync and install functions work
9. ✅ Experiment toggles update `ai_experiments` table
10. ✅ Help buttons open appropriate resources
11. ✅ All event-driven database I/O (no direct access)
12. ✅ Performance targets met (load < 50ms, interactions < 16ms)
13. ✅ NO STUBS in implementation
14. ✅ Tests pass with 100% success

---

## Estimated Time Breakdown

- UI setup and entity pre-allocation: 2 hours
- Database integration (8 tables): 2 hours
- Control behavior and event handlers: 2 hours
- MCP and Ollama integration: 1 hour
- Testing and polish: 1 hour

**Total: 5-7 hours**

**Ready for code review** ✅
