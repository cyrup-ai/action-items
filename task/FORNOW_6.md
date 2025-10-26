# Task: Fix "Allow All" Security Issue in Service Bridge

## OBJECTIVE
Replace "For now, allow all registered plugins to communicate" with proper permission/capability checking at line 604-615 in the message validation system.

## PRIORITY
P1 - CRITICAL - SECURITY ISSUE - Plugins can communicate with any other plugin without authorization

## SECURITY IMPACT
Currently, ANY registered plugin can send messages to ANY other plugin without permission checks. A malicious plugin could:
- Exfiltrate data by messaging other plugins
- Inject commands into privileged plugins
- Bypass security boundaries between plugin contexts
- Access sensitive capabilities via inter-plugin messaging

---

## FILE TO FIX
**Primary Target:** [`packages/ecs-service-bridge/src/systems/messaging.rs:604-615`](../packages/ecs-service-bridge/src/systems/messaging.rs)

**Related Files to Reference:**
- [permissions.rs](../packages/ecs-service-bridge/src/systems/plugin_management/permissions.rs) - Existing bitfield permission system (O(1) operations)
- [capability_index.rs](../packages/ecs-service-bridge/src/systems/plugin_management/capability_index.rs) - Fast capability lookup infrastructure
- [registration.rs](../packages/ecs-service-bridge/src/systems/plugin_management/registration.rs) - How plugins register with capabilities
- [components.rs](../packages/ecs-service-bridge/src/components.rs) - Capability component definition
- [resources.rs](../packages/ecs-service-bridge/src/resources.rs) - PluginInfo and Capability structures

---

## EXISTING INFRASTRUCTURE DISCOVERED

### 1. **Permission System** (Already Implemented)
Location: `packages/ecs-service-bridge/src/systems/plugin_management/permissions.rs`

```rust
#[derive(Debug, Clone, Component, Default)]
#[repr(C)]
pub struct PluginPermissions {
    permissions: u64,  // Bitfield for O(1) checks
    extended_permissions: FxHashMap<String, bool>,
}

// Existing permission constants:
const CLIPBOARD_READ: u64 = 1 << 0;
const CLIPBOARD_WRITE: u64 = 1 << 1;
const STORAGE_READ: u64 = 1 << 2;
const HTTP_REQUEST: u64 = 1 << 4;
// etc...
```

**Action:** Add new permission bit for inter-plugin messaging.

### 2. **Capability System** (Already Implemented)
Location: `packages/ecs-service-bridge/src/components.rs:67-82`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)]
pub struct Capability {
    pub name: String,
    pub version: String,
    pub description: String,
    pub metadata: HashMap<String, String>,  // ← USE THIS for allowed targets
}
```

**Key Finding:** The `metadata` field can store messaging target declarations!

### 3. **Plugin Registration** (Already Implemented)
Location: `packages/ecs-service-bridge/src/systems/plugin_management/registration.rs`

Plugins register with:
```rust
pub struct PluginRegistrationRequest {
    pub plugin_id: String,
    pub capabilities: Vec<Capability>,  // ← Plugins declare what they can do
    pub permissions: PluginPermissions,
    // ...
}
```

### 4. **Current Validation Function** (TO BE MODIFIED)
Location: `packages/ecs-service-bridge/src/systems/messaging.rs:560-651`

```rust
fn validate_plugin_permissions(
    plugin_registry: &PluginRegistryResource,
    envelope: &MessageEnvelope,
) -> bool {
    let plugin_id = envelope.routing.from.plugin_id();
    let target_plugin_id = envelope.routing.to.plugin_id();

    // ... existing checks for source registration, system plugins, target capabilities ...

    // ⚠️ SECURITY HOLE at lines 604-615 ⚠️
    if target_plugin_id != "system" && target_plugin_id != "broadcast" {
        // In production, this would check a permission matrix
        // For now, allow all registered plugins to communicate  ← THIS IS THE PROBLEM
        if plugin_registry.get_plugin(target_plugin_id).is_none() {
            warn!("Message to unregistered plugin: {} -> {}", plugin_id, target_plugin_id);
            return false;
        }
    }
    
    // ... rest of function ...
}
```

---

## IMPLEMENTATION PLAN

### **STEP 1: Define Inter-Plugin Messaging Permission Bit**

**File:** `packages/ecs-service-bridge/src/systems/plugin_management/permissions.rs`  
**Location:** After line 28 (after existing permission constants)

**Add:**
```rust
/// Permission to send inter-plugin messages
pub const INTER_PLUGIN_MESSAGING: u64 = 1 << 11;
```

### **STEP 2: Define Capability Name Constant**

**File:** `packages/ecs-service-bridge/src/components.rs`  
**Location:** After line 82 (after Capability impl block)

**Add:**
```rust
// Capability name constants for inter-plugin messaging
pub const CAPABILITY_INTER_PLUGIN_MESSAGING: &str = "inter_plugin_messaging";

