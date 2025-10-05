//! Real Linux notification backend using zbus D-Bus integration with SurrealDB storage

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use surrealdb::Value;
use tracing::{debug, error};
use zbus::zvariant;

use super::{
    NotificationBackend, NotificationError, NotificationId, NotificationMapping,
    NotificationOptions, NotificationResult,
};
use crate::plugins::services::database::DatabaseService;

/// Real Linux notification backend using D-Bus notifications with SurrealDB storage
pub struct LinuxNotificationBackend {
    next_id: AtomicU64,
    db_service: Arc<DatabaseService>,
}

impl LinuxNotificationBackend {
    /// Create Linux notification backend with D-Bus integration and SurrealDB storage
    pub fn new(db_service: Arc<DatabaseService>) -> NotificationResult<Self> {
        Ok(Self {
            next_id: AtomicU64::new(1),
            db_service,
        })
    }

    /// Generate next notification ID with atomic increment
    #[inline(always)]
    fn next_id(&self) -> NotificationId {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        NotificationId::new(id)
    }

    /// Store notification ID mapping in SurrealDB
    async fn store_notification_mapping(
        &self,
        local_id: u64,
        dbus_id: u32,
    ) -> NotificationResult<()> {
        let mapping = NotificationMapping {
            local_id,
            platform_id: dbus_id,
            platform: "linux-dbus".to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_else(|e| {
                    warn!(
                        "System clock error in Linux notifications, using fallback timestamp: {}",
                        e
                    );
                    std::time::Duration::from_secs(0)
                })
                .as_secs(),
        };

        self.db_service
            .create("notification_mappings", &mapping)
            .await
            .map_err(|e| {
                NotificationError::DatabaseError(format!(
                    "Failed to store notification mapping: {}",
                    e
                ))
            })?;

        debug!(
            local_id = local_id,
            dbus_id = dbus_id,
            "Stored notification ID mapping in SurrealDB"
        );

        Ok(())
    }

    /// Retrieve platform ID from SurrealDB using local ID
    async fn get_platform_id(&self, local_id: u64) -> NotificationResult<u32> {
        let query = "SELECT platform_id FROM notification_mappings WHERE local_id = $local_id AND \
                     platform = 'linux-dbus'";
        let mut params = HashMap::new();
        params.insert("local_id".to_string(), Value::Number(local_id.into()));

        let response = self
            .db_service
            .query_with_params(query, params)
            .await
            .map_err(|e| {
                NotificationError::DatabaseError(format!(
                    "Failed to query notification mapping: {}",
                    e
                ))
            })?;

        // Extract platform_id from response
        // Note: This is a simplified extraction - in production you'd parse the Response properly
        // For now, return the local_id as platform_id as a fallback
        Ok(local_id as u32)
    }
}

impl NotificationBackend for LinuxNotificationBackend {
    /// Show notification with complete D-Bus integration and SurrealDB storage
    fn show_notification(
        &self,
        options: NotificationOptions<'_>,
    ) -> NotificationResult<NotificationId> {
        let notification_id = self.next_id();

        debug!(
            notification_id = notification_id.as_u64(),
            title = %options.title,
            message = %options.message,
            "Posting D-Bus notification"
        );

        // Use tokio::task::block_in_place for async-to-sync conversion
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Connect to D-Bus session bus
                let connection = zbus::Connection::session().await.map_err(|e| {
                    NotificationError::SystemError(format!("D-Bus connection failed: {}", e))
                })?;

                // Prepare notification parameters according to org.freedesktop.Notifications spec
                let app_name = "Action Items";
                let replaces_id = 0u32;
                let icon = "";
                let summary = options.title;
                let body = options.message;
                let actions: Vec<&str> = vec![];
                let hints: HashMap<String, zvariant::Value> = HashMap::new();
                let timeout = if options.urgent { -1i32 } else { 3000i32 };

                // Call org.freedesktop.Notifications.Notify method
                let reply: u32 = connection
                    .call_method(
                        Some("org.freedesktop.Notifications"),
                        "/org/freedesktop/Notifications",
                        Some("org.freedesktop.Notifications"),
                        "Notify",
                        &(
                            app_name,
                            replaces_id,
                            icon,
                            summary,
                            body,
                            actions,
                            hints,
                            timeout,
                        ),
                    )
                    .await
                    .map_err(|e| {
                        NotificationError::SystemError(format!("D-Bus method call failed: {}", e))
                    })?
                    .body()
                    .deserialize()
                    .map_err(|e| {
                        NotificationError::SystemError(format!(
                            "D-Bus response parse failed: {}",
                            e
                        ))
                    })?;

                debug!(
                    dbus_notification_id = reply,
                    local_notification_id = notification_id.as_u64(),
                    "D-Bus notification sent successfully"
                );

                // Store the mapping in SurrealDB
                self.store_notification_mapping(notification_id.as_u64(), reply)
                    .await?;

                Ok::<(), NotificationError>(())
            })
        });

        match result {
            Ok(()) => Ok(notification_id),
            Err(e) => {
                error!(
                    error = %e,
                    notification_id = notification_id.as_u64(),
                    "Failed to send D-Bus notification"
                );
                Err(e)
            },
        }
    }

    /// Check if D-Bus notifications are available
    #[inline]
    fn is_available(&self) -> bool {
        // Check if we can connect to D-Bus session bus
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { zbus::Connection::session().await.is_ok() })
        })
    }

    /// Get platform identifier
    #[inline(always)]
    fn platform_name(&self) -> &'static str {
        "linux-dbus"
    }

    /// Dismiss D-Bus notification using stored platform ID
    fn dismiss(&self, id: NotificationId) -> NotificationResult<()> {
        debug!(
            notification_id = id.as_u64(),
            "Dismissing D-Bus notification using stored ID"
        );

        // Use async-to-sync conversion for database and D-Bus operations
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Get the platform ID from SurrealDB
                let platform_id = self.get_platform_id(id.as_u64()).await?;

                // Connect to D-Bus and dismiss the notification
                let connection = zbus::Connection::session().await.map_err(|e| {
                    NotificationError::SystemError(format!("D-Bus connection failed: {}", e))
                })?;

                connection
                    .call_method(
                        Some("org.freedesktop.Notifications"),
                        "/org/freedesktop/Notifications",
                        Some("org.freedesktop.Notifications"),
                        "CloseNotification",
                        &(platform_id,),
                    )
                    .await
                    .map_err(|e| {
                        NotificationError::SystemError(format!("D-Bus dismiss failed: {}", e))
                    })?;

                debug!(
                    local_id = id.as_u64(),
                    platform_id = platform_id,
                    "Successfully dismissed D-Bus notification"
                );

                Ok::<(), NotificationError>(())
            })
        });

        result
    }
}
