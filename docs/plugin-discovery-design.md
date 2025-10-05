# Plugin Discovery Design for Production

## Current Problems
- Mixed development and production paths
- Hardcoded directory names with inconsistent naming
- Auto-creation of system directories (security issue)
- Auto-building Rust plugins in production (security/performance issue)
- Poor XDG Base Directory compliance

## Production Design

### 1. Environment-Based Configuration

```rust
#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Production,
    Test,
}

impl Environment {
    pub fn detect() -> Self {
        if cfg!(debug_assertions) {
            Self::Development
        } else {
            Self::Production
        }
    }
    
    pub fn from_env() -> Self {
        match std::env::var("ACTION_ITEMS_ENV").as_deref() {
            Ok("dev") | Ok("development") => Self::Development,
            Ok("test") => Self::Test,
            _ => Self::detect()
        }
    }
}
```

### 2. Platform-Specific Directory Resolution

```rust
pub struct PluginDirectories {
    user_config: PathBuf,    // User-specific plugins
    user_data: PathBuf,      // Downloaded/installed plugins  
    system: Vec<PathBuf>,    // System-wide plugins
    bundled: Option<PathBuf>, // Bundled with app
}

impl PluginDirectories {
    pub fn for_environment(env: Environment) -> Self {
        match env {
            Environment::Development => Self::development_dirs(),
            Environment::Production => Self::production_dirs(),
            Environment::Test => Self::test_dirs(),
        }
    }
    
    fn production_dirs() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "linux")] {
                Self::linux_dirs()
            } else if #[cfg(target_os = "macos")] {
                Self::macos_dirs()
            } else if #[cfg(target_os = "windows")] {
                Self::windows_dirs()
            } else {
                Self::fallback_dirs()
            }
        }
    }
    
    #[cfg(target_os = "linux")]
    fn linux_dirs() -> Self {
        // Proper XDG Base Directory compliance
        let config_home = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".config"));
            
        let data_home = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".local/share"));
            
        let data_dirs = std::env::var("XDG_DATA_DIRS")
            .unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string())
            .split(':')
            .map(|p| PathBuf::from(p).join("action-items/plugins"))
            .collect();

        Self {
            user_config: config_home.join("action-items/plugins"),
            user_data: data_home.join("action-items/plugins"),
            system: data_dirs,
            bundled: None,
        }
    }
    
    #[cfg(target_os = "macos")]
    fn macos_dirs() -> Self {
        let home = dirs::home_dir().expect("Could not find home directory");
        
        Self {
            user_config: home.join("Library/Application Support/Action Items/Plugins"),
            user_data: home.join("Library/Application Support/Action Items/Plugins"),
            system: vec![
                PathBuf::from("/Library/Application Support/Action Items/Plugins"),
                PathBuf::from("/Applications/Action Items.app/Contents/PlugIns"),
            ],
            bundled: Some(PathBuf::from("/Applications/Action Items.app/Contents/Resources/Plugins")),
        }
    }
    
    #[cfg(target_os = "windows")]
    fn windows_dirs() -> Self {
        let roaming = dirs::config_dir().expect("Could not find config directory");
        let local = dirs::data_local_dir().expect("Could not find local data directory");
        
        let mut system = Vec::new();
        if let Ok(program_files) = std::env::var("ProgramFiles") {
            system.push(PathBuf::from(program_files).join("Action Items\\Plugins"));
        }
        
        Self {
            user_config: roaming.join("Action Items\\Plugins"),
            user_data: local.join("Action Items\\Plugins"),
            system,
            bundled: None,
        }
    }
    
    fn development_dirs() -> Self {
        Self {
            user_config: PathBuf::from("./dev/plugins"),
            user_data: PathBuf::from("./dev/plugins"), 
            system: vec![PathBuf::from("./plugins")],
            bundled: Some(PathBuf::from("./plugins/bundled")),
        }
    }
}
```

### 3. Discovery Priority and Security

