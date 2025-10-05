# Bevy API Misuse Mapping and Fixes

## 1. ChildBuilder Import Issues
**WRONG**: `use bevy::hierarchy::ChildBuilder;` - hierarchy module doesn't exist
**CORRECT**: ChildBuilder is available through `bevy::prelude::*`

## 2. Function Parameter Types
**WRONG**: Explicit `&mut ChildBuilder` parameters
**CORRECT**: Let Rust infer the type in `with_children` closures

## 3. spawn() Method Usage
**ISSUE**: Calling spawn on EntityCommands instead of ChildBuilder
**CORRECT**: spawn() should be called on the builder parameter in with_children closures

## 4. macOS API Issues
**WRONG**: 
- `ns_window` field on AppKitWindowHandle
- `BorderlessWindowMask`, `NonactivatingPanelMask` on NSWindowStyleMask

**CORRECT**: Need to check current macOS API patterns

## 5. HotKey Privacy Issues
**WRONG**: `struct HotKey is private`
**CORRECT**: Use public API for hotkey creation

## 6. WindowMode Usage
**WRONG**: `mismatched types: expected WindowMode, found enum constructor`
**CORRECT**: Use proper WindowMode enum values