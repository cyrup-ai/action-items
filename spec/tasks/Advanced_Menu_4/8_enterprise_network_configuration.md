# Task 8: Enterprise Network Configuration System

## Implementation Details

**File**: `ui/src/ui/enterprise_network.rs`  
**Lines**: 325-445  
**Architecture**: Enterprise-grade proxy and certificate management with compliance monitoring  
**Integration**: NetworkManager, SecurityManager, CertificateStore, ProxyHandler  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct EnterpriseNetworkManager {
    pub proxy_configuration: ProxyConfiguration,
    pub certificate_management: CertificateManagement,
    pub network_policies: NetworkPolicies,
    pub compliance_monitor: ComplianceMonitor,
    pub audit_logger: AuditLogger,
    pub security_settings: NetworkSecuritySettings,
}

#[derive(Clone, Debug)]
pub struct ProxyConfiguration {
    pub enabled: bool,
    pub use_system_settings: bool,
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub authentication: Option<ProxyAuthentication>,
    pub bypass_list: Vec<String>,
    pub pac_url: Option<String>,
    pub connection_timeout_ms: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProxyType {
    HTTP,
    HTTPS,
    SOCKS4,
    SOCKS5,
    PAC, // Proxy Auto-Configuration
    WPAD, // Web Proxy Auto-Discovery
}

#[derive(Clone, Debug)]
pub struct ProxyAuthentication {
    pub method: AuthenticationMethod,
    pub username: String,
    pub password_encrypted: Vec<u8>, // Encrypted password
    pub domain: Option<String>, // For NTLM/Kerberos
    pub use_current_credentials: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AuthenticationMethod {
    None,
    Basic,
    NTLM,
    Kerberos,
    Negotiate,
}

#[derive(Clone, Debug)]
pub struct CertificateManagement {
    pub certificate_store: CertificateStore,
    pub validation_settings: CertificateValidationSettings,
    pub trusted_roots: Vec<Certificate>,
    pub client_certificates: Vec<ClientCertificate>,
    pub revocation_checking: RevocationCheckingMode,
    pub certificate_policies: Vec<CertificatePolicy>,
}

pub fn enterprise_network_system(
    mut network_manager: ResMut<EnterpriseNetworkManager>,
    mut network_events: EventReader<NetworkConfigurationEvent>,
    mut certificate_events: EventWriter<CertificateEvent>,
    mut proxy_events: EventWriter<ProxyEvent>,
    mut audit_events: EventWriter<AuditEvent>,
    mut ui_events: EventWriter<UINotificationEvent>,
) {
    // Process network configuration changes
    for event in network_events.read() {
        match event {
            NetworkConfigurationEvent::ConfigureProxy { configuration } => {
                let result = configure_proxy_settings(
                    &mut network_manager.proxy_configuration,
                    configuration,
                    &proxy_events,
                    &audit_events,
                );

                match result {
                    Ok(changes) => {
                        ui_events.send(UINotificationEvent::Success {
                            message: "Proxy configuration updated successfully".to_string(),
                        });

                        // Log configuration change
                        audit_events.send(AuditEvent::ProxyConfigurationChanged {
                            old_config: network_manager.proxy_configuration.clone(),
                            new_config: configuration.clone(),
                            changed_by: get_current_user(),
                            timestamp: chrono::Utc::now(),
                        });

                        // Apply proxy changes
                        for change in changes {
                            apply_proxy_change(change, &proxy_events);
                        }
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to configure proxy: {}", error),
                            action: Some(UIAction::OpenNetworkSettings),
                        });

                        audit_events.send(AuditEvent::ProxyConfigurationFailed {
                            error: error.to_string(),
                            attempted_config: configuration.clone(),
                            timestamp: chrono::Utc::now(),
                        });
                    }
                }
            }

            NetworkConfigurationEvent::ConfigureCertificates { store_type } => {
                let result = configure_certificate_store(
                    &mut network_manager.certificate_management,
                    *store_type,
                    &certificate_events,
                );

                match result {
                    Ok(_) => {
                        ui_events.send(UINotificationEvent::Info {
                            message: format!("Certificate store set to {:?}", store_type),
                            duration: Some(Duration::from_secs(3)),
                        });

                        // Validate certificate store
                        validate_certificate_store(&network_manager.certificate_management, &ui_events);
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Warning {
                            message: format!("Certificate store configuration warning: {}", error),
                            details: Some("Application restart may be required".to_string()),
                        });
                    }
                }
            }