// Metadata keys for messaging capabilities
pub const METADATA_ALLOWED_TARGETS: &str = "allowed_targets";
pub const METADATA_ALLOWED_MESSAGE_TYPES: &str = "allowed_message_types";
```

### **STEP 3: Add Helper Function for Target Validation**

**File:** `packages/ecs-service-bridge/src/systems/messaging.rs`  
**Location:** Before validate_plugin_permissions function (around line 555)

**Add:**
```rust
/// Check if a plugin has permission to message a specific target plugin
/// Uses capability metadata to validate allowed messaging targets
#[inline]
fn check_inter_plugin_messaging_permission(
    source_plugin: &PluginInfo,
    target_plugin_id: &str,
) -> bool {
    // Find inter_plugin_messaging capability
    let messaging_capability = source_plugin
        .capabilities
        .iter()
        .find(|cap| cap.name == CAPABILITY_INTER_PLUGIN_MESSAGING);

    let Some(capability) = messaging_capability else {
        // Plugin doesn't have inter-plugin messaging capability at all
        return false;
    };

    // Check allowed targets in metadata
    if let Some(allowed_targets_str) = capability.metadata.get(METADATA_ALLOWED_TARGETS) {
        // Parse allowed targets (comma-separated list)
        let allowed_targets: Vec<&str> = allowed_targets_str
            .split(',')
            .map(|s| s.trim())
            .collect();

        // Check for wildcard (allow all)
        if allowed_targets.contains(&"*") {
            return true;
        }

        // Check if target is in allowed list
        if allowed_targets.contains(&target_plugin_id) {
            return true;
        }

        // Check for pattern matching (e.g., "system.*" matches "system.core")
        for pattern in allowed_targets {
            if pattern.ends_with(".*") {
                let prefix = &pattern[..pattern.len() - 2];
                if target_plugin_id.starts_with(prefix) {
                    return true;
                }
            }
        }

        return false;
    }

    // If no allowed_targets specified, deny by default
    false
}

/// Optional: Check if message type is allowed for this plugin
#[inline]
fn check_message_type_permission(
    source_plugin: &PluginInfo,
    message_type: &str,
    target_plugin_id: &str,
) -> bool {
    // Find inter_plugin_messaging capability
    let messaging_capability = source_plugin
        .capabilities
        .iter()
        .find(|cap| cap.name == CAPABILITY_INTER_PLUGIN_MESSAGING);

    let Some(capability) = messaging_capability else {
        return false;
    };

    // Check allowed message types in metadata
    if let Some(allowed_types_str) = capability.metadata.get(METADATA_ALLOWED_MESSAGE_TYPES) {
        let allowed_types: Vec<&str> = allowed_types_str
            .split(',')
            .map(|s| s.trim())
            .collect();

        // Wildcard allows all message types
        if allowed_types.contains(&"*") {
            return true;
        }

        // Check if message type is allowed
        if allowed_types.contains(&message_type) {
            return true;
        }

        return false;
    }

    // If no message type restrictions specified, allow all types
    // (the capability itself is the permission gate)
    true
}
```

### **STEP 4: Replace "Allow All" with Proper Permission Check**

**File:** `packages/ecs-service-bridge/src/systems/messaging.rs`  
**Location:** Lines 604-615 (replace the entire block)

**Replace:**
```rust
    // Check cross-plugin communication permissions
    if target_plugin_id != "system" && target_plugin_id != "broadcast" {
        // In production, this would check a permission matrix
        // For now, allow all registered plugins to communicate
        if plugin_registry.get_plugin(target_plugin_id).is_none() {
            warn!(
                "Message to unregistered plugin: {} -> {}",
                plugin_id, target_plugin_id
            );
            return false;
        }
    }
