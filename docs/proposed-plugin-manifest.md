# Proposed Plugin Manifest Structure

Based on the Raycast model, here's a proposed manifest structure for Action Items plugins:

## Manifest Format (plugin.toml)

```toml
[plugin]
id = "git-helper"
name = "Git Helper"
description = "Git workflow automation and utilities"
version = "1.0.0"
author = "username"
license = "MIT"
icon = "icon.png"
homepage = "https://example.com"
repository = "https://github.com/user/plugin"

# Categories from Raycast model
categories = ["Developer Tools", "Productivity"]

# Plugin type - native (Rust) or wasm
type = "native" # or "wasm"

# For native plugins
[plugin.native]
library = "libgit_helper.dylib"
# Platform-specific libraries
macos = "libgit_helper.dylib"
linux = "libgit_helper.so"
windows = "git_helper.dll"

# For WASM plugins
[plugin.wasm]
module = "git_helper.wasm"

# Commands - similar to Raycast but adapted for our needs
[[commands]]
id = "git-status"
name = "Git Status"
description = "Show git repository status"
keywords = ["git", "status", "vcs"]
# UI mode: search (default launcher), detail, form, no-view
mode = "search"
# Icon for this specific command
icon = "git-status.png"

[[commands]]
id = "git-commit"
name = "Git Commit"
description = "Create a git commit with AI-generated message"
keywords = ["git", "commit", "ai"]
mode = "form"

[[commands]]
id = "git-branch"
name = "Switch Git Branch"
description = "Quick switch between git branches"
keywords = ["git", "branch", "checkout"]
mode = "search"

# Capabilities/Permissions (more explicit than Raycast)
[capabilities]
filesystem = ["read", "write"]
network = true
clipboard = true
notifications = true
shell_execute = true
# Specific paths the plugin can access
allowed_paths = ["~/.gitconfig", "**/.git"]

# Preferences/Configuration (user-configurable)
[[preferences]]
name = "default_branch"
title = "Default Branch"
description = "Default branch name for new repositories"
type = "text"
default = "main"
required = false

[[preferences]]
name = "commit_style"
title = "Commit Message Style"
description = "Preferred commit message format"
type = "dropdown"
default = "conventional"
options = [
    { value = "conventional", title = "Conventional Commits" },
    { value = "simple", title = "Simple" },
    { value = "detailed", title = "Detailed with body" }
]

[[preferences]]
name = "auto_stage"
title = "Auto Stage Files"
description = "Automatically stage all changes before commit"
type = "checkbox"
default = false

# Plugin-specific metadata
[metadata]
min_launcher_version = "0.1.0"
max_launcher_version = "1.0.0"

# Search index hints
[search]
# Additional keywords for finding this plugin
keywords = ["version control", "vcs", "source control"]
# Weight for search ranking
weight = 1.0
```

## Key Improvements Over Current System

1. **Declarative Commands**: Commands are defined in the manifest, not in code
2. **Explicit Capabilities**: Clear permission model
3. **User Preferences**: Built-in preference system
4. **Search Optimization**: Keywords and search hints
5. **Platform Support**: Multi-platform binary support
6. **Versioning**: Clear version compatibility

## Comparison with Current Manifest

| Feature | Current (Raycast-style) | Proposed |
|---------|------------------------|----------|
| Format | JSON (package.json) | TOML |
| Commands | In manifest | In manifest |
| Permissions | Implicit | Explicit capabilities |
| Preferences | Basic | Rich types with UI hints |
| Search | Basic keywords | Advanced with weights |
| Icons | Single icon | Per-command icons |

## Migration Path

To migrate from our current system:

1. Convert existing PluginManifest to new format
2. Extract commands from code to manifest
3. Define explicit capabilities
4. Add search keywords and metadata
5. Support both formats during transition

## Example Usage in Code

```rust
// Plugin trait simplified - commands come from manifest
#[async_trait]
pub trait ActionItemsPlugin: Send + Sync {
    // Called when a command is executed
    async fn execute_command(&self, command_id: &str, context: PluginContext) -> Result<CommandResult>;
    
    // Lifecycle hooks
    async fn activate(&self) -> Result<()>;
    async fn deactivate(&self) -> Result<()>;
}

// Command results can specify UI updates
pub enum CommandResult {
    // Show search results
    Search(Vec<SearchResult>),
    // Show detail view
    Detail(DetailView),
    // Show form
    Form(FormDefinition),
    // No UI, just an action
    NoView,
    // Update existing view
    Update(ViewUpdate),
}
```

This approach combines the best of Raycast's declarative model with our Rust-based plugin system.