            NetworkConfigurationEvent::SyncSystemSettings => {
                let result = synchronize_system_network_settings(&mut network_manager);
                
                match result {
                    Ok(changes) => {
                        if !changes.is_empty() {
                            ui_events.send(UINotificationEvent::Info {
                                message: format!("Synchronized {} system network settings", changes.len()),
                                duration: Some(Duration::from_secs(5)),
                            });

                            // Show restart notification if needed
                            if changes.iter().any(|change| change.requires_restart()) {
                                ui_events.send(UINotificationEvent::Warning {
                                    message: "Some network changes require application restart".to_string(),
                                    details: Some("Click here to restart now".to_string()),
                                });
                            }
                        }
                    }
                    Err(error) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to sync system settings: {}", error),
                            action: Some(UIAction::CheckSystemPermissions),
                        });
                    }
                }
            }
        }
    }

    // Monitor network compliance
    monitor_network_compliance(&mut network_manager, &audit_events, &ui_events);

    // Update network statistics
    update_network_statistics(&mut network_manager);
}

fn configure_proxy_settings(
    proxy_config: &mut ProxyConfiguration,
    new_config: &ProxyConfiguration,
    proxy_events: &EventWriter<ProxyEvent>,
    audit_events: &EventWriter<AuditEvent>,
) -> Result<Vec<ProxyChange>, NetworkConfigurationError> {
    let mut changes = Vec::new();

    // Validate proxy configuration
    validate_proxy_configuration(new_config)?;

    // Compare configurations and track changes
    if proxy_config.enabled != new_config.enabled {
        changes.push(ProxyChange::EnabledChanged {
            old: proxy_config.enabled,
            new: new_config.enabled,
        });
    }

    if proxy_config.use_system_settings != new_config.use_system_settings {
        changes.push(ProxyChange::SystemSettingsChanged {
            old: proxy_config.use_system_settings,
            new: new_config.use_system_settings,
        });

        if new_config.use_system_settings {
            // Synchronize with system proxy settings
            synchronize_system_proxy_settings(proxy_config)?;
        }
    }

    if proxy_config.host != new_config.host || proxy_config.port != new_config.port {
        changes.push(ProxyChange::EndpointChanged {
            old_host: proxy_config.host.clone(),
            old_port: proxy_config.port,
            new_host: new_config.host.clone(),
            new_port: new_config.port,
        });
    }

    // Update configuration
    *proxy_config = new_config.clone();

    // Test proxy connectivity
    test_proxy_connectivity(proxy_config, proxy_events)?;

    Ok(changes)
}

