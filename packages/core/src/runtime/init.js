// NOTE: Commented out problematic imports that cause runtime panic
// These modules should be automatically loaded by the extensions when using init()
// If needed, they can be dynamically imported later in plugin code
//
// import "ext:deno_webidl/00_webidl.js";
// import "ext:deno_console/01_console.js";
// import "ext:deno_url/00_url.js";
// import "ext:deno_url/01_urlpattern.js";

// JavaScript initialization code for the ActionItems Deno runtime
// This code is loaded when the runtime starts and provides the global API for plugins

/**
 * ActionItems API for JavaScript/TypeScript plugins
 *
 * This global object provides access to all action items functionality
 * from within Deno-based plugins.
 */
globalThis.ActionItems = {
  /**
   * Create a new action item
   * @param {Object} item - The action item to create
   * @param {string} item.title - The item title
   * @param {string} [item.description] - Optional description
   * @param {string[]} [item.tags] - Optional tags array
   * @param {string} [item.priority] - Priority level: "Low", "Medium", "High", "Critical"
   * @param {string} [item.status] - Status: "Todo", "InProgress", "Completed", "Archived"
   * @returns {Promise<string>} The ID of the created item
   */
  async create(item) {
    // Validate required fields
    if (!item || typeof item !== "object") {
      throw new Error("Item must be an object");
    }
    if (!item.title || typeof item.title !== "string") {
      throw new Error("Item title is required and must be a string");
    }

    // Set defaults
    const actionItem = {
      id:
        item.id ||
        `item_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      title: item.title,
      description: item.description || null,
      tags: Array.isArray(item.tags) ? item.tags : [],
      priority: item.priority || "Medium",
      status: item.status || "Todo",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };

    return await Deno.core.ops.op_create_action_item(
      JSON.stringify(actionItem),
    );
  },

  /**
   * List all action items
   * @param {Object} [filter] - Optional filter criteria
   * @returns {Promise<Object[]>} Array of action items
   */
  async list(filter = {}) {
    const result = await Deno.core.ops.op_list_action_items();
    return JSON.parse(result);
  },

  /**
   * Update an existing action item
   * @param {string} id - The item ID to update
   * @param {Object} updates - Fields to update
   * @returns {Promise<Object>} The updated action item
   */
  async update(id, updates) {
    if (!id || typeof id !== "string") {
      throw new Error("Item ID is required and must be a string");
    }
    if (!updates || typeof updates !== "object") {
      throw new Error("Updates must be an object");
    }

    // Add updated timestamp
    const updatesWithTimestamp = {
      ...updates,
      updated_at: new Date().toISOString(),
    };

    const result = await Deno.core.ops.op_update_action_item(
      id,
      JSON.stringify(updatesWithTimestamp),
    );
    return JSON.parse(result);
  },

  /**
   * Delete an action item
   * @param {string} id - The item ID to delete
   * @returns {Promise<string>} The ID of the deleted item
   */
  async delete(id) {
    if (!id || typeof id !== "string") {
      throw new Error("Item ID is required and must be a string");
    }

    return await Deno.core.ops.op_delete_action_item(id);
  },

  /**
   * Search action items
   * @param {string} query - Search query
   * @param {Object} [options] - Search options
   * @param {string[]} [options.tags] - Filter by tags
   * @param {string} [options.priority] - Filter by priority
   * @param {string} [options.status] - Filter by status
   * @param {number} [options.limit] - Maximum number of results
   * @returns {Promise<Object[]>} Array of matching action items
   */
  async search(query, options = {}) {
    if (!query || typeof query !== "string") {
      throw new Error("Search query is required and must be a string");
    }

    const result = await Deno.core.ops.op_search_action_items(
      query,
      JSON.stringify(options),
    );
    return JSON.parse(result);
  },
};

/**
 * Plugin utilities and helper functions
 */
globalThis.PluginUtils = {
  /**
   * Log a message with the specified level
   * @param {string} level - Log level: "debug", "info", "warn", "error"
   * @param {string} message - Message to log
   */
  log(level, message) {
    if (!["debug", "info", "warn", "error"].includes(level)) {
      level = "info";
    }
    Deno.core.ops.op_log_message(level, String(message));
  },

  /**
   * Log an info message
   * @param {string} message - Message to log
   */
  info(message) {
    this.log("info", message);
  },

  /**
   * Log a warning message
   * @param {string} message - Message to log
   */
  warn(message) {
    this.log("warn", message);
  },

  /**
   * Log an error message
   * @param {string} message - Message to log
   */
  error(message) {
    this.log("error", message);
  },

  /**
   * Log a debug message
   * @param {string} message - Message to log
   */
  debug(message) {
    this.log("debug", message);
  },

  /**
   * Validate an action item object
   * @param {Object} item - Item to validate
   * @returns {boolean} True if valid
   * @throws {Error} If validation fails
   */
  validateActionItem(item) {
    if (!item || typeof item !== "object") {
      throw new Error("Item must be an object");
    }

    if (!item.title || typeof item.title !== "string") {
      throw new Error("Item title is required and must be a string");
    }

    if (
      item.priority &&
      !["Low", "Medium", "High", "Critical"].includes(item.priority)
    ) {
      throw new Error("Priority must be one of: Low, Medium, High, Critical");
    }

    if (
      item.status &&
      !["Todo", "InProgress", "Completed", "Archived"].includes(item.status)
    ) {
      throw new Error(
        "Status must be one of: Todo, InProgress, Completed, Archived",
      );
    }

    if (item.tags && !Array.isArray(item.tags)) {
      throw new Error("Tags must be an array");
    }

    return true;
  },

  /**
   * Generate a unique ID for action items
   * @returns {string} Unique identifier
   */
  generateId() {
    return `item_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  },

  /**
   * Format a date string for action items
   * @param {Date} [date] - Date to format (defaults to now)
   * @returns {string} ISO date string
   */
  formatDate(date = new Date()) {
    return date.toISOString();
  },

  /**
   * Parse tags from a string
   * @param {string} tagString - Comma-separated tags
   * @returns {string[]} Array of trimmed tags
   */
  parseTags(tagString) {
    if (!tagString || typeof tagString !== "string") {
      return [];
    }
    return tagString
      .split(",")
      .map((tag) => tag.trim())
      .filter((tag) => tag.length > 0);
  },

  /**
   * Sleep for a specified number of milliseconds
   * @param {number} ms - Milliseconds to sleep
   * @returns {Promise<void>}
   */
  async sleep(ms) {
    return new Promise((resolve) => setTimeout(resolve, ms));
  },
};

/**
 * Enhanced console that integrates with the plugin logging system
 */
const originalConsole = globalThis.console;
globalThis.console = {
  ...originalConsole,

  log(...args) {
    const message = args
      .map((arg) =>
        typeof arg === "object" ? JSON.stringify(arg, null, 2) : String(arg),
      )
      .join(" ");
    PluginUtils.log("info", message);
  },

  info(...args) {
    const message = args
      .map((arg) =>
        typeof arg === "object" ? JSON.stringify(arg, null, 2) : String(arg),
      )
      .join(" ");
    PluginUtils.log("info", message);
  },

  warn(...args) {
    const message = args
      .map((arg) =>
        typeof arg === "object" ? JSON.stringify(arg, null, 2) : String(arg),
      )
      .join(" ");
    PluginUtils.log("warn", message);
  },

  error(...args) {
    const message = args
      .map((arg) =>
        typeof arg === "object" ? JSON.stringify(arg, null, 2) : String(arg),
      )
      .join(" ");
    PluginUtils.log("error", message);
  },

  debug(...args) {
    const message = args
      .map((arg) =>
        typeof arg === "object" ? JSON.stringify(arg, null, 2) : String(arg),
      )
      .join(" ");
    PluginUtils.log("debug", message);
  },
};

/**
 * Plugin lifecycle hooks - plugins can override these
 */
globalThis.PluginLifecycle = {
  /**
   * Called when the plugin is first loaded
   * Plugins should override this method for initialization
   */
  async onLoad() {
    // Default implementation - no-op
    PluginUtils.debug("Plugin loaded");
  },

  /**
   * Called when the plugin is unloaded
   * Plugins should override this method for cleanup
   */
  async onUnload() {
    // Default implementation - no-op
    PluginUtils.debug("Plugin unloaded");
  },

  /**
   * Called when the plugin is activated/enabled
   * Plugins should override this method for activation logic
   */
  async onActivate() {
    // Default implementation - no-op
    PluginUtils.debug("Plugin activated");
  },

  /**
   * Called when the plugin is deactivated/disabled
   * Plugins should override this method for deactivation logic
   */
  async onDeactivate() {
    // Default implementation - no-op
    PluginUtils.debug("Plugin deactivated");
  },
};

// Initialize the plugin environment
PluginUtils.info("ActionItems Deno runtime initialized");

// Export types for TypeScript (when available)
if (typeof globalThis.exports !== "undefined") {
  globalThis.exports = {
    ActionItems: globalThis.ActionItems,
    PluginUtils: globalThis.PluginUtils,
    PluginLifecycle: globalThis.PluginLifecycle,
  };
}
