# Advanced_Menu_4 Task 8: Proxy Configuration System

## Task Overview
Implement enterprise proxy and network settings with HTTP/HTTPS proxy support, authentication, proxy auto-configuration (PAC), and network routing for corporate environments.

## Implementation Requirements

### Core Components
```rust
// Enterprise proxy configuration system
#[derive(Resource, Reflect, Debug)]
pub struct ProxyConfigurationResource {
    pub proxy_settings: ProxySettings,
    pub authentication: ProxyAuthentication,
    pub pac_config: PACConfiguration,
    pub network_routing: NetworkRoutingConfig,
}

#[derive(Reflect, Debug, Clone)]
pub struct ProxySettings {
    pub enabled: bool,
    pub proxy_type: ProxyType,
    pub http_proxy: Option<ProxyEndpoint>,
    pub https_proxy: Option<ProxyEndpoint>,
    pub ftp_proxy: Option<ProxyEndpoint>,
    pub socks_proxy: Option<ProxyEndpoint>,
    pub bypass_list: Vec<String>,
    pub auto_detect: bool,
}

#[derive(Reflect, Debug, Clone)]
pub enum ProxyType {
    Manual,
    Automatic,
    PAC { script_url: String },
    SystemDefault,
    Direct,
}

#[derive(Reflect, Debug, Clone)]
pub struct ProxyEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: ProxyProtocol,
    pub timeout: Duration,
}

#[derive(Reflect, Debug, Clone)]
pub enum ProxyProtocol {
    HTTP,
    HTTPS,
    SOCKS4,
    SOCKS5,
}

#[derive(Component, Reflect, Debug)]
pub struct ProxyConfigurationComponent {
    pub settings_panel: Entity,
    pub authentication_panel: Entity,
    pub pac_editor: Entity,
    pub connection_tester: Entity,
}

pub fn proxy_configuration_system(
    mut proxy_res: ResMut<ProxyConfigurationResource>,
    proxy_events: EventReader<ProxyEvent>,
    mut connection_events: EventWriter<ConnectionEvent>,
) {
    for event in proxy_events.read() {
        match event {
            ProxyEvent::UpdateSettings { settings } => {
                proxy_res.proxy_settings = settings.clone();
                connection_events.send(ConnectionEvent::ProxyConfigChanged);
            }
            ProxyEvent::TestConnection { endpoint } => {
                test_proxy_connection(endpoint, &proxy_res.authentication);
            }
        }
    }
}
```

### Authentication System
```rust
// Proxy authentication management
#[derive(Reflect, Debug)]
pub struct ProxyAuthentication {
    pub auth_methods: Vec<AuthenticationMethod>,
    pub credentials: SecureCredentialStore,
    pub ntlm_settings: Option<NTLMSettings>,
    pub kerberos_settings: Option<KerberosSettings>,
}

#[derive(Reflect, Debug)]
pub enum AuthenticationMethod {
    None,
    Basic { username: String, password_hash: String },
    Digest,
    NTLM,
    Kerberos,
    Certificate { cert_path: PathBuf },
}

#[derive(Reflect, Debug)]
pub struct NTLMSettings {
    pub domain: String,
    pub workstation: Option<String>,
    pub version: NTLMVersion,
}

#[derive(Reflect, Debug)]
pub enum NTLMVersion {
    V1,
    V2,
}

async fn authenticate_proxy_connection(
    endpoint: &ProxyEndpoint,
    auth: &ProxyAuthentication,
) -> Result<AuthenticatedConnection, ProxyError> {
    for auth_method in &auth.auth_methods {
        match auth_method {
            AuthenticationMethod::Basic { username, password_hash } => {
                let credentials = retrieve_credentials(&auth.credentials, username)?;
                match try_basic_auth(endpoint, &credentials).await {
                    Ok(connection) => return Ok(connection),
                    Err(e) => continue, // Try next method
                }
            }
            AuthenticationMethod::NTLM => {
                if let Some(ntlm_settings) = &auth.ntlm_settings {
                    match try_ntlm_auth(endpoint, ntlm_settings).await {
                        Ok(connection) => return Ok(connection),
                        Err(e) => continue,
                    }
                }
            }
            _ => continue,
        }
    }
    
    Err(ProxyError::AuthenticationFailed)
}
```