#[cfg(target_os = "macos")]
fn synchronize_system_proxy_settings(proxy_config: &mut ProxyConfiguration) -> Result<(), NetworkConfigurationError> {
    use core_foundation::*;
    use system_configuration::*;
    
    unsafe {
        let store = SCDynamicStoreCreate(
            std::ptr::null(),
            CFStringCreateWithCString(
                std::ptr::null(),
                "com.raycast.network\0".as_ptr() as *const i8,
                kCFStringEncodingUTF8,
            ),
            std::ptr::null(),
            std::ptr::null(),
        );
        
        if store.is_null() {
            return Err(NetworkConfigurationError::SystemIntegrationFailed(
                "Failed to create system configuration store".to_string()
            ));
        }

        let proxy_settings = SCDynamicStoreCopyProxies(store);
        if proxy_settings.is_null() {
            return Err(NetworkConfigurationError::SystemProxyNotFound);
        }

        // Extract HTTP proxy settings
        let http_enable_key = CFStringCreateWithCString(
            std::ptr::null(),
            "HTTPEnable\0".as_ptr() as *const i8,
            kCFStringEncodingUTF8,
        );
        
        if CFDictionaryContainsKey(proxy_settings, http_enable_key as *const _) {
            let enabled = CFDictionaryGetValue(proxy_settings, http_enable_key as *const _);
            if !enabled.is_null() {
                proxy_config.enabled = CFBooleanGetValue(enabled as CFBooleanRef);
            }
        }

        // Extract proxy host and port
        let http_proxy_key = CFStringCreateWithCString(
            std::ptr::null(),
            "HTTPProxy\0".as_ptr() as *const i8,
            kCFStringEncodingUTF8,
        );
        
        if CFDictionaryContainsKey(proxy_settings, http_proxy_key as *const _) {
            let proxy_host = CFDictionaryGetValue(proxy_settings, http_proxy_key as *const _);
            if !proxy_host.is_null() {
                let host_str = CFStringGetCStringPtr(proxy_host as CFStringRef, kCFStringEncodingUTF8);
                if !host_str.is_null() {
                    proxy_config.host = std::ffi::CStr::from_ptr(host_str).to_string_lossy().to_string();
                }
            }
        }

        CFRelease(proxy_settings);
        CFRelease(store);
    }

    proxy_config.use_system_settings = true;
    proxy_config.proxy_type = ProxyType::HTTP;

    Ok(())
}

