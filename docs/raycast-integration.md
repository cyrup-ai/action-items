# Raycast Extension Integration

This document describes the Raycast extension integration for Action Items launcher.

## Architecture Overview

The integration allows running existing Raycast extensions seamlessly through our WASM plugin system using Deno as the JavaScript runtime.

### Components

1. **RaycastLoader** (`core/src/raycast_loader.rs`)
   - Clones the Raycast extensions repository
   - Manages automatic updates every 24 hours
   - Lists available extensions
   - Parses `package.json` files

2. **RaycastAdapter** (`core/src/raycast_adapter.rs`)
   - Converts Raycast manifests to our plugin format
   - Provides `@raycast/api` shim for compatibility
   - Maps Raycast UI components to our search results
   - Handles host function bridging

3. **RaycastPlugin** (`core/src/raycast_plugin.rs`)
   - Wrapper that makes Raycast extensions behave like native plugins
   - Implements our `NativePlugin` trait
   - Handles search, command execution, and actions

4. **RaycastDiscovery** (`core/src/raycast_discovery.rs`)
   - Discovers and loads Raycast extensions at startup
   - Registers them with our plugin system
   - Manages periodic updates

5. **DenoRuntime** (`core/src/deno_runtime.rs`)
   - Placeholder for Deno WASM integration
   - Will execute TypeScript/JavaScript code
   - Provides host functions for system access

## How It Works

1. **Initialization**:
   - On first run, clones `https://github.com/raycast/extensions.git`
   - Stores in `~/.config/action-items/raycast/extensions/`
   - Creates sync state file to track updates

2. **Discovery**:
   - Scans the extensions directory for valid extensions
   - Parses each extension's `package.json`
   - Creates a `RaycastPlugin` wrapper for each
   - Registers with our plugin system

3. **Search Integration**:
   - Raycast extensions appear alongside native plugins
   - Search queries are passed to the extension
   - Results are converted to our format

4. **Execution** (TODO):
   - Load Deno runtime in WASM
   - Load `@raycast/api` shim
   - Execute extension's TypeScript code
   - Bridge results back to our system

## @raycast/api Shim

The shim maps Raycast's React-based API to our plugin interface:

```javascript
// Raycast components
export function List({ searchBarPlaceholder, children }) {
    return {
        type: 'list',
        placeholder: searchBarPlaceholder,
        items: items.map(child => child.props)
    };
}

// Maps to our SearchResult
{
    id: string,
    title: string,
    subtitle: string,
    icon: Icon,
    score: number,
    action_id: string,
    metadata: any
}
```

## Auto-Update Mechanism

- Checks every hour if 24 hours have passed since last sync
- Runs `git pull --rebase` to get latest extensions
- Updates sync state with commit hash and extension count
- Non-blocking - failures don't affect existing extensions

## Current Status

✅ Implemented:
- Extension repository cloning and updates
- Extension discovery and parsing
- Plugin manifest conversion
- Basic search result mocking
- Integration with existing plugin system

🚧 TODO:
- Actual Deno WASM runtime integration
- Full `@raycast/api` implementation
- TypeScript compilation pipeline
- Proper command/action execution
- React component lifecycle handling

## Usage

Once fully implemented, Raycast extensions will:
1. Appear automatically in search results
2. Execute in a sandboxed Deno environment
3. Have full access to Raycast API features
4. Update automatically from the official repository

No manual installation required - it's completely transparent to the user!