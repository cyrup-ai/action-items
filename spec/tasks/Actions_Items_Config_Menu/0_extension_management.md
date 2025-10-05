# Actions_Items_Config_Menu Task 0: Extension Management Data Models

## Task Overview
Implement comprehensive extension and command hierarchy data structures for the Actions Items Config menu, supporting hierarchical extension → command relationships, metadata management, and dynamic loading.

## Implementation Requirements

### Core Data Models
```rust
// Extension management system
#[derive(Resource, Reflect, Debug)]
pub struct ExtensionManagementResource {
    pub installed_extensions: HashMap<ExtensionId, Extension>,
    pub extension_hierarchy: ExtensionHierarchy,
    pub command_registry: CommandRegistry,
    pub extension_metadata: HashMap<ExtensionId, ExtensionMetadata>,
    pub loading_state: ExtensionLoadingState,
}

#[derive(Reflect, Debug, Clone, Hash, PartialEq, Eq)]
pub struct ExtensionId(pub String);

#[derive(Reflect, Debug, Clone)]
pub struct Extension {
    pub id: ExtensionId,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: Option<String>,
    pub commands: Vec<Command>,
    pub permissions: Vec<Permission>,
    pub status: ExtensionStatus,
    pub installation_path: PathBuf,
    pub configuration: ExtensionConfiguration,
}

#[derive(Reflect, Debug, Clone)]
pub enum ExtensionStatus {
    Active,
    Inactive,
    Loading,
    Error { message: String },
    UpdateAvailable { new_version: String },
    Disabled,
}

#[derive(Reflect, Debug, Clone)]
pub struct Command {
    pub id: CommandId,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub icon: Option<String>,
    pub hotkey: Option<Hotkey>,
    pub aliases: Vec<String>,
    pub category: Option<String>,
    pub execution_context: ExecutionContext,
    pub parameters: Vec<CommandParameter>,
}
```

### Command Registry System
```rust
// Command registry and hierarchy management
#[derive(Resource, Reflect, Debug)]
pub struct CommandRegistry {
    pub commands: HashMap<CommandId, Command>,
    pub command_index: CommandIndex,
    pub execution_history: Vec<CommandExecution>,
    pub favorite_commands: HashSet<CommandId>,
}

#[derive(Reflect, Debug, Clone, Hash, PartialEq, Eq)]
pub struct CommandId(pub String);

#[derive(Reflect, Debug)]
pub struct CommandIndex {
    pub by_name: HashMap<String, Vec<CommandId>>,
    pub by_keyword: HashMap<String, Vec<CommandId>>,
    pub by_category: HashMap<String, Vec<CommandId>>,
    pub by_extension: HashMap<ExtensionId, Vec<CommandId>>,
}

#[derive(Reflect, Debug, Clone)]
pub struct CommandExecution {
    pub command_id: CommandId,
    pub execution_time: DateTime<Utc>,
    pub execution_result: ExecutionResult,
    pub execution_duration: Duration,
}

#[derive(Reflect, Debug, Clone)]
pub enum ExecutionResult {
    Success,
    Warning { message: String },
    Error { message: String },
    Cancelled,
}
```

### Extension Hierarchy System  
```rust
// Hierarchical extension organization
#[derive(Resource, Reflect, Debug)]
pub struct ExtensionHierarchy {
    pub root_extensions: Vec<ExtensionId>,
    pub extension_tree: HashMap<ExtensionId, ExtensionNode>,
    pub collapsed_state: HashMap<ExtensionId, bool>,
    pub display_order: Vec<ExtensionId>,
}

#[derive(Reflect, Debug)]
pub struct ExtensionNode {
    pub extension_id: ExtensionId,
    pub parent: Option<ExtensionId>,
    pub children: Vec<ExtensionId>,
    pub commands: Vec<CommandId>,
    pub display_metadata: DisplayMetadata,
}

#[derive(Reflect, Debug)]
pub struct DisplayMetadata {
    pub display_name: String,
    pub sort_order: u32,
    pub color_theme: Option<String>,
    pub custom_icon: Option<String>,
    pub visibility: VisibilityState,
}

#[derive(Reflect, Debug)]
pub enum VisibilityState {
    Visible,
    Hidden,
    Conditional { condition: String },
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `ecs/hierarchy.rs` - Hierarchical entity relationships
- `reflection/reflection.rs` - Data serialization patterns
- `asset/asset_loading.rs` - Dynamic extension loading

### Implementation Pattern
```rust
// Based on hierarchy.rs for extension tree management
fn extension_hierarchy_system(
    parent_query: Query<&Children, With<ExtensionComponent>>,
    command_query: Query<&Command>,
    mut hierarchy_res: ResMut<ExtensionHierarchy>,
) {
    for children in &parent_query {
        for &child in children.iter() {
            if let Ok(command) = command_query.get(child) {
                // Update hierarchy structure
                update_extension_hierarchy(&mut hierarchy_res, command);
            }
        }
    }
}

// Based on asset_loading.rs for dynamic extension loading
fn extension_loading_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut extension_res: ResMut<ExtensionManagementResource>,
) {
    for (extension_id, extension) in &extension_res.installed_extensions {
        if extension.status == ExtensionStatus::Loading {
            let handle = asset_server.load(&extension.installation_path);
            commands.spawn(ExtensionLoadingTask {
                extension_id: extension_id.clone(),
                asset_handle: handle,
            });
        }
    }
}
```

## Extension Configuration
- Dynamic extension discovery and loading
- Configuration validation and schema enforcement
- Extension dependency resolution
- Version compatibility checking

## Performance Constraints
- **ZERO ALLOCATIONS** during command lookup operations
- Efficient hierarchical tree traversal
- Cached command index for fast searching
- Lazy loading of extension metadata

## Success Criteria
- Complete extension management data model implementation
- Efficient hierarchical command organization
- No unwrap()/expect() calls in production code
- Zero-allocation command registry operations
- Comprehensive extension metadata support

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for extension hierarchy logic
- Integration tests for command registry operations
- Performance tests for command lookup efficiency
- Validation tests for extension configuration schemas