#[cfg(target_os = "windows")]
fn synchronize_system_proxy_settings(proxy_config: &mut ProxyConfiguration) -> Result<(), NetworkConfigurationError> {
    use winapi::um::wininet::*;
    use winapi::um::winreg::*;
    
    unsafe {
        let mut internet_per_conn_option_list = INTERNET_PER_CONN_OPTION_LIST {
            dwSize: std::mem::size_of::<INTERNET_PER_CONN_OPTION_LIST>() as u32,
            pszConnection: std::ptr::null_mut(),
            dwOptionCount: 2,
            dwOptionError: 0,
            pOptions: std::ptr::null_mut(),
        };

        let mut options = vec![
            INTERNET_PER_CONN_OPTION {
                dwOption: INTERNET_PER_CONN_FLAGS,
                Value: std::mem::zeroed(),
            },
            INTERNET_PER_CONN_OPTION {
                dwOption: INTERNET_PER_CONN_PROXY_SERVER,
                Value: std::mem::zeroed(),
            },
        ];

        internet_per_conn_option_list.pOptions = options.as_mut_ptr();

        let mut buffer_size = std::mem::size_of::<INTERNET_PER_CONN_OPTION_LIST>() as u32;
        
        let result = InternetQueryOptionA(
            std::ptr::null_mut(),
            INTERNET_OPTION_PER_CONN_FLAGS,
            &mut internet_per_conn_option_list as *mut _ as *mut _,
            &mut buffer_size,
        );

        if result != 0 {
            let flags = options[0].Value.dwValue;
            proxy_config.enabled = (flags & PROXY_TYPE_PROXY) != 0;
            proxy_config.use_system_settings = true;
            
            if proxy_config.enabled {
                let proxy_server = options[1].Value.pszValue;
                if !proxy_server.is_null() {
                    let proxy_str = std::ffi::CStr::from_ptr(proxy_server).to_string_lossy();
                    if let Some((host, port)) = parse_proxy_string(&proxy_str) {
                        proxy_config.host = host;
                        proxy_config.port = port;
                        proxy_config.proxy_type = ProxyType::HTTP;
                    }
                }
            }
        } else {
            return Err(NetworkConfigurationError::SystemIntegrationFailed(
                "Failed to query Windows proxy settings".to_string()
            ));
        }
    }

    Ok(())
}
```

### Certificate Management System

**Reference**: `./docs/bevy/examples/security/certificate_validation.rs:185-228`

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum CertificateStoreType {
    System,           // OS keychain/certificate store
    Keychain,         // macOS Keychain
    WindowsCertStore, // Windows Certificate Store
    CustomStore(PathBuf), // Custom certificate store
}

#[derive(Clone, Debug)]
pub struct CertificateValidationSettings {
    pub validate_chain: bool,
    pub check_revocation: bool,
    pub allow_self_signed: bool,
    pub require_valid_hostname: bool,
    pub custom_ca_certificates: Vec<Certificate>,
    pub certificate_pinning: Vec<CertificatePin>,
}

fn configure_certificate_store(
    cert_management: &mut CertificateManagement,
    store_type: CertificateStoreType,
    certificate_events: &EventWriter<CertificateEvent>,
) -> Result<(), NetworkConfigurationError> {
    match store_type {
        CertificateStoreType::System | CertificateStoreType::Keychain => {
            #[cfg(target_os = "macos")]
            {
                configure_keychain_access(cert_management)?;
            }
            
            #[cfg(target_os = "windows")]
            {
                configure_windows_cert_store(cert_management)?;
            }
            
            #[cfg(target_os = "linux")]
            {
                configure_linux_cert_store(cert_management)?;
            }
        }
        CertificateStoreType::WindowsCertStore => {
            #[cfg(target_os = "windows")]
            {
                configure_windows_cert_store(cert_management)?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                return Err(NetworkConfigurationError::UnsupportedPlatform(
                    "Windows Certificate Store not available on this platform".to_string()
                ));
            }
        }
        CertificateStoreType::CustomStore(path) => {
            configure_custom_certificate_store(cert_management, &path)?;
        }
    }

    cert_management.certificate_store.store_type = store_type;

    // Validate certificate store configuration
    validate_certificate_store_configuration(cert_management)?;

    // Load trusted root certificates
    load_trusted_root_certificates(cert_management)?;

    certificate_events.send(CertificateEvent::StoreConfigured {
        store_type: cert_management.certificate_store.store_type.clone(),
        certificate_count: cert_management.trusted_roots.len(),
    });

    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_keychain_access(cert_management: &mut CertificateManagement) -> Result<(), NetworkConfigurationError> {
    use security_framework::keychain::*;
    use security_framework::certificate::*;
    
    // Access system keychain
    let keychain = SecKeychain::default()
        .map_err(|e| NetworkConfigurationError::CertificateStoreAccess(e.to_string()))?;

    // Search for certificates
    let search = SecCertificateSearch::new()
        .map_err(|e| NetworkConfigurationError::CertificateSearch(e.to_string()))?;

    let certificates: Vec<SecCertificate> = search.search()
        .map_err(|e| NetworkConfigurationError::CertificateEnumeration(e.to_string()))?;

    // Convert to internal certificate format
    for sec_cert in certificates {
        let der_data = sec_cert.to_der();
        let certificate = Certificate::from_der(&der_data)
            .map_err(|e| NetworkConfigurationError::CertificateParsing(e.to_string()))?;
        
        cert_management.trusted_roots.push(certificate);
    }

    cert_management.certificate_store.keychain_access = Some(keychain);

    Ok(())
}

#[cfg(target_os = "windows")]
fn configure_windows_cert_store(cert_management: &mut CertificateManagement) -> Result<(), NetworkConfigurationError> {
    use winapi::um::wincrypt::*;
    
    unsafe {
        let cert_store = CertOpenSystemStoreA(std::ptr::null_mut(), "ROOT\0".as_ptr() as *const i8);
        if cert_store.is_null() {
            return Err(NetworkConfigurationError::CertificateStoreAccess(
                "Failed to open Windows certificate store".to_string()
            ));
        }

        let mut cert_context = CertEnumCertificatesInStore(cert_store, std::ptr::null_mut());
        
        while !cert_context.is_null() {
            let cert_data = std::slice::from_raw_parts(
                (*cert_context).pbCertEncoded,
                (*cert_context).cbCertEncoded as usize,
            );
            
            if let Ok(certificate) = Certificate::from_der(cert_data) {
                cert_management.trusted_roots.push(certificate);
            }
            
            cert_context = CertEnumCertificatesInStore(cert_store, cert_context);
        }

        CertCloseStore(cert_store, 0);
    }

    Ok(())
}

fn validate_certificate_store(
    cert_management: &CertificateManagement,
    ui_events: &EventWriter<UINotificationEvent>,
) {
    let mut validation_issues = Vec::new();

    // Check if trusted roots are available
    if cert_management.trusted_roots.is_empty() {
        validation_issues.push("No trusted root certificates found".to_string());
    }

    // Check certificate store accessibility
    if !is_certificate_store_accessible(&cert_management.certificate_store) {
        validation_issues.push("Certificate store is not accessible".to_string());
    }

    // Check for expired certificates
    let expired_count = cert_management.trusted_roots.iter()
        .filter(|cert| cert.is_expired())
        .count();
    
    if expired_count > 0 {
        validation_issues.push(format!("{} expired certificates found", expired_count));
    }

    if !validation_issues.is_empty() {
        ui_events.send(UINotificationEvent::Warning {
            message: format!("Certificate validation found {} issues", validation_issues.len()),
            details: Some(validation_issues.join("; ")),
        });
    }
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_dropdown.rs:485-528`

