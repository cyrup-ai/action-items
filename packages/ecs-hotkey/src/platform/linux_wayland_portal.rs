//! XDG Desktop Portal GlobalShortcuts backend
//!
//! Implements global shortcuts via org.freedesktop.portal.GlobalShortcuts

use super::{WaylandBackend, WaylandError};
use crate::{HotkeyBinding, HotkeyDefinition};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use zbus::{
    proxy,
    zvariant::{ObjectPath, OwnedObjectPath, Value},
    Connection,
};

#[proxy(
    interface = "org.freedesktop.portal.GlobalShortcuts",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait GlobalShortcuts {
    /// Create a shortcuts session
    async fn create_session(
        &self,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    /// Bind shortcuts to session
    async fn bind_shortcuts(
        &self,
        session_handle: ObjectPath<'_>,
        shortcuts: Vec<(String, HashMap<&str, Value<'_>>)>,
        parent_window: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    /// Activated signal
    #[zbus(signal)]
    fn activated(
        session_handle: ObjectPath<'_>,
        shortcut_id: &str,
        timestamp: u64,
        options: HashMap<String, Value<'_>>,
    ) -> zbus::Result<()>;
}

pub struct XdgPortalBackend {
    connection: Connection,
    proxy: GlobalShortcutsProxy<'static>,
    session_handle: Option<OwnedObjectPath>,
    triggered_queue: Arc<RwLock<Vec<String>>>,
}

impl XdgPortalBackend {
    pub async fn new() -> Result<Self, WaylandError> {
        let connection = Connection::session().await?;
        let proxy = GlobalShortcutsProxy::new(&connection).await?;

        Ok(Self {
            connection,
            proxy,
            session_handle: None,
            triggered_queue: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Convert HotkeyDefinition to XDG Portal trigger format
    fn hotkey_to_xdg_trigger(hotkey: &HotkeyDefinition) -> String {
        use global_hotkey::hotkey::{Code, Modifiers};

        // XDG format: "<Control><Shift>space"
        let mut trigger = String::new();

        if hotkey.modifiers.contains(Modifiers::CONTROL) {
            trigger.push_str("<Control>");
        }
        if hotkey.modifiers.contains(Modifiers::SHIFT) {
            trigger.push_str("<Shift>");
        }
        if hotkey.modifiers.contains(Modifiers::ALT) {
            trigger.push_str("<Alt>");
        }
        if hotkey.modifiers.contains(Modifiers::SUPER) || hotkey.modifiers.contains(Modifiers::META)
        {
            trigger.push_str("<Super>");
        }

        // Map key code to GTK key name
        let key_str = match hotkey.code {
            // Letters (lowercase for GTK)
            Code::KeyA => "a",
            Code::KeyB => "b",
            Code::KeyC => "c",
            Code::KeyD => "d",
            Code::KeyE => "e",
            Code::KeyF => "f",
            Code::KeyG => "g",
            Code::KeyH => "h",
            Code::KeyI => "i",
            Code::KeyJ => "j",
            Code::KeyK => "k",
            Code::KeyL => "l",
            Code::KeyM => "m",
            Code::KeyN => "n",
            Code::KeyO => "o",
            Code::KeyP => "p",
            Code::KeyQ => "q",
            Code::KeyR => "r",
            Code::KeyS => "s",
            Code::KeyT => "t",
            Code::KeyU => "u",
            Code::KeyV => "v",
            Code::KeyW => "w",
            Code::KeyX => "x",
            Code::KeyY => "y",
            Code::KeyZ => "z",
            // Numbers
            Code::Digit0 => "0",
            Code::Digit1 => "1",
            Code::Digit2 => "2",
            Code::Digit3 => "3",
            Code::Digit4 => "4",
            Code::Digit5 => "5",
            Code::Digit6 => "6",
            Code::Digit7 => "7",
            Code::Digit8 => "8",
            Code::Digit9 => "9",
            // Special keys
            Code::Space => "space",
            Code::Enter => "Return",
            Code::Escape => "Escape",
            Code::Backspace => "BackSpace",
            Code::Tab => "Tab",
            // Function keys
            Code::F1 => "F1",
            Code::F2 => "F2",
            Code::F3 => "F3",
            Code::F4 => "F4",
            Code::F5 => "F5",
            Code::F6 => "F6",
            Code::F7 => "F7",
            Code::F8 => "F8",
            Code::F9 => "F9",
            Code::F10 => "F10",
            Code::F11 => "F11",
            Code::F12 => "F12",
            // Arrow keys
            Code::ArrowUp => "Up",
            Code::ArrowDown => "Down",
            Code::ArrowLeft => "Left",
            Code::ArrowRight => "Right",
            // Other common keys
            Code::Home => "Home",
            Code::End => "End",
            Code::PageUp => "Page_Up",
            Code::PageDown => "Page_Down",
            Code::Insert => "Insert",
            Code::Delete => "Delete",
            // Fallback
            _ => "unknown",
        };

        trigger.push_str(key_str);
        trigger
    }
}

#[async_trait::async_trait]
impl WaylandBackend for XdgPortalBackend {
    async fn init(&mut self) -> Result<(), WaylandError> {
        // Create session
        let options = HashMap::from([("session_handle_token", Value::from("action_items_session"))]);

        let session_handle = self.proxy.create_session(options).await?;

        // Set up Activated signal handler
        let triggered_queue = Arc::clone(&self.triggered_queue);
        let mut stream = self.proxy.receive_activated().await?;

        tokio::spawn(async move {
            while let Some(signal) = stream.next().await {
                if let Ok(args) = signal.args() {
                    debug!("XDG Portal shortcut activated: {}", args.shortcut_id);
                    triggered_queue
                        .write()
                        .await
                        .push(args.shortcut_id.to_string());
                }
            }
        });

        self.session_handle = Some(session_handle);
        info!("XDG Portal backend initialized");
        Ok(())
    }

    async fn register(&mut self, binding: &HotkeyBinding) -> Result<(), WaylandError> {
        let session_handle = self
            .session_handle
            .as_ref()
            .ok_or_else(|| WaylandError::BackendUnavailable("Session not initialized".to_string()))?;

        let trigger = Self::hotkey_to_xdg_trigger(&binding.definition);

        let shortcut = (
            binding.action.clone(),
            HashMap::from([
                ("description", Value::from(&binding.definition.description)),
                ("preferred_trigger", Value::from(trigger)),
            ]),
        );

        self.proxy
            .bind_shortcuts(
                session_handle.as_ref(),
                vec![shortcut],
                "", // No parent window
                HashMap::new(),
            )
            .await?;

        info!("Registered XDG Portal shortcut: {}", binding.action);
        Ok(())
    }

    async fn unregister(&mut self, _action_id: &str) -> Result<(), WaylandError> {
        // XDG Portal doesn't have explicit unregister - shortcuts are session-bound
        // They're automatically cleaned up when session ends
        Ok(())
    }

    async fn poll_events(&mut self) -> Result<Vec<String>, WaylandError> {
        let mut queue = self.triggered_queue.write().await;
        let events = queue.drain(..).collect();
        Ok(events)
    }

    async fn is_available() -> bool {
        if let Ok(conn) = Connection::session().await {
            GlobalShortcutsProxy::new(&conn).await.is_ok()
        } else {
            false
        }
    }
}
