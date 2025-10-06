//! KDE KGlobalAccel DBus backend for Wayland
//!
//! Implements global shortcuts via org.kde.KGlobalAccel interface

use super::{WaylandBackend, WaylandError};
use crate::{HotkeyBinding, HotkeyDefinition};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use zbus::{proxy, Connection};

const APP_COMPONENT: &str = "action_items";

/// KDE KGlobalAccel proxy interface
#[proxy(
    interface = "org.kde.KGlobalAccel",
    default_service = "org.kde.kglobalaccel",
    default_path = "/kglobalaccel"
)]
trait KGlobalAccel {
    /// Register an action for global shortcuts
    async fn do_register(&self, action_id: &[&str]) -> zbus::Result<()>;

    /// Set shortcut keys (v2 API)
    async fn set_shortcut_keys(
        &self,
        action_id: &[&str],
        keys: &[(Vec<i32>,)],
        flags: u32,
    ) -> zbus::Result<Vec<(Vec<i32>,)>>;

    /// Unregister a shortcut
    async fn unregister(
        &self,
        component_unique: &str,
        shortcut_unique: &str,
    ) -> zbus::Result<bool>;

    /// Get component object path
    async fn get_component(
        &self,
        component_unique: &str,
    ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
}

/// KDE Component proxy for receiving shortcut signals
#[proxy(
    interface = "org.kde.kglobalaccel.Component",
    default_service = "org.kde.kglobalaccel"
)]
trait KGlobalAccelComponent {
    /// Signal emitted when global shortcut is pressed
    #[zbus(signal)]
    fn global_shortcut_pressed(
        component_unique: &str,
        action_unique: &str,
        timestamp: i64,
    ) -> zbus::Result<()>;
}

pub struct KdeGlobalAccelBackend {
    connection: Connection,
    proxy: KGlobalAccelProxy<'static>,
    component_proxy: Option<KGlobalAccelComponentProxy<'static>>,
    registered_actions: Arc<RwLock<HashMap<String, HotkeyDefinition>>>,
    triggered_queue: Arc<RwLock<Vec<String>>>,
}