```

**With:**
```rust
    // Check cross-plugin communication permissions
    if target_plugin_id != "system" && target_plugin_id != "broadcast" {
        // Verify target plugin exists
        if plugin_registry.get_plugin(target_plugin_id).is_none() {
            warn!(
                "Message to unregistered plugin: {} -> {}",
                plugin_id, target_plugin_id
            );
            return false;
        }

        // Check if source plugin has permission to message this target
        if !check_inter_plugin_messaging_permission(source_plugin, target_plugin_id) {
            warn!(
                "Plugin '{}' not authorized to message plugin '{}'. Missing or insufficient '{}' capability.",
                plugin_id,
                target_plugin_id,
                CAPABILITY_INTER_PLUGIN_MESSAGING
            );
            return false;
        }

        // Optional: Check message type permissions
        if !check_message_type_permission(
            source_plugin,
            &envelope.metadata.message_type,
            target_plugin_id,
        ) {
            warn!(
                "Plugin '{}' not authorized to send message type '{}' to plugin '{}'",
                plugin_id,
                envelope.metadata.message_type,
                target_plugin_id
            );
            return false;
        }
    }
```

### **STEP 5: Add Required Import**

**File:** `packages/ecs-service-bridge/src/systems/messaging.rs`  
**Location:** Top of file with other imports (around line 14)

**Add:**
```rust
use crate::components::{CAPABILITY_INTER_PLUGIN_MESSAGING, METADATA_ALLOWED_TARGETS, METADATA_ALLOWED_MESSAGE_TYPES};
```

---

## CAPABILITY DECLARATION EXAMPLES

### Example 1: Plugin that can message specific plugins
```rust
let messaging_capability = Capability {
    name: "inter_plugin_messaging".to_string(),
    version: "1.0.0".to_string(),
    description: "Allows messaging to system and data plugins".to_string(),
    metadata: {
        let mut map = HashMap::new();
        map.insert("allowed_targets".to_string(), "system,data_processor".to_string());
        map.insert("allowed_message_types".to_string(), "*".to_string());
        map
    },
};
```

### Example 2: Plugin that can message all plugins (trusted plugin)
```rust
let messaging_capability = Capability {
    name: "inter_plugin_messaging".to_string(),
    version: "1.0.0".to_string(),
    description: "Full inter-plugin messaging access".to_string(),
    metadata: {
        let mut map = HashMap::new();
        map.insert("allowed_targets".to_string(), "*".to_string());
        map
    },
};
```

### Example 3: Plugin with pattern-based access
```rust
let messaging_capability = Capability {
    name: "inter_plugin_messaging".to_string(),
    version: "1.0.0".to_string(),
    description: "Can message system plugins and UI plugins".to_string(),
    metadata: {
        let mut map = HashMap::new();
        map.insert("allowed_targets".to_string(), "system.*,ui.*".to_string());
        map.insert("allowed_message_types".to_string(), "request,response,notification".to_string());
        map
    },
};
```

### Example 4: Plugin with NO inter-plugin messaging
```rust
// Simply don't include the inter_plugin_messaging capability
// The plugin can only receive messages, not send them to other plugins
let capabilities = vec![
    Capability::new("storage_access", "1.0.0", "Can access storage"),
    // No inter_plugin_messaging capability
];
```

---

## IMPLEMENTATION DETAILS

### **Validation Flow:**
1. **Existing check:** Source plugin is registered ✓ (already implemented)
2. **Existing check:** System plugins bypass restrictions ✓ (already implemented)
3. **Existing check:** Target capability permissions ✓ (already implemented)
4. **NEW CHECK:** Cross-plugin messaging permission (ADD THIS)
   - Check if source plugin has `inter_plugin_messaging` capability
   - Validate target plugin ID is in allowed_targets
   - Optionally validate message type is allowed
5. **Existing check:** Message type permissions for system commands ✓ (already implemented)

### **Error Handling:**
- Log all denied messaging attempts with WARN level
- Include plugin IDs, target, and reason for denial
- Return `false` from validation to block the message
- The calling code in `route_message_envelope` already handles failures

### **Performance Considerations:**
- Capability lookup: O(n) where n = number of capabilities per plugin (typically < 10)
- String comparison: O(m) where m = length of target plugin ID
- Overall: Negligible performance impact, runs in microseconds
- No allocations in hot path (uses iterators and string slices)

---

## DEFINITION OF DONE

### Functional Requirements:
- ✓ Replace "allow all" comment and logic with actual permission checking
- ✓ Validate inter-plugin messaging using capability metadata
- ✓ Support wildcard (`*`) for trusted plugins
- ✓ Support pattern matching (e.g., `system.*`)
- ✓ Support message type restrictions (optional but recommended)
- ✓ Log all permission denials with clear warning messages
- ✓ System and broadcast targets bypass restrictions (preserve existing behavior)
- ✓ Code compiles without errors or warnings

### Security Requirements:
- ✓ Default deny: Plugins without `inter_plugin_messaging` capability cannot message other plugins
- ✓ Explicit allow: Only plugins with declared targets in capability metadata can message those targets
- ✓ Audit trail: All denied attempts are logged for security review
- ✓ No bypass paths: All inter-plugin messages must pass through validation

### Code Quality:
- ✓ Follow existing code style (see messaging.rs for patterns)
- ✓ Use inline documentation for new functions
- ✓ Use existing error logging patterns (warn! macro)
- ✓ Use #[inline] attribute for hot-path functions
- ✓ No unsafe code
- ✓ No new dependencies required

---

## MIGRATION NOTES

### For Existing Plugins:
Plugins that need to message other plugins must be updated to include the `inter_plugin_messaging` capability with appropriate `allowed_targets` metadata. Plugins without this capability will be unable to send inter-plugin messages.

### Backward Compatibility:
This is a **breaking security fix**. Existing plugins that rely on unrestricted messaging will need capability declarations added. This is intentional - security over convenience.

### Rollout Strategy:
1. Implement the validation code (this task)
2. Update system/core plugins with appropriate capabilities
3. Update user plugins with required capabilities
4. Deploy with monitoring of warning logs
5. Review denied messages and adjust capabilities as needed

---

## CONSTRAINTS

- DO implement proper security - this is critical P1 issue
- DO log all permission denials for security audit trail
- DO use existing capability infrastructure (no new systems)
- DO preserve existing behavior for system/broadcast targets
- DO NOT write unit tests (handled separately)
- DO NOT write functional tests (handled separately)
- DO NOT write benchmarks (not required for this fix)
- DO NOT create extensive documentation (code comments sufficient)

---

## REFERENCES

**Related Source Files:**
- [`messaging.rs:560-651`](../packages/ecs-service-bridge/src/systems/messaging.rs) - Target validation function
- [`permissions.rs`](../packages/ecs-service-bridge/src/systems/plugin_management/permissions.rs) - Permission bit definitions
- [`capability_index.rs`](../packages/ecs-service-bridge/src/systems/plugin_management/capability_index.rs) - Capability indexing (not needed for this fix, but available)
- [`registration.rs:24-108`](../packages/ecs-service-bridge/src/systems/plugin_management/registration.rs) - How capabilities are declared
- [`components.rs:67-82`](../packages/ecs-service-bridge/src/components.rs) - Capability struct definition
- [`resources.rs:164-197`](../packages/ecs-service-bridge/src/resources.rs) - PluginInfo with capabilities

**Security Context:**
Inter-plugin communication without permission checking is a critical vulnerability. Malicious or compromised plugins could abuse this to access data or functionality they shouldn't have access to. This fix implements defense-in-depth security with capability-based access control.
