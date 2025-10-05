# Raycast Plugin Model Research

## Overview

Raycast uses a TypeScript/React-based plugin system where extensions are distributed through the Raycast Store. Their model is significantly different from our current Rust-based approach.

## Key Components

### 1. Extension Manifest (package.json)

Raycast extensions use a `package.json` with a specific schema:

```json
{
  "$schema": "https://www.raycast.com/schemas/extension.json",
  "name": "extension-id",
  "title": "Human Readable Title",
  "description": "What the extension does",
  "icon": "icon.png",
  "author": "username",
  "categories": ["Productivity", "Developer Tools"],
  "license": "MIT",
  "commands": [
    {
      "name": "command-id",
      "title": "Command Title",
      "subtitle": "Optional subtitle",
      "description": "What this command does",
      "mode": "view" // or "no-view"
    }
  ],
  "preferences": [...],
  "dependencies": {
    "@raycast/api": "^1.100.0"
  }
}
```

### 2. Command Types

Commands can have different modes:
- **view**: Opens a UI (React-based)
- **no-view**: Runs in background without UI
- **menu-bar**: Shows in menu bar

### 3. API Architecture

Raycast provides:
- **@raycast/api**: Main API package with React components
- **@raycast/utils**: Utility functions
- Built-in UI components (List, Detail, Form, etc.)
- Hooks for state management
- Native integrations (clipboard, notifications, etc.)

### 4. Development Workflow

```bash
# Raycast CLI commands
ray build      # Build the extension
ray develop    # Start development mode
ray lint       # Lint the code
ray publish    # Publish to store
```

### 5. Key Differences from Our Model

| Aspect | Raycast | Our Current Model |
|--------|---------|-------------------|
| Language | TypeScript/React | Rust (native), WASM |
| UI | React Components | Custom UI system |
| Distribution | Raycast Store | Local plugins |
| Manifest | package.json | Custom manifest |
| API | NPM package | FFI/WASM interface |
| Commands | Declarative in manifest | Programmatic |
| Development | ray CLI | cargo |

## Raycast Plugin Capabilities

### 1. User Interface
- List views with search
- Detail views
- Forms with validation
- Actions and keyboard shortcuts
- Custom icons and metadata

### 2. System Integration
- File system access
- Clipboard operations
- Notifications
- HTTP requests
- Local storage/cache
- OAuth authentication

### 3. AI Integration (Recent Addition)
Raycast now includes AI capabilities:
- Tools system for LLM integration
- Evaluation framework for testing
- Direct AI assistance in extensions

## Example: Simple Extension Structure

```
extension/
├── package.json
├── src/
│   ├── index.tsx      // Main command
│   ├── list.tsx       // List view command
│   └── utils.ts       // Helper functions
├── assets/
│   └── icon.png
└── README.md
```

## Recommendations for Our Plugin System

Based on Raycast's model, we should consider:

1. **Simplified Manifest**: Adopt a cleaner manifest structure similar to package.json
2. **Declarative Commands**: Define commands in the manifest rather than programmatically
3. **Standard UI Components**: Provide a set of pre-built UI components
4. **Better TypeScript Support**: If we support WASM plugins, provide TypeScript definitions
5. **CLI Tooling**: Create developer tools for building/testing plugins
6. **Plugin Store**: Consider a distribution mechanism for plugins

## Next Steps

1. Define our plugin manifest structure
2. Create standard plugin interfaces
3. Build developer tooling
4. Document the plugin API
5. Create example plugins