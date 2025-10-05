# GitHub Search WASM Plugin

This is a complete example of a WASM plugin for Modern Launcher that demonstrates:
- Async API calls using the callback pattern
- Configuration/preferences
- Search history
- Multiple action types
- Error handling

## Features

- **Search GitHub repositories** with real-time results
- **Configuration options**:
  - GitHub Personal Access Token (for private repos and higher rate limits)
  - Include/exclude forks
  - Include/exclude archived repos
  - Results per page (10-100)
  - Sort by: best match, stars, forks, or recently updated
- **Search history** - Shows recent searches when query is empty
- **Actions for each result**:
  - Open in browser
  - Copy repository URL
  - Copy clone command
  - Open in VS Code

## Building

```bash
# Install dependencies
npm install

# Build the plugin
npm run build

# Output will be in dist/github-search.wasm
```

## How It Works

### 1. Manifest
The plugin exports a `plugin_manifest()` function that returns:
- Plugin metadata (id, name, version, etc.)
- Capabilities (what the plugin can do)
- Preferences schema (configuration options)

### 2. Search Flow
When the user types a query:
1. `plugin_search()` is called with the query and context
2. Plugin builds GitHub API request parameters
3. Calls the async HTTP host function with a callback
4. Returns a loading state immediately
5. When HTTP completes, `github_search_callback()` is invoked
6. Callback processes the response and returns final results

### 3. Async Pattern
```typescript
// 1. Make async request
Host.outputString(JSON.stringify({
  request: "http",
  url: "https://api.github.com/search/repositories",
  request_id: "req_123",
  callback_fn: "github_search_callback"
}));

// 2. Return immediately with loading state
return JSON.stringify({ is_loading: true });

// 3. Callback invoked when complete
export function github_search_callback(result: string) {
  // Process and return final results
}
```

### 4. Actions
Each search result can have actions:
- `open_url` - Opens URL in default browser
- `copy` - Copies text to clipboard (uses async clipboard API)
- `search` - Re-runs search with a specific query

### 5. Configuration
The plugin reads preferences from `context.preferences`:
```typescript
const config = {
  github_token: context.preferences?.github_token,
  include_forks: context.preferences?.include_forks || false,
  // ...
};
```

## Plugin Interface

### Exported Functions

#### `plugin_manifest(): string`
Returns the plugin manifest as JSON.

#### `plugin_search(input: string): string`
Main search function. Input is JSON with:
```json
{
  "query": "search terms",
  "context": {
    "preferences": { /* user preferences */ }
  }
}
```

#### `plugin_action(input: string): string`
Executes an action. Input is JSON with:
```json
{
  "action_id": "open_browser",
  "context": { /* ... */ },
  "metadata": { "url": "https://..." }
}
```

#### `github_search_callback(result: string): string`
Callback for HTTP responses. Input is JSON with:
```json
{
  "request_id": "req_123",
  "result": {
    "status": 200,
    "body": "{ /* GitHub API response */ }"
  }
}
```

#### `clipboard_callback(result: string): string`
Callback for clipboard operations.

#### `plugin_validate_config(input: string): string`
Validates user configuration before saving.

## Testing

To test this plugin:

1. Build it: `npm run build`
2. Copy `dist/github-search.wasm` to your launcher's `plugins` directory
3. Restart the launcher
4. Type to search GitHub repositories

## Extending

This plugin demonstrates patterns you can use for:
- Any REST API integration
- Async operations with callbacks
- User preferences
- Search history
- Error handling
- Multiple action types

You can adapt this for other services like:
- GitLab/Bitbucket search
- NPM package search
- Documentation search
- Any API that returns searchable results