import { Host, Config, Memory } from "@extism/js-pdk";

// Plugin manifest
export function plugin_manifest() {
  return JSON.stringify({
    id: "com.github.search",
    name: "GitHub Search",
    version: "1.0.0",
    author: "Modern Launcher Example",
    description: "Search GitHub repositories",
    icon: "github",
    capabilities: {
      search: true,
      actions: true,
      background_refresh: false,
      configuration: true
    },
    preferences: [
      {
        name: "github_token",
        title: "GitHub Personal Access Token",
        description: "Required for private repos and higher rate limits",
        type: "password",
        required: false
      },
      {
        name: "include_forks",
        title: "Include Forks",
        description: "Include forked repositories in search results",
        type: "checkbox",
        default: false
      },
      {
        name: "include_archived",
        title: "Include Archived",
        description: "Include archived repositories in search results",
        type: "checkbox",
        default: false
      },
      {
        name: "per_page",
        title: "Results Per Page",
        description: "Number of results to show (10-100)",
        type: "textfield",
        default: "30"
      },
      {
        name: "sort_by",
        title: "Sort By",
        description: "How to sort search results",
        type: "dropdown",
        default: "best-match",
        options: [
          { value: "best-match", label: "Best Match" },
          { value: "stars", label: "Stars" },
          { value: "forks", label: "Forks" },
          { value: "updated", label: "Recently Updated" }
        ]
      }
    ]
  });
}

// Configuration interface
interface PluginConfig {
  github_token?: string;
  include_forks: boolean;
  include_archived: boolean;
  per_page: number;
  sort_by: string;
}

// Search result types matching our plugin interface
interface SearchResult {
  id: string;
  title: string;
  subtitle?: string;
  icon?: string;
  score: number;
  actions?: Action[];
  metadata?: any;
}

interface Action {
  id: string;
  title: string;
  type: string;
  metadata?: any;
}

// GitHub API types
interface GitHubRepo {
  id: number;
  name: string;
  full_name: string;
  description: string | null;
  html_url: string;
  stargazers_count: number;
  forks_count: number;
  language: string | null;
  updated_at: string;
  archived: boolean;
  fork: boolean;
  private: boolean;
  owner: {
    login: string;
    avatar_url: string;
  };
}

interface GitHubSearchResponse {
  total_count: number;
  items: GitHubRepo[];
}

// State management
interface SearchState {
  lastQuery: string;
  cachedResults: SearchResult[];
  searchHistory: string[];
}

let state: SearchState = {
  lastQuery: "",
  cachedResults: [],
  searchHistory: []
};

// Helper to generate unique request IDs
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

// Main search function
export function plugin_search(input: string) {
  const { query, context } = JSON.parse(input);
  
  // If empty query, show search history
  if (!query || query.trim() === "") {
    const historyResults = state.searchHistory.map((q, idx) => ({
      id: `history_${idx}`,
      title: q,
      subtitle: "Search history",
      icon: "history",
      score: 100 - idx,
      actions: [{
        id: "search",
        title: "Search Again",
        type: "search",
        metadata: { query: q }
      }]
    }));
    
    return JSON.stringify({
      results: historyResults,
      total: historyResults.length
    });
  }
  
  // Get configuration
  const config: PluginConfig = {
    github_token: context.preferences?.github_token,
    include_forks: context.preferences?.include_forks || false,
    include_archived: context.preferences?.include_archived || false,
    per_page: parseInt(context.preferences?.per_page || "30"),
    sort_by: context.preferences?.sort_by || "best-match"
  };
  
  // Build GitHub search query
  let searchQuery = query;
  if (!config.include_forks) {
    searchQuery += " fork:false";
  }
  if (!config.include_archived) {
    searchQuery += " archived:false";
  }
  
  // Prepare API request
  const requestId = generateRequestId();
  const searchParams = new URLSearchParams({
    q: searchQuery,
    sort: config.sort_by === "best-match" ? "" : config.sort_by,
    order: "desc",
    per_page: config.per_page.toString()
  }).toString();
  
  const headers: Record<string, string> = {
    "Accept": "application/vnd.github.v3+json",
    "User-Agent": "Modern-Launcher-GitHub-Plugin"
  };
  
  if (config.github_token) {
    headers["Authorization"] = `token ${config.github_token}`;
  }
  
  // Call async HTTP host function
  Host.outputString(JSON.stringify({
    request: "http",
    method: "GET",
    url: `https://api.github.com/search/repositories?${searchParams}`,
    headers,
    request_id: requestId,
    callback_fn: "github_search_callback"
  }));
  
  // Store search in history
  if (!state.searchHistory.includes(query)) {
    state.searchHistory.unshift(query);
    if (state.searchHistory.length > 10) {
      state.searchHistory.pop();
    }
  }
  
  // Return loading state
  return JSON.stringify({
    results: [{
      id: "loading",
      title: "Searching GitHub...",
      subtitle: `Query: ${query}`,
      icon: "loading",
      score: 0
    }],
    total: 1,
    is_loading: true
  });
}

