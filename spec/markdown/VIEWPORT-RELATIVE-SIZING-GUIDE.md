# Viewport-Relative Sizing Guide for Action Items

## Overview

All Action Items launcher UI specifications have been updated to use **viewport-relative sizing** instead of fixed pixel dimensions. This ensures the interface scales responsively across different screen sizes while maintaining visual consistency.

## Bevy Viewport Units

### Available Units

Based on Bevy's layout system (from `bevy/examples/ui/`):

1. **`Val::Vw(n)`** - Percentage of viewport width
   - Example: `Val::Vw(60.0)` = 60% of screen width
   - Use for: Container widths, horizontal spacing

2. **`Val::Vh(n)`** - Percentage of viewport height  
   - Example: `Val::Vh(5.0)` = 5% of screen height
   - Use for: Container heights, vertical spacing, search bar height

3. **`Val::VMin(n)`** - Percentage of minimum viewport dimension
   - Example: `Val::VMin(1.0)` = 1% of smaller dimension (width or height)
   - Use for: Padding, margins, border radius, icon spacing
   - **Why VMin**: Ensures consistent spacing regardless of aspect ratio

4. **`Val::VMax(n)`** - Percentage of maximum viewport dimension
   - Less commonly used
   - Use for: Special cases where larger dimension matters

5. **`Val::Percent(n)`** - Percentage of parent container
   - Example: `Val::Percent(100.0)` = 100% of parent
   - Use for: Child elements within fixed containers

6. **`Val::Auto`** - Automatically determined by layout
   - Use for: Flexible dimensions, centered positioning

## Core Sizing Standards

### Container Dimensions
```rust
// Main launcher container
width: Val::Vw(60.0),           // 60% viewport width
max_width: Val::Vw(60.0),       // Constrain to viewport percentage
min_width: Val::Vw(35.0),       // Minimum 35% viewport width
height: Val::Auto,              // Auto height based on content
max_height: Val::Vh(60.0),      // Maximum 60% viewport height
```

### Search Bar
```rust
// Search input container
width: Val::Percent(100.0),     // Full width of parent
height: Val::Vh(5.0),           // 5% of viewport height
```

### Spacing System
```rust
// Padding: Use VMin for consistent spacing across aspect ratios
padding: UiRect::all(Val::VMin(1.0)),    // 1% of min viewport dimension

// Margins
margin: UiRect::top(Val::Vh(12.0)),      // 12% from top
margin: UiRect::right(Val::VMin(0.8)),   // Icon spacing

// Gaps between elements
row_gap: Val::VMin(0.6),                 // 0.6% min viewport dimension
```

### Border Radius
```rust
// Always use VMin for consistent rounded corners
BorderRadius::all(Val::VMin(1.2)),      // Container: 1.2% VMin
BorderRadius::all(Val::VMin(0.6)),      // Search bar: 0.6% VMin
```

### Typography Sizing
Typography should use responsive calculations in systems:
```rust
// In a responsive typography system
font_size: window.height() * 0.022,  // 2.2% of viewport height
```

Static font sizes in specs should be noted as requiring responsive calculations.

## Responsive Patterns

### Portrait vs Landscape Mode
```rust
fn responsive_container_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut container: Query<&mut Node, With<LauncherContainer>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok(mut node) = container.get_single_mut() else { return };
    
    if window.width() < window.height() {
        // Portrait mode: use more width
        node.width = Val::Vw(90.0);
    } else {
        // Landscape mode: standard width
        node.width = Val::Vw(60.0);
    }
}
```

### Component-Based Constraints
```rust
#[derive(Component, Debug, Clone)]
pub struct CompactContainer {
    base_width_vw: f32,        // 60.0 (60% viewport width)
    min_width_vw: f32,         // 35.0 (35% viewport width)  
    portrait_width_vw: f32,    // 90.0 (90% for portrait mode)
}
```

## Migration Summary