### PAC Configuration
```rust
// Proxy Auto-Configuration support
#[derive(Reflect, Debug)]
pub struct PACConfiguration {
    pub pac_enabled: bool,
    pub pac_url: Option<String>,
    pub pac_script: Option<String>,
    pub pac_cache: PACCache,
    pub fallback_config: Option<ProxySettings>,
}

#[derive(Reflect, Debug)]
pub struct PACCache {
    pub cached_decisions: HashMap<String, ProxyDecision>,
    pub cache_ttl: Duration,
    pub max_cache_size: u32,
}

#[derive(Reflect, Debug)]
pub struct ProxyDecision {
    pub proxy_string: String,
    pub decision_time: DateTime<Utc>,
    pub hit_count: u32,
}

async fn evaluate_pac_script(
    url: &str,
    host: &str,
    pac_script: &str,
) -> Result<String, PACError> {
    // JavaScript engine execution for PAC script
    let js_engine = create_pac_js_engine();
    let result = js_engine.evaluate_pac_function(url, host, pac_script).await?;
    Ok(result)
}

pub fn pac_resolution_system(
    pac_config: Res<PACConfiguration>,
    resolution_requests: EventReader<ProxyResolutionRequest>,
    mut resolution_responses: EventWriter<ProxyResolutionResponse>,
) {
    for request in resolution_requests.read() {
        if pac_config.pac_enabled {
            if let Some(cached_decision) = pac_config.pac_cache.cached_decisions.get(&request.url) {
                if !is_cache_expired(cached_decision, &pac_config.pac_cache) {
                    resolution_responses.send(ProxyResolutionResponse {
                        request_id: request.request_id,
                        proxy_string: cached_decision.proxy_string.clone(),
                    });
                    continue;
                }
            }
            
            // Evaluate PAC script for new decision
            spawn_pac_evaluation_task(request.clone(), &pac_config);
        }
    }
}
```

### Network Routing Configuration
```rust
// Advanced network routing for enterprise environments
#[derive(Reflect, Debug)]
pub struct NetworkRoutingConfig {
    pub routing_rules: Vec<RoutingRule>,
    pub bypass_rules: Vec<BypassRule>,
    pub dns_settings: DNSSettings,
    pub ssl_settings: SSLSettings,
}

#[derive(Reflect, Debug)]
pub struct RoutingRule {
    pub rule_id: String,
    pub pattern: RoutingPattern,
    pub proxy_override: Option<ProxyEndpoint>,
    pub priority: u8,
    pub enabled: bool,
}

#[derive(Reflect, Debug)]
pub enum RoutingPattern {
    Domain(String),
    IPRange { start: IpAddr, end: IpAddr },
    Regex(String),
    Port(u16),
}

#[derive(Reflect, Debug)]
pub struct DNSSettings {
    pub dns_servers: Vec<IpAddr>,
    pub dns_over_https: bool,
    pub dns_cache_ttl: Duration,
}

#[derive(Reflect, Debug)]
pub struct SSLSettings {
    pub verify_certificates: bool,
    pub custom_ca_bundle: Option<PathBuf>,
    pub ssl_version_min: SSLVersion,
    pub cipher_suites: Vec<String>,
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `async_compute/async_compute.rs` - Async proxy connection testing
- `reflection/reflection.rs` - Configuration serialization
- `ecs/change_detection.rs` - Configuration change detection

### Implementation Pattern
```rust
// Based on async_compute.rs for proxy testing
fn async_proxy_test_system(
    mut commands: Commands,
    test_requests: Query<Entity, With<ProxyTestRequest>>,
) {
    for request_entity in &test_requests {
        let task = commands.spawn_task(async move {
            // Async proxy connection testing
            test_proxy_connectivity().await
        });
    }
}

// Based on reflection.rs for configuration persistence
#[derive(Reflect, Resource)]
#[reflect(Resource)]
pub struct ProxyConfigurationResource {
    // Configuration fields with Reflect for serialization
}
```

## Enterprise Integration
- Windows domain authentication support
- Active Directory integration
- Group Policy configuration import
- Corporate certificate store integration

## Performance Constraints
- **ZERO ALLOCATIONS** during proxy resolution
- Efficient PAC script caching
- Optimized authentication credential storage
- Minimal network overhead for proxy detection

## Success Criteria
- Complete enterprise proxy configuration implementation
- Robust authentication and PAC support
- No unwrap()/expect() calls in production code
- Zero-allocation proxy resolution
- Comprehensive network routing capabilities

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for proxy configuration logic
- Integration tests for authentication methods
- Performance tests for PAC script evaluation
- Network connectivity tests for various proxy scenarios