// Callback for HTTP response
export function github_search_callback(result: string) {
  const { request_id, result: httpResult } = JSON.parse(result);
  
  if (httpResult.error) {
    const errorResults: SearchResult[] = [{
      id: "error",
      title: "Search Failed",
      subtitle: httpResult.error,
      icon: "error",
      score: 0
    }];
    
    Host.outputString(JSON.stringify({
      results: errorResults,
      total: 1,
      error: httpResult.error
    }));
    return;
  }
  
  try {
    const response: GitHubSearchResponse = JSON.parse(httpResult.body);
    
    // Convert GitHub repos to search results
    const results: SearchResult[] = response.items.map((repo, idx) => ({
      id: repo.id.toString(),
      title: repo.name,
      subtitle: `${repo.owner.login} • ⭐ ${repo.stargazers_count} • ${repo.description || "No description"}`,
      icon: repo.private ? "lock" : "github",
      score: 100 - idx,
      metadata: {
        full_name: repo.full_name,
        url: repo.html_url,
        stars: repo.stargazers_count,
        forks: repo.forks_count,
        language: repo.language,
        updated: repo.updated_at,
        owner: repo.owner.login
      },
      actions: [
        {
          id: "open_browser",
          title: "Open in Browser",
          type: "open_url",
          metadata: { url: repo.html_url }
        },
        {
          id: "copy_url",
          title: "Copy URL",
          type: "copy",
          metadata: { text: repo.html_url }
        },
        {
          id: "copy_clone_url",
          title: "Copy Clone URL",
          type: "copy",
          metadata: { text: `git clone https://github.com/${repo.full_name}.git` }
        },
        {
          id: "open_vscode",
          title: "Open in VS Code",
          type: "open_url",
          metadata: { url: `vscode://vscode.git/clone?url=https://github.com/${repo.full_name}.git` }
        }
      ]
    }));
    
    // Cache results
    state.cachedResults = results;
    
    Host.outputString(JSON.stringify({
      results,
      total: response.total_count,
      metadata: {
        per_page: response.items.length,
        total_count: response.total_count
      }
    }));
  } catch (error) {
    Host.outputString(JSON.stringify({
      results: [{
        id: "parse_error",
        title: "Failed to parse response",
        subtitle: error.toString(),
        icon: "error",
        score: 0
      }],
      total: 1,
      error: error.toString()
    }));
  }
}

// Execute action
export function plugin_action(input: string) {
  const { action_id, context, metadata } = JSON.parse(input);
  
  switch (action_id) {
    case "open_browser":
      if (metadata?.url) {
        Host.outputString(JSON.stringify({
          type: "open_url",
          url: metadata.url
        }));
      }
      break;
      
    case "copy_url":
    case "copy_clone_url":
      if (metadata?.text) {
        // Call clipboard write host function
        const requestId = generateRequestId();
        Host.outputString(JSON.stringify({
          request: "clipboard_write",
          text: metadata.text,
          request_id: requestId,
          callback_fn: "clipboard_callback"
        }));
      }
      break;
      
    case "open_vscode":
      if (metadata?.url) {
        Host.outputString(JSON.stringify({
          type: "open_url",
          url: metadata.url
        }));
      }
      break;
      
    case "search":
      // Re-run search with history query
      if (metadata?.query) {
        return plugin_search(JSON.stringify({
          query: metadata.query,
          context
        }));
      }
      break;
  }
  
  return JSON.stringify({ success: true });
}

// Clipboard callback
export function clipboard_callback(result: string) {
  const { request_id, result: clipboardResult } = JSON.parse(result);
  
  if (clipboardResult.error) {
    Host.outputString(JSON.stringify({
      notification: {
        title: "Failed to copy",
        body: clipboardResult.error,
        type: "error"
      }
    }));
  } else {
    Host.outputString(JSON.stringify({
      notification: {
        title: "Copied to clipboard",
        body: "URL copied successfully",
        type: "success"
      }
    }));
  }
}

// Configuration validation
export function plugin_validate_config(input: string) {
  const config = JSON.parse(input);
  const errors: string[] = [];
  
  if (config.per_page) {
    const perPage = parseInt(config.per_page);
    if (isNaN(perPage) || perPage < 10 || perPage > 100) {
      errors.push("Results per page must be between 10 and 100");
    }
  }
  
  if (config.github_token && config.github_token.length < 40) {
    errors.push("GitHub token appears to be invalid");
  }
  
  return JSON.stringify({
    valid: errors.length === 0,
    errors
  });
}