impl KdeGlobalAccelBackend {
    pub async fn new() -> Result<Self, WaylandError> {
        let connection = Connection::session().await?;
        let proxy = KGlobalAccelProxy::new(&connection).await?;

        Ok(Self {
            connection,
            proxy,
            component_proxy: None,
            registered_actions: Arc::new(RwLock::new(HashMap::new())),
            triggered_queue: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Convert HotkeyDefinition to KDE key sequence format
    fn hotkey_to_kde_keyseq(hotkey: &HotkeyDefinition) -> Vec<i32> {
        use global_hotkey::hotkey::{Code, Modifiers};

        let mut keys = Vec::new();

        // Map modifiers to Qt key codes
        if hotkey.modifiers.contains(Modifiers::SHIFT) {
            keys.push(0x01000020); // Qt::Key_Shift
        }
        if hotkey.modifiers.contains(Modifiers::CONTROL) {
            keys.push(0x01000021); // Qt::Key_Control
        }
        if hotkey.modifiers.contains(Modifiers::ALT) {
            keys.push(0x01000023); // Qt::Key_Alt
        }
        if hotkey.modifiers.contains(Modifiers::META)
            || hotkey.modifiers.contains(Modifiers::SUPER)
        {
            keys.push(0x01000022); // Qt::Key_Meta
        }

        // Map key code to Qt key - comprehensive mapping
        let qt_key = match hotkey.code {
            // Letters A-Z
            Code::KeyA => 0x41,
            Code::KeyB => 0x42,
            Code::KeyC => 0x43,
            Code::KeyD => 0x44,
            Code::KeyE => 0x45,
            Code::KeyF => 0x46,
            Code::KeyG => 0x47,
            Code::KeyH => 0x48,
            Code::KeyI => 0x49,
            Code::KeyJ => 0x4a,
            Code::KeyK => 0x4b,
            Code::KeyL => 0x4c,
            Code::KeyM => 0x4d,
            Code::KeyN => 0x4e,
            Code::KeyO => 0x4f,
            Code::KeyP => 0x50,
            Code::KeyQ => 0x51,
            Code::KeyR => 0x52,
            Code::KeyS => 0x53,
            Code::KeyT => 0x54,
            Code::KeyU => 0x55,
            Code::KeyV => 0x56,
            Code::KeyW => 0x57,
            Code::KeyX => 0x58,
            Code::KeyY => 0x59,
            Code::KeyZ => 0x5a,
            // Numbers 0-9
            Code::Digit0 => 0x30,
            Code::Digit1 => 0x31,
            Code::Digit2 => 0x32,
            Code::Digit3 => 0x33,
            Code::Digit4 => 0x34,
            Code::Digit5 => 0x35,
            Code::Digit6 => 0x36,
            Code::Digit7 => 0x37,
            Code::Digit8 => 0x38,
            Code::Digit9 => 0x39,
            // Special keys
            Code::Space => 0x20,
            Code::Enter => 0x01000004,
            Code::Escape => 0x01000000,
            Code::Backspace => 0x01000003,
            Code::Tab => 0x01000001,
            // Function keys
            Code::F1 => 0x01000030,
            Code::F2 => 0x01000031,
            Code::F3 => 0x01000032,
            Code::F4 => 0x01000033,
            Code::F5 => 0x01000034,
            Code::F6 => 0x01000035,
            Code::F7 => 0x01000036,
            Code::F8 => 0x01000037,
            Code::F9 => 0x01000038,
            Code::F10 => 0x01000039,
            Code::F11 => 0x0100003a,
            Code::F12 => 0x0100003b,
            // Arrow keys
            Code::ArrowUp => 0x01000013,
            Code::ArrowDown => 0x01000015,
            Code::ArrowLeft => 0x01000012,
            Code::ArrowRight => 0x01000014,
            // Other common keys
            Code::Home => 0x01000010,
            Code::End => 0x01000011,
            Code::PageUp => 0x01000016,
            Code::PageDown => 0x01000017,
            Code::Insert => 0x01000006,
            Code::Delete => 0x01000007,
            // Fallback for unmapped keys
            _ => hotkey.code as i32,
        };
        keys.push(qt_key);

        keys
    }
}

#[async_trait::async_trait]
impl WaylandBackend for KdeGlobalAccelBackend {
    async fn init(&mut self) -> Result<(), WaylandError> {
        // Get component proxy for signal reception
        let component_path = self.proxy.get_component(APP_COMPONENT).await?;
        let component_proxy = KGlobalAccelComponentProxy::builder(&self.connection)
            .path(component_path)?
            .build()
            .await?;

        // Set up signal handler
        let triggered_queue = Arc::clone(&self.triggered_queue);

        let mut stream = component_proxy.receive_global_shortcut_pressed().await?;

        tokio::spawn(async move {
            while let Some(signal) = stream.next().await {
                if let Ok(args) = signal.args() {
                    let action_id = args.action_unique;
                    debug!("KDE shortcut pressed: {}", action_id);
                    triggered_queue.write().await.push(action_id.to_string());
                }
            }
        });

        self.component_proxy = Some(component_proxy);

        info!("KDE kglobalaccel backend initialized");
        Ok(())
    }

    async fn register(&mut self, binding: &HotkeyBinding) -> Result<(), WaylandError> {
        let action_id = binding.action.clone();
        let action_id_parts = [
            APP_COMPONENT,
            &action_id,
            &binding.definition.description,
        ];

        // Register action with KDE
        self.proxy.do_register(&action_id_parts).await?;

        // Convert hotkey to KDE format
        let key_sequence = Self::hotkey_to_kde_keyseq(&binding.definition);
        let keys = vec![(key_sequence,)];

        // Set shortcut (flags: 0 = default)
        let result = self
            .proxy
            .set_shortcut_keys(&action_id_parts, &keys, 0)
            .await?;

        if result.is_empty() {
            return Err(WaylandError::InvalidShortcut(
                "KDE rejected shortcut - may be in use".to_string(),
            ));
        }

        // Store registration
        self.registered_actions
            .write()
            .await
            .insert(action_id.clone(), binding.definition.clone());

        info!(
            "Registered KDE shortcut: {} -> {}",
            action_id, binding.definition.description
        );
        Ok(())
    }

    async fn unregister(&mut self, action_id: &str) -> Result<(), WaylandError> {
        let success = self.proxy.unregister(APP_COMPONENT, action_id).await?;

        if !success {
            return Err(WaylandError::NotFound(action_id.to_string()));
        }

        self.registered_actions.write().await.remove(action_id);

        info!("Unregistered KDE shortcut: {}", action_id);
        Ok(())
    }

    async fn poll_events(&mut self) -> Result<Vec<String>, WaylandError> {
        let mut queue = self.triggered_queue.write().await;
        let events = queue.drain(..).collect();
        Ok(events)
    }

    async fn is_available() -> bool {
        if let Ok(conn) = Connection::session().await {
            KGlobalAccelProxy::new(&conn).await.is_ok()
        } else {
            false
        }
    }
}