```rust
#[derive(Debug, Clone)]
pub struct DiscoveryConfig {
    pub directories: PluginDirectories,
    pub auto_build: bool,           // Only in development
    pub auto_create_dirs: bool,     // Only user dirs, never system
    pub max_depth: usize,
    pub allowed_extensions: HashSet<String>,
    pub security_policy: SecurityPolicy,
}

#[derive(Debug, Clone)]
pub enum SecurityPolicy {
    Permissive,     // Development: load from source, auto-build
    Restrictive,    // Production: only pre-built plugins
    Sandboxed,      // Future: run plugins in sandbox
}

impl DiscoveryConfig {
    pub fn for_environment(env: Environment) -> Self {
        let directories = PluginDirectories::for_environment(env.clone());
        
        match env {
            Environment::Development => Self {
                directories,
                auto_build: true,
                auto_create_dirs: true,
                max_depth: 3,
                allowed_extensions: ["wasm", "dll", "dylib", "so"].iter().map(|s| s.to_string()).collect(),
                security_policy: SecurityPolicy::Permissive,
            },
            Environment::Production => Self {
                directories,
                auto_build: false,
                auto_create_dirs: false, // Only create user dirs, never system
                max_depth: 2,
                allowed_extensions: ["wasm", "dll", "dylib", "so"].iter().map(|s| s.to_string()).collect(),
                security_policy: SecurityPolicy::Restrictive,
            },
            Environment::Test => Self {
                directories,
                auto_build: false,
                auto_create_dirs: true,
                max_depth: 1,
                allowed_extensions: ["wasm"].iter().map(|s| s.to_string()).collect(),
                security_policy: SecurityPolicy::Restrictive,
            },
        }
    }
}
```

### 4. Safe Directory Creation

```rust
impl DiscoveryConfig {
    pub fn ensure_user_directories(&self) -> Result<()> {
        if !self.auto_create_dirs {
            return Ok(());
        }
        
        // Only create user directories, never system directories
        let dirs_to_create = [
            &self.directories.user_config,
            &self.directories.user_data,
        ];
        
        for dir in dirs_to_create {
            if !dir.exists() {
                match std::fs::create_dir_all(dir) {
                    Ok(()) => info!("Created user plugin directory: {}", dir.display()),
                    Err(e) => warn!("Failed to create user plugin directory {}: {}", dir.display(), e),
                }
            }
        }
        
        Ok(())
    }
}
```

### 5. Plugin Loading with Security

```rust
pub fn discover_plugins_safe(config: &DiscoveryConfig) -> Result<Vec<DiscoveredPlugin>> {
    let mut plugins = Vec::new();
    
    // Load in priority order: bundled -> system -> user_data -> user_config
    let search_order = [
        config.directories.bundled.as_ref().map(|p| (p, PluginSource::Bundled)),
        Some((&config.directories.user_config, PluginSource::UserConfig)),
        Some((&config.directories.user_data, PluginSource::UserData)),
    ].into_iter().flatten()
    .chain(
        config.directories.system.iter().map(|p| (p, PluginSource::System))
    );
    
    for (dir, source) in search_order {
        if !dir.exists() {
            debug!("Plugin directory does not exist: {}", dir.display());
            continue;
        }
        
        match scan_directory_safe(dir, config, source) {
            Ok(mut found) => plugins.append(&mut found),
            Err(e) => warn!("Failed to scan plugin directory {}: {}", dir.display(), e),
        }
    }
    
    Ok(plugins)
}

#[derive(Debug, Clone)]
pub enum PluginSource {
    Bundled,    // Shipped with app
    System,     // System-wide installation
    UserData,   // User installed/downloaded
    UserConfig, // User configuration
}

#[derive(Debug)]
pub struct DiscoveredPlugin {
    pub path: PathBuf,
    pub source: PluginSource,
    pub plugin_type: PluginType,
    pub metadata: Option<PluginMetadata>,
}

#[derive(Debug)]
pub enum PluginType {
    WasmBinary(PathBuf),
    NativeBinary(PathBuf),
    RustProject(PathBuf),  // Only allowed in development
}
```

### 6. Configuration Override

```rust
// Allow environment variable overrides
impl PluginDirectories {
    pub fn with_overrides(mut self) -> Self {
        if let Ok(override_dir) = std::env::var("ACTION_ITEMS_PLUGIN_DIR") {
            // Add to user_config as highest priority
            self.user_config = PathBuf::from(override_dir);
        }
        
        if let Ok(extra_dirs) = std::env::var("ACTION_ITEMS_EXTRA_PLUGIN_DIRS") {
            for dir in extra_dirs.split(':') {
                self.system.push(PathBuf::from(dir));
            }
        }
        
        self
    }
}
```

## Usage

```rust
// In main application startup
let env = Environment::from_env();
let config = DiscoveryConfig::for_environment(env)
    .with_overrides();

config.ensure_user_directories()?;

let discovered = discover_plugins_safe(&config)?;
load_discovered_plugins(registry, discovered)?;
```

## Benefits

1. **Environment-aware**: Different behavior for dev vs production
2. **Platform-native**: Follows OS conventions (XDG, macOS app bundles, Windows)  
3. **Secure**: No auto-building in production, limited directory creation
4. **Configurable**: Environment variable overrides
5. **Prioritized**: Clear loading order with bundled > system > user
6. **Maintainable**: Centralized configuration, easy to test

## Migration Path

1. Keep current discovery for backward compatibility
2. Add new environment-aware discovery alongside  
3. Use feature flag to switch between implementations
4. Deprecate old implementation after testing