### Old Pattern (Fixed Pixels)
```rust
❌ width: Val::Px(600.0),
❌ height: Val::Px(40.0),
❌ padding: UiRect::all(Val::Px(12.0)),
❌ border_radius: Val::Px(8.0),
```

### New Pattern (Viewport-Relative)
```rust
✅ width: Val::Vw(60.0),
✅ height: Val::Vh(5.0),
✅ padding: UiRect::all(Val::VMin(1.0)),
✅ border_radius: Val::VMin(0.6),
```

## Updated Specifications

All specifications have been updated with viewport-relative sizing:

### Primary Specs
- ✅ `01-container-layout-LAUNCHER_UI.md`
- ✅ `01-container-layout-system.md`
- ✅ `03-compact-search-design.md`
- ✅ `04-window-LAUNCHER_UI.md`
- ✅ `05-window-sizing-strategy.md`
- ✅ `README.md`
- ✅ `BEVY-FLEX-LAYOUT-CORRECTION.md`

### Menu Specs
- ✅ `Main_Menu.md`
- ✅ `Main_Menu_2.md`
- ✅ `About_Menu.md`
- ✅ `Account_Menu.md`

## Key Benefits

1. **Screen Size Independence**: Interface scales naturally across different monitor sizes
2. **Aspect Ratio Handling**: VMin ensures consistent spacing on all aspect ratios
3. **Future-Proof**: Works with any screen resolution (4K, 1080p, mobile, etc.)
4. **Consistent Proportions**: Maintains visual hierarchy regardless of display size
5. **Responsive by Default**: No media queries or breakpoints needed

## Implementation Notes

### Shadow System
Shadows still use `Val::Px(0.0)` for offsets where appropriate, but blur and spread should use VMin:
```rust
BoxShadow(vec![ShadowStyle {
    color: Color::BLACK.with_alpha(0.3),
    x_offset: Val::Px(0.0),           // Zero offset is fine
    y_offset: Val::VMin(0.6),         // Responsive offset
    blur_radius: Val::VMin(1.8),      // Responsive blur
    spread_radius: Val::Px(0.0),      // Zero spread is fine
}])
```

### When to Use Each Unit

| Use Case | Unit | Example |
|----------|------|---------|
| Container width | `Val::Vw` | `Val::Vw(60.0)` |
| Container height | `Val::Vh` | `Val::Vh(5.0)` |
| Padding/Margin | `Val::VMin` | `Val::VMin(1.0)` |
| Border radius | `Val::VMin` | `Val::VMin(0.6)` |
| Icon spacing | `Val::VMin` | `Val::VMin(0.8)` |
| Gap spacing | `Val::VMin` | `Val::VMin(0.3)` |
| Child of container | `Val::Percent` | `Val::Percent(100.0)` |
| Flexible dimension | `Val::Auto` | `Val::Auto` |

## Testing Recommendations

1. **Test on Multiple Resolutions**: 1920x1080, 2560x1440, 3840x2160
2. **Test Portrait Mode**: Rotate display or use portrait monitor
3. **Test Edge Cases**: Very small (1024x768) and very large (5K+) displays
4. **Verify Spacing**: Ensure VMin-based spacing looks consistent across all sizes
5. **Check Font Scaling**: Ensure typography remains readable at all scales

## Bevy Documentation References

All patterns follow official Bevy examples:
- `docs/bevy/examples/ui/flex_layout.rs` - Percentage-based layouts
- `docs/bevy/examples/ui/ui_scaling.rs` - Responsive scaling
- `docs/bevy/examples/ui/viewport_debug.rs` - Viewport units (Vw, Vh, VMin, VMax)
- `docs/bevy/examples/ui/size_constraints.rs` - Min/max constraints

## Success Criteria

✅ Zero `Val::Px` references for dimensions (except zero offsets)  
✅ All widths/heights use viewport units  
✅ All spacing uses VMin for consistency  
✅ Responsive systems handle portrait/landscape  
✅ Border radius scales with screen size  
✅ Typography notes responsive calculation needs
