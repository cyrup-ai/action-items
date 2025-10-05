# Actions_Items_Config_Menu Task 2: Hierarchical Display System

## Task Overview
Implement collapsible extension → command tree display system with interactive expansion/collapse, visual hierarchy indicators, and efficient rendering of large command trees.

## Implementation Requirements

### Core Components
```rust
// Hierarchical display system
#[derive(Component, Reflect, Debug)]
pub struct HierarchicalDisplayComponent {
    pub tree_root_entity: Entity,
    pub expansion_state: HashMap<ExtensionId, bool>,
    pub selected_item: Option<SelectableItem>,
    pub scroll_position: f32,
    pub display_mode: DisplayMode,
}

#[derive(Reflect, Debug)]
pub enum SelectableItem {
    Extension(ExtensionId),
    Command(CommandId),
}

#[derive(Reflect, Debug)]
pub enum DisplayMode {
    Tree,
    List,
    Grid,
    Compact,
}

#[derive(Component, Reflect, Debug)]
pub struct TreeNodeComponent {
    pub item_type: TreeNodeType,
    pub depth_level: u32,
    pub is_expanded: bool,
    pub has_children: bool,
    pub parent_entity: Option<Entity>,
    pub child_entities: Vec<Entity>,
}

#[derive(Reflect, Debug)]
pub enum TreeNodeType {
    ExtensionNode {
        extension_id: ExtensionId,
        command_count: u32,
    },
    CommandNode {
        command_id: CommandId,
        parent_extension: ExtensionId,
    },
    CategoryNode {
        category_name: String,
        item_count: u32,
    },
}
```

### Visual Hierarchy System
```rust
// Visual hierarchy rendering
#[derive(Resource, Reflect, Debug)]
pub struct HierarchyVisualizationResource {
    pub indent_size: f32,
    pub expansion_icons: ExpansionIcons,
    pub tree_colors: TreeColorScheme,
    pub animation_settings: HierarchyAnimations,
}

#[derive(Reflect, Debug)]
pub struct ExpansionIcons {
    pub collapsed_icon: String,
    pub expanded_icon: String,
    pub leaf_icon: String,
    pub loading_icon: String,
}

#[derive(Reflect, Debug)]
pub struct TreeColorScheme {
    pub extension_color: Color,
    pub command_color: Color,
    pub category_color: Color,
    pub selected_color: Color,
    pub hover_color: Color,
    pub disabled_color: Color,
}

pub fn hierarchical_display_system(
    mut commands: Commands,
    mut display_query: Query<&mut HierarchicalDisplayComponent>,
    tree_node_query: Query<&TreeNodeComponent>,
    extension_res: Res<ExtensionManagementResource>,
) {
    for mut display in &mut display_query {
        if extension_res.is_changed() {
            rebuild_hierarchy_ui(&mut commands, &mut display, &extension_res);
        }
    }
}
```

### Interactive Tree Operations
```rust
// Tree expansion and interaction
#[derive(Event)]
pub struct TreeInteractionEvent {
    pub event_type: TreeInteractionType,
    pub target_entity: Entity,
    pub modifier_keys: ModifierKeys,
}

#[derive(Reflect, Debug)]
pub enum TreeInteractionType {
    Expand,
    Collapse,
    Toggle,
    Select,
    DoubleClick,
    RightClick,
}

pub fn tree_interaction_system(
    mut interaction_events: EventReader<TreeInteractionEvent>,
    mut display_query: Query<&mut HierarchicalDisplayComponent>,
    mut tree_node_query: Query<&mut TreeNodeComponent>,
) {
    for event in interaction_events.read() {
        match event.event_type {
            TreeInteractionType::Toggle => {
                toggle_tree_node(&mut tree_node_query, event.target_entity);
            }
            TreeInteractionType::Select => {
                update_selection(&mut display_query, event.target_entity);
            }
            _ => {} // Handle other interaction types
        }
    }
}

fn toggle_tree_node(
    tree_node_query: &mut Query<&mut TreeNodeComponent>,
    target_entity: Entity,
) {
    if let Ok(mut node) = tree_node_query.get_mut(target_entity) {
        if node.has_children {
            node.is_expanded = !node.is_expanded;
            // Update child visibility with zero allocations
        }
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `ui/ui_texture_atlas.rs` - UI texture management for icons
- `ui/scroll_area.rs` - Scrollable tree view
- `ui/ui_stack.rs` - Layered UI management

### Implementation Pattern
```rust
// Based on ui_stack.rs for hierarchical UI layout
fn hierarchy_ui_layout_system(
    mut commands: Commands,
    hierarchy_query: Query<&HierarchicalDisplayComponent, Changed<HierarchicalDisplayComponent>>,
) {
    for hierarchy in &hierarchy_query {
        let ui_stack = UiStack::new();
        
        for extension_id in &hierarchy.expansion_state {
            let extension_ui = create_extension_ui_node(extension_id);
            ui_stack.push(extension_ui);
            
            if hierarchy.expansion_state.get(extension_id).unwrap_or(&false) {
                let commands_ui = create_commands_ui_nodes(extension_id);
                for command_ui in commands_ui {
                    ui_stack.push_child(command_ui);
                }
            }
        }
    }
}

// Based on scroll_area.rs for scrollable tree
fn scrollable_tree_system(
    mut scroll_events: EventReader<MouseWheel>,
    mut hierarchy_query: Query<&mut HierarchicalDisplayComponent>,
) {
    for scroll_event in scroll_events.read() {
        for mut hierarchy in &mut hierarchy_query {
            hierarchy.scroll_position += scroll_event.y * 20.0;
            hierarchy.scroll_position = hierarchy.scroll_position.clamp(0.0, calculate_max_scroll(&hierarchy));
        }
    }
}
```

## Tree Virtualization
- Efficient rendering of large command trees
- Virtual scrolling for performance optimization
- Lazy loading of tree nodes
- Memory-efficient tree state management

## Performance Constraints
- **ZERO ALLOCATIONS** during tree expansion/collapse
- Efficient tree traversal algorithms
- Optimized rendering for large hierarchies
- Smooth animation during state transitions

## Success Criteria
- Complete hierarchical display implementation
- Smooth interactive tree operations
- No unwrap()/expect() calls in production code
- Zero-allocation tree state management
- Responsive performance with large extension sets

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for tree node operations
- Integration tests for hierarchy rendering
- Performance tests for large tree handling
- UI interaction tests for expansion/collapse behavior