```rust
// Enterprise Network Configuration section
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(16.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    // Section header with important notice
    (TextBundle::from_section(
        "Proxy and Certificate Settings",
        TextStyle {
            font: asset_server.load("fonts/Inter-SemiBold.ttf"),
            font_size: 16.0,
            color: Color::rgb(0.95, 0.95, 0.95),
        },
    ),),
    (TextBundle::from_section(
        "If you change your system's proxy settings or Keychain certificates later, you need to restart Raycast.",
        TextStyle {
            font: asset_server.load("fonts/Inter-Regular.ttf"),
            font_size: 11.0,
            color: Color::rgb(0.8, 0.6, 0.4), // Warning color
        },
    ),),
    
    // Web Proxy configuration
    (SettingsRowBundle {
        label: "Web Proxy".to_string(),
        control: ControlType::Checkbox {
            checked: network_manager.proxy_configuration.use_system_settings,
        },
        sublabel: Some("Use System Network Settings".to_string()),
        tooltip: Some("Inherit proxy settings from system network configuration for enterprise integration".to_string()),
        ..default()
    },),
    
    // Certificate store selection
    (SettingsRowBundle {
        label: "Certificates".to_string(),
        control: ControlType::Dropdown {
            options: vec![
                "Keychain".to_string(),
                "System Store".to_string(),
                "Windows Certificate Store".to_string(),
                "Custom Store...".to_string(),
            ],
            selected: match network_manager.certificate_management.certificate_store.store_type {
                CertificateStoreType::Keychain => 0,
                CertificateStoreType::System => 1,
                CertificateStoreType::WindowsCertStore => 2,
                CertificateStoreType::CustomStore(_) => 3,
            },
        },
        tooltip: Some("Select certificate store for HTTPS connections and security validation".to_string()),
        ..default()
    },),
    
    // Advanced proxy settings (expandable)
    (ExpansionPanelBundle {
        header: "Advanced Proxy Configuration".to_string(),
        expanded: !network_manager.proxy_configuration.use_system_settings,
        content: NodeBundle {
            children: &[
                // Proxy type selection
                (SettingsRowBundle {
                    label: "Proxy Type".to_string(),
                    control: ControlType::Dropdown {
                        options: vec![
                            "HTTP".to_string(),
                            "HTTPS".to_string(),
                            "SOCKS4".to_string(),
                            "SOCKS5".to_string(),
                            "PAC".to_string(),
                        ],
                        selected: match network_manager.proxy_configuration.proxy_type {
                            ProxyType::HTTP => 0,
                            ProxyType::HTTPS => 1,
                            ProxyType::SOCKS4 => 2,
                            ProxyType::SOCKS5 => 3,
                            ProxyType::PAC => 4,
                            ProxyType::WPAD => 4, // Fallback to PAC
                        },
                    },
                    ..default()
                },),
                
                // Proxy host and port
                (NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        width: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                children: &[
                    (SettingsRowBundle {
                        label: "Host".to_string(),
                        control: ControlType::TextInput {
                            value: network_manager.proxy_configuration.host.clone(),
                            placeholder: "proxy.company.com".to_string(),
                            validation: InputValidation::Hostname,
                        },
                        ..default()
                    },),
                    (SettingsRowBundle {
                        label: "Port".to_string(),
                        control: ControlType::NumberInput {
                            value: network_manager.proxy_configuration.port as f32,
                            min: 1.0,
                            max: 65535.0,
                            step: 1.0,
                        },
                        ..default()
                    },),
                ]),
                
                // Authentication settings
                if network_manager.proxy_configuration.authentication.is_some() {
                    (SettingsRowBundle {
                        label: "Authentication Method".to_string(),
                        control: ControlType::Dropdown {
                            options: vec![
                                "None".to_string(),
                                "Basic".to_string(),
                                "NTLM".to_string(),
                                "Kerberos".to_string(),
                                "Negotiate".to_string(),
                            ],
                            selected: match network_manager.proxy_configuration.authentication.as_ref().unwrap().method {
                                AuthenticationMethod::None => 0,
                                AuthenticationMethod::Basic => 1,
                                AuthenticationMethod::NTLM => 2,
                                AuthenticationMethod::Kerberos => 3,
                                AuthenticationMethod::Negotiate => 4,
                            },
                        },
                        ..default()
                    },)
                } else { () },
            ],
            ..default()
        },
        ..default()
    },),
    
    // Network status and diagnostics
    (ExpansionPanelBundle {
        header: "Network Diagnostics".to_string(),
        expanded: false,
        content: NodeBundle {
            children: &[
                // Connection status
                (InfoRowBundle {
                    label: "Proxy Status".to_string(),
                    value: if network_manager.proxy_configuration.enabled {
                        "Connected".to_string()
                    } else {
                        "Disabled".to_string()
                    },
                    status_color: if network_manager.proxy_configuration.enabled {
                        Some(Color::rgb(0.3, 0.8, 0.3))
                    } else {
                        Some(Color::rgb(0.6, 0.6, 0.6))
                    },
                    ..default()
                },),
                
                // Certificate count
                (InfoRowBundle {
                    label: "Trusted Certificates".to_string(),
                    value: format!("{} certificates", network_manager.certificate_management.trusted_roots.len()),
                    ..default()
                },),
                
                // Test connection button
                (ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(8.0)),
                        ..default()
                    },
                    background_color: Color::rgb(0.3, 0.6, 0.9).into(),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                children: &[
                    (TextBundle::from_section(
                        "Test Connection",
                        TextStyle {
                            font: asset_server.load("fonts/Inter-Medium.ttf"),
                            font_size: 12.0,
                            color: Color::WHITE,
                        },
                    ),),
                ]),
            ],
            ..default()
        },
        ..default()
    },),
]
```

### Architecture Notes

- Enterprise-grade proxy support with multiple authentication methods (Basic, NTLM, Kerberos)
- Cross-platform certificate store integration (macOS Keychain, Windows Certificate Store)
- Comprehensive network policy compliance monitoring and audit logging
- Real-time proxy connectivity testing and validation
- Automatic synchronization with system network configuration changes
- Security-aware credential management with encrypted password storage
- PAC (Proxy Auto-Configuration) and WPAD support for dynamic proxy discovery
- Certificate chain validation with customizable revocation checking

**Bevy Examples**: `./docs/bevy/examples/networking/http_client.rs:385-422`, `./docs/bevy/examples/security/certificate_validation.rs:285-322`  
**Integration Points**: NetworkManager, SecurityManager, AuditLogger, ComplianceMonitor  
**Dependencies**: ProxyClient, CertificateValidator, SystemIntegration, SecurityFramework